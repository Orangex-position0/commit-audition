use crate::logic::model::CommitMessageEntity;

/// 将 CommitMessageEntity 组装为符合规范的纯文本字符串
///
/// ```text
/// <type>: <title>           ← 标题行（必有）
///                           ← 空行（仅当有正文或 Issue 时）
/// <body>                    ← 正文（可选）
///                           ← 空行（仅当同时有正文和 Issue 时）
/// #<issue>                  ← Issue 关联（可选）
/// ```
pub fn build_message(msg: &CommitMessageEntity) -> String {
    let mut parts = Vec::new();

    // 标题行
    parts.push(format!("{}: {}", msg.commit_tag_type.as_str(), msg.title));

    // 空行 + 正文
    if let Some(body) = &msg.body
        && !body.trim().is_empty()
    {
        parts.push(String::new());
        parts.push(body.clone());
    }

    // 空行 + Issue 关联
    if let Some(issue) = msg.issue_num {
        parts.push(String::new());
        parts.push(format!("#{}", issue));
    }

    parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::model::CommitTagType;

    fn make_msg(
        commit_tag_type: CommitTagType,
        title: &str,
        body: Option<&str>,
        issue_num: Option<u32>,
    ) -> CommitMessageEntity {
        CommitMessageEntity {
            commit_tag_type,
            title: title.to_string(),
            body: body.map(|s| s.to_string()),
            issue_num,
        }
    }

    #[test]
    fn title_only() {
        let msg = make_msg(CommitTagType::Feat, "Add user login feature", None, None);
        assert_eq!(build_message(&msg), "feat: Add user login feature");
    }

    #[test]
    fn title_with_body() {
        let msg = make_msg(
            CommitTagType::Fix,
            "Fix login timeout",
            Some("The timeout was too short.\nIncreased to 30 seconds."),
            None,
        );
        let expected =
            "fix: Fix login timeout\n\nThe timeout was too short.\nIncreased to 30 seconds.";
        assert_eq!(build_message(&msg), expected);
    }

    #[test]
    fn title_with_issue() {
        let msg = make_msg(CommitTagType::Feat, "Add export feature", None, Some(42));
        assert_eq!(build_message(&msg), "feat: Add export feature\n\n#42");
    }

    #[test]
    fn full_message() {
        let msg = make_msg(
            CommitTagType::Feat,
            "Add user login",
            Some("Implemented OAuth2 flow."),
            Some(42),
        );
        let expected = "feat: Add user login\n\nImplemented OAuth2 flow.\n\n#42";
        assert_eq!(build_message(&msg), expected);
    }

    #[test]
    fn empty_body_ignored() {
        let msg = make_msg(CommitTagType::Docs, "Update readme", Some(""), None);
        assert_eq!(build_message(&msg), "docs: Update readme");
    }

    #[test]
    fn whitespace_body_ignored() {
        let msg = make_msg(CommitTagType::Docs, "Update readme", Some("   "), None);
        assert_eq!(build_message(&msg), "docs: Update readme");
    }
}
