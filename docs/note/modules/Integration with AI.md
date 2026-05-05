# Integration with AI
本项目提供 AI 模型集成功能，可通过 AI 辅助生成符合 Conventional Commits 规范的 commit message。

核心功能：
- 根据 `git diff --staged` 自动分析变更内容
- 调用 LLM（OpenAI / Claude / Ollama / DeepSeek / GLM 等）生成 commit message 建议
- AI 建议预填充到现有 TUI（vim 模式 / inquire 模式），用户可编辑确认
- AI 层完全可选，未配置 `[ai]` 段时工具行为不变


# 架构

## 模块
```text
src/
├── logic/
│   ├── ai/                    # AI 层（trait + prompt + diff）
│   │   ├── mod.rs             # 模块导出 + create_provider 工厂函数 + generate_suggestion 主流程
│   │   ├── provider.rs        # LLMProvider trait + AiSuggestion + AiError + parse_suggestion
│   │   ├── openai_compat.rs   # OpenAI 兼容 provider（覆盖 OpenAI / Ollama / DeepSeek / GLM 等）
│   │   ├── claude.rs          # Claude Messages API 独立实现
│   │   ├── prompt.rs          # prompt 加载逻辑（内置默认 + 自定义文件）+ build_user_prompt
│   │   ├── diff.rs            # git diff 获取与混合截断预处理
│   │   └── default_prompt.md  # 内置默认 system prompt
│   └── config.rs              # AppConfig 扩展：新增 AiConfig 结构体
├── cli/args.rs                # 新增 Commands::Ai, CliConfig::Ai
├── ui/
│   ├── prompt.rs              # 新增 run_prompt_with_suggestion()
│   └── vim/
│       ├── app.rs             # 新增 ai_suggestion, ai_loading 字段 + with_suggestion()
│       ├── event.rs           # 新增 Ctrl+A 触发 AI 重新生成
│       └── view.rs            # 新增 AI 加载状态渲染
└── main.rs                    # #[tokio::main] + Ai 子命令分发 + fallback 手动输入
```

## 核心组成
统一 trait + 各个 AI Model Chat 数据结构 + ratatui 渲染页面

- `LLMProvider`: LLM 提供者的统一抽象，定义 `async fn generate(AiPrompt) -> Result<AiSuggestion, AiError>`
- `AiPrompt`: 发送给 LLM 的完整提示（system + user 两个字段）
- `AiSuggestion`: AI 返回的结构化建议（commit_type, title, body）
- `AiError`: AI 调用的统一错误类型（Network / Auth / Parse / Config 四种变体）
- `LLMJsonResponse`: LLM JSON 响应的反序列化结构，内部使用

## Provider 实现策略

采用混合方案：
- **`OpenAiCompatibleProvider`**：一个通用 struct，通过配置不同 endpoint / api_key / model 适配所有 OpenAI 兼容 API（OpenAI / Ollama / DeepSeek / GLM 等）
- **`ClaudeProvider`**：独立实现，因为 Claude Messages API 格式不同（system prompt 在顶级字段、使用 `x-api-key` header）

内置 provider 默认配置：

| provider 值 | 默认 endpoint | 默认模型 | 认证方式 |
|---|---|---|---|
| `claude` | ClaudeProvider 独立实现 | `claude-sonnet-4-20250514` | `x-api-key` header |
| `openai` | `https://api.openai.com/v1/chat/completions` | `gpt-4o` | `Bearer` token |
| `ollama` | `http://localhost:11434/v1/chat/completions` | `codellama` | 无 |
| `deepseek` | `https://api.deepseek.com/v1/chat/completions` | `deepseek-chat` | `Bearer` token |
| `glm` | `https://open.bigmodel.cn/api/paas/v4/chat/completions` | `glm-4-flash` | `Bearer` token |
| 其他任意值 | 需手动配置 endpoint | 需手动配置 | `Bearer` token |

# 数据流

