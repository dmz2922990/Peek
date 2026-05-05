use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    widgets::Paragraph,
};

use crate::app::state::App;
use crate::ui::theme;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let term = app.diff_view.search_term.as_deref().unwrap_or("");
    let text = format!("/{}", term);
    let matches = app.diff_view.search_matches.len();
    let hint = if matches > 0 {
        format!(" ({} matches)", matches)
    } else if !term.is_empty() {
        " (no matches)".to_string()
    } else {
        String::new()
    };

    let paragraph = Paragraph::new(format!("{}{}", text, hint))
        .style(Style::default().fg(theme::YELLOW));
    f.render_widget(paragraph, area);
}
