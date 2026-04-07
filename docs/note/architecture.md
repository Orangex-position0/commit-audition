# 项目架构
---
version: 1.1
---

# 四层极简架构

项目采用 CLI 经典四层极简架构：

| 层 | 目录 | 职责 | 关键约束 |
|---|---|---|---|
| **CLI Layer** | `cli/` | 输入标准化：解析命令行参数、校验、默认值填充，输出 `CliConfig` | 不含业务逻辑 |
| **Logic Layer** | `logic/` | 纯逻辑：数据模型定义、规范校验规则、消息组装、配置加载、Hook 管理 | 核心模块为纯函数、无副作用、无 IO |
| **Integration Layer** | `integration/` | 副作用边界：外部依赖隔离（文件 IO、git 交互等） | 当前为空层，为后续扩展预留 |
| **UI Layer** | `ui/` | 终端交互：问答引导、编辑器模式、着色渲染、预览展示 | 仅处理展示和交互，不含业务规则 |

## 模块依赖关系

```text
main.rs
  ├── cli (parse_args → CliConfig)
  ├── ui::prompt (run_prompt → CommitMessageEntity)
  │     ├── ui::editor (input_body → Option<String>)
  │     │     └── logic::config (load_config)
  │     └── ui::render (render_colored_preview)
  └── logic::builder (build_message → String)
        └── logic::model (CommitTagType, CommitMessageEntity)
```

```text
logic::rules (validate_title / validate_body / validate_raw_commit_msg)
  └── logic::model (CommitTagType, TitleError, BodyError, CommitMsgError)

logic::hook (install_hook / uninstall_hook)
  └── 无内部依赖，直接调用 std::fs 和 git 命令

logic::config (load_config)
  └── 读取 $HOME/.commit-audition/config.toml
```

## 数据流

### 交互模式（默认）

```text
用户执行 commit-audition
       ↓
CLI Layer (parse_args → CliConfig::Interactive)
       ↓
UI Layer (run_prompt → 交互收集用户输入)
  ├── select_type()    → CommitTagType
  ├── input_title()    → String（经 rules::validate_title 校验）
  ├── input_body()     → Option<String>（经 rules::validate_body 校验）
  └── input_issue()    → Option<u32>
       ↓
组装为 CommitMessageEntity
       ↓
UI Layer (render_colored_preview → 着色预览)
       ↓
用户确认 → Logic Layer (build_message → 纯文本输出)
```

### 校验模式

```text
git commit → commit-msg hook → commit-audition validate <file>
       ↓
CLI Layer (parse_args → CliConfig::Validate { file })
       ↓
Logic Layer (validate_raw_commit_msg → Result<(), CommitMsgError>)
       ↓
退出码 0 (通过) / 1 (失败 + 错误信息)
```

## 各层公共接口一览

| 层 | 入口函数 | 输出类型 | 用途 |
|---|---|---|---|
| CLI | `parse_args()` | `CliConfig` | 解析命令行参数为统一配置 |
| Logic | `validate_title(title)` | `Result<(), TitleError>` | 校验 commit 标题 |
| Logic | `validate_body(body)` | `Result<(), BodyError>` | 校验 commit 正文行宽 |
| Logic | `validate_raw_commit_msg(content)` | `Result<(), CommitMsgError>` | 校验完整 commit message |
| Logic | `build_message(msg)` | `String` | 将实体组装为规范文本 |
| Logic | `load_config()` | `AppConfig` | 加载用户配置 |
| Logic | `install_hook()` / `uninstall_hook()` | `Result<String, String>` | 管理 git commit-msg hook |
| UI | `run_prompt()` | `Option<CommitMessageEntity>` | 交互式收集 commit message |
| UI | `render_colored_preview(msg)` | `String` | 着色预览 commit message |
| UI | `input_body()` | `Option<Option<String>>` | 正文输入（含编辑模式选择） |