```
用户执行 `ca ai` 或在 TUI 中按 Ctrl+A
  → diff.rs: get_staged_diff() + get_staged_stat()
  → diff.rs: truncate_diff() 根据大小选择截断策略
  → prompt.rs: load_system_prompt() + build_user_prompt(stat, diff)
  → provider.generate(prompt): async HTTP 调用 LLM
  → 解析 JSON 响应 → AiSuggestion
  → 预填充到 App 各字段（type / title / body）
  → 用户在现有 TUI 中编辑确认
```

# 相关 Git 命令

项目中使用了两个 git diff 命令，分别用于获取完整变更内容和文件级统计摘要。

## `git diff --cached --no-color`

**作用**：获取暂存区（staged）的完整变更内容。

**参数解释**：
- `--cached`：只看暂存区的变更（即已经 `git add` 但还没 `git commit` 的内容）。不带此参数的 `git diff` 看的是未暂存的变更（已修改但还没 `git add` 的内容）。
- `--no-color`：禁用终端颜色编码（ANSI 转义序列）。颜色编码对人类阅读友好，但会污染文本内容，导致 AI 收到 `ESC[32m+helloESC[m` 这样的乱码。

**输出格式**：
```diff
diff --git a/src/main.rs b/src/main.rs        ← 文件变更开始标记
index abc1234..def5678 100644                  ← git 内部哈希信息
--- a/src/main.rs                              ← 变更前文件路径
+++ b/src/main.rs                              ← 变更后文件路径
@@ -10,6 +10,8 @@ fn main() {                  @@ hunk 头：旧文件第10行起6行 → 新文件第10行起8行
 fn existing_function() {                       上下文行（无前缀）：未变更的代码
     println!("hello");                        上下文行
+    let x = 42;                               + 开头：新增的行
+    println!("{}", x);                        + 开头：新增的行
 }                                             上下文行
-    // old comment                            - 开头：删除的行
```

**行前缀含义**：
- 无前缀：上下文行（未变更，用于提供代码语境）
- `+` 开头：新增的行
- `-` 开头：删除的行

**本项目用途**：`get_staged_diff()` 调用此命令获取完整 diff，作为 AI 生成 commit message 的核心输入。

## `git diff --cached --stat --no-color`

**作用**：获取暂存区变更的文件级统计摘要（不含具体代码变更）。

**参数解释**：
- `--cached --no-color`：同上
- `--stat`：只输出文件级统计信息，不输出具体 diff 内容

**输出格式**：
```
 src/logic/ai.rs      | 168 ++++
 src/main.rs          | 125 +++-
 Cargo.lock           | 1112 ++++++++++++++++++++++++++++++
 3 files changed, 1405 insertions(+), 72 deletions(-)
```

**每列含义**：
- 第一列：文件路径
- `|` 后的数字：该文件变更的总行数
- `+++`/`---`/`++-`：可视化表示新增（`+`）和删除（`-`）的比例
- 最后一行：总计 N 个文件，M 次新增，K 次删除

**本项目用途**：`get_staged_stat()` 调用此命令获取文件摘要，始终附加在 AI prompt 中，确保 AI 知道所有变更了哪些文件，即使 diff 被截断。

## 两个命令的对比

| | `git diff --cached --no-color` | `git diff --cached --stat --no-color` |
|---|---|---|
| 输出内容 | 每一行代码的增删详情 | 每个文件的行数统计 |
| 大小 | 可能很大（几十 KB） | 通常很小（< 500 字符） |
| 用途 | AI 分析具体改了什么 | AI 了解改了哪些文件 |
| 被截断？ | 是（超过 8000 字符时截断） | 否（始终完整发送） |

# 核心功能
## diff truncate
获取 git diff，根据大小选择截断策略，返回 `String`。

始终附加 `get_staged_stat()` 的文件级摘要（`git diff --cached --stat`），确保 AI 知道所有变更了哪些文件。stat 获取失败时不影响主流程（返回空字符串降级）。

## truncate strategy 选型

