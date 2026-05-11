use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::state::{App, HelpTab};
use crate::config::types::{CONFIG_ENTRIES, KEYBINDING_ENTRIES, REVIEW_CONFIG_ENTRIES};
use crate::ui::theme;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO_URL: &str = env!("CARGO_PKG_REPOSITORY");

const FIXED_KEYS: &[(&str, &str)] = &[
    ("g/G", "Jump to top/bottom"),
    ("n/N", "Next/previous hunk"),
    ("R", "AI code review"),
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
        HelpTab::Ai => draw_ai(f, app, content_area, viewport_height),
    }

    // Edit popup overlay
    if let Some(edit_idx) = app.help_editing {
        draw_edit_popup(f, app, edit_idx);
    }
}

fn draw_settings(f: &mut Frame, app: &mut App, area: Rect, viewport_height: usize) {
    let mut lines: Vec<Line> = Vec::new();

    for (i, (_field, desc)) in KEYBINDING_ENTRIES.iter().enumerate() {
        let binding = app.config.keybindings.get_binding(i).unwrap_or("?");
        lines.push(entry_line(i, app.help_cursor, *desc, binding, theme::CYAN, 20));
    }

    lines.push(separator(" Configuration "));
    for (i, (_field, desc)) in CONFIG_ENTRIES.iter().enumerate() {
        let row_idx = CFG_OFFSET + i;
        let value = app.config.diff.get_entry(i).unwrap_or_else(|| "?".into());
        lines.push(entry_line(row_idx, app.help_cursor, *desc, &value, theme::GREEN, 20));
    }

    lines.push(separator(" Fixed keys "));
    for (offset, (key, desc)) in FIXED_KEYS.iter().enumerate() {
        let row_idx = FIXED_OFFSET + offset;
        lines.push(entry_line(row_idx, app.help_cursor, *desc, key, theme::MAGENTA, 20));
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

const AI_CFG_COUNT: usize = REVIEW_CONFIG_ENTRIES.len();

fn draw_ai(f: &mut Frame, app: &mut App, area: Rect, viewport_height: usize) {
    // Right margin: 3 chars for left-right symmetry (left has marker 2 + space 1 = 3)
    let content_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area)[0];

    let desc_width = 14;
    let prefix_len = 2 + 1 + desc_width; // marker(2) + space(1) + desc_field(14) = 17
    let max_value_w = (content_area.width as usize).saturating_sub(prefix_len);
    let prompt_idx = REVIEW_CONFIG_ENTRIES.len() - 1;

    let mut display_lines: Vec<Line> = Vec::new();
    let mut line_entry: Vec<usize> = Vec::new();

    display_lines.push(Line::from(Span::styled(
        "  AI Review Configuration",
        Style::default().fg(theme::TEXT_PRIMARY).add_modifier(Modifier::BOLD),
    )));
    line_entry.push(usize::MAX);
    display_lines.push(Line::from(""));
    line_entry.push(usize::MAX);

    for (i, (_field, desc)) in REVIEW_CONFIG_ENTRIES.iter().enumerate() {
        let value = app.config.review.get_entry(i).unwrap_or_else(|| "?".into());

        if i == prompt_idx {
            let is_cursor = i == app.help_cursor;
            let marker = if is_cursor { "▸ " } else { "  " };
            let wrapped = wrap_text_lines(&value, max_value_w);

            display_lines.push(Line::from(vec![
                Span::styled(marker, Style::default().fg(theme::YELLOW)),
                Span::styled(format!(" {:<w$}", desc, w = desc_width), cursor_style(is_cursor, theme::TEXT_PRIMARY)),
                Span::styled(wrapped.first().unwrap_or(&"").to_string(), cursor_style(is_cursor, theme::GREEN)),
            ]));
            line_entry.push(i);

            for chunk in wrapped.iter().skip(1) {
                display_lines.push(Line::from(vec![
                    Span::styled(" ".repeat(prefix_len), Style::default()),
                    Span::styled(chunk.to_string(), cursor_style(is_cursor, theme::GREEN)),
                ]));
                line_entry.push(i);
            }
        } else {
            display_lines.push(entry_line(i, app.help_cursor, *desc, &value, theme::GREEN, desc_width));
            line_entry.push(i);
        }
    }

    // Scroll: use display line range of cursor entry
    let cursor_entry = app.help_cursor;
    let cursor_start = line_entry.iter().position(|&e| e == cursor_entry).unwrap_or(0);
    let cursor_end = line_entry.iter().rposition(|&e| e == cursor_entry).unwrap_or(cursor_start);
    let max_scroll = display_lines.len().saturating_sub(viewport_height);
    if cursor_start < app.help_scroll {
        app.help_scroll = cursor_start;
    } else if cursor_end >= app.help_scroll + viewport_height {
        app.help_scroll = cursor_end + 1 - viewport_height;
    }
    if app.help_scroll > max_scroll {
        app.help_scroll = max_scroll;
    }

    let visible: Vec<Line> = display_lines
        .into_iter()
        .enumerate()
        .skip(app.help_scroll)
        .take(viewport_height)
        .map(|(idx, mut line)| {
            if line_entry[idx] == cursor_entry {
                Line::from(std::mem::take(&mut line.spans))
                    .patch_style(Style::default().bg(theme::HELP_CURSOR_BG))
            } else {
                line
            }
        })
        .collect();

    f.render_widget(Paragraph::new(visible), content_area);
}

// ── Tab bar ──

fn make_tab_bar(active: &HelpTab) -> Line<'static> {
    let (s_style, ai_style, a_style) = match active {
        HelpTab::Settings => (
            Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            Style::default().fg(theme::TEXT_SECONDARY),
            Style::default().fg(theme::TEXT_SECONDARY),
        ),
        HelpTab::Ai => (
            Style::default().fg(theme::TEXT_SECONDARY),
            Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            Style::default().fg(theme::TEXT_SECONDARY),
        ),
        HelpTab::About => (
            Style::default().fg(theme::TEXT_SECONDARY),
            Style::default().fg(theme::TEXT_SECONDARY),
            Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ),
    };

    Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(" Settings ", s_style),
        Span::styled("  ", Style::default()),
        Span::styled(" AI ", ai_style),
        Span::styled("  ", Style::default()),
        Span::styled(" About ", a_style),
    ])
}

