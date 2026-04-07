# commit-audition

交互式 Git Commit Message 生成与校验工具，让每一次提交都符合 [Conventional Commits](https://www.conventionalcommits.org/) 规范。

[English Version](README.en.md)

## 特性

- **交互式生成** — 通过引导式问答，逐步收集 type / title / body / issue，零记忆成本
- **实时校验** — 标题行宽 ≤50、正文行宽 ≤72、禁止句号结尾，输入即校验
- **三种编辑模式** — 终端内联输入、系统默认编辑器、自定义编辑器（VS Code / Vim 等）
- **Git Hook 集成** — 一键安装 `commit-msg` hook，在 `git commit` 时自动拦截不合规消息
- **CJK 友好** — 基于 Unicode 显示宽度计算，中文标题/正文校验准确
- **着色预览** — 提交前预览带颜色的 commit message，确认无误再输出

## 安装

```bash
git clone https://github.com/<your-username>/commit-audition.git
cd commit-audition
cargo build --release
```

编译产物位于 `target/release/commit-audition`，可将其加入 `PATH` 或复制到 `/usr/local/bin`。

## 快速开始

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
[editor]
command = "code --wait"    # 自定义编辑器命令
extension = "md"            # 临时文件扩展名
```

详见 [配置文档](docs/note/Configuration.md)。

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

详见 [架构文档](docs/note/architecture.md)。

## 文档

| 文档 | 说明 |
|---|---|
| [架构总览](docs/note/architecture.md) | 四层架构设计与数据流 |
| [CLI Layer](docs/note/Architecture-CLI%20Layer.md) | 命令行参数定义 |
| [Logic Layer](docs/note/Architecture-Logic%20Layer.md) | 业务逻辑与校验规则 |
| [UI Layer](docs/note/Architecture-UI%20Layer.md) | 终端交互设计 |
| [Integration Layer](docs/note/Architecture-Integration%20Layer.md) | 副作用边界设计 |
| [配置说明](docs/note/Configuration.md) | 配置文件格式与使用 |
| [依赖说明](docs/note/dependencies.md) | 依赖选型理由 |
| [校验策略](docs/note/Commit%20Message%20校验策略.md) | Commit message 校验方案 |
| [使用指南](docs/Usage.md) | 详细使用文档 |

## 许可证

MIT
