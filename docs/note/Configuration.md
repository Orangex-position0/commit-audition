# Configuration Module

本文档描述 `commit-audition` 的配置体系，包括配置文件路径、TOML 结构、字段说明和加载逻辑。

## 配置文件

- **路径**：`$HOME/.commit-audition/config.toml`
- **格式**：TOML
- **可选**：配置文件不存在时使用默认值，工具可正常运行

## 配置结构

```toml
[editor]
command = "code --wait"    # 编辑器命令（可选）
extension = "md"            # 临时文件扩展名（默认 "md"）
```

## 字段说明

### [editor] 段 — 编辑器配置

| 字段 | 类型 | 默认值 | 必填 | 说明 |
|---|---|---|---|---|
| `command` | `String` | `None` | 否 | 自定义编辑器命令，仅在 Custom Editor 模式下使用 |
| `extension` | `String` | `"md"` | 否 | Custom Editor 模式下临时文件的扩展名 |

### command 字段

`command` 支持带参数的编辑器命令，空格分隔：
- `"code --wait"` → 使用 VS Code 打开并等待关闭
- `"vim"` → 使用 Vim 打开
- `"nano"` → 使用 nano 打开
- `"subl --wait"` → 使用 Sublime Text 打开并等待

该字段仅在用户选择 **Custom Editor** 编辑模式时被使用。选择 Default Editor 模式时，系统通过 `VISUAL`/`EDITOR` 环境变量自动检测编辑器，不需要此配置。

### extension 字段

控制 Custom Editor 模式创建的临时文件扩展名，影响编辑器的语法高亮和行为：
- `"md"` → Markdown 文件，大多数编辑器支持语法高亮
- `"txt"` → 纯文本文件

## 加载逻辑

```text
load_config()
  ├── dirs::home_dir() 获取用户 home 目录
  │     └── 失败 → 返回 AppConfig::default()
  ├── 检查 $HOME/.commit-audition/config.toml 是否存在
  │     └── 不存在 → 返回 AppConfig::default()
  ├── std::fs::read_to_string() 读取文件内容
  │     └── 失败 → 返回 AppConfig::default()
  └── toml::from_str() 解析 TOML
        └── 失败 → 返回 AppConfig::default()
```

**设计原则**：任何环节失败都返回默认配置而非报错，确保"零配置即可使用"。

## 使用场景

### 场景 1：使用 VS Code 编辑 commit 正文

配置文件 `~/.commit-audition/config.toml`：

```toml
[editor]
command = "code --wait"
extension = "md"
```

运行 `commit-audition`，在"是否添加正文?"选择 Yes → 选择 Custom Editor → 自动使用 VS Code 打开临时 `.md` 文件。

### 场景 2：使用终端内联输入（默认）

无需任何配置文件。运行 `commit-audition`，在"是否添加正文?"选择 Yes → 选择 Terminal Inline → 在终端逐行输入。

### 场景 3：使用系统默认编辑器

无需配置文件，但需设置环境变量：

```bash
export EDITOR=vim
# 或
export VISUAL=code
```

运行 `commit-audition`，在"是否添加正文?"选择 Yes → 选择 Default Editor → 系统自动使用 `$VISUAL` 或 `$EDITOR` 指定的编辑器。
