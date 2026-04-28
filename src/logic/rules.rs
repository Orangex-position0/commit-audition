use crate::logic::model::CommitTagType;
use rust_i18n::t;
use unicode_width::UnicodeWidthStr;

/// 计算 UTF-8 字符串的 Unicode 显示宽度
pub fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// 标题行最大显示宽度
pub const TITLE_MAX_WIDTH: usize = 50;

/// 正文每行最大显示宽度
pub const BODY_LINE_MAX_WIDTH: usize = 72;

/// 标题校验错误枚举
#[derive(Debug, PartialEq, Eq)]
pub enum TitleError {
    /// 标题不能为空
    Empty,
    /// 标题不能超过 {max} 个字符
    TooLong { width: usize, max: usize },
    /// 标题不能以句号结尾
    EndsWithPeriod,
}

/// 校验 commit 标题
/// 判空 -> 长度判断 -> 结尾句号判断
pub fn validate_title(title: &str) -> Result<(), TitleError> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(TitleError::Empty);
    }

    let width = display_width(trimmed);
    if width > TITLE_MAX_WIDTH {
        return Err(TitleError::TooLong {
            width,
            max: TITLE_MAX_WIDTH,
        });
    }

    if trimmed.ends_with('.') {
        return Err(TitleError::EndsWithPeriod);
    }

    Ok(())
}

/// 正文行校验错误枚举
#[derive(Debug, PartialEq, Eq)]
pub enum BodyError {
    /// 正文行不能超过 {max} 个字符
    LineTooLong {
        /// 超限所在行号（从 1 开始计数）
        line_number: usize,
        /// 该行实际显示宽度
        width: usize,
        /// 允许的最大显示宽度
        max: usize,
    },
}

/// 校验 commit 正文行
pub fn validate_body(body: &str) -> Result<(), BodyError> {
    for (i, line) in body.lines().enumerate() {
        let width = display_width(line);
        if width > BODY_LINE_MAX_WIDTH {
            return Err(BodyError::LineTooLong {
                line_number: i + 1,
                width,
                max: BODY_LINE_MAX_WIDTH,
            });
        }
    }

    Ok(())
}

/// commit message 原始文本校验-错误枚举
#[derive(Debug, PartialEq, Eq)]
pub enum CommitMsgError {
    /// message 为空
    Empty,
    /// 缺少 "<type>: " 前缀
    MissingType {
        /// 用户输入的第一行内容
        line: String,
    },
    /// 类型前缀不合法（如 "unknown: ..."）
    InvalidType {
        /// 用户使用的类型
        found: String,
        /// 所有合法类型列表
        valid: String,
    },
    /// 标题不合规（复用 TitleError）
    TitleError(TitleError),
    /// 正文不合规（复用 BodyError）
    BodyError(BodyError),
}

impl std::fmt::Display for CommitMsgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommitMsgError::Empty => write!(f, "{}", t!("rules.msg_empty")),
            CommitMsgError::MissingType { line } => {
                write!(
                    f,
                    "{}",
                    t!("rules.msg_missing_type_with_input", line = line)
                )
            }
            CommitMsgError::InvalidType { found, valid } => {
                write!(
                    f,
                    "{}\n  {}",
                    t!("rules.msg_invalid_type", found = found),
                    t!("rules.msg_valid_types", valid = valid)
                )
            }
            CommitMsgError::TitleError(e) => {
                let msg = match e {
                    TitleError::Empty => t!("rules.title_empty"),
                    TitleError::TooLong { width, max } => {
                        t!("rules.title_too_long", width = width, max = max)
                    }
                    TitleError::EndsWithPeriod => t!("rules.title_period"),
                };
                write!(f, "{}", msg)
            }
            CommitMsgError::BodyError(e) => {
                let msg = match e {
                    BodyError::LineTooLong {
                        line_number,
                        width,
                        max,
                    } => t!(
                        "rules.body_line_too_long",
                        line = line_number,
                        width = width,
                        max = max
                    ),
                };
                write!(f, "{}", msg)
            }
        }
    }
}

