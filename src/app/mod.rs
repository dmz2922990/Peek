pub mod message;
pub mod state;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use message::{Command, Message};
use state::{App, AppMode};

use crate::diff::types::DiffMode;

pub fn update(app: &mut App, msg: Message) -> Command {
    match msg {
        Message::Key(key) => handle_key(app, key),
        Message::Resize(w, h) => {
            app.size = (w, h);
            Command::None
        }
        Message::DiffLoaded(files) => {
            app.diff_data = files;
            app.file_tree.selected = 0;
            app.diff_view.scroll = 0;
            app.diff_view.status_message = None;
            Command::None
        }
        Message::DiffError(e) => {
            app.diff_view.status_message = Some(format!("Error: {}", e));
            Command::None
        }
        Message::GitCommandDone(result) => {
            match result {
                Ok(msg) => {
                    app.diff_view.status_message = Some(msg);
                    // Refresh diff after git operations
                    Command::LoadDiff(app.diff_view.mode.clone())
                }
                Err(e) => {
                    app.diff_view.status_message = Some(format!("Git error: {}", e));
                    Command::None
                }
            }
        }
        Message::ClipboardCopyDone => {
            app.diff_view.status_message = Some("Copied to clipboard".into());
            app.diff_view.selection_start = None;
            app.diff_view.selection_end = None;
            if app.mode == AppMode::VisualSelect {
                app.mode = AppMode::DiffViewFocus;
            }
            Command::None
        }
        Message::ConfigLoaded(config) => {
            app.config = config;
            Command::None
        }
        Message::Tick => Command::None,
        Message::Mouse(_) => Command::None,
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> Command {
    match app.mode {
        AppMode::Normal | AppMode::DiffViewFocus => handle_diff_view_key(app, key),
        AppMode::FileTreeFocus => handle_file_tree_key(app, key),
        AppMode::GitCommit => handle_commit_key(app, key),
        AppMode::GitPush => handle_push_key(app, key),
        AppMode::FindBar => handle_find_key(app, key),
        AppMode::VisualSelect => handle_visual_select_key(app, key),
        AppMode::Editor => handle_editor_key(app, key),
    }
}

fn handle_diff_view_key(app: &mut App, key: KeyEvent) -> Command {
    let kb = &app.config.keybindings;

    // Quit
    if is_key(&key, &kb.quit) {
        app.should_quit = true;
        return Command::None;
    }

    // Toggle file tree
    if is_key(&key, &kb.toggle_file_tree) {
        app.file_tree.visible = !app.file_tree.visible;
        if app.file_tree.visible && app.mode == AppMode::DiffViewFocus {
            app.mode = AppMode::FileTreeFocus;
        }
        return Command::None;
    }

    // Scroll
    if is_key(&key, &kb.scroll_down) || matches!(key.code, KeyCode::Down) {
        app.diff_view.scroll = app.diff_view.scroll.saturating_add(1);
        return Command::None;
    }
    if is_key(&key, &kb.scroll_up) || matches!(key.code, KeyCode::Up) {
        app.diff_view.scroll = app.diff_view.scroll.saturating_sub(1);
        return Command::None;
    }

    // Half-page scroll
    if matches!(key.code, KeyCode::PageDown) {
        let half = (app.size.1 as usize) / 2;
        app.diff_view.scroll = app.diff_view.scroll.saturating_add(half);
        return Command::None;
    }
    if matches!(key.code, KeyCode::PageUp) {
        let half = (app.size.1 as usize) / 2;
        app.diff_view.scroll = app.diff_view.scroll.saturating_sub(half);
        return Command::None;
    }

    // Jump to top/bottom
    if matches!(key.code, KeyCode::Char('g')) {
        app.diff_view.scroll = 0;
        return Command::None;
    }
    if matches!(key.code, KeyCode::Char('G')) {
        app.diff_view.scroll = usize::MAX;
        return Command::None;
    }

    // Context expand/collapse
    if is_key(&key, &kb.context_expand) {
        app.diff_view.extra_context = app.diff_view.extra_context.saturating_add(3);
        return Command::None;
    }
    if is_key(&key, &kb.context_collapse) {
        app.diff_view.extra_context = app.diff_view.extra_context.saturating_sub(3);
        return Command::None;
    }

    // Diff target switch
    if is_key(&key, &kb.diff_target_switch) {
        let new_mode = match &app.diff_view.mode {
            DiffMode::Head => DiffMode::MainBranch,
            DiffMode::MainBranch => DiffMode::Head,
            DiffMode::OtherBranch(_) => DiffMode::Head,
        };
        let cmd = Command::LoadDiff(new_mode.clone());
        app.diff_view.mode = new_mode;
        return cmd;
    }

    // Commit
    if is_key(&key, &kb.commit) {
        app.mode = AppMode::GitCommit;
        app.diff_view.commit_message.clear();
        return Command::None;
    }

    // Push
    if is_key(&key, &kb.push) {
        app.mode = AppMode::GitPush;
        return Command::None;
    }

    // Open PR
    if is_key(&key, &kb.open_pr) {
        let branch = app.current_branch.clone().unwrap_or_default();
        return Command::OpenUrl(branch);
    }

    // Find
    if is_key(&key, &kb.find) {
        app.mode = AppMode::FindBar;
        app.diff_view.search_term = Some(String::new());
        return Command::None;
    }

    // Copy
    if is_key(&key, &kb.copy) {
        return Command::CopyToClipboard(String::new());
    }

    // Open editor
    if is_key(&key, &kb.open_editor) {
        if let Some(file) = app.current_file().or_else(|| app.diff_data.first()) {
            app.editing_file = Some(file.display_path().to_path_buf());
            app.mode = AppMode::Editor;
        }
        return Command::None;
    }

    // Visual select
    if is_key(&key, &kb.visual_select) {
        app.mode = AppMode::VisualSelect;
        app.diff_view.selection_start = Some(app.diff_view.scroll);
        app.diff_view.selection_end = Some(app.diff_view.scroll);
        return Command::None;
    }

    Command::None
}

fn handle_file_tree_key(app: &mut App, key: KeyEvent) -> Command {
    let kb = &app.config.keybindings;

    if is_key(&key, &kb.toggle_file_tree) {
        app.file_tree.visible = false;
        app.mode = AppMode::DiffViewFocus;
        return Command::None;
    }

    if is_key(&key, &kb.quit) {
        app.should_quit = true;
        return Command::None;
    }

    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            if app.file_tree.selected + 1 < app.diff_data.len() {
                app.file_tree.selected += 1;
            }
            Command::None
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.file_tree.selected = app.file_tree.selected.saturating_sub(1);
            Command::None
        }
        KeyCode::Enter => {
            app.mode = AppMode::DiffViewFocus;
            app.diff_view.selected_file = Some(app.file_tree.selected);
            app.diff_view.scroll = 0;
            Command::None
        }
        KeyCode::Tab => {
            app.mode = AppMode::DiffViewFocus;
            Command::None
        }
        _ => Command::None,
    }
}

