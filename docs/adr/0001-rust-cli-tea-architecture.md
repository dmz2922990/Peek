# ADR 0001: Rust CLI + TEA Architecture + Smart Copy Model

## Status

Accepted

## Context

We need to build a cross-platform code review tool that runs in any terminal emulator. The tool should allow developers to view diffs, edit files, perform git operations, and share diff content with external AI tools for review.

Reference implementation: Warp's built-in Code Review feature (Rust, TEA pattern, ~15k LOC).

## Decision

1. **Rust + ratatui** — Chosen for performance, cross-platform single binary, and direct architectural alignment with Warp's implementation.
2. **TEA (The Elm Architecture)** — Model → Update → View pattern for predictable state management in the TUI.
3. **Shell out to git CLI** — All git operations via `std::process::Command`. No libgit2 dependency.
4. **No in-app AI** — Instead of embedding AI, we provide "smart copy" that enriches clipboard content with file path, line numbers, and function name in Markdown format. Users paste into their preferred AI tool.
5. **Self-parse git diff stdout** — Custom parser for `git diff` output. Context expansion reads source files independently.
6. **L2 embedded editor** — Multi-line editing via `tui-textarea` crate, not a full editor.
7. **TOML configuration** — `~/.config/review-helper/config.toml` via `serde` + `toml`.

## Consequences

- **Positive**: Zero runtime dependencies (no Node.js, no Python). Binary works on any machine with git installed. TEA pattern scales well as features grow. Smart copy decouples us from any specific AI provider.
- **Negative**: Rust development speed is slower than TypeScript/Python. TEA requires disciplined message/event design. L2 editor will not satisfy users who expect vim-level editing. No in-app AI means a multi-step workflow for AI review.
- **Risk**: `tui-textarea` crate maturity. Mitigated by falling back to `$EDITOR` if needed.
