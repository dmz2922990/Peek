use std::fs;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::app::state::App;

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    f.render_widget(Clear, area);

    let Some(ref file_path) = app.editing_file else {
        let widget = Paragraph::new("No file open for editing. Press Esc to return.")
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(widget, area);
        return;
    };

    let content = fs::read_to_string(file_path)
        .unwrap_or_else(|_| String::from("(unable to read file)"));

    let file_name = file_path.display();
    let header = format!(" Editing: {} ", file_name);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(header, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));

    let lines: Vec<Line> = content.lines()
        .enumerate()
        .map(|(i, line)| {
            let line_num = format!("{:>4} ", i + 1);
            Line::from(vec![
                Span::styled(line_num, Style::default().fg(Color::DarkGray)),
                Span::raw(line.to_string()),
            ])
        })
        .collect();

    let widget = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.diff_view.scroll as u16, 0));

    f.render_widget(widget, area);

    // Bottom hint bar
    let hint_area = Rect::new(0, area.height - 1, area.width, 1);
    let hints = " Ctrl+S: save  Ctrl+R: find/replace  Esc: return to diff";
    let hint = Paragraph::new(hints)
        .style(Style::default().fg(Color::DarkGray).bg(Color::Black));
    f.render_widget(hint, hint_area);
}
