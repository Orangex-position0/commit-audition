# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - 2025-05-05

### Added

- 集成 LLM 提供商用于 AI 辅助生成 commit message，支持 Claude、OpenAI、Ollama、DeepSeek 和 GLM 后端
- 新增 `ai` 子命令及交互式流程中的 AI 建议功能
- AI 功能支持 staged diff 自动收集、自适应截断和 i18n 提示模板
- 新增配置文件模板（`templates/config.en.toml`、`templates/config.zh.toml`）
- 新增 AI 集成相关文档（`docs/note/modules/Integration with AI.md`）

### Changed

- 将终端 UI 库从 `dialoguer` 迁移到 `inquire` (v0.9)，修复文本输入时光标不可见的问题
- Vim 模式下新增光标显示支持

### Fixed

- 修复 `dialoguer::Input` 隐藏终端光标的已知问题（dialoguer#77）

---

## [0.4.0] - 2025-04-21

### Added

- 使用 `rust-i18n` 添加 i18n 国际化支持，提供中英双语 UI
- CI、文档和 License 相关基础设施
- 本地 CI 检查脚本和 pre-push hook

### Fixed

- 将 MSRV 从 1.85.0 提升到 1.88.0 以兼容 ratatui 0.29
- 修复 StepBar 中非激活步骤的前景色不可见问题
- 修复 clippy `collapsible_match` 警告
- 修复 `ci-check.sh` 中 `set -e` 误退出问题

---

## [0.3.0] - 2025-04-11

### Added

- 编辑/查看模式、循环选择、步骤跳转和正文预览功能

### Changed

- 同步键位绑定并更新 man page

---

## [0.2.0] - 2025-04-06

### Added

- Vim 模式 TUI 界面
- 配置文件、键位绑定文档和 man pages

---

## [0.1.0] - 2025-03-30

### Added

- 交互式 commit message 生成与校验功能
- Conventional Commits 规范校验
- Git Hook 集成
- CI/CD 基础设施和自动化发布配置

[unreleased]: https://github.com/USER/commit-audition/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/USER/commit-audition/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/USER/commit-audition/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/USER/commit-audition/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/USER/commit-audition/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/USER/commit-audition/releases/tag/v0.1.0
