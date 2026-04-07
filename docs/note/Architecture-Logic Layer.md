# Logic Layer

Logic 层是项目的业务核心，负责数据模型定义、规范校验规则、消息组装、配置加载和 Hook 管理。本层的核心子模块（model/rules/builder）严格遵循纯函数约束，不包含副作用和 IO 操作。

## 模块结构

```text
src/logic.rs         → 模块入口，声明 5 个子模块
src/logic/model.rs   → 领域模型定义
src/logic/rules.rs   → 校验规则
src/logic/builder.rs → 消息组装
src/logic/config.rs  → 配置加载（含文件 IO）
src/logic/hook.rs    → Hook 管理（含文件系统操作）
```

## 1. model.rs — 领域模型

### CommitTagType — Commit 类型标签枚举

| 变体 | `as_str()` | `get_description()` |
|---|---|---|
| `Feat` | `"feat"` | 新功能 (feature) |
| `Fix` | `"fix"` | 修补 bug |
| `Docs` | `"docs"` | 文档改变 |
| `Style` | `"style"` | 格式（不影响代码运行的变动） |
| `Refactor` | `"refactor"` | 重构 |
| `Test` | `"tests"` | 增加测试 |
| `Chore` | `"chore"` | 构建过程或辅助工具的变动 |

**设计要点**：
- `ALL` 常量数组提供遍历能力，用于 UI 下拉列表和校验白名单
- `as_str()` 用于消息组装，`get_description()` 用于 UI 展示，职责分离
- 注意 `Test` 变体的 `as_str()` 返回 `"tests"` 而非 `"test"`

### CommitMessageEntity — 核心实体

```rust
pub struct CommitMessageEntity {
    pub commit_tag_type: CommitTagType,
    pub title: String,
    pub body: Option<String>,
    pub issue_num: Option<u32>,
}
```

这是项目的核心实体，承载一条完整 commit message 的结构化数据。`body` 和 `issue_num` 为可选字段，对应规范中正文和 Issue 关联的可选性。

### EditorMode — 编辑模式枚举

| 变体 | `display_label()` | 说明 |
|---|---|---|
| `TerminalInline` | "Terminal Inline - 在终端中逐行输入" | 终端内逐行输入 |
| `DefaultEditor` | "Default Editor - 使用系统默认编辑器" | VISUAL/EDITOR 环境变量 |
| `CustomEditor` | "Custom Editor - 使用配置文件中指定的编辑器" | 配置文件驱动 |

**设计要点**：枚举定义在 `model.rs`（领域层），但交互实现在 `ui/editor.rs`（UI 层），遵循"模型与展示分离"原则。

### CommitMsgParsed — 原始文本解析结构

```rust
pub struct CommitMsgParsed {
    pub type_prefix: Option<String>,
    pub title: String,
    pub body: Option<String>,
}
```

从 git commit message 原始文本解析而来，用于 `validate` 子命令的校验流程。`type_prefix` 为 `None` 表示缺少类型前缀。

**解析规则** (`CommitMsgParsed::parse`)：
1. 取第一行，按第一个 `:` 分割为 `type_prefix` 和 `title`
2. 找到第一个 `\n\n` 后的内容作为 `body`（过滤空值）

## 2. rules.rs — 校验规则

### 校验层级

```text
validate_raw_commit_msg(content)
  ├── 非空检查       → CommitMsgError::Empty
  ├── 类型前缀存在   → CommitMsgError::MissingType
  ├── 类型合法性     → CommitMsgError::InvalidType
  ├── validate_title(title)
  │     ├── 非空检查 → TitleError::Empty
  │     ├── 长度检查 → TitleError::TooLong (≤50 Unicode 显示宽度)
  │     └── 句号检查 → TitleError::EndsWithPeriod
  └── validate_body(body)
        └── 行宽检查 → BodyError::LineTooLong (≤72 Unicode 显示宽度/行)
```

### 常量

| 常量 | 值 | 说明 |
|---|---|---|
| `TITLE_MAX_WIDTH` | 50 | 标题行最大 Unicode 显示宽度 |
| `BODY_LINE_MAX_WIDTH` | 72 | 正文每行最大 Unicode 显示宽度 |

### 关键设计：Unicode 显示宽度

使用 `unicode-width` crate 的 `UnicodeWidthStr::width()` 计算字符串显示宽度，而非字节长度或字符数。这确保中文字符（占 2 个显示宽度）的标题行宽计算正确。

### 错误类型层次