### 1. AI 自主提炼摘要
核心：**两次AI调用** → 先让AI生成diff精简摘要 → 再用摘要生成commit message

| ✅️优点 | ❌️缺点 |
|-------|--------|
| 保留完整上下文，生成准确率极高 | 需调用两次AI，成本翻倍、耗时增加 |
| 大diff也能稳定输出高质量结果 | 实现逻辑更复杂，需管理双请求流程 |
| 无需手动编写截断规则 | 极端超大diff仍可能触发首次请求超限 |

---

### 2. 按文件生成摘要
核心：**本地预处理** → 遍历变更文件 → 单文件截取片段/统计信息 → 结构化发送AI

| ✅️优点 | ❌️缺点 |
|-------|--------|
| 完全可控，绝对不会触发Token超限 | 本地硬截断可能丢失diff中间的核心逻辑 |
| 速度极快、AI调用成本极低 | 需要自主开发文件遍历、片段提取逻辑 |
| 结构化信息，AI解析效率更高 | 复杂变更场景下准确率会下降 |

---

### 3. 混合截断策略
核心：**截断规则 + AI**

1. 先判断 diff 大小
    - 小 diff：直接完整发送
    - 中 diff：按文件取片段（方案 2）
    - 大 diff：只发送文件列表 + 增删行数
2. 再将处理后的内容发给 AI 生成 commit message

| ✅️优点 | ❌️缺点 |
|-------|--------|
| 最均衡，质量/速度/成本三者兼顾 | 实现复杂 |
| 几乎适配所有项目规模 | 超大 diff 仍会丢失细节 |

---

### 4. 直接完整发送（无截断）
核心：**无处理** → 直接将`git diff`完整内容发送给AI生成message

| ✅️优点 | ❌️缺点 |
|-------|--------|
| 实现最简单，开发成本极低 | 大diff直接超限，兼容性极差 |
| AI获取全量上下文，理解最准确 | 耗时久、Token成本高 |
| 适配小型/单次变更 | 易被无关代码干扰，生成冗余信息 |

---

### 5. 仅发送元数据（极简截断）
核心：**无代码片段** → 仅发送文件列表、增删行数、文件类型给AI

| ✅️优点 | ❌️缺点 |
|-------|--------|
| 极致省Token，响应速度最快 | 无代码上下文，生成准确率极低 |
| 完全不泄露源码，安全性最高 | 复杂逻辑变更完全无法识别 |
| 实现极简，无任何截断逻辑 | 仅能生成基础格式的commit |


## truncate strategy 最终方案
**混合阶段策略**

核心原则: 三级处理
- 小 diff 不截断，原样发送
- 中 diff 按文件均衡采样，每个文件分配公平预算
- 大 diff 仅保留极简摘要（文件列表 + 每个文件前几行）
- stat 摘要始终附加，确保 AI 知道全貌

### 三级处理

| 级别 | diff 大小                   | 策略               | 实现函数                  |
|----|---------------------------|------------------|-----------------------|
| 小  | ≤ `MAX_DIFF_CHARS` (8000) | 直接完整发送，不做任何处理    | 无截断，原样返回              |
| 中  | 8000 ~ 32000 字符           | 按文件均衡采样          | `balanced_truncate()` |
| 大  | > 32000 字符                | 极简摘要：文件头 + 前 N 行 | `minimal_truncate()`  |


### 核心函数

```text
truncate_diff(diff, max_chars)          ← 唯一的公开截断入口
    │
    ├── diff.len() <= max_chars         → 直接返回（小 diff）
    │
    ├── diff.len() <= max_chars * 4     → balanced_truncate()（中 diff）
    │       │
    │       ├── split_into_file_blocks()    ← 把完整 diff 切成一个个文件块
    │       ├── 算每块预算
    │       ├── truncate_single_block()     ← 截断单个文件块到预算内
    │       └── append_truncation_notice()  ← 末尾加 "... (truncated)" 提示
    │
    └── 否则                            → minimal_truncate()（大 diff）
            │
            ├── 逐行遍历，每文件只取前 10 行
            └── append_truncation_notice()

调用方 (mod.rs):
    generate_suggestion()
        ├── get_staged_diff()        ← 拿原始 diff
        ├── get_staged_stat()        ← 拿文件统计摘要
        ├── truncate_diff()          ← 截断
        └── build_user_prompt(stat, diff)  ← 组装完整 prompt
```

