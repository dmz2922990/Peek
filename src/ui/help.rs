use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::state::App;
use crate::config::types::KEYBINDING_ENTRIES;

// Fixed (non-configurable) keys shown for reference
const FIXED_KEYS: &[(&str, &str)] = &[
    ("g/G", "Jump to top/bottom"),
    ("n/N", "Next/previous hunk"),
    ("Tab", "Switch focus"),
    ("PgDn/PgUp", "Half-page scroll"),
    ("Esc", "Cancel/close"),
    ("Ctrl+Enter", "Confirm commit"),
];

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = centered_rect(70, 80, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Keybindings (Enter to edit, Esc to close) ")
        .style(Style::default().bg(Color::DarkGray));
    f.render_widget(block, area);

    let inner = area.inner(Margin::new(1, 1));
    let viewport_height = inner.height as usize;

    let configurable_count = KEYBINDING_ENTRIES.len();

    let mut lines: Vec<Line> = Vec::new();

    // Configurable keybindings
    for (i, (_field, desc)) in KEYBINDING_ENTRIES.iter().enumerate() {
        let binding = app.config.keybindings.get_binding(i).unwrap_or("?");

        if app.help_editing == Some(i) {
            lines.push(Line::from(vec![
                Span::styled(format!("  {:>20} ", *desc), Style::default().fg(Color::Yellow)),
                Span::styled(" <press new key... Esc=cancel>".to_string(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]));
        } else {
            let is_cursor = i == app.help_cursor;
            let desc_style = if is_cursor {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let key_style = if is_cursor {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Cyan)
            };
            let cursor_marker = if is_cursor { "▸ " } else { "  " };

            lines.push(Line::from(vec![
                Span::styled(cursor_marker.to_string(), Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:>20} ", *desc), desc_style),
                Span::styled(binding.to_string(), key_style),
            ]));
        }
    }

    // Separator
    lines.push(Line::from(Span::styled(
        "  ── Fixed keys ──".to_string(),
        Style::default().fg(Color::DarkGray),
    )));

    // Fixed keys
    for (offset, (key, desc)) in FIXED_KEYS.iter().enumerate() {
        let row_idx = configurable_count + 1 + offset;
        let is_cursor = app.help_cursor == row_idx;

        let desc_style = if is_cursor {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let key_style = if is_cursor {
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Magenta)
        };
        let cursor_marker = if is_cursor { "▸ " } else { "  " };

        lines.push(Line::from(vec![
            Span::styled(cursor_marker.to_string(), Style::default().fg(Color::Yellow)),
            Span::styled(format!("{:>20} ", *desc), desc_style),
            Span::styled(key.to_string(), key_style),
        ]));
    }

    let total_lines = lines.len();
    let max_scroll = total_lines.saturating_sub(viewport_height);

    // Clamp scroll to keep cursor visible
    if app.help_cursor < app.help_scroll {
        app.help_scroll = app.help_cursor;
    } else if app.help_cursor >= app.help_scroll + viewport_height {
        app.help_scroll = app.help_cursor + 1 - viewport_height;
    }
    if app.help_scroll > max_scroll {
        app.help_scroll = max_scroll;
    }

    let visible: Vec<Line> = lines
        .into_iter()
        .enumerate()
        .skip(app.help_scroll)
        .take(viewport_height)
        .map(|(idx, mut line)| {
            if idx == app.help_cursor {
                Line::from(std::mem::take(&mut line.spans))
                    .patch_style(Style::default().bg(Color::Rgb(40, 40, 60)))
            } else {
                line
            }
        })
        .collect();

    let widget = Paragraph::new(visible);
    f.render_widget(widget, inner);
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
