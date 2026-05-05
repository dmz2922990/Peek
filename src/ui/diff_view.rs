use std::fs;
use std::path::Path;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::state::{App, AppMode};
use crate::diff::types::DiffLine;
use crate::ui::theme;

pub fn draw(f: &mut Frame, app: &mut App, area: Rect, focused: bool) {
    let border_style = if focused {
        Style::default().fg(theme::CYAN)
    } else {
        Style::default().fg(theme::UNFOCUSED_BORDER)
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
            .style(Style::default().fg(theme::TEXT_SECONDARY))
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
        Style::default().fg(theme::TEXT_SECONDARY),
    )));
    line_map.push(None);
    lines.push(Line::from(Span::styled(
        format!("+++ {}", file.new_path.display()),
        Style::default().fg(theme::TEXT_SECONDARY),
    )));
    line_map.push(None);

    // Load source file for context expansion
    let source_lines = load_source_lines(&file.new_path);
    let expanded_folds = &app.diff_view.expanded_folds;

    for (hunk_idx, hunk) in file.hunks.iter().enumerate() {
        let down_key = hunk_idx * 2;
        let up_key = hunk_idx * 2 + 1;
        let down_count = expanded_folds.get(&down_key).copied().unwrap_or(0);
        let up_count = expanded_folds.get(&up_key).copied().unwrap_or(0);

        // ── Fold indicator / expanded context before hunk ──
        let prev_hunk_end = if hunk_idx == 0 {
            1
        } else {
            file.hunks[hunk_idx - 1].new_start + file.hunks[hunk_idx - 1].new_count
        };
        let current_hunk_start = hunk.new_start;
        let gap = current_hunk_start.saturating_sub(prev_hunk_end);

        if gap > 0 {
            let total_shown = down_count + up_count;
            if total_shown >= gap {
                // All context visible
                for line_no in prev_hunk_end..current_hunk_start {
                    if let Some(content) = get_source_line(&source_lines, line_no) {
                        lines.push(make_expanded_context_line(&content, line_no, line_no));
                        line_map.push(Some((hunk_idx, None)));
                    }
                }
            } else {
                let hidden = gap - total_shown;
                let use_dual = gap > app.config.diff.default_context_lines;

                // Context expanded from top
                for i in 0..down_count {
                    let line_no = prev_hunk_end + i;
                    if let Some(content) = get_source_line(&source_lines, line_no) {
                        lines.push(make_expanded_context_line(&content, line_no, line_no));
                        line_map.push(Some((hunk_idx, None)));
                    }
                }

                if use_dual {
                    // ↓ fold indicator
                    let fold_idx = lines.len();
                    lines.push(make_fold_line_down(hidden));
                    line_map.push(None);
                    fold_positions.push((fold_idx, down_key));

                    // ↑ fold indicator
                    let fold_idx = lines.len();
                    lines.push(make_fold_line_up(hidden));
                    line_map.push(None);
                    fold_positions.push((fold_idx, up_key));
                } else {
                    // Single fold indicator for small gap
                    let fold_idx = lines.len();
                    lines.push(make_fold_line(hidden));
                    line_map.push(None);
                    fold_positions.push((fold_idx, down_key));
                }

                // Context expanded from bottom
                for i in 0..up_count {
                    let line_no = current_hunk_start - up_count + i;
                    if let Some(content) = get_source_line(&source_lines, line_no) {
                        lines.push(make_expanded_context_line(&content, line_no, line_no));
                        line_map.push(Some((hunk_idx, None)));
                    }
                }
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
            Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD),
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
                let tail_extra = expanded_folds.get(&usize::MAX).copied().unwrap_or(0);
                if tail_extra > 0 {
                    let expand_count = tail_gap.min(tail_extra);
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
                    Span::styled("▸", Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD)),
                ];
                new_spans.append(&mut line.spans);
                let bg = if is_selected { theme::VISUAL_SELECT_CURSOR } else { theme::SELECTION_BG };
                Line::from(new_spans).patch_style(Style::default().bg(bg))
            } else if is_selected {
                Line::from(
                    std::mem::take(&mut line.spans)
                ).patch_style(Style::default().bg(theme::VISUAL_SELECT_RANGE))
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
                Span::styled(format!(" {:>4} {:>4} ", old_line, new_line), Style::default().fg(theme::TEXT_SECONDARY)),
                Span::styled(format!(" {}", content), Style::default().fg(theme::TEXT_PRIMARY)),
            ])
        }
        DiffLine::Add { content, new_line } => {
            Line::from(vec![
                Span::styled(format!("      {:>4} ", new_line), Style::default().fg(theme::TEXT_SECONDARY)),
                Span::styled(format!("+{}", content), Style::default().fg(theme::GREEN)),
            ])
        }
        DiffLine::Delete { content, old_line } => {
            Line::from(vec![
                Span::styled(format!(" {:>4}      ", old_line), Style::default().fg(theme::TEXT_SECONDARY)),
                Span::styled(format!("-{}", content), Style::default().fg(theme::RED)),
            ])
        }
    }
}

fn make_expanded_context_line(content: &str, old_line: usize, new_line: usize) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!(" {:>4} {:>4} ", old_line, new_line), Style::default().fg(theme::BLUE)),
        Span::styled(format!(" {}", content), Style::default().fg(theme::TEXT_PRIMARY).add_modifier(Modifier::DIM)),
    ])
}

fn make_fold_line(hidden: usize) -> Line<'static> {
    Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            format!("· · · {} line{} hidden · · ·", hidden, if hidden > 1 { "s" } else { "" }),
            Style::default().fg(theme::BLUE).add_modifier(Modifier::DIM),
        ),
    ])
}

fn make_fold_line_down(hidden: usize) -> Line<'static> {
    Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            format!("↓ · · · {} line{} hidden · · ·", hidden, if hidden > 1 { "s" } else { "" }),
            Style::default().fg(theme::BLUE).add_modifier(Modifier::DIM),
        ),
    ])
}

fn make_fold_line_up(hidden: usize) -> Line<'static> {
    Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            format!("↑ · · · {} line{} hidden · · ·", hidden, if hidden > 1 { "s" } else { "" }),
            Style::default().fg(theme::BLUE).add_modifier(Modifier::DIM),
        ),
    ])
}
