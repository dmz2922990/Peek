pub mod message;
pub mod state;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use message::{Command, Message};
use state::{App, AppMode, HelpTab};

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
            app.diff_view.status_message = None;
            app.diff_view.cache_key = None;
            app.diff_view.hscroll = 0;
            app.review.cache.clear();
            app.review.visible = false;
            app.review.comments.clear();
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
        Message::FileChanged => {
            if matches!(app.mode, AppMode::GitCommit | AppMode::GitPush | AppMode::Help) {
                Command::None
            } else {
                app.review.cache.clear();
                app.review.visible = false;
                app.review.comments.clear();
                Command::LoadDiff(app.diff_view.mode.clone())
            }
        }
        Message::Tick => Command::None,
        Message::Mouse(_) => Command::None,
        Message::ReviewResult { path, comments } => {
            app.review.reviewing = false;
            app.review.status_message = None;
            app.review.cache.insert(path, comments.clone());
            app.review.comments = comments;
            app.review.selected = 0;
            app.review.scroll = 0;
            app.review.expanded = None;
            app.review.visible = true;
            app.mode = AppMode::ReviewFocus;
            Command::None
        }
        Message::ReviewError(e) => {
            app.review.reviewing = false;
            app.review.status_message = Some(format!("Error: {}", e));
            app.review.visible = true;
            Command::None
        }
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> Command {
    // Global keys: H toggles Help from any mode (except Help itself)
    if matches!(key.code, KeyCode::Char('H')) && app.mode != AppMode::Help {
        app.mode = AppMode::Help;
        app.help_cursor = 0;
        app.help_scroll = 0;
        app.help_editing = None;
        return Command::None;
    }

    match app.mode {
        AppMode::Normal | AppMode::DiffViewFocus => handle_diff_view_key(app, key),
        AppMode::FileTreeFocus => handle_file_tree_key(app, key),
        AppMode::GitCommit => handle_commit_key(app, key),
        AppMode::GitPush => handle_push_key(app, key),
        AppMode::FindBar => handle_find_key(app, key),
        AppMode::VisualSelect => handle_visual_select_key(app, key),
        AppMode::Help => handle_help_key(app, key),
        AppMode::ReviewFocus => handle_review_key(app, key),
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

    // Tab to cycle focus: DiffView → FileTree → Review → DiffView
    if matches!(key.code, KeyCode::Tab) {
        if app.file_tree.visible {
            app.mode = AppMode::FileTreeFocus;
        } else if app.review.visible || app.review.reviewing {
            app.mode = AppMode::ReviewFocus;
        }
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

    // Horizontal scroll (arrow keys and vim h/l)
    if matches!(key.code, KeyCode::Right) || matches!(key.code, KeyCode::Char('l')) {
        app.diff_view.hscroll += 4;
        return Command::None;
    }
    if matches!(key.code, KeyCode::Left) || matches!(key.code, KeyCode::Char('h')) {
        app.diff_view.hscroll = app.diff_view.hscroll.saturating_sub(4);
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

    // Context expand/collapse
    let cursor = app.diff_view.cursor;
    let folds = &app.diff_view.fold_positions;
    // Visible fold indicators near cursor
    let fold_above_key = folds.iter().rev()
        .find(|(pos, _)| *pos <= cursor)
        .map(|(_, k)| *k);
    let fold_below_key = folds.iter()
        .find(|(pos, _)| *pos > cursor)
        .map(|(_, k)| *k);

    // Hunk-based fold keys (used for collapse when no visible fold indicators)
    let current_hunk = if cursor < app.diff_view.rendered_line_map.len() {
        app.diff_view.rendered_line_map[..=cursor].iter().rev()
            .find_map(|e| e.as_ref().map(|(h, _, _)| *h))
    } else {
        None
    };
    let hunk_above_key = current_hunk.map(|h| h * 2 + 1); // up_key for gap before current hunk
    let hunk_below_key = current_hunk.map(|h| {
        // Find actual fold key after this hunk from fold_positions, or compute
        let computed = (h + 1) * 2;
        if folds.iter().any(|(_, k)| *k == computed || *k == computed + 1) {
            computed
        } else if folds.iter().any(|(_, k)| *k == usize::MAX) {
            usize::MAX
        } else {
            computed
        }
    });

    let ctx = app.config.diff.default_context_lines;

    // Context expand (only visible folds)
    if is_key(&key, &kb.context_expand) {
        if let Some(k) = fold_above_key { *app.diff_view.expanded_folds.entry(k).or_insert(0) += ctx; }
        if let Some(k) = fold_below_key { *app.diff_view.expanded_folds.entry(k).or_insert(0) += ctx; }
        return Command::None;
    }
    // Context collapse (always hunk-based — works even when fully expanded)
    if is_key(&key, &kb.context_collapse) {
        let keys: Vec<usize> = hunk_above_key.into_iter().chain(hunk_below_key.into_iter()).collect();
        for k in keys {
            if let Some(count) = app.diff_view.expanded_folds.get_mut(&k) {
                *count = count.saturating_sub(ctx);
                if *count == 0 { app.diff_view.expanded_folds.remove(&k); }
            }
        }
        return Command::None;
    }
    // Context expand all (only visible folds)
    if is_key(&key, &kb.context_expand_all) {
        if let Some(k) = fold_above_key { app.diff_view.expanded_folds.insert(k, 1000); }
        if let Some(k) = fold_below_key { app.diff_view.expanded_folds.insert(k, 1000); }
        return Command::None;
    }
    if is_key(&key, &kb.context_collapse_all) {
        app.diff_view.expanded_folds.clear();
        return Command::None;
    }

    // Enter: expand fold at cursor
    if matches!(key.code, KeyCode::Enter) {
        let cursor = app.diff_view.cursor;
        if let Some((_, target)) = app.diff_view.fold_positions.iter().find(|(pos, _)| *pos == cursor) {
            let entry = app.diff_view.expanded_folds.entry(*target).or_insert(0);
            *entry = entry.saturating_add(app.config.diff.default_context_lines);
            app.diff_view.jump_to_fold = Some(*target);
            return Command::None;
        }
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

    // Manual refresh
    if matches!(key.code, KeyCode::Char('r')) {
        app.diff_view.cache_key = None;
        return Command::LoadDiff(app.diff_view.mode.clone());
    }

    // AI Review
    if matches!(key.code, KeyCode::Char('R')) {
        return start_review(app);
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
            let line = cursor_line_number(app);
            return Command::OpenFile { path: file.display_path().to_path_buf(), line };
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

    if matches!(key.code, KeyCode::Char('R')) {
        return start_review(app);
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
            app.diff_view.expanded_folds.clear();
            // Load cached review for new file
            let path = app.current_file().map(|f| f.new_path.clone());
            if let Some(ref p) = path {
                if let Some(cached) = app.review.cache.get(p).cloned() {
                    app.review.comments = cached;
                    app.review.visible = true;
                } else {
                    app.review.comments.clear();
                    app.review.visible = false;
                }
                app.review.selected = 0;
                app.review.scroll = 0;
                app.review.expanded = None;
            }
            Command::None
        }
        KeyCode::Tab => {
            if app.review.visible || app.review.reviewing {
                app.mode = AppMode::ReviewFocus;
            } else {
                app.mode = AppMode::DiffViewFocus;
            }
            Command::None
        }
        KeyCode::Char('y') => {
            if let Some(file) = app.diff_data.get(app.file_tree.selected) {
                let path = file.display_path().display().to_string();
                app.diff_view.status_message = Some(format!("Copied: {}", path));
                return Command::CopyToClipboard(path);
            }
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

/// Compute the source file line number at or near the current diff cursor position.
fn cursor_line_number(app: &App) -> usize {
    let cursor = app.diff_view.cursor;
    let map = &app.diff_view.rendered_line_map;
    let file = match app.current_file().or_else(|| app.diff_data.first()) {
        Some(f) => f,
        None => return 1,
    };

    // Try exact cursor position
    if let Some(Some((hunk_idx, Some(line_idx), _))) = map.get(cursor) {
        return line_number_from_idx(&file.hunks[*hunk_idx], *line_idx);
    }

    // Search backward for nearest diff line
    for i in (0..cursor).rev() {
        if let Some(Some((hunk_idx, Some(line_idx), _))) = map.get(i) {
            return line_number_from_idx(&file.hunks[*hunk_idx], *line_idx);
        }
    }

    // Fallback: first line
    1
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

fn start_review(app: &mut App) -> Command {
    if app.review.reviewing {
        return Command::None;
    }
    if app.config.review.effective_api_key().is_empty() {
        app.review.status_message = Some("API key not configured".into());
        app.review.visible = true;
        return Command::None;
    }
    app.review.status_message = None;
    app.review.reviewing = true;
    let file_idx = app.file_tree.selected;
    let file = match app.diff_data.get(file_idx) {
        Some(f) => f.clone(),
        None => return Command::None,
    };
    Command::StartReview {
        file,
        repo_root: app.repo_root.clone(),
        config: app.config.review.clone(),
    }
}

fn handle_review_key(app: &mut App, key: KeyEvent) -> Command {
    let max = app.review.comments.len().saturating_sub(1);
    crate::review::log(&format!("review_key: {:?} selected={} max={}", key.code, app.review.selected, max));

    match key.code {
        KeyCode::Char('j') | KeyCode::Char('J') | KeyCode::Down => {
            if app.review.selected < max {
                app.review.selected += 1;
                let viewport = 5; // approximate
                if app.review.selected >= app.review.scroll + viewport {
                    app.review.scroll = app.review.selected + 1 - viewport;
                }
            }
            Command::None
        }
        KeyCode::Char('k') | KeyCode::Char('K') | KeyCode::Up => {
            if app.review.selected > 0 {
                app.review.selected -= 1;
                if app.review.selected < app.review.scroll {
                    app.review.scroll = app.review.selected;
                }
            }
            Command::None
        }
        KeyCode::Enter => {
            if let Some(comment) = app.review.comments.get(app.review.selected) {
                if app.review.expanded == Some(app.review.selected) {
                    app.review.expanded = None;
                } else {
                    app.review.expanded = Some(app.review.selected);
                }
                let line_no = comment.line_no;
                if line_no > 0 {
                    let found = app.diff_view.rendered_line_map.iter().enumerate()
                        .find_map(|(i, entry)| {
                            entry.as_ref().and_then(|(_, _, nl)| {
                                if *nl == Some(line_no) { Some(i) } else { None }
                            })
                        });
                    if let Some(pos) = found {
                        app.diff_view.cursor = pos;
                    }
                }
            }
            app.mode = AppMode::DiffViewFocus;
            Command::None
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            app.review.visible = false;
            app.mode = AppMode::DiffViewFocus;
            Command::None
        }
        KeyCode::Tab => {
            app.mode = AppMode::DiffViewFocus;
            Command::None
        }
        _ => Command::None,
    }
}

fn handle_help_key(app: &mut App, key: KeyEvent) -> Command {
    use crate::config::types::{CONFIG_ENTRIES, KEYBINDING_ENTRIES, REVIEW_CONFIG_ENTRIES};

    // Settings tab offsets
    let kb_count = KEYBINDING_ENTRIES.len();
    let cfg_offset = kb_count + 1;
    let cfg_count = CONFIG_ENTRIES.len();
    let fixed_offset = cfg_offset + cfg_count + 1;
    let fixed_count = 7;

    // AI tab
    let ai_count = REVIEW_CONFIG_ENTRIES.len();

    let total_entries = match app.help_tab {
        HelpTab::Settings => fixed_offset + fixed_count,
        HelpTab::About => 0,
        HelpTab::Ai => ai_count,
    };

    // ── Editing mode ──
    if let Some(edit_idx) = app.help_editing {
        if matches!(key.code, KeyCode::Esc) {
            app.help_editing = None;
            app.help_input_buffer.clear();
            app.help_input_cursor = 0;
            return Command::None;
        }

        match app.help_tab {
            HelpTab::Settings => {
                if edit_idx < kb_count {
                    let binding_str = key_event_to_binding(&key);
                    if !binding_str.is_empty() {
                        app.help_input_buffer = binding_str.clone();
                        app.config.keybindings.set_binding(edit_idx, binding_str);
                        crate::config::save(&app.config);
                        app.help_editing = None;
                        app.help_input_buffer.clear();
                    }
                } else if edit_idx >= cfg_offset && edit_idx < cfg_offset + cfg_count {
                    let cfg_idx = edit_idx - cfg_offset;
                    match key.code {
                        KeyCode::Tab => {
                            if app.config.diff.set_entry(cfg_idx, app.help_input_buffer.clone()) {
                                crate::config::save(&app.config);
                            }
                            app.help_editing = None;
                            app.help_input_buffer.clear();
                            app.help_input_cursor = 0;
                        }
                        KeyCode::Backspace => {
                            if app.help_input_cursor > 0 {
                                let pos = app.help_input_cursor - 1;
                                if let Some((bp, ch)) = app.help_input_buffer.char_indices().nth(pos) {
                                    app.help_input_buffer.drain(bp..bp + ch.len_utf8());
                                }
                                app.help_input_cursor = pos;
                            }
                        }
                        KeyCode::Delete => {
                            if let Some((bp, ch)) = app.help_input_buffer.char_indices().nth(app.help_input_cursor) {
                                app.help_input_buffer.drain(bp..bp + ch.len_utf8());
                            }
                        }
                        KeyCode::Left => { app.help_input_cursor = app.help_input_cursor.saturating_sub(1); }
                        KeyCode::Right => {
                            let len = app.help_input_buffer.chars().count();
                            if app.help_input_cursor < len { app.help_input_cursor += 1; }
                        }
                        KeyCode::Home => { app.help_input_cursor = 0; }
                        KeyCode::End => { app.help_input_cursor = app.help_input_buffer.chars().count(); }
                        KeyCode::Char(c) => {
                            let bp = app.help_input_buffer.char_indices()
                                .nth(app.help_input_cursor)
                                .map_or(app.help_input_buffer.len(), |(i, _)| i);
                            app.help_input_buffer.insert(bp, c);
                            app.help_input_cursor += 1;
                        }
                        _ => {}
                    }
                }
            }
            HelpTab::Ai => {
                if edit_idx < ai_count {
                    let is_prompt = edit_idx + 1 == ai_count;
                    match key.code {
                        KeyCode::Enter => {
                            if is_prompt {
                                let bp = app.help_input_buffer.char_indices()
                                    .nth(app.help_input_cursor)
                                    .map_or(app.help_input_buffer.len(), |(i, _)| i);
                                app.help_input_buffer.insert(bp, '\n');
                                app.help_input_cursor += 1;
                            }
                        }
                        KeyCode::Tab => {
                            if app.config.review.set_entry(edit_idx, app.help_input_buffer.clone()) {
                                crate::config::save(&app.config);
                            }
                            app.help_editing = None;
                            app.help_input_buffer.clear();
                            app.help_input_cursor = 0;
                        }
                        KeyCode::Backspace => {
                            if app.help_input_cursor > 0 {
                                let pos = app.help_input_cursor - 1;
                                if let Some((bp, ch)) = app.help_input_buffer.char_indices().nth(pos) {
                                    app.help_input_buffer.drain(bp..bp + ch.len_utf8());
                                }
                                app.help_input_cursor = pos;
                            }
                        }
                        KeyCode::Delete => {
                            if let Some((bp, ch)) = app.help_input_buffer.char_indices().nth(app.help_input_cursor) {
                                app.help_input_buffer.drain(bp..bp + ch.len_utf8());
                            }
                        }
                        KeyCode::Left => { app.help_input_cursor = app.help_input_cursor.saturating_sub(1); }
                        KeyCode::Right => {
                            let len = app.help_input_buffer.chars().count();
                            if app.help_input_cursor < len { app.help_input_cursor += 1; }
                        }
                        KeyCode::Home => { app.help_input_cursor = 0; }
                        KeyCode::End => { app.help_input_cursor = app.help_input_buffer.chars().count(); }
                        KeyCode::Char(c) => {
                            let bp = app.help_input_buffer.char_indices()
                                .nth(app.help_input_cursor)
                                .map_or(app.help_input_buffer.len(), |(i, _)| i);
                            app.help_input_buffer.insert(bp, c);
                            app.help_input_cursor += 1;
                        }
                        _ => {}
                    }
                }
            }
            HelpTab::About => {}
        }
        return Command::None;
    }

    // ── Navigation mode ──
    match key.code {
        KeyCode::Esc => {
            app.mode = AppMode::DiffViewFocus;
            app.help_editing = None;
            app.help_input_buffer.clear();
            app.help_input_cursor = 0;
            Command::None
        }
        KeyCode::Tab => {
            app.help_tab = match app.help_tab {
                HelpTab::Settings => HelpTab::Ai,
                HelpTab::Ai => HelpTab::About,
                HelpTab::About => HelpTab::Settings,
            };
            app.help_cursor = 0;
            app.help_scroll = 0;
            app.help_editing = None;
            Command::None
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if app.help_cursor + 1 < total_entries {
                app.help_cursor += 1;
            }
            Command::None
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.help_cursor > 0 {
                app.help_cursor -= 1;
            }
            Command::None
        }
        KeyCode::Enter => {
            let cursor = app.help_cursor;
            match app.help_tab {
                HelpTab::Settings => {
                    if cursor < kb_count {
                        app.help_input_buffer = app.config.keybindings.get_binding(cursor)
                            .unwrap_or("").to_string();
                        app.help_editing = Some(cursor);
                    } else if cursor >= cfg_offset && cursor < cfg_offset + cfg_count {
                        let cfg_idx = cursor - cfg_offset;
                        app.help_input_buffer = app.config.diff.get_entry(cfg_idx)
                            .unwrap_or_default();
                        app.help_editing = Some(cursor);
                        app.help_input_cursor = app.help_input_buffer.chars().count();
                    }
                }
                HelpTab::Ai => {
                    if cursor < ai_count {
                        app.help_input_buffer = app.config.review.get_entry(cursor)
                            .unwrap_or_default();
                        app.help_editing = Some(cursor);
                        app.help_input_cursor = app.help_input_buffer.chars().count();
                    }
                }
                HelpTab::About => {}
            }
            Command::None
        }
        _ => Command::None,
    }
}

/// Convert a KeyEvent to a binding string like "ctrl+j", "alt+x", "q"
fn key_event_to_binding(key: &KeyEvent) -> String {
    let mut parts = Vec::new();
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("ctrl");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        parts.push("alt");
    }

    let key_part = match key.code {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Tab => "tab".into(),
        KeyCode::Enter => "enter".into(),
        KeyCode::Esc => "esc".into(),
        KeyCode::Backspace => "backspace".into(),
        KeyCode::Up => "up".into(),
        KeyCode::Down => "down".into(),
        KeyCode::Left => "left".into(),
        KeyCode::Right => "right".into(),
        KeyCode::PageUp => "pageup".into(),
        KeyCode::PageDown => "pagedown".into(),
        KeyCode::Home => "home".into(),
        KeyCode::End => "end".into(),
        _ => return String::new(), // unsupported key
    };

    parts.push(&key_part);
    parts.join("+")
}

fn copy_cursor_line(app: &mut App) -> Command {
    let cursor = app.diff_view.cursor;
    let map = &app.diff_view.rendered_line_map;

    // Only copy actual diff lines (Some(hunk_idx, Some(line_idx)))
    let Some(&Some((hunk_idx, Some(line_idx), _))) = map.get(cursor) else {
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
        if let Some(Some((hunk_idx, Some(line_idx), _))) = map.get(i) {
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
        .and_then(|opt| opt.map(|(h, _, _)| h));

    let target_hunk: usize = match (current_hunk, direction) {
        (Some(h), 1) => h + 1,
        (Some(h), -1) => h.saturating_sub(1),
        (None, 1) => 0,
        (None, -1) => return, // already before first hunk
        _ => return,
    };

    // Find the first diff line of the target hunk in the rendered map
    let target_pos = map.iter().enumerate().find(|(_, entry)| {
        matches!(entry, Some((h, Some(0), _)) if *h == target_hunk)
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
