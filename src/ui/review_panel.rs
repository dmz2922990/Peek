use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
};

use crate::app::state::{App, ReviewSeverity};
use crate::ui::theme;

pub fn draw(f: &mut Frame, app: &mut App, area: Rect, focused: bool) {
    if area.height == 0 { return; }

    f.render_widget(Clear, area);

    let border_style = if focused {
        Style::default().fg(theme::CYAN)
    } else {
        Style::default().fg(theme::TEXT_SECONDARY)
    };

    let title = if app.review.reviewing {
        " Reviewing... "
    } else if let Some(ref msg) = app.review.status_message {
        Box::leak(format!(" Review: {} ", msg).into_boxed_str())
    } else {
        let n = app.review.comments.len();
        if focused { Box::leak(format!(" Review ({}) > ", n).into_boxed_str()) }
        else { Box::leak(format!(" Review ({}) ", n).into_boxed_str()) }
    };

    let items: Vec<ListItem> = app.review.comments.iter().enumerate().map(|(idx, comment)| {
        let (icon, color) = match comment.severity {
            ReviewSeverity::Error => ("\u{2717}", theme::RED),
            ReviewSeverity::Warning => ("\u{26A0}", theme::YELLOW),
            ReviewSeverity::Suggestion => ("\u{1F4A1}", theme::BLUE),
        };

        let is_selected = idx == app.review.selected;
        let is_expanded = app.review.expanded == Some(idx);

        let line_no = if comment.line_no > 0 {
            format!("L{:<4}", comment.line_no)
        } else {
            "      ".to_string()
        };

        let bg = if is_selected {
            theme::SELECTION_BG
        } else {
            theme::DEFAULT_BG
        };

        let summary_style = Style::default().fg(theme::TEXT_PRIMARY).bg(bg);
        let line_style = Style::default().fg(theme::TEXT_SECONDARY).bg(bg);
        let icon_style = Style::default().fg(color).bg(bg);

        let mut spans = vec![
            Span::styled(format!(" {} ", icon), icon_style),
            Span::styled(line_no, line_style),
            Span::styled(&comment.summary, summary_style),
        ];

        if is_expanded && !comment.detail.is_empty() {
            let detail_style = Style::default().fg(theme::TEXT_SECONDARY).bg(theme::DEFAULT_BG).add_modifier(Modifier::DIM);
            spans.push(Span::styled("\n", Style::default()));
            spans.push(Span::styled(format!("       {}", comment.detail), detail_style));
        }

        ListItem::new(Line::from(spans))
    }).collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::TOP)
            .title(title)
            .border_style(border_style)
            .style(Style::default().bg(theme::DEFAULT_BG)));

    let mut state = ListState::default();
    state.select(Some(app.review.selected));

    f.render_stateful_widget(list, area, &mut state);
}
