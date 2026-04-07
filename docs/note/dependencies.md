# Dependencies Docs

本文档列出 `commit-audition` 的所有依赖及其选型理由。

## 依赖列表

| 依赖 | 版本 | 用途 | 使用位置 |
|---|---|---|---|
| clap | 4.5.5 | CLI 参数解析 | `cli/args.rs` |
| dialoguer | 0.12.0 | 终端交互组件 | `ui/prompt.rs`, `ui/editor.rs` |
| colored | 2.2.0 | 终端着色输出 | `ui/render.rs`, `ui/prompt.rs`, `ui/editor.rs`, `main.rs` |
| unicode-width | 0.2.0 | Unicode 字符显示宽度计算 | `logic/rules.rs` |
| dirs | 5.0.1 | 跨平台 home 目录定位 | `logic/config.rs` |
| toml | 0.9.0 | TOML 配置文件解析 | `logic/config.rs` |
| serde | 1.0.2 | 序列化/反序列化框架 | `logic/config.rs` |
| tempfile | 3.23.0 | 临时文件管理 | `ui/editor.rs` |

## 选型理由

### clap — CLI 参数解析

选择 `clap` 并启用 `derive` feature，而非 `structopt`：
- `clap v4` 已内置 derive 宏支持，`structopt` 已进入维护模式
- derive 模式通过结构体注解声明参数，代码简洁且类型安全
- 自动生成 `--help` 信息和版本号

```rust
#[derive(Parser)]
#[command(name = "commit-audition", version, about)]
pub struct CliArgs { ... }
```

### dialoguer — 终端交互组件

提供 Select（下拉选择）、Input（文本输入）、Confirm（确认对话框）、Editor（外部编辑器）四种交互组件：
- 统一的 `ColorfulTheme` 主题，视觉一致性好
- 内置输入校验支持（`validate_with`），与 Logic 层校验规则无缝集成
- `Editor` 组件封装了系统编辑器调用逻辑

### colored — 终端着色

轻量级终端着色库，提供链式 API：
- 用于 commit message 预览的着色展示（类型标签青色、标题白色粗体、Issue 黄色）
- 用于错误信息的红色高亮和提示信息的黄色/青色

### unicode-width — CJK 字符宽度计算

**核心选型原因**：commit message 的标题行宽限制（50 字符）和正文行宽限制（72 字符）基于**终端显示宽度**，而非字节长度或字符数。中文字符在终端中占 2 个显示宽度，需要专门的宽度计算：
- `"feat: 新功能"` 的显示宽度为 10（而非字节长度 14）
- 确保中文标题行宽校验的准确性

### dirs — 跨平台路径

提供 `home_dir()` 函数，跨平台（Windows/macOS/Linux）获取用户 home 目录：
- 配置文件路径 `$HOME/.commit-audition/config.toml` 的基础
- 比 `std::env::var("HOME")` 更可靠（Windows 下 HOME 环境变量不一定存在）

### toml + serde — 配置文件解析

- `serde` 提供 `Deserialize` derive 宏，将 TOML 文件直接反序列化为 `AppConfig` 结构体
- `toml` crate 负责实际的 TOML 格式解析
- 两者配合使用是 Rust 生态中配置文件处理的标准方案

### tempfile — 临时文件管理

用于 Custom Editor 模式下创建临时文件：
- `NamedTempFile` 在 drop 时自动清理，防止临时文件残留
- 支持修改文件扩展名（`.with_extension()`），确保编辑器正确识别文件类型
