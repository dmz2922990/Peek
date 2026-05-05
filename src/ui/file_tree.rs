use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::app::state::App;
use crate::diff::types::FileStatus;
use crate::ui::theme;

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

        let line = Line::from(vec![
            Span::styled(format!("{} ", indicator), Style::default().fg(color)),
            Span::styled(path, Style::default()),
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

    f.render_stateful_widget(list, area, &mut state);
}