fn handle_commit_key(app: &mut App, key: KeyEvent) -> Command {
    match key.code {
        KeyCode::Esc => {
            app.mode = AppMode::DiffViewFocus;
            Command::None
        }
        KeyCode::Enter => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                let msg = app.diff_view.commit_message.clone();
                if !msg.is_empty() {
                    app.mode = AppMode::DiffViewFocus;
                    return Command::RunGitCommit { message: msg };
                }
            } else {
                app.diff_view.commit_message.push('\n');
            }
            Command::None
        }
        KeyCode::Char(c) => {
            app.diff_view.commit_message.push(c);
            Command::None
        }
        KeyCode::Backspace => {
            app.diff_view.commit_message.pop();
            Command::None
        }
        _ => Command::None,
    }
}

fn handle_push_key(app: &mut App, key: KeyEvent) -> Command {
    match key.code {
        KeyCode::Esc => {
            app.mode = AppMode::DiffViewFocus;
            Command::None
        }
        KeyCode::Char('y') | KeyCode::Enter => {
            let branch = app.current_branch.clone().unwrap_or_else(|| "main".into());
            let set_upstream = true; // Always try -u for simplicity
            app.mode = AppMode::DiffViewFocus;
            Command::RunGitPush {
                remote: "origin".into(),
                branch,
                set_upstream,
            }
        }
        KeyCode::Char('n') => {
            app.mode = AppMode::DiffViewFocus;
            Command::None
        }
        _ => Command::None,
    }
}

