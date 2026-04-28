rust_i18n::i18n!("locales", fallback = "en");

use commit_audition::logic::model::CommitTagType;
use commit_audition::logic::rules::CommitMsgError;
use commit_audition::ui::vim::app::Step;
use rust_i18n::t;
use std::sync::Mutex;

/// i18n 测试使用全局 locale，必须串行执行
static LOCALE_LOCK: Mutex<()> = Mutex::new(());

/// 验证英文 locale 下 Step label 返回正确字符串
#[test]
fn step_labels_english() {
    let _lock = LOCALE_LOCK.lock().unwrap();
    rust_i18n::set_locale("en");
    assert_eq!(Step::SelectType.label(), "1.Type");
    assert_eq!(Step::InputTitle.label(), "2.Title");
    assert_eq!(Step::SelectBody.label(), "3.Body");
    assert_eq!(Step::InputIssue.label(), "4.Issue");
    assert_eq!(Step::Preview.label(), "5.Preview");
}

/// 验证中文 locale 下 Step label 返回正确字符串
#[test]
fn step_labels_chinese() {
    let _lock = LOCALE_LOCK.lock().unwrap();
    rust_i18n::set_locale("zh");
    assert_eq!(Step::SelectType.label(), "1.类型");
    assert_eq!(Step::InputTitle.label(), "2.标题");
    assert_eq!(Step::SelectBody.label(), "3.正文");
    assert_eq!(Step::InputIssue.label(), "4.Issue");
    assert_eq!(Step::Preview.label(), "5.预览");
}

/// 验证 CommitTagType 描述在英文 locale 下正确
#[test]
fn type_descriptions_english() {
    let _lock = LOCALE_LOCK.lock().unwrap();
    rust_i18n::set_locale("en");
    assert_eq!(CommitTagType::Feat.get_description(), "A new feature");
    assert_eq!(CommitTagType::Fix.get_description(), "A bug fix");
    assert_eq!(
        CommitTagType::Docs.get_description(),
        "Documentation changes"
    );
    assert_eq!(
        CommitTagType::Style.get_description(),
        "Formatting (no code behavior change)"
    );
    assert_eq!(
        CommitTagType::Refactor.get_description(),
        "Refactoring (not a feature or bug fix)"
    );
    assert_eq!(CommitTagType::Test.get_description(), "Adding tests");
    assert_eq!(
        CommitTagType::Chore.get_description(),
        "Build or auxiliary tool changes"
    );
}

/// 验证 CommitTagType 描述在中文 locale 下正确
#[test]
fn type_descriptions_chinese() {
    let _lock = LOCALE_LOCK.lock().unwrap();
    rust_i18n::set_locale("zh");
    assert_eq!(CommitTagType::Feat.get_description(), "新功能 (feature)");
    assert_eq!(CommitTagType::Fix.get_description(), "修补 bug");
    assert_eq!(CommitTagType::Docs.get_description(), "文档改变");
    assert_eq!(
        CommitTagType::Style.get_description(),
        "格式（不影响代码运行的变动）"
    );
    assert_eq!(
        CommitTagType::Refactor.get_description(),
        "重构（既不是新增功能，也不是修改 bug 的代码变动）"
    );
    assert_eq!(CommitTagType::Test.get_description(), "增加测试");
    assert_eq!(
        CommitTagType::Chore.get_description(),
        "构建过程或辅助工具的变动"
    );
}

/// 验证 CommitMsgError 的 Display 在英文 locale 下使用翻译
#[test]
fn commit_msg_error_display_english() {
    let _lock = LOCALE_LOCK.lock().unwrap();
    rust_i18n::set_locale("en");
    let err = CommitMsgError::Empty;
    assert_eq!(format!("{err}"), "Commit message cannot be empty");
}

/// 验证 CommitMsgError 的 Display 在中文 locale 下使用翻译
#[test]
fn commit_msg_error_display_chinese() {
    let _lock = LOCALE_LOCK.lock().unwrap();
    rust_i18n::set_locale("zh");
    let err = CommitMsgError::Empty;
    assert_eq!(format!("{err}"), "commit message 不能为空");
}

/// 验证带参数的翻译键在英文 locale 下正确插值
#[test]
fn interpolated_translations_english() {
    let _lock = LOCALE_LOCK.lock().unwrap();
    rust_i18n::set_locale("en");
    let msg = t!("vim.width_counter", width = 42).to_string();
    assert_eq!(msg, "Width: 42/50");
}

/// 验证带参数的翻译键在中文 locale 下正确插值
#[test]
fn interpolated_translations_chinese() {
    let _lock = LOCALE_LOCK.lock().unwrap();
    rust_i18n::set_locale("zh");
    let msg = t!("vim.width_counter", width = 42).to_string();
    assert_eq!(msg, "宽度: 42/50");
}

/// 验证 locale 切换是全局生效的：设置 zh 后切回 en
#[test]
fn locale_switching() {
    let _lock = LOCALE_LOCK.lock().unwrap();
    rust_i18n::set_locale("en");
    assert_eq!(CommitTagType::Feat.get_description(), "A new feature");

    rust_i18n::set_locale("zh");
    assert_eq!(CommitTagType::Feat.get_description(), "新功能 (feature)");

    rust_i18n::set_locale("en");
    assert_eq!(CommitTagType::Feat.get_description(), "A new feature");
}
