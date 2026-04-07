use colored::Colorize;
use crate::logic::model::CommitMessageEntity;

/// 将 commit message 渲染为终端可读的预览文本
pub fn render_colored_preview(msg: &CommitMessageEntity) -> String {
    let type_label = format!("{}:", msg.commit_tag_type.as_str()).cyan().bold();
    let title = msg.title.white().bold();

    let mut parts = vec![format!("{} {}", type_label, title)];

    if let Some(body) = &msg.body && !body.trim().is_empty()
    {
        parts.push(String::new());
        parts.push(body.clone());
    }

    if let Some(issue) = msg.issue_num {
        parts.push(String::new());
        parts.push(format!("#{}", issue).yellow().bold().to_string());
    }

    parts.join("\n")
}