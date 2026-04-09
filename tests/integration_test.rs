use commit_audition::logic::builder::build_message;
use commit_audition::logic::model::{CommitMessageEntity, CommitTagType};
use commit_audition::logic::rules::{validate_body, validate_title};

/// 端到端流程测试：校验 → 构建 → 输出
#[test]
fn e2e_title_only_flow() {
    let title = "Add user login feature";

    // 校验通过
    assert!(validate_title(title).is_ok());

    // 构建
    let msg = CommitMessageEntity {
        commit_tag_type: CommitTagType::Feat,
        title: title.to_string(),
        body: None,
        issue_num: None,
    };
    let output = build_message(&msg);

    assert_eq!(output, "feat: Add user login feature");
}

#[test]
fn e2e_full_message_flow() {
    let title = "Fix login timeout issue";
    let body = "The timeout was set to 5 seconds.\nIncreased to 30 seconds for slow networks.";

    assert!(validate_title(title).is_ok());
    assert!(validate_body(body).is_ok());

    let msg = CommitMessageEntity {
        commit_tag_type: CommitTagType::Fix,
        title: title.to_string(),
        body: Some(body.to_string()),
        issue_num: Some(123),
    };
    let output = build_message(&msg);

    let expected = "fix: Fix login timeout issue\n\nThe timeout was set to 5 seconds.\nIncreased to 30 seconds for slow networks.\n\n#123";
    assert_eq!(output, expected);
}

#[test]
fn e2e_chore_with_body() {
    let title = "Update CI pipeline";
    let body = "Switched from Travis CI to GitHub Actions";

    assert!(validate_title(title).is_ok());
    assert!(validate_body(body).is_ok());

    let msg = CommitMessageEntity {
        commit_tag_type: CommitTagType::Chore,
        title: title.to_string(),
        body: Some(body.to_string()),
        issue_num: None,
    };
    let output = build_message(&msg);

    assert_eq!(
        output,
        "chore: Update CI pipeline\n\nSwitched from Travis CI to GitHub Actions"
    );
}
