use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::app::state::App;
use crate::ui::theme;

pub fn draw_commit(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 50, f.area());
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),    // branch
            Constraint::Length(1),    // files summary
            Constraint::Min(3),       // message editor
            Constraint::Length(1),    // hints
        ])
        .split(area.inner(ratatui::layout::Margin::new(1, 1)));

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Commit ")
        .style(Style::default().bg(theme::DIALOG_BG));
    f.render_widget(block, area);

    // Branch
    let branch = app.current_branch.as_deref().unwrap_or("unknown");
    let branch_text = Paragraph::new(format!("Branch: {}", branch))
        .style(Style::default().fg(theme::CYAN));
    f.render_widget(branch_text, chunks[0]);

    // Files summary
    let summary = format!("Files: {}  +{} -{}", app.files_changed(), app.total_additions(), app.total_deletions());
    let summary_text = Paragraph::new(summary)
        .style(Style::default().fg(theme::YELLOW));
    f.render_widget(summary_text, chunks[1]);

    // Message editor
    let msg = app.diff_view.commit_message.clone();
    let msg_text = Paragraph::new(msg)
        .style(Style::default().fg(theme::TEXT_PRIMARY))
        .wrap(Wrap { trim: false });
    f.render_widget(msg_text, chunks[2]);

    // Hints at bottom
    let hints = Paragraph::new(Line::from(vec![
        Span::styled("Enter", Style::default().fg(theme::CYAN)),
        Span::styled(":new line  ", Style::default().fg(theme::TEXT_PRIMARY)),
        Span::styled("Ctrl+Enter", Style::default().fg(theme::CYAN)),
        Span::styled(":commit  ", Style::default().fg(theme::TEXT_PRIMARY)),
        Span::styled("Esc", Style::default().fg(theme::CYAN)),
        Span::styled(":cancel", Style::default().fg(theme::TEXT_PRIMARY)),
    ]));
    f.render_widget(hints, chunks[3]);

    // Place cursor at end of commit message
    let msg = &app.diff_view.commit_message;
    if msg.is_empty() {
        f.set_cursor_position((chunks[2].x, chunks[2].y));
    } else {
        let line_count = msg.lines().count().max(1);
        let last_line_len = msg.lines().last().map(|l| l.len()).unwrap_or(0) as u16;
        let msg_row = (line_count - 1).min(chunks[2].height as usize - 1) as u16;
        f.set_cursor_position((chunks[2].x + last_line_len, chunks[2].y + msg_row));
    }
}

pub fn draw_push(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 30, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Push ")
        .style(Style::default().bg(theme::DIALOG_BG));
    f.render_widget(block, area);

    let branch = app.current_branch.as_deref().unwrap_or("unknown");
    let text = vec![
        Line::from(format!("Push to origin/{}?", branch)),
        Line::from(""),
        Line::from(Span::styled("y/Enter: confirm   n/Esc: cancel", Style::default().fg(theme::TEXT_SECONDARY))),
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
