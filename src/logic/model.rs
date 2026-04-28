use rust_i18n::t;

/// Commit 类型标签枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitTagType {
    /// 新功能
    Feat,
    /// 修复
    Fix,
    /// 文档
    Docs,
    /// 格式改变 (不影响代码运行的变动)
    Style,
    /// 重构
    Refactor,
    /// 增加测试
    Test,
    /// 构建过程或辅助工具的变动
    Chore,
}

impl CommitTagType {
    /// 所有可选的 commit 类型标签
    pub const ALL: [CommitTagType; 7] = [
        CommitTagType::Feat,
        CommitTagType::Fix,
        CommitTagType::Docs,
        CommitTagType::Style,
        CommitTagType::Refactor,
        CommitTagType::Test,
        CommitTagType::Chore,
    ];

    /// commit 类型标签转字符串，用于消息组装
    pub fn as_str(&self) -> &'static str {
        match self {
            CommitTagType::Feat => "feat",
            CommitTagType::Fix => "fix",
            CommitTagType::Docs => "docs",
            CommitTagType::Style => "style",
            CommitTagType::Refactor => "refactor",
            CommitTagType::Test => "tests",
            CommitTagType::Chore => "chore",
        }
    }

    /// 选择列表中的显示描述，用于 UI 展示
    pub fn get_description(&self) -> String {
        match self {
            CommitTagType::Feat => t!("type.feat").to_string(),
            CommitTagType::Fix => t!("type.fix").to_string(),
            CommitTagType::Docs => t!("type.docs").to_string(),
            CommitTagType::Style => t!("type.style").to_string(),
            CommitTagType::Refactor => t!("type.refactor").to_string(),
            CommitTagType::Test => t!("type.test").to_string(),
            CommitTagType::Chore => t!("type.chore").to_string(),
        }
    }
}

/// commit message 结构实体
#[derive(Debug, Clone)]
pub struct CommitMessageEntity {
    /// commit 类型标签
    pub commit_tag_type: CommitTagType,
    /// 标题
    pub title: String,
    /// 正文
    pub body: Option<String>,
    /// 关联的 issue 编号
    pub issue_num: Option<u32>,
}

/// Commit Body 编辑模式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    /// 终端模式 (终端内逐行输入)
    TerminalInline,
    /// 默认编辑器模式 (默认编辑器内输入)
    DefaultEditor,
    /// 自定义编辑器模式 (自定义编辑器内输入)
    CustomEditor,
}

impl EditorMode {
    /// 所有可选的编辑模式
    pub const ALL: [EditorMode; 3] = [
        EditorMode::TerminalInline,
        EditorMode::DefaultEditor,
        EditorMode::CustomEditor,
    ];

    /// 用于 Select 下拉列表的显示文本
    pub fn display_label(&self) -> String {
        match self {
            EditorMode::TerminalInline => t!("editor_mode.terminal_inline").to_string(),
            EditorMode::DefaultEditor => t!("editor_mode.default_editor").to_string(),
            EditorMode::CustomEditor => t!("editor_mode.custom_editor").to_string(),
        }
    }
}

/// 从 git commit message 原始文本解析出的结构
#[derive(Debug)]
pub struct CommitMsgParsed {
    /// commit 类型标签
    pub type_prefix: Option<String>,
    /// 标题
    pub title: String,
    /// 正文
    pub body: Option<String>,
}

impl CommitMsgParsed {
    /// 从 commit message 原始文本解析
    ///
    /// 格式: "<type>: <title>\n\n<body>\n\n#<issue>"
    pub fn parse(content: &str) -> Self {
        let content = content.trim();
        let first_line = content.lines().next().unwrap_or("");

        let (type_prefix, title) = match first_line.find(":") {
            None => (None, first_line.to_string()),
            Some(pos) => {
                let prefix = first_line[..pos].to_string();
                let title = first_line[pos + 1..].trim_start().to_string();
                (Some(prefix), title)
            }
        };

        let body = content
            .find("\n\n")
            .map(|pos| content[pos + 2..].trim().to_string())
            .filter(|b| !b.is_empty());

        Self {
            type_prefix,
            title,
            body,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_tag_type_as_str() {
        assert_eq!(CommitTagType::Feat.as_str(), "feat");
        assert_eq!(CommitTagType::Fix.as_str(), "fix");
        assert_eq!(CommitTagType::Chore.as_str(), "chore");
    }

    #[test]
    fn commit_tag_type_all_contains_seven() {
        assert_eq!(CommitTagType::ALL.len(), 7);
    }
}
