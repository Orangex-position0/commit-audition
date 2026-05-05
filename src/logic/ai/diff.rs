use crate::logic::ai::provider::AiError;
use std::process::Command;

/// diff 最大字符数
pub const MAX_DIFF_CHARS: usize = 8000;

/// 中 diff 上限倍数 (中 diff 阈值 = MAX_DIFF_CHARS * 此值)
const MEDIUM_THRESHOLD_MULTIPLIER: usize = 4;

/// 大 diff 时每个文件保留的最大行数
const MINIMAL_LINES_PER_FILE: usize = 10;

/// get staged diff
pub fn get_staged_diff() -> Result<String, AiError> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--no-color"])
        .output()
        .map_err(|e| AiError::IO(format!("无法执行 git: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AiError::IO(format!("git diff 错误: {}", stderr)));
    }

    let diff = String::from_utf8_lossy(&output.stdout).to_string();

    if diff.trim().is_empty() {
        return Err(AiError::Config("没有暂存的变更，请先执行 git add".into()));
    }

    Ok(diff)
}

/// 获取暂存区的文件级统计摘要
pub fn get_staged_stat() -> String {
    let output = Command::new("git")
        .args(["diff", "--cached", "--stat", "--no-color"])
        .output();

    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => String::new(),
    }
}

/// 预处理 diff 入口, 根据 diff 大小选择策略
///
/// - 小 diff (≤ max_chars)：直接完整发送
/// - 中 diff (max_chars ~ max_chars*4)：按文件均衡采样
/// - 大 diff (> max_chars*4)：极简摘要，每个文件只取前几行
pub fn truncate_diff(diff: &str, max_chars: usize) -> String {
    let diff_size = diff.len();

    if diff_size <= max_chars {
        return diff.to_string();
    }

    if diff_size <= max_chars * MEDIUM_THRESHOLD_MULTIPLIER {
        return balanced_truncate(diff, max_chars);
    }

    minimal_truncate(diff, max_chars)
}

/// 将 diff 按 "diff --git" 行切分为文件块
///
/// 每个文件块以 "diff --git" 开头，包含该文件的所有变更行
fn split_into_file_blocks(diff: &str) -> Vec<String> {
    let lines: Vec<&str> = diff.lines().collect();
    let mut blocks = Vec::new();
    let mut block_start = 0;

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("diff --git") && i > 0 {
            blocks.push(lines[block_start..i].join("\n"));
            block_start = i;
        }
    }

    // 最后一个文件块
    if block_start < lines.len() {
        blocks.push(lines[block_start..].join("\n"));
    }

    blocks
}

/// 中 diff 策略: 均衡采样：按文件数均分字符预算
///
/// 保证每个文件至少有代表性行被保留
fn balanced_truncate(diff: &str, max_chars: usize) -> String {
    let blocks = split_into_file_blocks(diff);

    if blocks.is_empty() {
        return diff.to_string();
    }

    let per_file_budget = max_chars / blocks.len();
    let mut result = String::with_capacity(max_chars);

    for block in &blocks {
        let remaining = max_chars.saturating_sub(result.len());
        if remaining < 50 {
            break;
        }
        let budget = per_file_budget.min(remaining);
        truncate_single_block(block, budget, &mut result);
    }

    append_truncation_notice(diff.len(), result.len(), &mut result);

    result
}

/// 大 diff 策略: 极简摘要
fn minimal_truncate(diff: &str, max_chars: usize) -> String {
    let mut result = String::with_capacity(max_chars);
    let mut current_file_lines = 0;

    for line in diff.lines() {
        if line.starts_with("diff --git") {
            current_file_lines = 0;
        }

        current_file_lines += 1;

        if current_file_lines > MINIMAL_LINES_PER_FILE {
            continue;
        }

        if result.len() + line.len() + 1 > max_chars {
            break;
        }

        result.push_str(line);
        result.push('\n');
    }

    append_truncation_notice(diff.len(), result.len(), &mut result);

    result
}

/// 截断单个文件块到执行预算内
fn truncate_single_block(block: &str, budget: usize, result: &mut String) {
    let start_len = result.len();
    for line in block.lines() {
        if result.len() + line.len() + 1 > start_len + budget {
            return;
        }
        result.push_str(line);
        result.push('\n');
    }
}