/// 校验原始 commit message 文本
///
/// 格式要求: `"<type>: <title>[\n\n<body>][\n\n#<issue>]"`
///
/// 校验步骤: 非空 → 类型前缀存在 → 类型合法 → 标题合规 → 正文合规
pub fn validate_raw_commit_msg(content: &str) -> Result<(), CommitMsgError> {
    let content = content.trim();

    if content.is_empty() {
        return Err(CommitMsgError::Empty);
    }

    let first_line = content.lines().next().unwrap_or("");

    let colon_pos = first_line.find(':');
    let type_prefix = match colon_pos {
        None => {
            return Err(CommitMsgError::MissingType {
                line: first_line.to_string(),
            });
        }
        Some(pos) => &first_line[..pos],
    };

    let valid_types: Vec<&str> = CommitTagType::ALL.iter().map(|t| t.as_str()).collect();
    if !valid_types.contains(&type_prefix) {
        return Err(CommitMsgError::InvalidType {
            found: type_prefix.to_string(),
            valid: valid_types.join(", "),
        });
    }

    let title = first_line
        .get(colon_pos.unwrap() + 1..)
        .unwrap_or("")
        .trim_start();

    validate_title(title).map_err(CommitMsgError::TitleError)?;

    let body_part = content
        .find("\n\n")
        .map(|pos| content[pos + 2..].trim().to_string())
        .filter(|b| !b.is_empty());

    if let Some(body) = &body_part {
        validate_body(body).map_err(CommitMsgError::BodyError)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_width_ascii() {
        assert_eq!(display_width("hello"), 5);
        assert_eq!(display_width(""), 0);
    }

    #[test]
    fn display_width_cjk() {
        // 每个中文字符占 2 个显示宽度
        assert_eq!(display_width("你好"), 4);
        assert_eq!(display_width("Hello世界"), 9);
    }

    #[test]
    fn validate_title_ok() {
        assert!(validate_title("Add user login feature").is_ok());
        assert!(validate_title("Fix bug in parser").is_ok());
    }

    #[test]
    fn validate_title_empty() {
        assert_eq!(validate_title(""), Err(TitleError::Empty));
        assert_eq!(validate_title("   "), Err(TitleError::Empty));
    }

    #[test]
    fn validate_title_too_long() {
        let long_title = "A".repeat(51);
        assert!(matches!(
            validate_title(&long_title),
            Err(TitleError::TooLong { .. })
        ));
    }

    #[test]
    fn validate_title_exactly_50() {
        let title = "A".repeat(50);
        assert!(validate_title(&title).is_ok());
    }

    #[test]
    fn validate_title_ends_with_period() {
        assert_eq!(
            validate_title("Fix the bug."),
            Err(TitleError::EndsWithPeriod)
        );
    }

    #[test]
    fn validate_title_no_period() {
        assert!(validate_title("Fix the bug").is_ok());
    }

    #[test]
    fn validate_body_ok() {
        assert!(validate_body("This is a valid body line.").is_ok());
        assert!(validate_body("Line one\nLine two").is_ok());
    }

    #[test]
    fn validate_body_line_too_long() {
        let long_line = "A".repeat(73);
        let body = format!("short line\n{}", long_line);
        let result = validate_body(&body);
        assert!(matches!(
            result,
            Err(BodyError::LineTooLong { line_number: 2, .. })
        ));
    }

    #[test]
    fn validate_body_exactly_72() {
        let line = "A".repeat(72);
        assert!(validate_body(&line).is_ok());
    }

    #[test]
    fn validate_raw_commit_msg_ok() {
        let msg = "feat: Add user login feature";
        assert!(validate_raw_commit_msg(msg).is_ok());
    }

    #[test]
    fn validate_raw_commit_msg_with_body() {
        let msg = "fix: Fix login timeout\n\nThe timeout was too short.";
        assert!(validate_raw_commit_msg(msg).is_ok());
    }

    #[test]
    fn validate_raw_commit_msg_empty() {
        assert_eq!(validate_raw_commit_msg(""), Err(CommitMsgError::Empty));
    }

    #[test]
    fn validate_raw_commit_msg_missing_type() {
        let msg = "fix bug without type prefix";
        assert!(matches!(
            validate_raw_commit_msg(msg),
            Err(CommitMsgError::MissingType { .. })
        ));
    }

    #[test]
    fn validate_raw_commit_msg_invalid_type() {
        let msg = "unknown: some message";
        assert!(matches!(
            validate_raw_commit_msg(msg),
            Err(CommitMsgError::InvalidType { .. })
        ));
    }
}