// ── Edit popup ──

fn draw_edit_popup(f: &mut Frame, app: &App, edit_idx: usize) {
    let (field_name, description, is_keybinding) = resolve_edit_field(app, edit_idx);
    let is_multiline = app.help_tab == HelpTab::Ai
        && edit_idx + 1 == REVIEW_CONFIG_ENTRIES.len();

    let current = app.help_input_buffer.clone();

    // Popup height: multiline uses 70% of frame, normal fits content
    let popup_h = if is_multiline {
        let max_h = f.area().height.saturating_sub(4);
        (max_h * 7 / 10).max(15).min(max_h)
    } else {
        let estimated_w = (f.area().width as usize / 2).saturating_sub(4).max(10);
        let value_lines = wrap_text_lines(&current, estimated_w);
        let mut lc = 3 + value_lines.len() + 2;
        if !is_keybinding {
            let hint = resolve_value_hint(app, edit_idx);
            if !hint.is_empty() { lc += 2; }
        }
        (lc as u16 + 2).min(f.area().height.saturating_sub(4))
    };

    // Centered popup: exact height, 50% width
    let top_pad = (f.area().height.saturating_sub(popup_h)) / 2;
    let bot_pad = f.area().height.saturating_sub(popup_h) - top_pad;
    let v_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_pad),
            Constraint::Length(popup_h),
            Constraint::Length(bot_pad),
        ])
        .split(f.area());
    let popup = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(v_layout[1])[1];

    f.render_widget(Clear, popup);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Edit: {} ", field_name))
        .style(Style::default().bg(theme::EDIT_POPUP_BG));
    f.render_widget(block, popup);

    let inner = popup.inner(Margin::new(1, 1));

    if is_multiline {
        draw_edit_multiline(f, app, inner, description, &current);
    } else {
        draw_edit_single(f, app, inner, description, is_keybinding, &current, edit_idx);
    }
}