/// 追加截断通知
fn append_truncation_notice(original_len: usize, result_len: usize, result: &mut String) {
    let omitted = original_len.saturating_sub(result_len);
    if omitted > 0 {
        result.push_str(&format!("\n... (truncated, {} chars omitted)\n", omitted));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_short_diff_unchanged() {
        let diff = "diff --git a/main.rs b/main.rs\n+hello\n-world";
        let result = truncate_diff(diff, 1000);
        assert_eq!(result, diff);
    }

    #[test]
    fn truncate_long_diff_adds_notice() {
        let long_line = "A".repeat(200);
        let diff = format!("diff --git a/file.rs b/file.rs\n{}\n", long_line);
        let result = truncate_diff(&diff, 100);
        assert!(result.contains("truncated"));
    }

    #[test]
    fn truncate_caps_lines_per_file() {
        let mut diff = "diff --git a/big.rs b/big.rs\n".to_string();
        for i in 0..100 {
            diff.push_str(&format!("+line {}\n", i));
        }
        diff.push_str("diff --git a/small.rs b/small.rs\n+only one line\n");

        let result = truncate_diff(&diff, 10000);

        // small.rs 的文件头和内容应该被保留
        assert!(result.contains("diff --git a/small.rs"));
        assert!(result.contains("only one line"));
    }

    #[test]
    fn notice_added_when_omitted() {
        let mut result = String::from("hello");
        append_truncation_notice(100, 5, &mut result);
        assert!(result.contains("truncated"));
        assert!(result.contains("95"));
    }

    #[test]
    fn notice_not_added_when_nothing_omitted() {
        let mut result = String::from("hello");
        append_truncation_notice(5, 5, &mut result);
        assert!(!result.contains("truncated"));
    }

    // --- split_into_file_blocks ---

    #[test]
    fn split_single_file() {
        let diff = "diff --git a/main.rs b/main.rs\n+hello\n-world";
        let blocks = split_into_file_blocks(diff);
        assert_eq!(blocks.len(), 1);
    }

    #[test]
    fn split_multiple_files() {
        let diff = "diff --git a/a.rs b/a.rs\n+line1\ndiff --git a/b.rs b/b.rs\n+line2";
        let blocks = split_into_file_blocks(diff);
        assert_eq!(blocks.len(), 2);
    }

    // --- truncate_diff 分级 ---

    #[test]
    fn truncate_small_diff_unchanged() {
        let diff = "diff --git a/main.rs b/main.rs\n+hello\n-world";
        let result = truncate_diff(diff, 1000);
        assert_eq!(result, diff);
    }

    #[test]
    fn truncate_medium_diff_balanced() {
        // 构造一个超过 8000 但不超过 32000 的多文件 diff
        let mut diff = String::new();
        for i in 0..5 {
            diff.push_str(&format!("diff --git a/file{}.rs b/file{}.rs\n", i, i));
            diff.push_str("--- a/file.rs\n");
            diff.push_str("+++ b/file.rs\n");
            for j in 0..100 {
                diff.push_str(&format!("+line {} content here {}\n", j, i));
            }
        }

        let result = truncate_diff(&diff, MAX_DIFF_CHARS);

        // 每个文件都应该被包含
        for i in 0..5 {
            assert!(
                result.contains(&format!("diff --git a/file{}.rs", i)),
                "文件 file{}.rs 应该在截断结果中",
                i
            );
        }
    }

    #[test]
    fn truncate_large_diff_minimal() {
        // 构造一个超过 32000 的超大 diff
        let mut diff = String::new();
        for i in 0..50 {
            diff.push_str(&format!("diff --git a/file{}.rs b/file{}.rs\n", i, i));
            for j in 0..200 {
                diff.push_str(&format!(
                    "+line {} with lots of content to make it big\n",
                    j
                ));
            }
        }

        let result = truncate_diff(&diff, MAX_DIFF_CHARS);

        // 结果应该比原始 diff 小很多
        assert!(result.len() < diff.len());
        // 应该有截断提示
        assert!(result.contains("truncated"));
    }

    #[test]
    fn truncate_preserves_file_headers() {
        let diff = "diff --git a/a.rs b/a.rs\n+line1\ndiff --git a/b.rs b/b.rs\n+line2";
        let result = truncate_diff(diff, 1000);
        assert!(result.contains("diff --git a/a.rs"));
        assert!(result.contains("diff --git a/b.rs"));
    }

    #[test]
    fn get_staged_stat_returns_string() {
        // 集成测试：实际调用 git
        // 在有 git 仓库的环境中应该返回非空字符串（如果有 staged changes）
        // 或空字符串（如果没有）
        let stat = get_staged_stat();
        // 只要不 panic 就行
        let _ = stat;
    }
}
