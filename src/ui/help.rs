use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::state::{App, HelpTab};
use crate::config::types::{CONFIG_ENTRIES, KEYBINDING_ENTRIES};
use crate::ui::theme;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO_URL: &str = env!("CARGO_PKG_REPOSITORY");

const FIXED_KEYS: &[(&str, &str)] = &[
    ("g/G", "Jump to top/bottom"),
    ("n/N", "Next/previous hunk"),
    ("Tab", "Switch focus / tab"),
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
        .style(Style::default().bg(theme::DIALOG_BG));
    f.render_widget(block, area);

    let inner = area.inner(Margin::new(1, 1));

    // ── Tab bar ──
    let tab_bar = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    let tab_line = make_tab_bar(&app.help_tab);
    f.render_widget(Paragraph::new(tab_line), tab_bar[0]);

    let content_area = tab_bar[1];
    let viewport_height = content_area.height as usize;

    match app.help_tab {
        HelpTab::Settings => draw_settings(f, app, content_area, viewport_height),
        HelpTab::About => draw_about(f, content_area),
    }

    // Edit popup overlay (Settings tab only)
    if app.help_tab == HelpTab::Settings {
        if let Some(edit_idx) = app.help_editing {
            draw_edit_popup(f, app, edit_idx);
        }
    }
}

fn draw_settings(f: &mut Frame, app: &mut App, area: Rect, viewport_height: usize) {
    let mut lines: Vec<Line> = Vec::new();

    for (i, (_field, desc)) in KEYBINDING_ENTRIES.iter().enumerate() {
        let binding = app.config.keybindings.get_binding(i).unwrap_or("?");
        lines.push(entry_line(i, app.help_cursor, *desc, binding, theme::CYAN));
    }

    lines.push(separator(" Configuration "));
    for (i, (_field, desc)) in CONFIG_ENTRIES.iter().enumerate() {
        let row_idx = CFG_OFFSET + i;
        let value = app.config.diff.get_entry(i).unwrap_or_else(|| "?".into());
        lines.push(entry_line(row_idx, app.help_cursor, *desc, &value, theme::GREEN));
    }

    lines.push(separator(" Fixed keys "));
    for (offset, (key, desc)) in FIXED_KEYS.iter().enumerate() {
        let row_idx = FIXED_OFFSET + offset;
        lines.push(entry_line(row_idx, app.help_cursor, *desc, key, theme::MAGENTA));
    }

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
                    .patch_style(Style::default().bg(theme::HELP_CURSOR_BG))
            } else {
                line
            }
        })
        .collect();

    f.render_widget(Paragraph::new(visible), area);
}

fn draw_about(f: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(""),
        logo_line1(),
        logo_line2(),
        logo_line3(),
        logo_line4(),
        logo_line5(),
        logo_line6(),
        Line::from(""),
        Line::from(Span::styled(
            format!("  Peek v{}", VERSION),
            Style::default().fg(theme::TEXT_PRIMARY).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  A terminal diff viewer with syntax highlighting",
            Style::default().fg(theme::TEXT_SECONDARY),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(REPO_URL, Style::default().fg(theme::BLUE).add_modifier(Modifier::UNDERLINED)),
        ]),
    ];

    f.render_widget(Paragraph::new(lines), area);
}

// ── Logo lines ──

fn logo_line1() -> Line<'static> {
    Line::from(Span::styled("   ╭━━━━━━╮", Style::default().fg(theme::CYAN)))
}

fn logo_line2() -> Line<'static> {
    Line::from(Span::styled("   ┃ ╭━━╮ ┃", Style::default().fg(theme::CYAN)))
}

fn logo_line3() -> Line<'static> {
    Line::from(vec![
        Span::styled("   ┃ ╰━━╯ ┃  ", Style::default().fg(theme::CYAN)),
        Span::styled("E", Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD)),
        Span::styled(" ", Style::default()),
        Span::styled("E", Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD)),
        Span::styled(" ", Style::default()),
        Span::styled("K", Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD)),
    ])
}

fn logo_line4() -> Line<'static> {
    Line::from(Span::styled("   ┃ ╭━━━━╯", Style::default().fg(theme::CYAN)))
}

fn logo_line5() -> Line<'static> {
    Line::from(vec![
        Span::styled("   ┃ ┃       ", Style::default().fg(theme::CYAN)),
        Span::styled("+", Style::default().fg(theme::GREEN).add_modifier(Modifier::BOLD)),
        Span::styled(" diff ", Style::default().fg(theme::TEXT_PRIMARY)),
        Span::styled("-", Style::default().fg(theme::RED).add_modifier(Modifier::BOLD)),
    ])
}

fn logo_line6() -> Line<'static> {
    Line::from(Span::styled("   ╰━╯", Style::default().fg(theme::CYAN)))
}

// ── Tab bar ──

fn make_tab_bar(active: &HelpTab) -> Line<'static> {
    let (s_style, a_style) = match active {
        HelpTab::Settings => (
            Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            Style::default().fg(theme::TEXT_SECONDARY),
        ),
        HelpTab::About => (
            Style::default().fg(theme::TEXT_SECONDARY),
            Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ),
    };

    Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(" Settings ", s_style),
        Span::styled("  ", Style::default()),
        Span::styled(" About ", a_style),
    ])
}

// ── Edit popup ──

fn draw_edit_popup(f: &mut Frame, app: &App, edit_idx: usize) {
    let (field_name, description, is_keybinding) = resolve_edit_field(edit_idx);

    let popup = centered_rect(50, 28, f.area());
    f.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Edit: {} ", field_name))
        .style(Style::default().bg(theme::EDIT_POPUP_BG));
    f.render_widget(block, popup);

    let inner = popup.inner(Margin::new(1, 1));

    let mut lines = Vec::new();

    lines.push(Line::from(Span::styled(
        description.to_string(),
        Style::default().fg(theme::TEXT_PRIMARY),
    )));
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        "Current value:",
        Style::default().fg(theme::TEXT_SECONDARY),
    )));

    let current = app.help_input_buffer.clone();
    lines.push(Line::from(Span::styled(
        format!("  {}", current),
        Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if is_keybinding {
        lines.push(Line::from(Span::styled(
            "Press any key to set new binding",
            Style::default().fg(theme::YELLOW),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "Type to edit value",
            Style::default().fg(theme::YELLOW),
        )));
    }
    lines.push(Line::from(Span::styled(
        "Enter: confirm   Esc: cancel",
        Style::default().fg(theme::TEXT_SECONDARY),
    )));

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
            Style::default().fg(theme::TEXT_SECONDARY),
        )));
    }

    f.render_widget(Paragraph::new(lines), inner);

    let buffer_len = app.help_input_buffer.len() as u16;
    let cursor_x = inner.x + 2 + buffer_len;
    let cursor_y = inner.y + 3;
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

fn entry_line(row_idx: usize, cursor: usize, desc: &str, value: &str, value_color: ratatui::style::Color) -> Line<'static> {
    let is_cursor = row_idx == cursor;
    let marker = if is_cursor { "▸ " } else { "  " };
    Line::from(vec![
        Span::styled(marker, Style::default().fg(theme::YELLOW)),
        Span::styled(format!("{:>20} ", desc), cursor_style(is_cursor, theme::TEXT_PRIMARY)),
        Span::styled(value.to_string(), cursor_style(is_cursor, value_color)),
    ])
}

fn cursor_style(is_cursor: bool, color: ratatui::style::Color) -> Style {
    if is_cursor {
        Style::default().fg(color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(color)
    }
}

fn separator(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("  ──{}──", title),
        Style::default().fg(theme::TEXT_SECONDARY),
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
