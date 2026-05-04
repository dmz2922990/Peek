## Why

开发者在 code review 时需要在终端和 AI 工具之间反复切换上下文。现有工具（VS Code、GitHub Web）要么离开终端，要么无法方便地将 diff 定位信息传递给 AI。需要一个终端内的 diff 查看器，让开发者浏览变更、就地编辑、提交推送，并通过智能复制将带定位信息的 diff 粘贴到任意 AI 工具中进行 review。

## What Changes

- 新建 Rust CLI 项目，基于 ratatui 构建 TUI 界面
- 左右分栏布局：左侧文件树 + 右侧 diff 视图
- 解析 `git diff` 输出为结构化数据，支持上下文行扩展/折叠
- 智能复制：选中 diff 内容时自动附加文件路径、行号、函数名的 Markdown 格式头
- L2 内嵌编辑器：基于 tui-textarea 的多行编辑，支持基础光标操作和搜索替换
- Git 操作：commit / push / 打开浏览器创建 PR
- 查找替换：diff 内搜索和导航
- 滚动位置保持：diff 刷新后恢复滚动位置
- 可配置快捷键：TOML 格式配置文件

## Capabilities

### New Capabilities

- `diff-engine`: Diff 数据类型定义、git diff stdout 解析器、上下文扩展（读取源文件）
- `tui-core`: TEA 架构框架（Model/Update/View）、主布局（左右分栏）、事件系统、状态栏、App 状态机
- `file-tree`: 文件树渲染、键盘导航、折叠/展开、变更状态图标、可配置快捷键开关
- `diff-view`: Diff 内容渲染（hunks/lines）、行号、+/- 标记、滚动、diff 目标切换（HEAD/main/branch）
- `smart-copy`: 选中 diff 范围、生成 Markdown 格式头（文件名+行号+函数名）、写入系统剪贴板
- `git-ops`: Commit 对话框（消息编辑+执行）、Push 确认+执行、打开浏览器创建 PR
- `editor`: L2 内嵌编辑器集成（tui-textarea）、打开文件、保存修改、返回 diff 自动刷新
- `find-replace`: 搜索栏 UI、diff 内关键词匹配、匹配项导航（上/下一个）
- `config`: TOML 配置文件解析、快捷键映射、diff 偏好设置、默认值
- `scroll-preservation`: 滚动位置锚点跟踪、diff 刷新后恢复

### Modified Capabilities

（无，全新项目）

## Impact

- **新增依赖**: ratatui, crossterm, tokio, serde, toml, tui-textarea, arboard (剪贴板), anyhow
- **构建目标**: Mac (aarch64/x86_64), Linux (x86_64), Windows (x86_64)
- **系统要求**: git CLI（运行时唯一外部依赖）
- **参考架构**: Warp Code Review（Rust, TEA, ~15k LOC）
