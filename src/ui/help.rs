use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::state::App;
use crate::config::types::{CONFIG_ENTRIES, KEYBINDING_ENTRIES};

const FIXED_KEYS: &[(&str, &str)] = &[
    ("g/G", "Jump to top/bottom"),
    ("n/N", "Next/previous hunk"),
    ("Tab", "Switch focus"),
    ("PgDn/PgUp", "Half-page scroll"),
    ("Ctrl+Enter", "Confirm commit"),
    ("Esc", "Cancel/close"),
];

const KB_COUNT: usize = KEYBINDING_ENTRIES.len();
const CFG_OFFSET: usize = KB_COUNT + 1;
const CFG_COUNT: usize = CONFIG_ENTRIES.len();
const FIXED_OFFSET: usize = CFG_OFFSET + CFG_COUNT + 1;

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = centered_rect(70, 80, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Settings (Enter to edit, Esc to close) ")
        .style(Style::default().bg(Color::DarkGray));
    f.render_widget(block, area);

    let inner = area.inner(Margin::new(1, 1));
    let viewport_height = inner.height as usize;

    let mut lines: Vec<Line> = Vec::new();

    // ── Keybindings ──
    for (i, (_field, desc)) in KEYBINDING_ENTRIES.iter().enumerate() {
        let binding = app.config.keybindings.get_binding(i).unwrap_or("?");
        lines.push(entry_line(i, app.help_cursor, *desc, binding, Color::Cyan));
    }

    // ── Configuration ──
    lines.push(separator(" Configuration "));
    for (i, (_field, desc)) in CONFIG_ENTRIES.iter().enumerate() {
        let row_idx = CFG_OFFSET + i;
        let value = app.config.diff.get_entry(i).unwrap_or_else(|| "?".into());
        lines.push(entry_line(row_idx, app.help_cursor, *desc, &value, Color::Green));
    }

    // ── Fixed keys ──
    lines.push(separator(" Fixed keys "));
    for (offset, (key, desc)) in FIXED_KEYS.iter().enumerate() {
        let row_idx = FIXED_OFFSET + offset;
        lines.push(entry_line(row_idx, app.help_cursor, *desc, key, Color::Magenta));
    }

    // Clamp scroll
    let total_lines = lines.len();
    let max_scroll = total_lines.saturating_sub(viewport_height);
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

    // ── Edit popup overlay ──
    if let Some(edit_idx) = app.help_editing {
        draw_edit_popup(f, app, edit_idx);
    }
}

fn draw_edit_popup(f: &mut Frame, app: &App, edit_idx: usize) {
    let (field_name, description, is_keybinding) = resolve_edit_field(edit_idx);

    let popup = centered_rect(50, 28, f.area());
    f.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Edit: {} ", field_name))
        .style(Style::default().bg(Color::Rgb(30, 30, 45)));
    f.render_widget(block, popup);

    let inner = popup.inner(Margin::new(1, 1));

    let mut lines = Vec::new();

    // Field description
    lines.push(Line::from(Span::styled(
        description.to_string(),
        Style::default().fg(Color::White),
    )));
    lines.push(Line::from(""));

    // Current value label
    lines.push(Line::from(Span::styled(
        "Current value:",
        Style::default().fg(Color::DarkGray),
    )));

    // Current value
    let current = app.help_input_buffer.clone();
    lines.push(Line::from(Span::styled(
        format!("  {}", current),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    // Hints
    if is_keybinding {
        lines.push(Line::from(Span::styled(
            "Press any key to set new binding",
            Style::default().fg(Color::Yellow),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "Type to edit value",
            Style::default().fg(Color::Yellow),
        )));
    }
    lines.push(Line::from(Span::styled(
        "Enter: confirm   Esc: cancel",
        Style::default().fg(Color::DarkGray),
    )));

    // Value hints for config entries
    if !is_keybinding {
        let cfg_idx = edit_idx - CFG_OFFSET;
        let hint = match cfg_idx {
            0 => "e.g. 3",
            1 => "e.g. 30",
            2 => "left or right",
            _ => "",
        };
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("Valid values: {}", hint),
            Style::default().fg(Color::DarkGray),
        )));
    }

    let widget = Paragraph::new(lines);
    f.render_widget(widget, inner);

    // Place terminal cursor at end of input value (line 3 = "  {value}")
    let buffer_len = app.help_input_buffer.len() as u16;
    let cursor_x = inner.x + 2 + buffer_len;
    let cursor_y = inner.y + 3; // line index 3 = the value line
    f.set_cursor_position((cursor_x, cursor_y));
}

fn resolve_edit_field(edit_idx: usize) -> (&'static str, &'static str, bool) {
    if edit_idx < KB_COUNT {
        let (field, desc) = &KEYBINDING_ENTRIES[edit_idx];
        return (*field, *desc, true);
    }
    if edit_idx >= CFG_OFFSET && edit_idx < CFG_OFFSET + CFG_COUNT {
        let cfg_idx = edit_idx - CFG_OFFSET;
        let (field, desc) = &CONFIG_ENTRIES[cfg_idx];
        return (*field, *desc, false);
    }
    ("unknown", "", false)
}

fn entry_line(row_idx: usize, cursor: usize, desc: &str, value: &str, value_color: Color) -> Line<'static> {
    let is_cursor = row_idx == cursor;
    let marker = if is_cursor { "▸ " } else { "  " };
    Line::from(vec![
        Span::styled(marker, Style::default().fg(Color::Yellow)),
        Span::styled(format!("{:>20} ", desc), cursor_style(is_cursor, Color::White)),
        Span::styled(value.to_string(), cursor_style(is_cursor, value_color)),
    ])
}

fn cursor_style(is_cursor: bool, color: Color) -> Style {
    if is_cursor {
        Style::default().fg(color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(color)
    }
}

fn separator(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("  ──{}──", title),
        Style::default().fg(Color::DarkGray),
    ))
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
