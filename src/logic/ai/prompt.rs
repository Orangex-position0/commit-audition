use crate::logic::ai::provider::AiError;
use crate::logic::config::AiConfig;
use rust_i18n::t;

pub fn load_system_prompt(ai_config: &AiConfig) -> Result<String, AiError> {
    match &ai_config.prompt_file {
        Some(path) => {
            let expanded = expand_tilde(path);
            std::fs::read_to_string(&expanded)
                .map_err(|e| AiError::Config(format!("无法读取 prompt 文件 {}: {}", path, e)))
        }
        None => Ok(t!("ai.system_prompt").to_string()),
    }
}

/// 组装完整的 user prompt (diff content)
pub fn build_user_prompt(stat: &str, diff: &str) -> String {
    if stat.is_empty() {
        format!("{}\n\n{}", t!("ai.user_prompt_without_stat"), diff)
    } else {
        format!(
            "{}\n{}\n\n{}\n\n{}",
            t!("ai.user_prompt_stat_label"),
            stat,
            t!("ai.user_prompt_diff_label"),
            diff
        )
    }
}

/// 展开 ~ 为 home directory
fn expand_tilde(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return format!("{}/{}", home.display(), rest);
    }
    path.to_string()
}
