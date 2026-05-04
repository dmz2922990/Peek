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
            app.diff_view.cursor = 0;
            app.diff_view.expand_hunk = None;
            app.diff_view.extra_context = 0;
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
        return Command::None;
    }

    // Tab to switch to file tree
    if matches!(key.code, KeyCode::Tab) && app.file_tree.visible {
        app.mode = AppMode::FileTreeFocus;
        return Command::None;
    }

    // Use viewport height written back by the renderer
    let viewport = app.diff_view.viewport_height.max(1);

    if is_key(&key, &kb.scroll_down) || matches!(key.code, KeyCode::Down) {
        let max = app.diff_view.total_lines.saturating_sub(1);
        if app.diff_view.cursor < max {
            app.diff_view.cursor += 1;
            if app.diff_view.cursor >= app.diff_view.scroll + viewport {
                app.diff_view.scroll = app.diff_view.cursor + 1 - viewport;
            }
        }
        return Command::None;
    }
    if is_key(&key, &kb.scroll_up) || matches!(key.code, KeyCode::Up) {
        if app.diff_view.cursor > 0 {
            app.diff_view.cursor -= 1;
            if app.diff_view.cursor < app.diff_view.scroll {
                app.diff_view.scroll = app.diff_view.cursor;
            }
        }
        return Command::None;
    }

    // Half-page scroll (Ctrl+D / Ctrl+U)
    if matches!(key.code, KeyCode::PageDown) {
        let half = viewport / 2;
        let max = app.diff_view.total_lines.saturating_sub(1);
        app.diff_view.cursor = (app.diff_view.cursor + half).min(max);
        app.diff_view.scroll = app.diff_view.scroll + half;
        clamp_scroll(app, viewport);
        return Command::None;
    }
    if matches!(key.code, KeyCode::PageUp) {
        let half = viewport / 2;
        app.diff_view.cursor = app.diff_view.cursor.saturating_sub(half);
        app.diff_view.scroll = app.diff_view.scroll.saturating_sub(half);
        if app.diff_view.scroll > app.diff_view.cursor {
            app.diff_view.scroll = app.diff_view.cursor;
        }
        return Command::None;
    }

    // Jump to top/bottom
    if matches!(key.code, KeyCode::Char('g')) {
        app.diff_view.cursor = 0;
        app.diff_view.scroll = 0;
        return Command::None;
    }
    if matches!(key.code, KeyCode::Char('G')) {
        let max = app.diff_view.total_lines.saturating_sub(1);
        app.diff_view.cursor = max;
        app.diff_view.scroll = max.saturating_sub(viewport);
        return Command::None;
    }

    // Jump to next/previous hunk (n/N)
    if matches!(key.code, KeyCode::Char('n')) {
        jump_hunk(app, 1);
        return Command::None;
    }
    if matches!(key.code, KeyCode::Char('N')) {
        jump_hunk(app, -1);
        return Command::None;
    }

    // Context expand/collapse (incremental)
    if is_key(&key, &kb.context_expand) {
        if let Some(Some((hunk_idx, _))) = app.diff_view.rendered_line_map.get(app.diff_view.cursor) {
            app.diff_view.expand_hunk = Some(*hunk_idx);
        }
        app.diff_view.extra_context = app.diff_view.extra_context.saturating_add(3);
        return Command::None;
    }
    if is_key(&key, &kb.context_collapse) {
        app.diff_view.extra_context = app.diff_view.extra_context.saturating_sub(3);
        if app.diff_view.extra_context == 0 {
            app.diff_view.expand_hunk = None;
        }
        return Command::None;
    }

    // Context expand/collapse (full — Shift+= / Shift+-)
    if is_key(&key, &kb.context_expand_all) {
        if let Some(Some((hunk_idx, _))) = app.diff_view.rendered_line_map.get(app.diff_view.cursor) {
            app.diff_view.expand_hunk = Some(*hunk_idx);
            app.diff_view.extra_context = 1000;
        }
        return Command::None;
    }
    if is_key(&key, &kb.context_collapse_all) {
        app.diff_view.extra_context = 0;
        app.diff_view.expand_hunk = None;
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

    // Copy current line
    if is_key(&key, &kb.copy) {
        return copy_cursor_line(app);
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
        app.diff_view.selection_start = Some(app.diff_view.cursor);
        app.diff_view.selection_end = Some(app.diff_view.cursor);
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
            app.diff_view.cursor = 0;
            app.diff_view.expand_hunk = None;
            app.diff_view.extra_context = 0;
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

fn clamp_scroll(app: &mut App, viewport: usize) {
    let max_scroll = app.diff_view.total_lines.saturating_sub(viewport);
    if app.diff_view.scroll > max_scroll {
        app.diff_view.scroll = max_scroll;
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
    let viewport = app.diff_view.viewport_height.max(1);

    if matches!(key.code, KeyCode::Esc) {
        app.mode = AppMode::DiffViewFocus;
        app.diff_view.selection_start = None;
        app.diff_view.selection_end = None;
        return Command::None;
    }

    // Yank (copy) selection
    if is_key(&key, &kb.copy) {
        return copy_selection_range(app);
    }

    // Move down — extend selection
    if is_key(&key, &kb.scroll_down) || matches!(key.code, KeyCode::Down) {
        let max = app.diff_view.total_lines.saturating_sub(1);
        if app.diff_view.cursor < max {
            app.diff_view.cursor += 1;
            app.diff_view.selection_end = Some(app.diff_view.cursor);
            if app.diff_view.cursor >= app.diff_view.scroll + viewport {
                app.diff_view.scroll = app.diff_view.cursor + 1 - viewport;
            }
        }
        return Command::None;
    }

    // Move up — extend selection
    if is_key(&key, &kb.scroll_up) || matches!(key.code, KeyCode::Up) {
        if app.diff_view.cursor > 0 {
            app.diff_view.cursor -= 1;
            app.diff_view.selection_end = Some(app.diff_view.cursor);
            if app.diff_view.cursor < app.diff_view.scroll {
                app.diff_view.scroll = app.diff_view.cursor;
            }
        }
        return Command::None;
    }

    Command::None
}

fn copy_cursor_line(app: &mut App) -> Command {
    let cursor = app.diff_view.cursor;
    let map = &app.diff_view.rendered_line_map;

    // Only copy actual diff lines (Some(hunk_idx, Some(line_idx)))
    let Some(&Some((hunk_idx, Some(line_idx)))) = map.get(cursor) else {
        app.diff_view.status_message = Some("Nothing to copy here".into());
        return Command::None;
    };

    let file = match app.current_file().or_else(|| app.diff_data.first()) {
        Some(f) => f,
        None => return Command::None,
    };

    if let Some(hunk) = file.hunks.get(hunk_idx) {
        let content = crate::clipboard::smart_copy::format_single_line_copy(file, hunk, line_idx);
        if content.is_empty() {
            app.diff_view.status_message = Some("Nothing to copy here".into());
            return Command::None;
        }
        return Command::CopyToClipboard(content);
    }

    Command::None
}

fn copy_selection_range(app: &mut App) -> Command {
    let start = app.diff_view.selection_start;
    let end = app.diff_view.selection_end;

    let Some(start_idx) = start else { return Command::None };
    let Some(end_idx) = end else { return Command::None };

    let lo = start_idx.min(end_idx);
    let hi = start_idx.max(end_idx);
    let map = &app.diff_view.rendered_line_map;

    // Collect all (hunk_idx, line_idx) pairs in the selection (only actual diff lines)
    let mut entries: Vec<(usize, usize)> = Vec::new();
    for i in lo..=hi {
        if let Some(Some((hunk_idx, Some(line_idx)))) = map.get(i) {
            entries.push((*hunk_idx, *line_idx));
        }
    }

    if entries.is_empty() {
        app.diff_view.status_message = Some("No diff lines in selection".into());
        return Command::None;
    }

    let file = match app.current_file().or_else(|| app.diff_data.first()) {
        Some(f) => f,
        None => return Command::None,
    };

    // Determine the range of hunks and line numbers involved
    let first = entries[0];
    let last = entries[entries.len() - 1];

    let content = if first.0 == last.0 {
        // Single hunk
        let hunk = &file.hunks[first.0];
        crate::clipboard::smart_copy::format_smart_copy(
            file, &file.hunks[first.0..=first.0],
            line_number_from_idx(hunk, first.1),
            line_number_from_idx(hunk, last.1),
        )
    } else {
        // Multiple hunks — collect them
        let min_hunk = first.0;
        let max_hunk = last.0;
        let start_line = line_number_from_idx(&file.hunks[min_hunk], first.1);
        let end_line = line_number_from_idx(&file.hunks[max_hunk], last.1);
        crate::clipboard::smart_copy::format_smart_copy(
            file, &file.hunks[min_hunk..=max_hunk],
            start_line, end_line,
        )
    };

    Command::CopyToClipboard(content)
}

fn line_number_from_idx(hunk: &crate::diff::types::Hunk, line_idx: usize) -> usize {
    match hunk.lines.get(line_idx) {
        Some(crate::diff::types::DiffLine::Context { new_line, .. }) => *new_line,
        Some(crate::diff::types::DiffLine::Add { new_line, .. }) => *new_line,
        Some(crate::diff::types::DiffLine::Delete { old_line, .. }) => *old_line,
        None => 0,
    }
}

/// Jump to the next (+1) or previous (-1) hunk relative to cursor position.
fn jump_hunk(app: &mut App, direction: i32) {
    let map = &app.diff_view.rendered_line_map;
    let cursor = app.diff_view.cursor;
    let viewport = app.diff_view.viewport_height.max(1);

    // Find current hunk index
    let current_hunk: Option<usize> = map.get(cursor)
        .and_then(|opt| opt.map(|(h, _)| h));

    let target_hunk: usize = match (current_hunk, direction) {
        (Some(h), 1) => h + 1,
        (Some(h), -1) => h.saturating_sub(1),
        (None, 1) => 0,
        (None, -1) => return, // already before first hunk
        _ => return,
    };

    // Find the first diff line of the target hunk in the rendered map
    let target_pos = map.iter().enumerate().find(|(_, entry)| {
        matches!(entry, Some((h, Some(0))) if *h == target_hunk)
    }).map(|(i, _)| i);

    if let Some(pos) = target_pos {
        app.diff_view.cursor = pos;
        // Adjust scroll to keep cursor visible
        if pos >= app.diff_view.scroll + viewport {
            app.diff_view.scroll = pos + 1 - viewport;
        } else if pos < app.diff_view.scroll {
            app.diff_view.scroll = pos;
        }
    }
}

fn is_key(event: &KeyEvent, binding: &str) -> bool {
    let lowered = binding.to_lowercase();

    // Parse modifiers and key from binding string (e.g. "ctrl+j", "alt+x")
    let mut expected_ctrl = false;
    let mut expected_alt = false;
    let mut expected_char: Option<char> = None;

    let mut remaining = lowered.as_str();
    loop {
        if let Some(rest) = remaining.strip_prefix("ctrl+") {
            if rest.is_empty() {
                // Binding was just "ctrl+" — treat "+" as the key
                expected_ctrl = true;
                expected_char = Some('+');
                break;
            }
            expected_ctrl = true;
            remaining = rest;
        } else if let Some(rest) = remaining.strip_prefix("alt+") {
            if rest.is_empty() {
                expected_alt = true;
                expected_char = Some('+');
                break;
            }
            expected_alt = true;
            remaining = rest;
        } else {
            break;
        }
    }

    if expected_char.is_none() {
        if remaining == "tab" {
            return event.code == KeyCode::Tab
                && event.modifiers.contains(KeyModifiers::CONTROL) == expected_ctrl
                && event.modifiers.contains(KeyModifiers::ALT) == expected_alt;
        } else if remaining == "enter" {
            return event.code == KeyCode::Enter
                && event.modifiers.contains(KeyModifiers::CONTROL) == expected_ctrl
                && event.modifiers.contains(KeyModifiers::ALT) == expected_alt;
        } else if remaining == "esc" {
            return event.code == KeyCode::Esc
                && event.modifiers.contains(KeyModifiers::CONTROL) == expected_ctrl
                && event.modifiers.contains(KeyModifiers::ALT) == expected_alt;
        } else if remaining.len() == 1 {
            expected_char = Some(remaining.chars().next().unwrap());
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
