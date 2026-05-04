use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::app::state::App;

pub fn draw_commit(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 50, f.area());
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),    // branch
            Constraint::Length(3),    // files summary
            Constraint::Min(5),       // message editor
            Constraint::Length(1),    // hints
        ])
        .split(area.inner(ratatui::layout::Margin::new(1, 1)));

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Commit ")
        .style(Style::default().bg(Color::DarkGray));
    f.render_widget(block, area);

    // Branch
    let branch = app.current_branch.as_deref().unwrap_or("unknown");
    let branch_text = Paragraph::new(format!("Branch: {}", branch))
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(branch_text, chunks[0]);

    // Files summary
    let summary = format!("Files changed: {} (+{} -{})", app.files_changed(), app.total_additions(), app.total_deletions());
    let summary_text = Paragraph::new(summary)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(summary_text, chunks[1]);

    // Message editor
    let msg = if app.diff_view.commit_message.is_empty() {
        "Type commit message...".to_string()
    } else {
        app.diff_view.commit_message.clone()
    };
    let msg_style = if app.diff_view.commit_message.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };
    let msg_text = Paragraph::new(msg).style(msg_style).wrap(Wrap { trim: false });
    f.render_widget(msg_text, chunks[2]);

    // Hints
    let hints = Paragraph::new("Ctrl+Enter: commit  Esc: cancel")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(hints, chunks[3]);
}

pub fn draw_push(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 30, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Push ")
        .style(Style::default().bg(Color::DarkGray));
    f.render_widget(block, area);

    let branch = app.current_branch.as_deref().unwrap_or("unknown");
    let text = vec![
        Line::from(format!("Push to origin/{}?", branch)),
        Line::from(""),
        Line::from(Span::styled("y/Enter: confirm   n/Esc: cancel", Style::default().fg(Color::DarkGray))),
    ];
    let widget = Paragraph::new(text);
    f.render_widget(widget, area.inner(ratatui::layout::Margin::new(2, 2)));
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
