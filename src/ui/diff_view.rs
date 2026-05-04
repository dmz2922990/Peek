use std::fs;
use std::path::Path;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::state::App;
use crate::diff::types::DiffLine;

pub fn draw(f: &mut Frame, app: &mut App, area: Rect, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = if focused {
        " Diff ◂ "
    } else {
        " Diff "
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

    // File header
    lines.push(Line::from(Span::styled(
        format!("--- {}", file.old_path.display()),
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(Span::styled(
        format!("+++ {}", file.new_path.display()),
        Style::default().fg(Color::DarkGray),
    )));

    // Load source file for context expansion
    let source_lines = load_source_lines(&file.new_path);
    let extra = app.diff_view.extra_context;

    for (hunk_idx, hunk) in file.hunks.iter().enumerate() {
        // Inject extra context before hunk (between hunks)
        if extra > 0 {
            let prev_hunk_end = if hunk_idx == 0 {
                1
            } else {
                file.hunks[hunk_idx - 1].new_start + file.hunks[hunk_idx - 1].new_count
            };
            let current_hunk_start = hunk.new_start;
            let gap = current_hunk_start.saturating_sub(prev_hunk_end);

            if gap > 0 {
                let expand_count = gap.min(extra);
                let start = current_hunk_start.saturating_sub(expand_count);
                for line_no in start..current_hunk_start {
                    if let Some(content) = get_source_line(&source_lines, line_no) {
                        lines.push(make_expanded_context_line(&content, line_no + prev_hunk_end.saturating_sub(1), line_no));
                    }
                }
                if gap > expand_count {
                    lines.push(Line::from(Span::styled(
                        format!("  ... ({} lines hidden)", gap.saturating_sub(expand_count)),
                        Style::default().fg(Color::DarkGray),
                    )));
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
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));

        for dl in &hunk.lines {
            lines.push(make_diff_line(dl));
        }

        // Inject extra context after last hunk
        if extra > 0 && hunk_idx == file.hunks.len() - 1 {
            let after_start = hunk.new_start + hunk.new_count;
            for offset in 0..extra {
                let line_no = after_start + offset;
                if let Some(content) = get_source_line(&source_lines, line_no) {
                    lines.push(make_expanded_context_line(&content, line_no, line_no));
                }
            }
        }
    }

    // Context indicator
    if extra > 0 {
        lines.push(Line::from(Span::styled(
            format!("  [context expanded: +{} lines]", extra),
            Style::default().fg(Color::Yellow),
        )));
    }

    // Write actual total back so key handler stays in sync
    let total_lines = lines.len();
    app.diff_view.total_lines = total_lines;

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

    let visible_lines: Vec<Line> = lines
        .into_iter()
        .enumerate()
        .skip(app.diff_view.scroll)
        .take(viewport_height)
        .map(|(idx, mut line)| {
            if focused && idx == cursor_line {
                let mut new_spans = vec![
                    Span::styled("▸", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                ];
                new_spans.append(&mut line.spans);
                Line::from(new_spans).patch_style(Style::default().bg(Color::DarkGray))
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
