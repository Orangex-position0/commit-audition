use crate::logic::ai::provider::{AiError, AiPrompt, AiSuggestion, LLMProvider};
use crate::logic::config::AiConfig;

mod claude;
mod diff;
mod openai_compat;
mod prompt;
pub mod provider;

/// 各个 provider 的默认 endpoint & model
struct ProviderDefaults {
    endpoint: &'static str,
    model: &'static str,
}

fn get_provider_defaults(provider: &str) -> Option<ProviderDefaults> {
    match provider {
        "openai" => Some(ProviderDefaults {
            endpoint: "https://api.openai.com/v1/chat/completions",
            model: "gpt-4o",
        }),
        "ollama" => Some(ProviderDefaults {
            endpoint: "http://localhost:11434/v1/chat/completions",
            model: "codellama",
        }),
        "deepseek" => Some(ProviderDefaults {
            endpoint: "https://api.deepseek.com/v1/chat/completions",
            model: "deepseek-chat",
        }),
        "glm" => Some(ProviderDefaults {
            endpoint: "https://open.bigmodel.cn/api/paas/v4/chat/completions",
            model: "glm-4-flash",
        }),
        _ => None,
    }
}

/// 根据配置创建对应 provider
pub fn create_provider(config: &AiConfig) -> Result<Box<dyn LLMProvider>, AiError> {
    match config.provider.as_str() {
        "claude" => {
            let key = config
                .api_key
                .as_ref()
                .ok_or_else(|| AiError::Config("Claude 需要配置 api_key".into()))?;
            Ok(Box::new(claude::ClaudeProvider::new(
                key.clone(),
                config.model.clone(),
            )))
        }
        other => {
            let defaults = get_provider_defaults(other);
            let endpoint = config
                .endpoint
                .clone()
                .or_else(|| defaults.as_ref().map(|d| d.endpoint.to_string()))
                .ok_or_else(|| {
                    AiError::Config(format!("未知的 provider '{}', 请手动配置 endpoint", other))
                })?;

            let model = config.model.clone().unwrap_or_else(|| {
                defaults
                    .map(|d| d.model.to_string())
                    .unwrap_or_else(|| "default".to_string())
            });

            Ok(Box::new(openai_compat::OpenAiCompatibleProvider::new(
                endpoint,
                config.api_key.clone(),
                model,
            )))
        }
    }
}

/// 完整的 AI 生成流程: 获取 diff + stat -> 加载 prompt -> 调用 LLM -> 返回建议
pub async fn generate_suggestion(
    provider: &dyn LLMProvider,
    ai_config: AiConfig,
) -> Result<AiSuggestion, AiError> {
    // 1. diff
    let raw_diff = diff::get_staged_diff()?;
    let stat = diff::get_staged_stat();
    let truncated_diff = diff::truncate_diff(&raw_diff, diff::MAX_DIFF_CHARS);

    // 2. load prompt
    let system_prompt = prompt::load_system_prompt(&ai_config)?;
    let user_prompt = prompt::build_user_prompt(&stat, &truncated_diff);

    // 3. call LLM
    let prompt = AiPrompt {
        system: system_prompt,
        user: user_prompt,
    };

    provider.generate(prompt).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::ai::prompt::build_user_prompt;

    fn make_ai_config(provider: &str) -> AiConfig {
        AiConfig {
            provider: provider.to_string(),
            api_key: None,
            endpoint: None,
            model: None,
            prompt_file: None,
        }
    }

    #[test]
    fn create_provider_ollama_no_key_needed() {
        let config = make_ai_config("ollama");
        assert!(create_provider(&config).is_ok());
    }

    #[test]
    fn create_provider_openai_no_key_allowed() {
        // OpenAI 兼容 provider 不强制要求 key（某些本地 API 无需认证）
        let config = make_ai_config("openai");
        assert!(create_provider(&config).is_ok());
    }

    #[test]
    fn create_provider_claude_requires_key() {
        let config = make_ai_config("claude");
        let result = create_provider(&config);
        assert!(matches!(result, Err(AiError::Config(_))));
    }

    #[test]
    fn create_provider_unknown_without_endpoint_fails() {
        let config = make_ai_config("unknown_provider");
        let result = create_provider(&config);
        assert!(matches!(result, Err(AiError::Config(_))));
    }

    #[test]
    fn create_provider_unknown_with_endpoint_ok() {
        let config = AiConfig {
            provider: "custom".to_string(),
            api_key: None,
            endpoint: Some("https://custom.api.com/v1/chat/completions".to_string()),
            model: None,
            prompt_file: None,
        };
        assert!(create_provider(&config).is_ok());
    }

    #[test]
    fn create_provider_claude_with_key() {
        let config = AiConfig {
            provider: "claude".to_string(),
            api_key: Some("sk-test-key".to_string()),
            endpoint: None,
            model: None,
            prompt_file: None,
        };
        assert!(create_provider(&config).is_ok());
    }

    #[test]
    fn create_provider_glm_no_key_allowed() {
        let config = make_ai_config("glm");
        assert!(create_provider(&config).is_ok());
    }

    #[test]
    fn build_user_prompt_with_stat() {
        rust_i18n::set_locale("zh");
        let stat = " src/main.rs | 5 +\n 1 file changed, 5 insertions(+)";
        let diff = "+hello";
        let prompt = build_user_prompt(stat, diff);
        assert!(prompt.contains("文件摘要"));
        assert!(prompt.contains("main.rs"));
        assert!(prompt.contains("hello"));
    }

    #[test]
    fn build_user_prompt_without_stat() {
        rust_i18n::set_locale("zh");
        let diff = "+hello";
        let prompt = build_user_prompt("", diff);
        assert!(!prompt.contains("文件摘要"));
        assert!(prompt.contains("hello"));
    }
}
