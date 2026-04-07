# UI Layer

UI 层负责所有终端交互逻辑：交互式问答引导、正文编辑器模式选择、着色预览渲染。本层仅处理展示和用户交互，不包含业务校验规则（校验由 Logic 层的 `rules.rs` 提供）。

## 模块结构

```text
src/ui.rs            → 模块入口，声明 3 个子模块
src/ui/prompt.rs     → 交互式问答主流程
src/ui/editor.rs     → 正文编辑器模式（pub，被 prompt.rs 调用）
src/ui/render.rs     → 着色预览渲染（模块私有）
```

## 1. prompt.rs — 交互式问答主流程

### run_prompt() — 主循环

交互式问答的主入口，采用**确认-重做**循环模型：

```text
loop {
    msg = collect_inputs()?      ← 收集用户输入
    render_colored_preview(&msg) ← 着色预览
    confirmed?                   ← 用户确认
    ├── 确认 → return Some(msg)
    └── 不确认
        ├── 重新编辑 → continue (重新进入循环)
        └── 放弃 → return None
}
```

**三态模型**：
1. **确认** = 正常完成，返回 `Some(CommitMessageEntity)`
2. **不确认 + 重新编辑** = 用户对内容不满意，回到循环起点重新收集
3. **不确认 + 不重新编辑** = 用户放弃操作，返回 `None`

### collect_inputs() — 逐步收集

按固定顺序收集 commit message 的四个部分：

```text
collect_inputs()
  ├── select_type()  → CommitTagType     (Select 下拉选择)
  ├── input_title()  → String            (Input 文本输入 + 实时校验)
  ├── input_body()   → Option<String>    (Confirm + EditorMode 选择)
  └── input_issue()  → Option<u32>       (Confirm + Input 数字输入)
```

### select_type() — 类型选择

使用 `dialoguer::Select` 构建 commit 类型下拉列表，展示格式为 `"feat - 新功能 (feature)"`。

### input_title() — 标题输入

使用 `dialoguer::Input` 接收标题文本，内置实时校验：
- 调用 `rules::validate_title()` 进行校验
- 校验失败时在输入框下方显示中文错误提示（如"标题长度超出限制"）
- 提示文案：`"输入 commit 标题 (命令式预期，首字母大写，<= 50 字符)"`

### input_issue() — Issue 编号输入

两步操作：
1. `Confirm` — 询问"是否添加关联 issue?"（默认 No）
2. `Input` — 输入 issue 编号，校验为有效正整数

## 2. editor.rs — 正文编辑器模式

### input_body() — 统一入口

```text
input_body()
  ├── Confirm: "是否添加正文?" (默认 No)
  │     └── No → return Some(None)  ← 不添加正文
  └── Yes → select_editor_mode()
        ├── TerminalInline  → edit_terminal_inline()
        ├── DefaultEditor   → edit_default_editor()
        └── CustomEditor    → edit_custom_editor()
```

返回类型 `Option<Option<String>>`：
- `None` → 用户取消操作
- `Some(None)` → 用户选择不添加正文
- `Some(Some(body))` → 用户输入了正文

### TerminalInline — 终端逐行输入

```text
edit_terminal_inline()
  ├── 循环接收用户输入
  │     ├── 非空行 → 加入 lines 列表
  │     └── 空行（且已有内容）→ 结束输入
  ├── join("\n") 合并所有行
  └── validate_body() 校验
        ├── 通过 → Some(Some(body))
        └── 失败 → 显示错误信息 + None
```

**设计要点**：第一个空行不终止输入（避免用户误触），只有输入了至少一行内容后空行才终止。

### DefaultEditor — 系统默认编辑器

```text
edit_default_editor()
  ├── detect_default_editor()
  │     ├── 检测 VISUAL 环境变量
  │     └── 回退到 EDITOR 环境变量
  ├── 提示将使用的编辑器名称
  └── dialoguer::Editor::new().edit(hint)
        ├── 编辑器返回内容 → filter_and_clean() → validate_body()
        └── 编辑器返回 None → Some(None)
```

### CustomEditor — 自定义编辑器

```text
edit_custom_editor(command, extension)
  ├── NamedTempFile::new() 创建临时文件
  ├── 写入提示注释 "# 输入正文内容，保存退出即可"
  ├── 修改文件扩展名为配置值（如 .md）
  ├── 解析 command 字符串为 (cmd, args)
  ├── Command::new(cmd).args(args + path).status()
  │     └── 非零退出码 → 提示异常 + Some(None)
  ├── 读取临时文件内容
  ├── 删除临时文件
  └── filter_and_clean() → validate_body()
```

**设计要点**：
- 命令字符串支持带参数（如 `"code --wait"` 会被拆分为 `cmd="code"`, `args=["--wait"]`）
- 临时文件扩展名由配置控制，确保编辑器能正确识别文件类型

### filter_and_clean() — 内容清理

过滤以 `#` 开头的注释行，合并后 `trim()`。确保编辑器中的提示注释不会被当作正文内容。

### 正文校验集成

三种编辑模式在返回前都调用 `validate_body()` 校验行宽。校验失败时显示错误信息并返回 `None`（让调用者重新触发编辑流程）。

## 3. render.rs — 着色预览渲染

### render_colored_preview() — 着色策略

```text
渲染格式：
  <cyan:bold>type:</cyan:bold> <white:bold>title</white:bold>
  [空行]
  <正文原色>
  [空行]
  <yellow:bold>#issue</yellow:bold>
```

| 部分 | 颜色 | 样式 |
|---|---|---|
| 类型标签 + 冒号 | 青色 (cyan) | 粗体 |
| 标题文本 | 白色 (white) | 粗体 |
| 正文 | 默认色 | 无 |
| Issue 编号 | 黄色 (yellow) | 粗体 |

### 与 build_message() 的差异

| 维度 | `render_colored_preview()` | `build_message()` |
|---|---|---|
| 输出 | 着色 ANSI 字符串 | 纯文本字符串 |
| 用途 | 终端预览展示 | git commit / 文件输出 |
| 空值处理 | 相同逻辑（跳过空 body） | 相同逻辑 |

两者使用相同的空值判断逻辑和片段拼接结构，区别仅在于着色处理。
