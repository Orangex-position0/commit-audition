use async_trait::async_trait;
use reqwest::Client;

use super::provider::{AiError, AiPrompt, AiSuggestion, LLMProvider, parse_suggestion};

/// 默认 Claude 模型
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";

/// Claude API 响应结构
#[derive(serde::Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(serde::Deserialize)]
struct ClaudeContent {
    text: String,
}

/// Claude API 请求结构
#[derive(serde::Serialize)]
#[serde(tag = "role")]
#[serde(rename_all = "lowercase")]
enum ClaudeMessage {
    User { content: String },
}

#[derive(serde::Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<ClaudeMessage>,
}

pub struct ClaudeProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl ClaudeProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
        }
    }
}

#[async_trait]
impl LLMProvider for ClaudeProvider {
    async fn generate(&self, prompt: AiPrompt) -> Result<AiSuggestion, AiError> {
        let request = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 1024,
            system: prompt.system,
            messages: vec![ClaudeMessage::User {
                content: prompt.user,
            }],
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AiError::Network(e.to_string()))?;

        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(AiError::Auth("API key 无效或已过期".into()));
        }
        if status == reqwest::StatusCode::FORBIDDEN {
            return Err(AiError::Auth("API 配额超限".into()));
        }
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AiError::Network(format!(
                "API 请求失败 ({}): {}",
                status, body
            )));
        }

        let claude_resp: ClaudeResponse = response
            .json()
            .await
            .map_err(|e| AiError::Parse(format!("响应解析失败: {}", e)))?;

        let text = claude_resp
            .content
            .first()
            .map(|c| c.text.as_str())
            .unwrap_or("");

        parse_suggestion(text)
    }
}
