use crate::logic::ai::provider::{AiError, AiPrompt, AiSuggestion, LLMProvider, parse_suggestion};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// OpenAI Chat Completions API Spec
#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: String,
}

#[derive(Serialize)]
#[serde(tag = "role")]
#[serde(rename_all = "lowercase")]
enum ChatRequestMessage {
    System { content: String },
    User { content: String },
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatRequestMessage>,
}

/// 通用 OpenAI 兼容 Provider
///
/// 通过不同的 endpoint / api_key / model 配置适配：
/// - OpenAI: endpoint = "https://api.openai.com/v1/chat/completions"
/// - Ollama: endpoint = "http://localhost:11434/v1/chat/completions"
/// - DeepSeek: endpoint = "https://api.deepseek.com/v1/chat/completions"
/// - 其他任何兼容 OpenAI Chat Completions 的 API
pub struct OpenAiCompatibleProvider {
    client: Client,
    endpoint: String,
    api_key: Option<String>,
    model: String,
}

impl OpenAiCompatibleProvider {
    pub fn new(endpoint: String, api_key: Option<String>, model: String) -> Self {
        // Ollama 等本地模型响应较慢，统一设置较长超时
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            endpoint,
            api_key,
            model,
        }
    }
}

#[async_trait]
impl LLMProvider for OpenAiCompatibleProvider {
    async fn generate(&self, prompt: AiPrompt) -> Result<AiSuggestion, AiError> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatRequestMessage::System {
                    content: prompt.system,
                },
                ChatRequestMessage::User {
                    content: prompt.user,
                },
            ],
        };

        let mut builder = self
            .client
            .post(&self.endpoint)
            .header("content-type", "application/json");

        if let Some(ref key) = self.api_key {
            builder = builder.header("Authorization", format!("Bearer {}", key));
        }

        let response = builder.json(&request).send().await.map_err(|e| {
            if e.is_connect() {
                AiError::Network("无法连接到 API 服务，请检查端点配置和服务状态".into())
            } else {
                AiError::Network(e.to_string())
            }
        })?;

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

        let chat_resp: ChatResponse = response
            .json()
            .await
            .map_err(|e| AiError::Parse(format!("响应解析失败: {}", e)))?;

        let text = chat_resp
            .choices
            .first()
            .map(|c| c.message.content.as_str())
            .unwrap_or("");

        parse_suggestion(text)
    }
}
