## 1. Project Setup

- [x] 1.1 Initialize Cargo project with `cargo init` and add all dependencies to `Cargo.toml` (ratatui, crossterm, tokio, serde, toml, tui-textarea, arboard, anyhow)
- [x] 1.2 Create module directory structure: `src/{app,ui,diff,git_cmd,clipboard,config,scroll}` with `mod.rs` in each
- [x] 1.3 Verify project compiles with empty module stubs (`cargo check`)

## 2. Config System

- [x] 2.1 Implement `Config`, `Keybindings`, `DiffConfig` structs in `src/config/types.rs` with serde derive and Default impl
- [x] 2.2 Implement `Config::load()` in `src/config/mod.rs` that reads `~/.config/review-helper/config.toml`, falls back to defaults on missing/invalid file
- [x] 2.3 Write tests for config loading: valid TOML, missing file, invalid TOML, partial override

## 3. Diff Data Types

- [x] 3.1 Implement `FileDiff`, `Hunk`, `DiffLine`, `FileStatus` in `src/diff/types.rs` per design doc D3 section
- [x] 3.2 Verify types compile and `Debug`/`Clone` derive correctly

## 4. Diff Parser

- [x] 4.1 Implement `parse_diff(input: &str) -> Result<Vec<FileDiff>>` in `src/diff/parser.rs` handling: file headers, hunk headers with function names, context/add/delete lines, line numbers, binary files, renames
- [x] 4.2 Write test: parse single modified file diff
- [x] 4.3 Write test: parse multiple files with different statuses (added/modified/deleted/renamed)
- [x] 4.4 Write test: parse hunk header with function name and without
- [x] 4.5 Write test: parse binary file diff
- [x] 4.6 Write test: parse malformed diff input (error handling)

## 5. Git Command Execution

- [x] 5.1 Implement `run_git_diff_head()` in `src/git_cmd/diff.rs` executing `git diff HEAD`, returning stdout
- [x] 5.2 Implement `run_git_diff_branch(branch: &str)` computing merge-base then diff
- [x] 5.3 Implement `git_status_porcelain()` in `src/git_cmd/ops.rs` for changed file detection
- [x] 5.4 Implement `git_commit(message: &str)`, `git_push(remote: &str, branch: &str)`, `git_remote_url()` in `src/git_cmd/ops.rs`
- [x] 5.5 Write integration test for git commands using a temp git repo

## 6. TEA Framework

- [x] 6.1 Define `Message` enum in `src/app/message.rs` with variants: Key, Resize, DiffLoaded, DiffError, GitCommandDone, ClipboardCopyDone, ConfigLoaded
- [x] 6.2 Define `AppMode` enum in `src/app/state.rs` with variants: Normal, FileTreeFocus, DiffViewFocus, Editor, GitCommit, GitPush, FindBar
- [x] 6.3 Implement `App` struct in `src/app/mod.rs` holding: config, mode, diff_data, file_tree_state, diff_scroll, selected_file, and all sub-states
- [x] 6.4 Implement `update(app: &mut App, msg: Message) -> Option<Command>` in `src/app/mod.rs` routing messages by mode
- [x] 6.5 Implement terminal init/cleanup in `src/main.rs`: raw mode, alternate screen, panic hook
- [x] 6.6 Implement main event loop in `src/main.rs`: crossterm poll → Message → update → view, with tokio MPSC channel for async results
- [x] 6.7 Implement async bridge: spawn git diff on tokio, send result via channel as Message::DiffLoaded

## 7. Layout and Status Bar

- [x] 7.1 Implement `view(app: &App, frame: &mut Frame)` in `src/ui/mod.rs` dispatching to sub-renderers
- [x] 7.2 Implement left-right split layout in `src/ui/layout.rs` with configurable file tree width (default 30%) and toggle collapse
- [x] 7.3 Implement status bar in `src/ui/status_bar.rs` showing: diff stats (+N -M files:F), current diff mode, contextual key hints

## 8. File Tree UI

