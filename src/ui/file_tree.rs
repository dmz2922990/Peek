use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
};

use crate::app::state::App;
use crate::diff::types::FileStatus;
use crate::ui::theme;

/// Compact a path for display: show filename, then abbreviated parent dirs.
/// e.g. "qca/src/qca-wam/src/ap/wam_ubus.c" -> "wam_ubus.c(q/s/q/s/a)"
fn compact_path(path: &str) -> String {
    let path_obj = std::path::Path::new(path);
    let filename = path_obj.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
    let parent = path_obj.parent();
    match parent {
        None => filename,
        Some(p) if p.as_os_str().is_empty() => filename,
        Some(p) => {
            let short: String = p.components()
                .filter_map(|c| {
                    let s = c.as_os_str().to_string_lossy();
                    s.chars().next()
                })
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join("/");
            format!("{}({})", filename, short)
        }
    }
}

pub fn draw(f: &mut Frame, app: &App, area: Rect, focused: bool) {
    let items: Vec<ListItem> = app.diff_data.iter().map(|file| {
        let (indicator, color) = match file.status {
            FileStatus::Added => ("+", theme::GREEN),
            FileStatus::Deleted => ("-", theme::RED),
            FileStatus::Modified => ("~", theme::YELLOW),
            FileStatus::Renamed => ("→", theme::CYAN),
            FileStatus::Copied => ("C", theme::MAGENTA),
        };

        let path = file.display_path().display().to_string();
        let display = compact_path(&path);

        let line = Line::from(vec![
            Span::styled(format!("{} ", indicator), Style::default().fg(color)),
            Span::styled(display, Style::default()),
            Span::styled(format!(" +{}", file.additions), Style::default().fg(theme::GREEN)),
            Span::styled(format!(" -{}", file.deletions), Style::default().fg(theme::RED)),
        ]);

        ListItem::new(line)
    }).collect();

    let border_style = if focused {
        Style::default().fg(theme::CYAN)
    } else {
        Style::default().fg(theme::UNFOCUSED_BORDER)
    };

    let title = if focused {
        " Files ▸ "
    } else {
        " Files "
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title).border_style(border_style))
        .highlight_style(
            Style::default()
                .bg(theme::SELECTION_BG)
                .add_modifier(Modifier::BOLD)
        );

    let mut state = ListState::default();
    state.select(Some(app.file_tree.selected));

    f.render_widget(Clear, area);
    f.render_stateful_widget(list, area, &mut state);
}
