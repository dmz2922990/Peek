# Review Helper - Context

## Glossary

### Review Helper
A cross-platform CLI tool that provides an interactive TUI for code review within any terminal emulator.

### Terminal Tool
Any terminal emulator (iTerm2, Warp, Terminal.app, Windows Terminal, Alacritty, etc.) — the tool runs as a standalone binary inside them.

### Smart Copy
Clipboard copy enriched with file path, line numbers, and function name in Markdown format, for pasting into external AI tools.

### Context Expansion
Diff view feature that allows expanding/collapsing surrounding lines around changes by reading source file content independently from diff output.

## Resolved Decisions

- **Delivery Model**: Standalone CLI tool. Runs in any terminal emulator.
- **Tech Stack**: Rust + ratatui. Cross-platform single binary distribution.
- **TUI Architecture**: TEA (The Elm Architecture) — Model → Update → View. Single-directional data flow, centralized state.
- **Async Runtime**: tokio.
- **Layout**: Left-right split — file tree on the left, diff view on the right. Git operations via modal dialogs.
  - Diff View supports context line expansion (collapse/expand surrounding lines).
  - File tree supports configurable keyboard shortcuts for toggle open/close.
- **Git Integration**: Shell out to system git CLI. Commit + push via git CLI. PR creation = open browser via `git remote get-url`.
- **Diff Parsing**: Self-parse `git diff` stdout. Context expansion reads source file content independently.
- **Function Name Detection**: From git diff hunk header (`@@ ... @@ function_name`).
- **Smart Copy**: Markdown block format (`📄 file:lines → function()` + fenced diff code block). No in-app AI interaction. Users paste into external AI tools.
- **File Editing**: L2 embedded editor — multi-line editing with cursor, insert, delete, copy/paste, search/replace. Built on `tui-textarea`. Syntax highlighting as later enhancement.
- **Configuration**: TOML format at `~/.config/review-helper/config.toml`. Covers keybindings and diff preferences. Parsed via `serde` + `toml`.
- **Distribution**: crates.io (`cargo install`) + GitHub Release cross-platform binaries (Mac/Linux/Windows). CI via GitHub Actions.

## Out of Scope

- AI interaction within the app (replaced by smart copy)
- Inline comment system
- GitHub/GitLab PR API integration
- Platform-specific features (PR comment import, etc.)
