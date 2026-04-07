# Introduction
本文档为项目中“Commit Message 校验策略”的设计文档。

# 背景
若开发者直接编辑 commit message，而没有使用本工具，则规范还是形同虚设。

所以项目需要 Commit Message 规范校验机制，能约束本地项目的 commit message。

# 实现方案选型

| 选项 | 拦截时机 | 实现方式 | 适用场景 |
|------|----------|----------|----------|
| 方案 A | `commit-msg` Hook | git hook 调用 `commit-audition validate` | commit 创建的瞬间拦截（推荐） |
| 方案 B | `pre-push` Hook | git hook 调用 `commit-audition validate` | push 前批量拦截 |
| 方案 C | 软件内子命令 | `commit-audition check <file>` 手动校验 | 不依赖 hook 的纯工具模式 |

## 校验时机的对比
先明确拦截时机 

### `commit-msg` Hook（commit 创建时）

```
git commit -m "fix bug"
        ↓
    commit-msg hook 触发
        ↓
    校验失败 → 阻止 commit 创建
    校验通过 → commit 正常创建
```

**优势**：
- **即时反馈**：不合规的 commit 根本不会产生，开发者立刻知道并修正
- **修复成本低**：只需重新编辑 message，无需 rebase/amend
- **符合 fail-fast 原则**：问题在最早的可能时刻被发现

**劣势**：
- 每次提交都触发，对"先提交再整理 message"的工作流有干扰
- hook 是本地配置，新人 clone 仓库后需要手动安装

### `pre-push` Hook（push 前）

```
git push origin main
        ↓
    pre-push hook 触发
        ↓
    逐个校验待 push 的 commit
    全部通过 → 允许 push
    有不合规的 → 阻止 push
```

**优势**：
- 不干扰本地提交体验，开发者在本地可以随意写
- 只在"即将公开"时强制规范，灵活性好

**劣势**：
- **修复成本高**：如果已经积累了 10 个 commit，其中 3 个不合规，需要 `git rebase -i` 逐个修改 message
- 延迟反馈：开发者可能已经忘了当初为什么这样写 message

### 最后的建议
使用 `commit-msg` hook 为主，`pre-push` hook 为辅 (后续可作为扩展项)

## 最终方案: `commit-msg` hook + validate 子命令

1. 在 `commit-audition` 中新增 `validate` 子命令，读取文件内容并校验
2. 提供 `hook install` 子命令，自动在 `.git/hooks/` 安装 `commit-msg` hook
3. Hook 脚本调用 `commit-audition validate <msg-file>`，校验失败时以非零退出码阻止 commit

### 合法类型列表

| 类型标签 | 说明 |
|---|---|
| `feat` | 新功能 |
| `fix` | 修补 bug |
| `docs` | 文档改变 |
| `style` | 格式（不影响代码运行的变动） |
| `refactor` | 重构 |
| `tests` | 增加测试（注意：对应枚举 `CommitTagType::Test`，输出为 `"tests"` 而非 `"test"`） |
| `chore` | 构建过程或辅助工具的变动 |

### 整体流程
```text
用户执行 git commit -m "fix bug"
        ↓
    .git/hooks/commit-msg 触发，传入临时文件路径
        ↓
    hook 脚本执行: commit-audition validate "$1"
        ↓
    commit-audition 内部:
    读取文件 → 解析 commit message → 复用 rules.rs 校验
        ↓
    退出码 0 (通过) / 1 (失败)
```