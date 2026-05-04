use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::app::state::App;
use crate::diff::types::FileStatus;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app.diff_data.iter().map(|file| {
        let (indicator, color) = match file.status {
            FileStatus::Added => ("+", Color::Green),
            FileStatus::Deleted => ("-", Color::Red),
            FileStatus::Modified => ("~", Color::Yellow),
            FileStatus::Renamed => ("→", Color::Cyan),
            FileStatus::Copied => ("C", Color::Magenta),
        };

        let path = file.display_path().display().to_string();
        let stats = format!("+{} -{}", file.additions, file.deletions);

        let line = Line::from(vec![
            Span::styled(format!("{} ", indicator), Style::default().fg(color)),
            Span::styled(path, Style::default()),
            Span::styled(format!(" {}", stats), Style::default().fg(Color::DarkGray)),
        ]);

        ListItem::new(line)
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::RIGHT).title("Files"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = ListState::default();
    state.select(Some(app.file_tree.selected));

    f.render_stateful_widget(list, area, &mut state);
}