- [x] 8.1 Implement file tree rendering in `src/ui/file_tree.rs`: directory grouping, file status indicators (+/-/~), per-file stats
- [x] 8.2 Implement keyboard navigation: Up/Down move selection, Enter selects file and jumps diff view to it, scroll for long lists
- [x] 8.3 Implement file tree toggle: Ctrl+T collapses/expands panel, preserves selection state

## 9. Diff View UI

- [x] 9.1 Implement diff rendering in `src/ui/diff_view.rs`: hunk headers as separators, line numbers, color-coded +/- lines, context lines
- [x] 9.2 Implement vertical scrolling: j/k line-by-line, Ctrl+D/Ctrl+U half-page, Page Up/Down full-page, g/G jump first/last
- [x] 9.3 Implement virtualized rendering: only render lines within viewport for performance with large diffs
- [x] 9.4 Implement diff target switcher: Tab cycles through HEAD/main/branch, triggers async diff reload

## 10. Smart Copy

- [x] 10.1 Implement visual selection mode in diff view: `v` enters selection, Up/Down extends, Escape cancels, selected lines highlighted
- [x] 10.2 Implement Markdown format generation in `src/clipboard/smart_copy.rs`: `📄 path:lines → function()` header + fenced diff block
- [x] 10.3 Implement `y` copy: sends formatted content to system clipboard via arboard crate
- [x] 10.4 Implement `yy` single-line yank without explicit selection
- [x] 10.5 Write tests for Markdown format generation with and without function name, multi-hunk selection

## 11. Context Expansion

- [x] 11.1 Implement source file reader in `src/diff/parser.rs` or separate module: read file lines by range, handle missing files gracefully
- [x] 11.2 Implement `+`/`-` context expand/collapse in diff view: read source file, inject additional context lines, update display
- [x] 11.3 Handle boundary conditions: beginning of file, end of file, binary files

## 12. Git Operations UI

- [x] 12.1 Implement commit dialog in `src/ui/git_dialog.rs`: centered modal, text area for message, branch name, changed files summary
- [x] 12.2 Implement Ctrl+Enter commit execution: run `git commit`, refresh diff on success, show error on failure
- [x] 12.3 Implement push dialog: show unpushed commit count, confirm/cancel, execute `git push` or `git push -u`
- [x] 12.4 Implement `o` open browser: construct PR URL from remote URL + branch name, open via `open` (Mac) / `xdg-open` (Linux) / `start` (Windows)

## 13. Embedded Editor

- [x] 13.1 Integrate `tui-textarea` in `src/ui/editor.rs`: open file at current diff line, full-file editing
- [x] 13.2 Implement Ctrl+S save: write file to disk, show confirmation
- [x] 13.3 Implement Escape exit: detect unsaved changes, prompt confirmation, return to diff view with refresh
- [x] 13.4 Implement Ctrl+R find/replace within editor

## 14. Find/Replace in Diff View

- [x] 14.1 Implement find bar UI in `src/ui/find_bar.rs`: bottom bar input, `/` activates, Escape closes
- [x] 14.2 Implement search: highlight all matches in diff, scroll to first match, "No matches found" feedback
- [x] 14.3 Implement `n`/`N` navigation: jump next/previous match with wrap-around

## 15. Scroll Preservation

- [x] 15.1 Implement scroll anchor in `src/scroll/mod.rs`: track position as (file_path, hunk_header, line_offset) tuple
- [x] 15.2 Implement position restore: after diff refresh, resolve anchor to new position, scroll to it
- [x] 15.3 Handle edge case: anchor points to removed hunk → jump to nearest remaining hunk
- [x] 15.4 Write tests for anchor calculation and restoration

## 16. Integration and Polish

- [x] 16.1 End-to-end test: open tool in a git repo, browse diff, copy with smart format, commit, push, quit
- [ ] 16.2 Verify cross-platform build: `cargo build --target` for Mac/Linux/Windows
- [x] 16.3 Add `clippy` and `rustfmt` checks, fix all warnings
- [ ] 16.4 Write README with installation, usage, configuration, keybindings reference