```text
CommitMsgError (完整消息校验错误)
  ├── Empty                    → 消息为空
  ├── MissingType { line }     → 缺少 "<type>: " 前缀
  ├── InvalidType { found, valid } → 类型前缀不合法
  ├── TitleError(TitleError)   → 标题校验错误
  │     ├── Empty              → 标题为空
  │     ├── TooLong { width, max } → 标题超长
  │     └── EndsWithPeriod     → 标题以句号结尾
  └── BodyError(BodyError)     → 正文校验错误
        └── LineTooLong { line_number, width, max } → 正文行超宽
```

`CommitMsgError` 实现了 `Display` trait，提供中文错误信息，用于终端输出。

## 3. builder.rs — 消息组装

### 组装策略：Vec 片段拼接

```rust
pub fn build_message(msg: &CommitMessageEntity) -> String
```

使用 `Vec<String>` 收集行片段，最终 `join("\n")` 拼接：

```text
片段收集顺序：
1. "<type>: <title>"     ← 必有
2. "" (空行)             ← 仅当 body 或 issue_num 存在
3. "<body>"              ← 仅当 body 非空且非纯空白
4. "" (空行)             ← 仅当 body 和 issue_num 同时存在
5. "#<issue>"            ← 仅当 issue_num 存在
```

### 空值处理

- `body` 为 `None` 或纯空白字符串 → 跳过正文和其前置空行
- `body` 存在但 `issue_num` 为 `None` → 仅输出标题 + 正文
- 两者都存在 → 标题 + 空行 + 正文 + 空行 + Issue

### 与 `render_colored_preview()` 的差异

- `build_message()` → 纯文本，用于 git commit 或文件输出
- `render_colored_preview()` → 着色文本（使用 `colored` crate），用于终端预览展示

## 4. config.rs — 配置加载

### 配置结构

```rust
pub struct AppConfig {
    pub editor: EditorConfig,
}

pub struct EditorConfig {
    pub command: Option<String>,
    pub extension: String,  // 默认 "md"
}
```

### 配置文件

- 路径：`$HOME/.commit-audition/config.toml`
- 格式：TOML
- 示例：
  ```toml
  [editor]
  command = "code --wait"
  extension = "md"
  ```

### 加载逻辑

```text
load_config()
  ├── 获取 home 目录 → 失败则返回默认配置
  ├── 检查文件存在 → 不存在则返回默认配置
  ├── 读取文件内容 → 失败则返回默认配置
  └── TOML 解析 → 失败则返回默认配置
```

**设计要点**：采用"优雅降级"策略，任何环节失败都返回默认值而非报错，确保无配置文件时工具也能正常使用。

## 5. hook.rs — Hook 管理

### commit-msg Hook 脚本模板

```sh
#!/bin/sh
# commit-audition commit-msg hook
# 由 `commit-audition hook install` 自动生成

commit-audition validate "$1"
```

### install_hook() — 安装流程

```text
1. 通过 git rev-parse --git-dir 定位 .git 目录
2. 确保 .git/hooks/ 目录存在
3. 检查 commit-msg 文件是否已存在
   ├── 不存在 → 写入 hook 脚本
   └── 已存在
       ├── 包含 "commit-audition" 标识 → 提示已安装，跳过
       └── 不包含标识 → 报错，要求手动检查
4. (Unix) 设置 0o755 可执行权限
```

### uninstall_hook() — 卸载流程

```text
1. 定位 .git/hooks/ 目录
2. 检查 commit-msg 文件存在
3. 安全检查：仅删除包含 "commit-audition" 标识的 hook
   └── 不包含标识 → 报错，要求手动检查
4. 删除文件
```

### 安全设计

- **标识检查**：通过 `content.contains("commit-audition")` 判断 hook 是否由本工具生成，防止误删用户的自定义 hook
- **幂等性**：重复安装不会覆盖已有 hook，仅提示"已安装"
- **跨平台**：权限设置仅在 Unix 系统执行（`#[cfg(unix)]`），Windows 不需要

## 纯函数约束与 IO 例外

Logic 层的设计理想是"纯函数、无副作用、无 IO"，但 `config.rs` 和 `hook.rs` 存在 IO 操作：

| 子模块 | 纯度 | IO 操作 | 原因 |
|---|---|---|---|
| `model.rs` | 纯 | 无 | 纯数据定义 |
| `rules.rs` | 纯 | 无 | 纯校验函数 |
| `builder.rs` | 纯 | 无 | 纯组装函数 |
| `config.rs` | **含 IO** | `std::fs::read_to_string` | 读取配置文件，Logic 层需要配置数据驱动行为 |
| `hook.rs` | **含 IO** | `std::fs` 写入/删除/权限 + `Command::new("git")` | 管理 git hook 文件 |

**未来规划**：随着项目演进，`config.rs` 和 `hook.rs` 中的 IO 操作可迁移至 Integration 层，通过依赖倒置（在 Logic 层定义 trait，在 Integration 层实现）恢复 Logic 层的纯度。
