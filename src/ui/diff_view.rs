use std::fs;
use std::path::Path;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::state::{App, AppMode};
use crate::diff::types::DiffLine;

pub fn draw(f: &mut Frame, app: &mut App, area: Rect, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = match app.mode {
        AppMode::VisualSelect => " VISUAL ",
        _ if focused => " Diff ◂ ",
        _ => " Diff ",
    };

    let block = Block::default()
        .borders(Borders::NONE)
        .title(Span::styled(title, border_style));

    if app.diff_data.is_empty() {
        let widget = Paragraph::new("No diff loaded. Run in a git repository with changes.")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        f.render_widget(widget, area);
        return;
    }

    let file = app.current_file().or_else(|| app.diff_data.first());
    let Some(file) = file else {
        return;
    };

    let mut lines: Vec<Line> = Vec::new();
    // (hunk_idx, Some(line_idx)) = diff line, (hunk_idx, None) = expanded context, None = header/fold
    let mut line_map: Vec<Option<(usize, Option<usize>)>> = Vec::new();
    let mut fold_positions: Vec<(usize, usize)> = Vec::new();

    // File header
    lines.push(Line::from(Span::styled(
        format!("--- {}", file.old_path.display()),
        Style::default().fg(Color::DarkGray),
    )));
    line_map.push(None);
    lines.push(Line::from(Span::styled(
        format!("+++ {}", file.new_path.display()),
        Style::default().fg(Color::DarkGray),
    )));
    line_map.push(None);

    // Use the locked expansion target from state (set by = key, not computed per frame)
    let expand_target = app.diff_view.expand_hunk;

    // Load source file for context expansion
    let source_lines = load_source_lines(&file.new_path);
    let extra = app.diff_view.extra_context;

    for (hunk_idx, hunk) in file.hunks.iter().enumerate() {
        // Only expand context near the locked target hunk (± 1)
        let expand_this = extra > 0 && expand_target.map_or(false, |et| {
            hunk_idx >= et.saturating_sub(1) && hunk_idx <= et + 1
        });

        // ── Fold indicator / expanded context before hunk ──
        let prev_hunk_end = if hunk_idx == 0 {
            1
        } else {
            file.hunks[hunk_idx - 1].new_start + file.hunks[hunk_idx - 1].new_count
        };
        let current_hunk_start = hunk.new_start;
        let gap = current_hunk_start.saturating_sub(prev_hunk_end);

        if gap > 0 {
            if expand_this {
                // Fold for remaining hidden lines (at top), then expanded context below
                let expand_count = gap.min(extra);
                let remaining = gap.saturating_sub(expand_count);
                if remaining > 0 {
                    let fold_idx = lines.len();
                    lines.push(make_fold_line(remaining));
                    line_map.push(None);
                    fold_positions.push((fold_idx, hunk_idx));
                }
                let start = current_hunk_start.saturating_sub(expand_count);
                for line_no in start..current_hunk_start {
                    if let Some(content) = get_source_line(&source_lines, line_no) {
                        lines.push(make_expanded_context_line(&content, line_no, line_no));
                        line_map.push(Some((hunk_idx, None)));
                    }
                }
            } else {
                // Show fold indicator for full gap
                let fold_idx = lines.len();
                lines.push(make_fold_line(gap));
                line_map.push(None);
                fold_positions.push((fold_idx, hunk_idx));
            }
        }

        // Hunk header
        let header_text = if let Some(ref func) = hunk.function_name {
            format!("@@ -{},{} +{},{} @@ {}", hunk.old_start, hunk.old_count, hunk.new_start, hunk.new_count, func)
        } else {
            format!("@@ -{},{} +{},{} @@", hunk.old_start, hunk.old_count, hunk.new_start, hunk.new_count)
        };
        lines.push(Line::from(Span::styled(
            header_text,
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));
        line_map.push(None);

        for (line_idx, dl) in hunk.lines.iter().enumerate() {
            lines.push(make_diff_line(dl));
            line_map.push(Some((hunk_idx, Some(line_idx))));
        }

        // ── Fold / expanded context after last hunk ──
        if hunk_idx == file.hunks.len() - 1 {
            let after_start = hunk.new_start + hunk.new_count;
            let total_file_lines = source_lines.len();
            let tail_gap = total_file_lines.saturating_sub(after_start);

            if tail_gap > 0 {
                if app.diff_view.expand_tail && extra > 0 {
                    let expand_count = tail_gap.min(extra);
                    for offset in 0..expand_count {
                        let line_no = after_start + offset;
                        if let Some(content) = get_source_line(&source_lines, line_no) {
                            lines.push(make_expanded_context_line(&content, line_no, line_no));
                            line_map.push(Some((hunk_idx, None)));
                        }
                    }
                    let remaining = tail_gap.saturating_sub(expand_count);
                    if remaining > 0 {
                        let fold_idx = lines.len();
                        lines.push(make_fold_line(remaining));
                        line_map.push(None);
                        fold_positions.push((fold_idx, usize::MAX));
                    }
                } else {
                    let fold_idx = lines.len();
                    lines.push(make_fold_line(tail_gap));
                    line_map.push(None);
                    fold_positions.push((fold_idx, usize::MAX));
                }
            }
        }
    }

    // Write state back for key handler
    let total_lines = lines.len();
    app.diff_view.total_lines = total_lines;
    app.diff_view.rendered_line_map = line_map;
    app.diff_view.fold_positions = fold_positions;

    // Resolve pending fold jump: move cursor to the new fold position
    if let Some(target) = app.diff_view.jump_to_fold.take() {
        if let Some((pos, _)) = app.diff_view.fold_positions.iter().find(|(_, t)| *t == target) {
            app.diff_view.cursor = *pos;
        }
    }

    let viewport_height = area.height.saturating_sub(1) as usize;
    app.diff_view.viewport_height = viewport_height;
    let max_scroll = total_lines.saturating_sub(viewport_height);

    // Clamp cursor within [0, total_lines-1]
    if app.diff_view.cursor >= total_lines {
        app.diff_view.cursor = total_lines.saturating_sub(1);
    }

    // Ensure scroll keeps cursor visible: scroll <= cursor <= scroll + viewport - 1
    if app.diff_view.cursor < app.diff_view.scroll {
        app.diff_view.scroll = app.diff_view.cursor;
    } else if app.diff_view.cursor >= app.diff_view.scroll + viewport_height {
        app.diff_view.scroll = app.diff_view.cursor + 1 - viewport_height;
    }

    // Clamp scroll and write back to app state
    if app.diff_view.scroll > max_scroll {
        app.diff_view.scroll = max_scroll;
    }

    let cursor_line = app.diff_view.cursor;

    // Compute visual select range
    let sel_range = if app.mode == AppMode::VisualSelect {
        app.diff_view.selection_start.and_then(|start| {
            app.diff_view.selection_end.map(|end| {
                let lo = start.min(end);
                let hi = start.max(end);
                (lo, hi)
            })
        })
    } else {
        None
    };

    let visible_lines: Vec<Line> = lines
        .into_iter()
        .enumerate()
        .skip(app.diff_view.scroll)
        .take(viewport_height)
        .map(|(idx, mut line)| {
            let is_cursor = focused && idx == cursor_line;
            let is_selected = sel_range.map_or(false, |(lo, hi)| idx >= lo && idx <= hi);

            if is_cursor {
                let mut new_spans = vec![
                    Span::styled("▸", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                ];
                new_spans.append(&mut line.spans);
                let bg = if is_selected { Color::Rgb(60, 60, 100) } else { Color::DarkGray };
                Line::from(new_spans).patch_style(Style::default().bg(bg))
            } else if is_selected {
                Line::from(
                    std::mem::take(&mut line.spans)
                ).patch_style(Style::default().bg(Color::Rgb(40, 40, 80)))
            } else {
                line
            }
        })
        .collect();

    let widget = Paragraph::new(visible_lines)
        .block(block);

    f.render_widget(widget, area);
}

fn load_source_lines(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .map(|content| content.lines().map(String::from).collect())
        .unwrap_or_default()
}

fn get_source_line(lines: &[String], line_no: usize) -> Option<String> {
    if line_no == 0 { return None; }
    lines.get(line_no - 1).cloned()
}

fn make_diff_line(dl: &DiffLine) -> Line<'static> {
    match dl {
        DiffLine::Context { content, old_line, new_line } => {
            Line::from(vec![
                Span::styled(format!(" {:>4} {:>4} ", old_line, new_line), Style::default().fg(Color::DarkGray)),
                Span::styled(format!(" {}", content), Style::default().fg(Color::White)),
            ])
        }
        DiffLine::Add { content, new_line } => {
            Line::from(vec![
                Span::styled(format!("      {:>4} ", new_line), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("+{}", content), Style::default().fg(Color::Green)),
            ])
        }
        DiffLine::Delete { content, old_line } => {
            Line::from(vec![
                Span::styled(format!(" {:>4}      ", old_line), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("-{}", content), Style::default().fg(Color::Red)),
            ])
        }
    }
}

fn make_expanded_context_line(content: &str, old_line: usize, new_line: usize) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!(" {:>4} {:>4} ", old_line, new_line), Style::default().fg(Color::Blue)),
        Span::styled(format!(" {}", content), Style::default().fg(Color::Blue).add_modifier(Modifier::DIM)),
    ])
}

fn make_fold_line(hidden: usize) -> Line<'static> {
    Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            format!("· · · {} line{} hidden · · ·", hidden, if hidden > 1 { "s" } else { "" }),
            Style::default().fg(Color::Blue).add_modifier(Modifier::DIM),
        ),
    ])
}