#### `truncate_diff()`
截断入口函数。根据 diff 字符长度判断落入哪个级别，分发到对应的处理函数。

#### `get_staged_stat()`
非致命函数。获取 `git diff --cached --stat --no-color` 的文件级统计摘要。执行失败时返回空字符串而不是 Err，因为 stat 只是辅助信息，不影响主流程。

#### `split_into_file_blocks()`
将完整 diff 按 `diff --git` 行切分为 `Vec<String>`，每个元素是一个文件的完整变更内容。
- 用 `block_start` 记录当前文件块的起始行号
- 遇到新的 `diff --git` 行时，把 `[block_start..i)` 的行 join 成一个 String 推入 blocks
- `i > 0` 条件避免在第一行（第一个文件头）时做无意义的空切分
- 循环结束后处理最后一个文件块

#### `balanced_truncate()`
中 diff 均衡采样策略：
1. 调用 `split_into_file_blocks` 切分文件块
2. 计算每块预算：`per_file_budget = max_chars / 文件数`
3. 遍历每个文件块，取 `min(per_file_budget, remaining)` 作为实际预算
4. 调用 `truncate_single_block` 截断到预算内
5. `remaining < 50` 时停止（空间太小写不下有意义内容）

#### `minimal_truncate()`
大 diff 极简摘要策略：
- 维护 `current_file_lines` 计数器，遇到 `diff --git` 行重置为 0
- 每个文件只保留前 `MINIMAL_LINES_PER_FILE`（10）行
- 超出 10 行的用 `continue` 跳过（不终止循环，后面还有别的文件）
- 总预算用完时 `break`

#### `truncate_single_block()`
截断单个文件块到预算内。关键：用 `start_len = result.len()` 记录写入前的总长度，比较 `result.len() + line.len() + 1 > start_len + budget`，确保每个文件块最多写入 `budget` 字符。

#### `append_truncation_notice()`
计算被省略的字符数（`original_len - result_len`），大于 0 时追加 `"... (truncated, N chars omitted)"` 提示。

# 配置

配置文件路径：`~/.commit-audition/config.toml`

```toml
# 方案一：Ollama 本地模型（无需 API key）
[ai]
provider = "ollama"
model = "qwen2.5-coder:7b"

# 方案二：GLM（智谱 AI，OpenAI 兼容）
# [ai]
# provider = "glm"
# api_key = "你的智谱API Key"
# model = "glm-4-flash"

# 方案三：Claude（使用独立 Messages API 实现）
# [ai]
# provider = "claude"
# api_key = "sk-ant-..."

# 方案四：任意 OpenAI 兼容 API（手动指定 endpoint）
# [ai]
# provider = "custom"
# endpoint = "https://your-api.example.com/v1/chat/completions"
# api_key = "your-key"
# model = "your-model"
```

# 错误处理

AI 错误不阻塞主流程，始终 fallback 到手动输入：

| 场景 | 表现 |
|---|---|
| `[ai]` 段未配置 | `ca ai` 报错退出，提示配置方法 |
| API key 无效 | TUI 显示错误 → fallback 手动输入 |
| Ollama 未启动 | 提示连接失败 → fallback |
| Diff 为空 | 提示 "没有暂存的变更" → fallback |
| Diff 过大 | 截断 + 追加提示 |
| 响应解析失败 | 提示 "AI 响应解析失败" → fallback |
| 网络超时 | 默认 120 秒超时 → fallback |
| stat 获取失败 | 非致命，stat 为空字符串，继续用 diff |
