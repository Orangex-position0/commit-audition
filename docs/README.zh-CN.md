# commit-audition

交互式 Git Commit Message 生成与校验工具，让每一次提交都符合 [Conventional Commits](https://www.conventionalcommits.org/) 规范。

[English Version](../README.md)

## 演示

![vim mode 演示](assets/demo-vim-mode.gif)

![普通模式演示](assets/demo-prompt-mode.gif)

## 特性

- **交互式生成** — 通过引导式问答，逐步收集 type / title / body / issue，零记忆成本
- **实时校验** — 标题行宽 ≤50、正文行宽 ≤72、禁止句号结尾，输入即校验
- **三种编辑模式** — 终端内联输入、系统默认编辑器、自定义编辑器（VS Code / Vim 等）
- **Vim Mode TUI** — 基于 ratatui 的全键盘驱动 TUI 界面（lazygit 风格）
- **Git Hook 集成** — 一键安装 `commit-msg` hook，在 `git commit` 时自动拦截不合规消息
- **CJK 友好** — 基于 Unicode 显示宽度计算，中文标题/正文校验准确
- **着色预览** — 提交前预览带颜色的 commit message，确认无误再输出

## 安装

### Cargo

```bash
cargo install commit-audition
```

### 一键安装

```bash
# macOS / Linux
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Orangex-position0/commit-audition/releases/latest/download/commit-audition-installer.sh | sh

# Windows PowerShell
powershell -c "irm https://github.com/Orangex-position0/commit-audition/releases/latest/download/commit-audition-installer.ps1 | iex"
```

### 从源码构建

```bash
git clone https://github.com/Orangex-position0/commit-audition.git
cd commit-audition
cargo build --release
```

编译产物位于 `target/release/commit-audition`，可将其加入 `PATH` 或复制到 `/usr/local/bin`。

## 使用

### 交互式生成 Commit Message

```bash
commit-audition
```

工具将引导你完成以下步骤：

1. 选择 commit 类型（feat / fix / docs / style / refactor / tests / chore）
2. 输入标题（命令式语气，首字母大写，≤50 字符）
3. 选择是否添加正文 → 选择编辑方式 → 输入正文
4. 选择是否关联 Issue
5. 预览确认 → 输出纯文本

### 校验已有 Commit Message

```bash
# 校验文件
commit-audition validate .git/COMMIT_EDITMSG

# 从 stdin 读取（CI 场景）
echo "feat: Add feature" | commit-audition validate -
```

### 安装 Git Hook

```bash
# 安装 commit-msg hook
commit-audition hook install

# 卸载
commit-audition hook uninstall
```

安装后，每次 `git commit` 都会自动校验 commit message，不合规将被拦截。

## 配置

配置文件：`~/.commit-audition/config.toml`

```toml
# 启用 vim mode TUI 界面
vim_mode = true

[editor]
command = "code --wait"    # 自定义编辑器命令
extension = "md"            # 临时文件扩展名
```

详见 [配置文档](Configuration.md)。

## 推荐别名

`commit-audition` 名字较长，推荐设置别名简化输入：

| Shell | 配置文件 | 命令 |
|---|---|---|
| bash | `~/.bashrc` | `alias cmt='commit-audition'` |
| zsh | `~/.zshrc` | `alias cmt='commit-audition'` |
| fish | `~/.config/fish/config.fish` | `alias cmt commit-audition` |
| PowerShell | `$PROFILE` | `Set-Alias -Name cmt -Value commit-audition` |

设置后即可使用 `cmt` 代替 `commit-audition`：

```bash
cmt          # 启动交互式生成
cmt validate # 校验模式
```

## Vim Mode 截图

| 类型选择 | 标题输入 |
|---|---|
| ![类型选择](assets/screenshots/step-select-type.png) | ![标题输入](assets/screenshots/step-input-title.png) |

| 正文编辑 | 预览确认 |
|---|---|
| ![正文编辑](assets/screenshots/step-select-body.png) | ![预览确认](assets/screenshots/step-preview.png) |

## Commit Message 格式

```text
<type>: <title>           ← 标题行（≤50 字符显示宽度）
                          ← 空行
<body>                    ← 正文（每行 ≤72 字符显示宽度，可选）
                          ← 空行
#<issue>                  ← Issue 关联（可选）
```

### 合法类型

| 类型 | 说明 |
|---|---|
| `feat` | 新功能 |
| `fix` | 修补 bug |
| `docs` | 文档改变 |
| `style` | 格式（不影响代码运行的变动） |
| `refactor` | 重构 |
| `tests` | 增加测试 |
| `chore` | 构建过程或辅助工具的变动 |

## 项目架构

采用 CLI 经典四层极简架构：

| 层 | 目录 | 职责 |
|---|---|---|
| **CLI Layer** | `cli/` | 命令行参数解析与标准化 |
| **Logic Layer** | `logic/` | 纯业务逻辑：模型、校验、组装 |
| **Integration Layer** | `integration/` | 副作用边界（预留） |
| **UI Layer** | `ui/` | 终端交互：问答、编辑器、渲染 |

详见 [架构文档](architecture.md)。

## 文档

| 文档 | 说明 |
|---|---|
| [架构总览](architecture.md) | 四层架构设计与数据流 |
| [Vim Mode 设计](vim-mode.md) | Vim mode TUI 详细设计文档 |
| [配置说明](Configuration.md) | 配置文件格式与使用 |
| [构建与发布指南](../release-guide.md) | dist + GitHub Actions 自动发布流程 |

## 许可证

MIT OR Apache-2.0
