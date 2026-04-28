use crate::prelude::{CommitMessageEntity, CommitTagType, EditorMode};
use rust_i18n::t;

/// APP 状态机
/// 当前所处的状态步骤
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    SelectType,
    InputTitle,
    SelectBody,
    InputIssue,
    Preview,
}

impl Step {
    pub fn all() -> [Step; 5] {
        [
            Step::SelectType,
            Step::InputTitle,
            Step::SelectBody,
            Step::InputIssue,
            Step::Preview,
        ]
    }

    /// 步骤栏显示标签
    pub fn label(&self) -> String {
        match self {
            Step::SelectType => t!("vim.step_type").to_string(),
            Step::InputTitle => t!("vim.step_title").to_string(),
            Step::SelectBody => t!("vim.step_body").to_string(),
            Step::InputIssue => t!("vim.step_issue").to_string(),
            Step::Preview => t!("vim.step_preview").to_string(),
        }
    }

    /// 切换到上一步 (循环)
    pub fn prev(&self) -> Self {
        let all = Self::all();
        let idx = all.iter().position(|&s| s == *self).unwrap_or(0);
        all[(idx + all.len() - 1) % all.len()]
    }

    /// 切换到下一步 (循环)
    pub fn next(&self) -> Self {
        let all = Self::all();
        let idx = all.iter().position(|&s| s == *self).unwrap_or(0);
        all[(idx + 1) % all.len()]
    }
}

/// 全屏 TUI 的应用状态
pub struct App {
    /// 当前所处的步骤
    pub step: Step,
    /// step1 - 类型选择列表的光标索引
    pub selected_type_index: usize,
    /// 用户是否已确认类型选择
    pub type_selected: bool,
    /// 已输入的 commit 标题
    pub title: String,
    /// step3 - 正文编辑方式列表的光标索引
    pub selected_body_index: usize,
    /// 用户是否已确认正文编辑方式选择
    pub body_selected: bool,
    /// 正文内容（外部编辑器返回后填入）
    pub body: Option<String>,
    /// 已输入的 issue 编号
    pub issue_num: String,
    /// 搜索过滤文本
    pub filter_text: String,
    /// 是否处于搜索模式
    pub searching: bool,
    /// 待执行的编辑器调用（由事件循环处理挂起/恢复）
    pub pending_editor: Option<EditorMode>,
    /// 是否退出事件循环
    pub quit: bool,
    /// 用户是否在预览步骤确认提交（区分 y 确认 和 q/Esc 取消）
    pub confirmed: bool,
    /// 是否处于编辑模式
    pub editing: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            step: Step::SelectType,
            selected_type_index: 0,
            type_selected: false,
            title: String::new(),
            selected_body_index: 0,
            body_selected: false,
            body: None,
            issue_num: String::new(),
            filter_text: String::new(),
            searching: false,
            pending_editor: None,
            quit: false,
            confirmed: false,
            editing: false,
        }
    }

    /// 将 APP 状态转为 CommitMessageEntity
    pub fn to_entity(&self) -> Option<CommitMessageEntity> {
        let commit_tag_type = CommitTagType::ALL.get(self.selected_type_index).copied()?;
        let title = self.title.trim().to_string();
        if title.is_empty() {
            return None;
        }

        let body = self.body.clone().filter(|b| !b.trim().is_empty());
        let issue_num = self.issue_num.trim().parse().ok();

        Some(CommitMessageEntity {
            commit_tag_type,
            title,
            body,
            issue_num,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_all_has_five_elements() {
        assert_eq!(Step::all().len(), 5);
    }

    #[test]
    fn step_next_wraps_around() {
        assert_eq!(Step::Preview.next(), Step::SelectType);
        assert_eq!(Step::SelectType.next(), Step::InputTitle);
        assert_eq!(Step::InputIssue.next(), Step::Preview);
    }

    #[test]
    fn step_prev_wraps_around() {
        assert_eq!(Step::SelectType.prev(), Step::Preview);
        assert_eq!(Step::Preview.prev(), Step::InputIssue);
        assert_eq!(Step::InputTitle.prev(), Step::SelectType);
    }

    #[test]
    fn step_labels() {
        rust_i18n::set_locale("en");
        assert_eq!(Step::SelectType.label(), "1.Type");
        assert_eq!(Step::InputTitle.label(), "2.Title");
        assert_eq!(Step::SelectBody.label(), "3.Body");
        assert_eq!(Step::InputIssue.label(), "4.Issue");
        assert_eq!(Step::Preview.label(), "5.Preview");
    }

    #[test]
    fn app_new_default_state() {
        let app = App::new();
        assert_eq!(app.step, Step::SelectType);
        assert!(!app.type_selected);
        assert!(app.title.is_empty());
        assert!(!app.body_selected);
        assert!(app.body.is_none());
        assert!(app.issue_num.is_empty());
        assert!(app.filter_text.is_empty());
        assert!(!app.searching);
        assert!(app.pending_editor.is_none());
        assert!(!app.quit);
        assert!(!app.confirmed);
    }

    #[test]
    fn to_entity_title_only() {
        let mut app = App::new();
        app.selected_type_index = 0;
        app.title = "Add new feature".to_string();

        let entity = app.to_entity().unwrap();
        assert_eq!(entity.commit_tag_type, CommitTagType::Feat);
        assert_eq!(entity.title, "Add new feature");
        assert!(entity.body.is_none());
        assert!(entity.issue_num.is_none());
    }

    #[test]
    fn to_entity_full_message() {
        let mut app = App::new();
        app.selected_type_index = 1;
        app.title = "Fix login bug".to_string();
        app.body = Some("Updated timeout".to_string());
        app.issue_num = "42".to_string();

        let entity = app.to_entity().unwrap();
        assert_eq!(entity.commit_tag_type, CommitTagType::Fix);
        assert_eq!(entity.body.as_deref(), Some("Updated timeout"));
        assert_eq!(entity.issue_num, Some(42));
    }

    #[test]
    fn to_entity_empty_title_returns_none() {
        let app = App::new();
        assert!(app.to_entity().is_none());
    }

    #[test]
    fn to_entity_whitespace_title_returns_none() {
        let mut app = App::new();
        app.title = "   ".to_string();
        assert!(app.to_entity().is_none());
    }

    #[test]
    fn to_entity_out_of_range_index_returns_none() {
        let mut app = App::new();
        app.selected_type_index = 100;
        app.title = "Test".to_string();
        assert!(app.to_entity().is_none());
    }

    #[test]
    fn to_entity_trims_title() {
        let mut app = App::new();
        app.title = "  Trimmed title  ".to_string();

        let entity = app.to_entity().unwrap();
        assert_eq!(entity.title, "Trimmed title");
    }

    #[test]
    fn to_entity_filters_empty_body() {
        let mut app = App::new();
        app.title = "Test".to_string();
        app.body = Some("   ".to_string());

        let entity = app.to_entity().unwrap();
        assert!(entity.body.is_none());
    }

    #[test]
    fn to_entity_parses_issue_num() {
        let mut app = App::new();
        app.title = "Test".to_string();
        app.issue_num = "123".to_string();

        let entity = app.to_entity().unwrap();
        assert_eq!(entity.issue_num, Some(123));
    }

    #[test]
    fn to_entity_invalid_issue_returns_none() {
        let mut app = App::new();
        app.title = "Test".to_string();
        app.issue_num = "abc".to_string();

        let entity = app.to_entity().unwrap();
        assert!(entity.issue_num.is_none());
    }
}
