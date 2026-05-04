## Context

Review Helper 是一个全新 Rust CLI 项目，目标是构建一个终端内的代码审查 TUI 工具。参考实现为 Warp Code Review（Rust, TEA 模式, ~15k LOC）。技术决策已通过 grilling session 确认并记录在 `CONTEXT.md` 和 `docs/adr/0001` 中。

核心约束：
- 运行时仅依赖系统 git CLI
- 跨平台单二进制分发（Mac/Linux/Windows）
- 不内嵌 AI，通过智能复制桥接外部 AI 工具
- 不做评论系统

## Goals / Non-Goals

### Goals
- 可在任意终端模拟器中运行的独立 CLI
- 左右分栏 diff 浏览体验（文件树 + diff 视图）
- 一键智能复制带定位信息的 diff 内容给 AI 工具
- 终端内完成 commit → push → 打开浏览器创建 PR 工作流
- 终端内 L2 级别代码编辑
- 可配置快捷键

### Non-Goals
- 不做 AI 交互（应用内无 AI 对话、无自动 review 建议）
- 不做评论系统（行内评论、文件评论、通用评论）
- 不做平台 API 集成（GitHub/GitLab PR 评论导入）
- 不做 L3 完整编辑器（多 buffer、多光标、插件系统）
- 不做协作功能（实时共享、多人 review）

## Decisions

### D1: TEA (Elm Architecture) 模式

选择 Model → Update → View 单向数据流。

**替代方案：** Component 模式（类 React 组件树），每个组件管理自己的状态。
**选择理由：** Warp 验证了 TEA 在 diff + git 操作多状态交互场景下的可维护性。Rust 所有权系统天然适合集中状态 + 纯函数渲染。ratatui 社区最佳实践也倾向 TEA。

核心循环：
```
Event (crossterm terminal event)
    ↓
Message (内部消息枚举)
    ↓
update(app, message) → 修改 App state，返回 Command
    ↓
view(app) → ratatui::Frame 渲染
```

### D2: 模块结构

```
src/
├── main.rs              # 入口：终端初始化、主循环、清理
├── app/
│   ├── mod.rs           # App struct（TEA Model），集中状态
│   ├── state.rs         # AppMode、PaneState 等状态枚举
│   └── message.rs       # Message 枚举（所有可能的内部消息）
├── ui/
│   ├── mod.rs           # view() 入口，分发到各子模块渲染
│   ├── layout.rs        # 主布局：左右分栏 + 底部状态栏
│   ├── file_tree.rs     # 文件树渲染 + ListState
│   ├── diff_view.rs     # Diff 内容渲染 + ScrollState
│   ├── status_bar.rs    # 底部状态栏（统计、快捷键提示）
│   ├── git_dialog.rs    # Git 操作模态对话框
│   ├── find_bar.rs      # 搜索栏
│   └── editor.rs        # L2 编辑器（tui-textarea 封装）
├── diff/
│   ├── mod.rs
│   ├── types.rs         # FileDiff, Hunk, DiffLine, FileStatus
│   └── parser.rs        # 解析 git diff stdout → Vec<FileDiff>
├── git_cmd/
│   ├── mod.rs
│   ├── diff.rs          # 执行 git diff 命令，返回 stdout
│   └── ops.rs           # git commit / push / remote URL
├── clipboard/
│   ├── mod.rs
│   └── smart_copy.rs    # Markdown 格式化 + 系统剪贴板写入
├── config/
│   ├── mod.rs
│   └── types.rs         # Config, Keybindings, DiffConfig + TOML 解析
└── scroll/
    └── mod.rs           # ScrollPreserver：锚点跟踪 + 恢复
```

模块间依赖方向（严格单向）：
```
main → app → (ui, diff, git_cmd, clipboard, config, scroll)
ui → (diff/types, config, app/state)
diff/parser → diff/types
git_cmd → (diff/types)
clipboard → (diff/types)
```

### D3: 核心数据类型

```rust
// diff/types.rs
struct FileDiff {
    old_path: PathBuf,
    new_path: PathBuf,
    status: FileStatus,        // Added/Modified/Deleted/Renamed
    hunks: Vec<Hunk>,
    additions: usize,
    deletions: usize,
    is_binary: bool,
}

struct Hunk {
    old_start: usize, old_count: usize,
    new_start: usize, new_count: usize,
    function_name: Option<String>,
    lines: Vec<DiffLine>,
}

enum DiffLine {
    Context { content: String, old_line: usize, new_line: usize },
    Add { content: String, new_line: usize },
    Delete { content: String, old_line: usize },
}

// app/state.rs
enum AppMode {
    Normal,           // 浏览 diff
    FileTreeFocus,    // 焦点在文件树
    DiffViewFocus,    // 焦点在 diff 视图
    Editor,           // 编辑模式
    GitCommit,        // commit 对话框
    GitPush,          // push 确认
    FindBar,          // 搜索模式
}

// app/message.rs
enum Message {
    Key(KeyEvent),
    Resize(u16, u16),
    DiffLoaded(Vec<FileDiff>),
    DiffError(String),
    GitCommandDone(Result<String, String>),
    ClipboardCopyDone,
    ConfigLoaded(Config),
}
```

### D4: Git 交互策略

所有 git 操作通过 `std::process::Command` 执行。

| 操作 | 命令 |
|------|------|
| Diff (HEAD) | `git diff HEAD` |
| Diff (main) | `git merge-base main HEAD` → `git diff <base> HEAD` |
| Diff (branch) | `git fetch origin <branch>` → `git merge-base <branch> HEAD` → `git diff <base> HEAD` |
| Commit | `git commit -m "<message>"` |
| Push | `git push` (首次 `git push -u origin <branch>`) |
| Remote URL | `git remote get-url origin` |
| File content | `git show HEAD:<path>` or direct file read |
| Status | `git status --porcelain` |

### D5: 智能复制格式

用户选中 diff 行后按 `y`，生成如下格式写入剪贴板：

```markdown
📄 src/diff/parser.rs:42-48 → parse_hunks()

` ``
diff
@@ -42,6 +42,8 @@ fn parse_hunks(content: &str) -> Vec<Hunk> {
-    let lines = content.lines();
+    let lines: Vec<&str> = content.lines().collect();
+    validate_lines(&lines)?;
```

函数名来自 hunk header 的 `@@ ... @@ function_name` 部分。
```

### D6: 异步架构

使用 tokio 异步运行时，但 TUI 渲染保持同步（ratatui 要求）。

```
主线程：crossterm event poll → Message → update() → view()
tokio 任务：git diff 执行、文件 I/O、剪贴板写入
通信：tokio::sync::mpsc channel 将异步结果发送回主循环
```

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| tui-textarea crate 不够成熟或停止维护 | 接口抽象为 trait，可替换为 `$EDITOR` fallback |
| git diff 格式在不同 git 版本间有差异 | 解析器只处理核心格式，异常时降级为原始文本显示 |
| 上下文扩展需要读取大文件 | 限制读取行数上限（如 10000 行），超限截断 |
| ratatui 渲染性能（超大 diff） | 虚拟化列表，只渲染可见行。参考 Warp 的 viewported_list |
| Windows 终端兼容性 | crossterm 已处理大部分跨平台差异，CI 中测试 Windows 构建 |

## Migration Plan

N/A（全新项目，无迁移）

## Open Questions

- tui-textarea 的具体 API 和限制待技术验证（Task 阶段首个 spike）
- arboard（剪贴板 crate）在 Wayland/Wayland-terminal 下的兼容性待确认
- 上下文扩展的默认折叠行数（建议 3，与 git diff 默认一致）
