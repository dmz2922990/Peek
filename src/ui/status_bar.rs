use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::state::App;
use crate::ui::theme;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let additions = app.total_additions();
    let deletions = app.total_deletions();
    let files = app.files_changed();

    let mode_str = format!("{}", app.diff_view.mode);
    let branch = app.current_branch.as_deref().unwrap_or("unknown");

    let mut spans = vec![
        Span::styled(
            format!(" +{} -{} files:{} ", additions, deletions, files),
            Style::default().fg(theme::GREEN),
        ),
        Span::styled(
            format!(" [{}:{}] ", branch, mode_str),
            Style::default().fg(theme::CYAN),
        ),
    ];

    if let Some(ref msg) = app.diff_view.status_message {
        spans.push(Span::styled(
            format!(" {} ", msg),
            Style::default().fg(theme::YELLOW),
        ));
    }

    if app.review.reviewing {
        let filename = app.current_file()
            .map(|f| f.new_path.display().to_string())
            .unwrap_or_default();
        spans.push(Span::styled(
            format!(" Reviewing {}... ", filename),
            Style::default().fg(theme::YELLOW).add_modifier(ratatui::style::Modifier::BOLD),
        ));
    }

    let hint = match app.mode {
        _ => " q:quit j/k:scroll =/-:ctx +/_:all ctx n/N:hunk R:review r:refresh c:commit p:push /:find",
    };
    spans.push(Span::styled(hint.to_string(), Style::default().fg(theme::TEXT_SECONDARY)));

    let paragraph = Paragraph::new(Line::from(spans)).style(Style::default().bg(theme::STATUS_BAR_BG));
    f.render_widget(paragraph, area);
}
