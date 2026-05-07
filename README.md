<p align="center">
  <pre>
   ╭━━━━━━╮
   ┃ ╭━━╮ ┃
   ┃ ╰━━╯ ┃  E E K
   ┃ ╭━━━━╯
   ┃ ┃       + diff -
   ╰━╯
  </pre>
  <h1 align="center">Peek</h1>
  <p align="center">A terminal diff viewer with syntax highlighting</p>
  <p align="center">
    <a href="https://github.com/dmz2922990/Peek/releases"><img src="https://img.shields.io/github/v/tag/dmz2922990/Peek?label=version" alt="version"></a>
    <img src="https://img.shields.io/crates/l/peek" alt="license">
    <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-blue" alt="platform">
  </p>
</p>

---

**Peek** is an interactive, full-screen terminal UI for viewing `git diff` output. It parses unified diff, highlights syntax with true color support, and lets you browse, search, copy, and commit — all without leaving your terminal.

## Features

- **Syntax Highlighting** — Token-level coloring for all major languages via syntect
- **Side-by-side File Tree** — Browse changed files in a collapsible sidebar (left or right)
- **Context Folding** — Expand / collapse context lines around hunks incrementally or all at once
- **Horizontal Scrolling** — Scroll wide diff lines with `h` / `l`
- **Smart Copy** — Copy a single line or visually-select a range, with line numbers included
- **Find in Diff** — Search and jump between matches
- **Git Commit & Push** — Write a commit message and push without leaving the TUI
- **Open in Editor** — Jump to the exact line in `$EDITOR` (default: vim)
- **Open PR** — One key to open the pull request URL in your browser
- **Auto-reload** — Watches the working directory and refreshes on file changes
- **Frozen File Headers** — `---` / `+++` headers stay pinned at the top while you scroll
- **Fully Configurable** — Remap all keys and tweak settings via config file or the built-in Help panel

## Installation

### npm (Recommended)

```bash
npm install -g @dmz2922990/peek
```

Supports macOS (Apple Silicon & Intel), Linux, and Windows.

### From Source

```bash
git clone https://github.com/dmz2922990/Peek.git
cd Peek
cargo install --path .
```

### Download Binary

Download the archive for your platform from the [Releases](https://github.com/dmz2922990/Peek/releases) page, extract it, and place `peek` (or `peek.exe`) on your `PATH`.

## Quick Start

```bash
# View diff in current git repository
peek

# View diff in a specific directory
peek ~/projects/my-repo

# Print version
peek --version
```

## Keybindings

### Diff View

| Key | Action |
|---|---|
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| `h` / `←` | Scroll left |
| `l` / `→` | Scroll right |
| `PgDn` / `PgUp` | Half-page scroll |
| `g` / `G` | Jump to top / bottom |
| `n` / `N` | Next / previous hunk |
| `Enter` | Expand fold at cursor |
| `=` / `-` | Expand / collapse context incrementally |
| `+` / `_` | Expand all / collapse all context |
| `v` | Visual selection mode |
| `y` | Copy current line to clipboard |
| `/` | Find in diff |
| `e` | Open file in editor |
| `S` | Switch diff target (HEAD / main) |
| `c` | Git commit |
| `p` | Git push |
| `o` | Open pull request |
| `Tab` | Switch focus to file tree |
| `Ctrl+T` | Toggle file tree panel |
| `H` | Open help |
| `q` | Quit |

### File Tree

| Key | Action |
|---|---|
| `j` / `k` | Navigate files |
| `Enter` | Select file |
| `y` | Copy file path |
| `Tab` | Switch focus back to diff |

### Visual Select

| Key | Action |
|---|---|
| `j` / `k` | Extend selection |
| `y` | Copy selected range |
| `Esc` | Cancel |

### Git Commit

| Key | Action |
|---|---|
| `Enter` | Newline |
| `Ctrl+Enter` | Confirm commit |
| `Esc` | Cancel |

## Configuration

Config file: `~/.config/review-helper/config.toml`

### Key Remapping

```toml
[keybindings]
quit = "q"
scroll_down = "j"
scroll_up = "k"
toggle_file_tree = "ctrl+t"
open_editor = "e"
commit = "c"
push = "p"
open_pr = "o"
find = "/"
copy = "y"
visual_select = "v"
context_expand = "="
context_collapse = "-"
context_expand_all = "+"
context_collapse_all = "_"
diff_target_switch = "S"
```

### Diff Settings

```toml
[diff]
default_context_lines = 3       # Lines added per incremental expand
file_tree_width_percent = 30    # File tree panel width (%)
file_tree_position = "left"     # "left" or "right"
```

All settings can also be changed interactively — press `H` to open the Help panel, navigate to any keybinding or config value, and press `Enter` to edit inline.

## License

This project is licensed under the terms included in the [LICENSE](LICENSE) file.

## Links

- **Repository:** https://github.com/dmz2922990/Peek
- **npm Package:** https://www.npmjs.com/package/@dmz2922990/peek
- **Releases:** https://github.com/dmz2922990/Peek/releases