fn handle_find_key(app: &mut App, key: KeyEvent) -> Command {
    match key.code {
        KeyCode::Esc => {
            app.diff_view.search_term = None;
            app.diff_view.search_matches.clear();
            app.mode = AppMode::DiffViewFocus;
            Command::None
        }
        KeyCode::Enter => {
            app.mode = AppMode::DiffViewFocus;
            // Search matches will be computed during rendering
            Command::None
        }
        KeyCode::Char(c) => {
            if let Some(ref mut term) = app.diff_view.search_term {
                term.push(c);
            }
            Command::None
        }
        KeyCode::Backspace => {
            if let Some(ref mut term) = app.diff_view.search_term {
                term.pop();
                if term.is_empty() {
                    app.diff_view.search_matches.clear();
                }
            }
            Command::None
        }
        _ => Command::None,
    }
}

fn handle_editor_key(app: &mut App, key: KeyEvent) -> Command {
    match key.code {
        KeyCode::Esc => {
            app.mode = AppMode::DiffViewFocus;
            app.editing_file = None;
            // Refresh diff after potential edits
            Command::LoadDiff(app.diff_view.mode.clone())
        }
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Save — for now just confirm, actual save done via external edit
            app.diff_view.status_message = Some("File saved".into());
            Command::None
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Find/replace placeholder
            app.diff_view.status_message = Some("Find/replace: not yet implemented".into());
            Command::None
        }
        _ => Command::None,
    }
}

fn handle_visual_select_key(app: &mut App, key: KeyEvent) -> Command {
    let kb = &app.config.keybindings;

    if matches!(key.code, KeyCode::Esc) {
        app.mode = AppMode::DiffViewFocus;
        app.diff_view.selection_start = None;
        app.diff_view.selection_end = None;
        return Command::None;
    }

    if is_key(&key, &kb.copy) || is_key(&key, &kb.scroll_down) {
        app.diff_view.selection_end = Some(app.diff_view.scroll);
        if is_key(&key, &kb.copy) {
            return Command::CopyToClipboard(String::new());
        }
        app.diff_view.scroll = app.diff_view.scroll.saturating_add(1);
        return Command::None;
    }

    if is_key(&key, &kb.scroll_up) {
        app.diff_view.selection_end = Some(app.diff_view.scroll);
        app.diff_view.scroll = app.diff_view.scroll.saturating_sub(1);
        return Command::None;
    }

    Command::None
}

fn is_key(event: &KeyEvent, binding: &str) -> bool {
    let lowered = binding.to_lowercase();
    let parts: Vec<&str> = lowered.split('+').collect();

    let mut expected_ctrl = false;
    let mut expected_alt = false;
    let mut expected_char: Option<char> = None;

    for part in parts {
        match part {
            "ctrl" => expected_ctrl = true,
            "alt" => expected_alt = true,
            "tab" if expected_char.is_none() => {
                return event.code == KeyCode::Tab
                    && event.modifiers.contains(KeyModifiers::CONTROL) == expected_ctrl
                    && event.modifiers.contains(KeyModifiers::ALT) == expected_alt;
            }
            c if c.len() == 1 => expected_char = Some(c.chars().next().unwrap()),
            _ => {}
        }
    }

    if let Some(ch) = expected_char {
        let char_match = matches!(event.code, KeyCode::Char(c) if c == ch);
        let ctrl_match = event.modifiers.contains(KeyModifiers::CONTROL) == expected_ctrl;
        let alt_match = event.modifiers.contains(KeyModifiers::ALT) == expected_alt;
        return char_match && ctrl_match && alt_match;
    }

    false
}
