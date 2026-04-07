# 使用指南

本文档详细说明 `commit-audition` 的所有使用方式。

## 目录

- [交互式生成](#交互式生成)
- [校验模式](#校验模式)
- [Git Hook 管理](#git-hook-管理)
- [正文编辑模式](#正文编辑模式)
- [配置](#配置)
- [Commit Message 规范](#commit-message-规范)

---

## 交互式生成

### 基本用法

```bash
commit-audition
```

启动交互式问答，按步骤生成 commit message。

### 步骤详解

#### 第 1 步：选择 Commit 类型

```
? 选择 commit 类型
> feat - 新功能 (feature)
  fix - 修补 bug
  docs - 文档改变
  style - 格式（不影响代码运行的变动）
  refactor - 重构
  tests - 增加测试
  chore - 构建过程或辅助工具的变动
```

使用上下箭头选择，回车确认。

#### 第 2 步：输入标题

```
? 输入 commit 标题 (命令式预期，首字母大写，<= 50 字符): Add user login feature
```

**校验规则**：
- 不能为空
- 显示宽度不超过 50 字符（中文每个字符占 2 宽度）
- 不能以句号结尾

校验失败时会在输入框下方显示错误提示，需重新输入。

#### 第 3 步：添加正文（可选）

```
? 是否添加正文? (y/N)
```

选择 `Yes` 后进入编辑模式选择：

```
? 选择编辑方式
> Terminal Inline - 在终端中逐行输入
  Default Editor - 使用系统默认编辑器
  Custom Editor - 使用配置文件中指定的编辑器
```

详见 [正文编辑模式](#正文编辑模式)。

#### 第 4 步：关联 Issue（可选）

```
? 是否添加关联 issue? (y/N)
? 输入 issue 编号: 42
```

#### 第 5 步：预览与确认

```
── 预览 ──
feat: Add user login feature

Implemented OAuth2 flow.

#42

? 确认使用这条 commit message? (Y/n)
```

- **确认**：输出纯文本 commit message
- **不确认**：询问是否重新编辑，重新编辑则回到第 1 步，否则取消

### 输出示例

```text
feat: Add user login feature

Implemented OAuth2 flow.

#42
```

可以直接复制使用，或通过管道传递给 `git commit`：

```bash
commit-audition | xargs -0 git commit -m
```

---

## 校验模式

### 校验文件

```bash
commit-audition validate .git/COMMIT_EDITMSG
```

常用于 CI/CD 流水线或手动检查。

### 从 stdin 读取

```bash
echo "feat: Add feature" | commit-audition validate -
```

使用 `-` 作为文件路径参数，从标准输入读取。

### 退出码

| 退出码 | 含义 |
|---|---|
| `0` | 校验通过 |
| `1` | 校验失败（输出错误信息到 stderr） |

### 校验失败示例

```bash
$ commit-audition validate .git/COMMIT_EDITMSG
标题: 长度超出限制，当前长度为 65，最大长度为 50
```

```bash
$ echo "unknown: some message" | commit-audition validate -
不合法的类型 "unknown"
  合法类型: feat, fix, docs, style, refactor, tests, chore
```

### 校验规则

| 规则 | 限制 | 错误类型 |
|---|---|---|
| 消息不能为空 | — | `Empty` |
| 必须有 `<type>: ` 前缀 | — | `MissingType` |
| 类型必须合法 | 7 种 | `InvalidType` |
| 标题不能为空 | — | `TitleError::Empty` |
| 标题显示宽度 | ≤50 | `TitleError::TooLong` |
| 标题不能以句号结尾 | — | `TitleError::EndsWithPeriod` |
| 正文每行显示宽度 | ≤72 | `BodyError::LineTooLong` |

---

## Git Hook 管理

### 安装 Hook

```bash
commit-audition hook install
```

在当前 git 仓库的 `.git/hooks/` 下创建 `commit-msg` hook 脚本。

**效果**：每次执行 `git commit` 时，hook 自动调用 `commit-audition validate` 校验 commit message。校验不通过时，commit 被阻止。

**安全检查**：
- 如果已存在本工具生成的 hook → 提示"已安装"，跳过
- 如果已存在其他 hook → 报错，要求手动检查，不会覆盖

### 卸载 Hook

```bash
commit-audition hook uninstall
```

删除当前仓库的 `commit-msg` hook。

**安全检查**：仅删除包含 `commit-audition` 标识的 hook，不会误删用户自定义 hook。

### 典型工作流

```bash
# 首次使用：安装 hook
cd your-project
commit-audition hook install

# 正常提交（hook 自动校验）
git add .
git commit -m "feat: Add new feature"
# → 校验通过，commit 成功

git commit -m "不好的message"
# → 校验失败，commit 被阻止
```

---

## 正文编辑模式

### Terminal Inline（终端内联）

在终端中逐行输入正文，输入空行结束。

```
逐行输入正文，输入空行结束:
? 第 1 行 (留空结束): The timeout was set to 5 seconds.
? 第 2 行 (留空结束): Increased to 30 seconds for slow networks.
? 第 3 行 (留空结束):
```

**特点**：
- 适合简短的正文（1-5 行）
- 无需离开终端
- 每行自动校验行宽（≤72 字符）

### Default Editor（系统默认编辑器）

使用 `VISUAL` 或 `EDITOR` 环境变量指定的编辑器。

```bash
# 设置默认编辑器
export EDITOR=vim
# 或
export VISUAL=code
```

**特点**：
- 适合较长的正文
- 支持编辑器的全部功能（语法高亮、搜索替换等）
- 注释行（以 `#` 开头）会自动过滤

### Custom Editor（自定义编辑器）

使用配置文件中指定的编辑器命令。

需要在 `~/.commit-audition/config.toml` 中配置：

```toml
[editor]
command = "code --wait"
extension = "md"
```

**特点**：
- 可指定任意编辑器及参数
- 临时文件扩展名可配置，确保编辑器正确识别文件类型
- 未配置时会提示配置示例

### 编辑模式对比

| 模式 | 适合场景 | 需要配置 | 离开终端 |
|---|---|---|---|
| Terminal Inline | 简短正文 | 否 | 否 |
| Default Editor | 较长正文 | 需设置环境变量 | 是 |
| Custom Editor | 特定编辑器偏好 | 需配置文件 | 是 |

---

## 配置

### 配置文件路径

```text
~/.commit-audition/config.toml
```

### 完整配置示例

```toml
[editor]
command = "code --wait"    # 自定义编辑器命令（可选）
extension = "md"            # 临时文件扩展名（默认 "md"）
```

### 常见编辑器配置

| 编辑器 | command 值 |
|---|---|
| VS Code | `"code --wait"` |
| Vim | `"vim"` |
| Neovim | `"nvim"` |
| Nano | `"nano"` |
| Sublime Text | `"subl --wait"` |
| Emacs | `"emacs"` |

### 配置加载规则

- 文件不存在 → 使用默认值
- 文件内容无效 → 使用默认值
- 部分字段缺失 → 该字段使用默认值

---

## Commit Message 规范

### 格式

```text
<type>: <title>           ← 标题行（必有，≤50 字符显示宽度）
                          ← 空行（仅当有正文或 Issue 时）
<body>                    ← 正文（可选，每行 ≤72 字符显示宽度）
                          ← 空行（仅当同时有正文和 Issue 时）
#<issue>                  ← Issue 关联（可选）
```

### 示例

#### 最简形式

```text
feat: Add user login feature
```

#### 标题 + 正文

```text
fix: Fix login timeout

The timeout was set to 5 seconds.
Increased to 30 seconds for slow networks.
```

#### 完整形式

```text
feat: Add user login

Implemented OAuth2 flow.

#42
```

### 标题规范

- 使用命令式语气（"Add" 而非 "Added" 或 "Adds"）
- 首字母大写
- 不超过 50 个字符显示宽度（中文每个字符占 2 宽度）
- 不以句号结尾

### 正文规范

- 每行不超过 72 个字符显示宽度
- 用于解释"为什么"而非"做了什么"（"做了什么"应该由标题和代码 diff 说明）
