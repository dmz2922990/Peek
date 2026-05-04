use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::app::state::App;
use crate::diff::types::FileStatus;

pub fn draw(f: &mut Frame, app: &App, area: Rect, focused: bool) {
    let items: Vec<ListItem> = app.diff_data.iter().map(|file| {
        let (indicator, color) = match file.status {
            FileStatus::Added => ("+", Color::Green),
            FileStatus::Deleted => ("-", Color::Red),
            FileStatus::Modified => ("~", Color::Yellow),
            FileStatus::Renamed => ("→", Color::Cyan),
            FileStatus::Copied => ("C", Color::Magenta),
        };

        let path = file.display_path().display().to_string();

        let line = Line::from(vec![
            Span::styled(format!("{} ", indicator), Style::default().fg(color)),
            Span::styled(path, Style::default()),
            Span::styled(format!(" +{}", file.additions), Style::default().fg(Color::Green)),
            Span::styled(format!(" -{}", file.deletions), Style::default().fg(Color::Red)),
        ]);

        ListItem::new(line)
    }).collect();

    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
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
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        );

    let mut state = ListState::default();
    state.select(Some(app.file_tree.selected));

    f.render_stateful_widget(list, area, &mut state);
}
