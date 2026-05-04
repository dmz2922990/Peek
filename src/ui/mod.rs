pub mod layout;
pub mod status_bar;
pub mod file_tree;
pub mod diff_view;
pub mod git_dialog;
pub mod find_bar;
pub mod editor;

use ratatui::{Frame, layout::{Constraint, Direction, Layout}};

use crate::app::state::{App, AppMode};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // main area
            Constraint::Length(1), // status bar
        ])
        .split(f.area());

    let main_area = chunks[0];
    let status_area = chunks[1];

    // Main area: file tree + diff view
    if app.file_tree.visible {
        let width_pct = app.config.diff.file_tree_width_percent;
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(width_pct),
                Constraint::Percentage(100 - width_pct),
            ])
            .split(main_area);

        file_tree::draw(f, app, main_chunks[0]);
        diff_view::draw(f, app, main_chunks[1]);
    } else {
        diff_view::draw(f, app, main_area);
    }

    // Status bar
    status_bar::draw(f, app, status_area);

    // Overlays / fullscreen modes
    match app.mode {
        AppMode::GitCommit => git_dialog::draw_commit(f, app),
        AppMode::GitPush => git_dialog::draw_push(f, app),
        AppMode::FindBar => find_bar::draw(f, app, status_area),
        AppMode::Editor => editor::draw(f, app),
        _ => {}
    }
}
