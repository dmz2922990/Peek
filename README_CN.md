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
  <p align="center">带语法高亮的终端 Diff 查看器</p>
  <p align="center">
    <a href="https://github.com/dmz2922990/Peek/releases"><img src="https://img.shields.io/github/v/tag/dmz2922990/Peek?label=version" alt="version"></a>
    <img src="https://img.shields.io/crates/l/peek" alt="license">
    <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-blue" alt="platform">
  </p>
</p>

---

**Peek** 是一个交互式全屏终端 `git diff` 查看器。它解析 unified diff 输出，使用真彩色进行语法高亮，让你在终端内浏览、搜索、复制和提交代码变更。

## 功能特性

- **语法高亮** — 基于 syntect 的词法级着色，支持所有主流编程语言
- **侧边文件树** — 在可折叠的侧栏中浏览变更文件（支持左侧或右侧显示）
- **上下文折叠** — 增量式展开 / 折叠 hunk 之间的上下文行，或一键全部展开 / 折叠
- **水平滚动** — 使用 `h` / `l` 滚动查看超长 diff 行
- **智能复制** — 复制单行或可视化选择多行，自动包含行号
- **Diff 内搜索** — 搜索并跳转到匹配位置
- **Git 提交与推送** — 无需离开 TUI 即可编写提交信息并推送
- **打开编辑器** — 一键跳转到 `$EDITOR`（默认 vim）中的对应行
- **打开 PR** — 一键在浏览器中打开当前分支的 Pull Request 链接
- **自动刷新** — 监听工作目录文件变化，自动重新加载 diff
- **冻结文件头** — `---` / `+++` 文件头固定在顶部，内容区域独立滚动
- **完全可配置** — 通过配置文件或内置 Help 面板重映射所有按键和调整设置

## 安装

### npm（推荐）

```bash
npm install -g @dmz2922990/peek
```

支持 macOS（Apple Silicon 和 Intel）、Linux 和 Windows。

### 从源码构建

```bash
git clone https://github.com/dmz2922990/Peek.git
cd Peek
cargo install --path .
```

### 下载二进制文件

从 [Releases](https://github.com/dmz2922990/Peek/releases) 页面下载对应平台的压缩包，解压后将 `peek`（或 `peek.exe`）放到 `PATH` 中。

## 快速开始

```bash
# 在当前 git 仓库中查看 diff
peek

# 在指定目录中查看 diff
peek ~/projects/my-repo

# 查看版本号
peek --version
```

## 快捷键

### Diff 视图

| 按键 | 功能 |
|---|---|
| `j` / `↓` | 向下滚动 |
| `k` / `↑` | 向上滚动 |
| `h` / `←` | 向左滚动 |
| `l` / `→` | 向右滚动 |
| `PgDn` / `PgUp` | 半页滚动 |
| `g` / `G` | 跳到顶部 / 底部 |
| `n` / `N` | 下一个 / 上一个 hunk |
| `Enter` | 展开光标处的折叠 |
| `=` / `-` | 增量展开 / 折叠上下文 |
| `+` / `_` | 全部展开 / 全部折叠上下文 |
| `v` | 可视选择模式 |
| `y` | 复制当前行到剪贴板 |
| `/` | 在 diff 中搜索 |
| `e` | 在编辑器中打开文件 |
| `S` | 切换 diff 目标（HEAD / main） |
| `c` | Git 提交 |
| `p` | Git 推送 |
| `o` | 打开 Pull Request |
| `Tab` | 切换焦点到文件树 |
| `Ctrl+T` | 切换文件树面板显隐 |
| `H` | 打开帮助 |
| `q` | 退出 |

### 文件树

| 按键 | 功能 |
|---|---|
| `j` / `k` | 导航文件 |
| `Enter` | 选择文件 |
| `y` | 复制文件路径 |
| `Tab` | 焦点切回 diff 视图 |

### 可视选择

| 按键 | 功能 |
|---|---|
| `j` / `k` | 扩展选择范围 |
| `y` | 复制选中范围 |
| `Esc` | 取消 |

### Git 提交

| 按键 | 功能 |
|---|---|
| `Enter` | 换行 |
| `Ctrl+Enter` | 确认提交 |
| `Esc` | 取消 |

## 配置

配置文件路径：`~/.config/peek/config.toml`

### 按键重映射

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

### Diff 设置

```toml
[diff]
default_context_lines = 3       # 每次增量展开的行数
file_tree_width_percent = 30    # 文件树面板宽度（百分比）
file_tree_position = "left"     # "left"（左侧）或 "right"（右侧）
```

所有设置也支持在 TUI 内交互修改 — 按 `H` 打开帮助面板，导航到任意按键绑定或配置项，按 `Enter` 即可内联编辑。

## 许可证

本项目遵循 [LICENSE](LICENSE) 文件中的许可证条款。

## 链接

- **代码仓库：** https://github.com/dmz2922990/Peek
- **npm 包：** https://www.npmjs.com/package/@dmz2922990/peek
- **发布页面：** https://github.com/dmz2922990/Peek/releases
