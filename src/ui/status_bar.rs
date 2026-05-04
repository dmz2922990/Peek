use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::state::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let additions = app.total_additions();
    let deletions = app.total_deletions();
    let files = app.files_changed();

    let mode_str = format!("{}", app.diff_view.mode);

    let mut spans = vec![
        Span::styled(
            format!(" +{} -{} files:{} ", additions, deletions, files),
            Style::default().fg(Color::Green),
        ),
        Span::styled(
            format!(" [{}] ", mode_str),
            Style::default().fg(Color::Cyan),
        ),
    ];

    if let Some(ref msg) = app.diff_view.status_message {
        spans.push(Span::styled(
            format!(" {} ", msg),
            Style::default().fg(Color::Yellow),
        ));
    }

    let hint = match app.mode {
        _ => " q:quit j/k:scroll =/-:context c:commit p:push o:PR /:find y:copy v:select Ctrl+T:tree",
    };
    spans.push(Span::styled(hint.to_string(), Style::default().fg(Color::DarkGray)));

    let paragraph = Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Black));
    f.render_widget(paragraph, area);
}
