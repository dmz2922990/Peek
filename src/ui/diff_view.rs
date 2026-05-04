use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

use crate::app::state::App;
use crate::diff::types::DiffLine;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    if app.diff_data.is_empty() {
        let widget = Paragraph::new("No diff loaded. Run in a git repository with changes.")
            .style(Style::default().fg(Color::DarkGray));
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

    for hunk in &file.hunks {
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
            let line = match dl {
                DiffLine::Context { content, old_line, new_line } => {
                    Line::from(vec![
                        Span::styled(format!("{:>4} {:>4} ", old_line, new_line), Style::default().fg(Color::DarkGray)),
                        Span::styled(format!(" {}", content), Style::default().fg(Color::White)),
                    ])
                }
                DiffLine::Add { content, new_line } => {
                    Line::from(vec![
                        Span::styled(format!("     {:>4} ", new_line), Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("+{}", content), Style::default().fg(Color::Green)),
                    ])
                }
                DiffLine::Delete { content, old_line } => {
                    Line::from(vec![
                        Span::styled(format!("{:>4}      ", old_line), Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("-{}", content), Style::default().fg(Color::Red)),
                    ])
                }
            };
            lines.push(line);
        }
    }

    let viewport_height = area.height as usize;
    let scroll = app.diff_view.scroll.min(lines.len().saturating_sub(1));

    let visible_lines: Vec<Line> = lines
        .into_iter()
        .skip(scroll)
        .take(viewport_height)
        .collect();

    let widget = Paragraph::new(visible_lines)
        .block(Block::default())
        .wrap(Wrap { trim: false });

    f.render_widget(widget, area);
}