fn draw_edit_single(
    f: &mut Frame, app: &App, inner: Rect, description: &str,
    is_keybinding: bool, current: &str, edit_idx: usize,
) {
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        description.to_string(),
        Style::default().fg(theme::TEXT_PRIMARY),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Current value:",
        Style::default().fg(theme::TEXT_SECONDARY),
    )));

    let actual_w = inner.width.saturating_sub(2) as usize;
    let wrapped = wrap_text_lines(current, actual_w);
    for chunk in &wrapped {
        lines.push(Line::from(Span::styled(
            format!("  {}", chunk),
            Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD),
        )));
    }
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        if is_keybinding { "Press any key to set new binding" } else { "Type to edit value" },
        Style::default().fg(theme::YELLOW),
    )));
    lines.push(Line::from(Span::styled(
        "Tab: confirm   Esc: cancel",
        Style::default().fg(theme::TEXT_SECONDARY),
    )));

    if !is_keybinding {
        let hint = resolve_value_hint(app, edit_idx);
        if !hint.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("Valid values: {}", hint),
                Style::default().fg(theme::TEXT_SECONDARY),
            )));
        }
    }

    f.render_widget(Paragraph::new(lines), inner);

    // Cursor
    let actual_w = inner.width.saturating_sub(2) as usize;
    let before_cursor: String = current.chars().take(app.help_input_cursor).collect();
    let cursor_wrapped = wrap_text_lines(&before_cursor, actual_w);
    let c_row = cursor_wrapped.len().saturating_sub(1);
    let c_col = display_width(cursor_wrapped.last().unwrap_or(&""));
    let cursor_x = (inner.x + 2 + c_col as u16).min(inner.x + inner.width.saturating_sub(1));
    let cursor_y = (inner.y + 3 + c_row as u16).min(inner.y + inner.height.saturating_sub(1));
    f.set_cursor_position((cursor_x, cursor_y));
}

fn draw_edit_multiline(f: &mut Frame, app: &App, inner: Rect, description: &str, current: &str) {
    // Split: header (3) | content area (fills remaining) | footer (3)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(inner);

    // Header
    f.render_widget(Paragraph::new(vec![
        Line::from(Span::styled(description.to_string(), Style::default().fg(theme::TEXT_PRIMARY))),
        Line::from(""),
        Line::from(Span::styled("Prompt:", Style::default().fg(theme::TEXT_SECONDARY))),
    ]), chunks[0]);

    // Content area with top border
    let content_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(theme::TEXT_SECONDARY))
        .style(Style::default().bg(theme::DEFAULT_BG));
    let content_inner = content_block.inner(chunks[1]);
    f.render_widget(content_block, chunks[1]);

    let actual_w = content_inner.width.saturating_sub(2) as usize;
    let wrapped = wrap_text_lines(current, actual_w);
    let mut content_lines: Vec<Line> = wrapped.iter().map(|chunk| {
        Line::from(Span::styled(
            format!("  {}", chunk),
            Style::default().fg(theme::CYAN),
        ))
    }).collect();
    // Pad to fill content area
    while content_lines.len() < content_inner.height as usize {
        content_lines.push(Line::from(""));
    }
    f.render_widget(Paragraph::new(content_lines), content_inner);

    // Footer
    f.render_widget(Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled("Enter: newline   Tab: confirm", Style::default().fg(theme::YELLOW))),
        Line::from(Span::styled("Esc: cancel", Style::default().fg(theme::TEXT_SECONDARY))),
    ]), chunks[2]);

    // Cursor
    let before_cursor: String = current.chars().take(app.help_input_cursor).collect();
    let cursor_wrapped = wrap_text_lines(&before_cursor, actual_w);
    let c_row = cursor_wrapped.len().saturating_sub(1);
    let c_col = display_width(cursor_wrapped.last().unwrap_or(&""));
    let cursor_x = (content_inner.x + 2 + c_col as u16)
        .min(content_inner.x + content_inner.width.saturating_sub(1));
    let cursor_y = (content_inner.y + c_row as u16)
        .min(content_inner.y + content_inner.height.saturating_sub(1));
    f.set_cursor_position((cursor_x, cursor_y));
}

