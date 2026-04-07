# CLI Layer

CLI 层负责命令行参数的解析和标准化，将用户输入转换为统一的 `CliConfig` 枚举输出，供下游模块分发处理。

## 模块结构

```text
src/cli.rs          → 模块入口，暴露 parse_args()
src/cli/args.rs     → 参数定义（CliArgs/Commands/HookAction）和统一输出（CliConfig）
```

## 核心类型

### CliArgs — 命令行参数结构体

使用 `clap::Parser` derive 宏定义，是 CLI 层的原始输入：

| 字段 | 类型 | 说明 |
|---|---|---|
| `command` | `Option<Commands>` | 子命令（可选，不传则进入交互模式） |
| `template` | `bool` | `--template` 标志，跳过交互直接输出模板（预留功能） |

### Commands — 子命令枚举

| 变体 | 参数 | 说明 |
|---|---|---|
| `Validate { file: String }` | 文件路径或 `"-"` (stdin) | 校验 commit message 文件 |
| `Hook { action: HookAction }` | Hook 子操作 | 管理 git hooks |

### HookAction — Hook 操作枚举

| 变体 | 说明 |
|---|---|
| `Install` | 安装 commit-msg hook 到当前仓库 |
| `Uninstall` | 卸载当前仓库的 commit-msg hook |

### CliConfig — 统一输出枚举

`CliConfig` 是 CLI 层的标准化输出，由 `From<CliArgs>` 自动转换：

| 变体 | 字段 | 对应场景 |
|---|---|---|
| `Interactive { template: bool }` | 无子命令时 | 启动交互式问答 |
| `Validate { file: String }` | `validate` 子命令 | 校验 commit message 文件 |
| `Hook(HookAction)` | `hook` 子命令 | Hook 安装/卸载 |

**转换逻辑** (`From<CliArgs> for CliConfig`)：
- `command = None` → `Interactive { template }`
- `command = Some(Validate { file })` → `Validate { file }`
- `command = Some(Hook { action })` → `Hook(action)`

## 入口函数

```rust
pub fn parse_args() -> CliConfig
```

调用 `CliArgs::parse()` 解析命令行参数，通过 `Into<CliConfig>` 转换为统一配置。

## 使用示例

```bash
# 默认模式：启动交互式问答
commit-audition

# 校验模式：校验 commit message 文件
commit-audition validate .git/COMMIT_EDITMSG

# 校验模式：从 stdin 读取（用于管道/CI 场景）
echo "feat: Add feature" | commit-audition validate -

# Hook 管理：安装 commit-msg hook
commit-audition hook install

# Hook 管理：卸载 commit-msg hook
commit-audition hook uninstall
```

## 设计约束

- CLI 层**不包含业务逻辑**，仅负责参数解析和标准化
- 所有业务校验逻辑由 Logic 层的 `rules.rs` 承担
- 终端交互由 UI 层的 `prompt.rs` 承担
