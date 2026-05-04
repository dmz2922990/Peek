pub mod layout;
pub mod status_bar;
pub mod file_tree;
pub mod diff_view;
pub mod git_dialog;
pub mod find_bar;
pub mod help;

use ratatui::{Frame, layout::{Constraint, Direction, Layout}};

use crate::app::state::{App, AppMode};

pub fn draw(f: &mut Frame, app: &mut App) {
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
        let tree_on_right = app.config.diff.file_tree_position == "right";

        let (tree_pct, diff_pct) = if tree_on_right {
            (100u16.saturating_sub(width_pct), width_pct)
        } else {
            (width_pct, 100u16.saturating_sub(width_pct))
        };

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(tree_pct),
                Constraint::Percentage(diff_pct),
            ])
            .split(main_area);

        let file_focused = matches!(app.mode, AppMode::FileTreeFocus);
        let diff_focused = matches!(app.mode, AppMode::DiffViewFocus | AppMode::VisualSelect);

        let (tree_area, diff_area) = if tree_on_right {
            (main_chunks[1], main_chunks[0])
        } else {
            (main_chunks[0], main_chunks[1])
        };

        file_tree::draw(f, app, tree_area, file_focused);
        diff_view::draw(f, app, diff_area, diff_focused);
    } else {
        diff_view::draw(f, app, main_area, true);
    }

    // Status bar
    status_bar::draw(f, app, status_area);

    // Overlays / fullscreen modes
    match app.mode {
        AppMode::GitCommit => git_dialog::draw_commit(f, app),
        AppMode::GitPush => git_dialog::draw_push(f, app),
        AppMode::FindBar => find_bar::draw(f, app, status_area),
        AppMode::Help => help::draw(f, app),
        _ => {}
    }
}