fn resolve_edit_field(app: &App, edit_idx: usize) -> (&'static str, &'static str, bool) {
    match app.help_tab {
        HelpTab::Settings => {
            if edit_idx < KB_COUNT {
                let (field, desc) = &KEYBINDING_ENTRIES[edit_idx];
                return (*field, *desc, true);
            }
            if edit_idx >= CFG_OFFSET && edit_idx < CFG_OFFSET + CFG_COUNT {
                let cfg_idx = edit_idx - CFG_OFFSET;
                let (field, desc) = &CONFIG_ENTRIES[cfg_idx];
                return (*field, *desc, false);
            }
        }
        HelpTab::Ai => {
            if edit_idx < AI_CFG_COUNT {
                let (field, desc) = &REVIEW_CONFIG_ENTRIES[edit_idx];
                return (*field, *desc, false);
            }
        }
        HelpTab::About => {}
    }
    ("unknown", "", false)
}

fn resolve_value_hint(app: &App, edit_idx: usize) -> &'static str {
    match app.help_tab {
        HelpTab::Settings => {
            let cfg_idx = edit_idx - CFG_OFFSET;
            match cfg_idx {
                0 => "e.g. 3",
                1 => "e.g. 30",
                2 => "left or right",
                _ => "",
            }
        }
        HelpTab::Ai => {
            match edit_idx {
                0 => "e.g. sk-ant-... or env:PEEK_API_KEY",
                1 => "e.g. claude-sonnet-4-20250514",
                2 => "e.g. https://api.anthropic.com",
                3 => "min: 5000, e.g. 16384",
                4 => "e.g. 5 or full",
                5 => "e.g. en, zh, ja",
                _ => "",
            }
        }
        HelpTab::About => "",
    }
}

fn entry_line(row_idx: usize, cursor: usize, desc: &str, value: &str, value_color: ratatui::style::Color, desc_width: usize) -> Line<'static> {
    let is_cursor = row_idx == cursor;
    let marker = if is_cursor { "▸ " } else { "  " };
    Line::from(vec![
        Span::styled(marker, Style::default().fg(theme::YELLOW)),
        Span::styled(format!(" {:<w$}", desc, w = desc_width), cursor_style(is_cursor, theme::TEXT_PRIMARY)),
        Span::styled(value.to_string(), cursor_style(is_cursor, value_color)),
    ])
}

fn display_width(text: &str) -> usize {
    unicode_width::UnicodeWidthStr::width(text)
}

fn wrap_text_lines<'a>(text: &'a str, max_width: usize) -> Vec<&'a str> {
    let mut result = Vec::new();
    for segment in text.split('\n') {
        if max_width > 0 && display_width(segment) > max_width {
            result.extend(wrap_text(segment, max_width));
        } else {
            result.push(segment);
        }
    }
    if result.is_empty() { result.push(""); }
    result
}

fn wrap_text(text: &str, max_width: usize) -> Vec<&str> {
    if max_width == 0 { return vec![text]; }
    let mut result = Vec::new();
    let mut line_start = 0;
    let mut width = 0;
    for (pos, ch) in text.char_indices() {
        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + cw > max_width {
            if pos > line_start {
                result.push(&text[line_start..pos]);
            }
            line_start = pos;
            width = cw;
        } else {
            width += cw;
        }
    }
    if line_start < text.len() {
        result.push(&text[line_start..]);
    }
    if result.is_empty() { result.push(text); }
    result
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
