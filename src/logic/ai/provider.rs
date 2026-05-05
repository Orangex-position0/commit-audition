use crate::prelude::CommitTagType;
use async_trait::async_trait;
use serde::Deserialize;

/// 发送给 LLM 的完整提示
pub struct AiPrompt {
    /// system prompt
    pub system: String,
    /// user prompt
    pub user: String,
}

/// AI 返回的结构化建议
#[derive(Debug, Clone)]
pub struct AiSuggestion {
    pub commit_type: CommitTagType,
    pub title: String,
    pub body: Option<String>,
}

/// AI 调用错谁
#[derive(Debug)]
pub enum AiError {
    /// 网关错误 / 连接失败
    Network(String),
    /// API 认证失败 / 配额超限
    Auth(String),
    /// 响应解析失败 (LLM 返回的格式错误)
    Parse(String),
    /// 配置缺失
    Config(String),
    /// 本地命令执行 / 文件 IO 错误
    IO(String),
}

impl std::fmt::Display for AiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiError::Network(msg) => write!(f, "网络错误: {}", msg),
            AiError::Auth(msg) => write!(f, "认证失败: {}", msg),
            AiError::Parse(msg) => write!(f, "AI 响应解析失败: {}", msg),
            AiError::Config(msg) => write!(f, "配置错误: {}", msg),
            AiError::IO(msg) => write!(f, "IO 错误: {}", msg),
        }
    }
}

impl std::error::Error for AiError {}

#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// 根据给定的 prompt 生成 commit message 建议
    async fn generate(&self, prompt: AiPrompt) -> Result<AiSuggestion, AiError>;
}

/// LLM JSON 响应的反序列化结构
#[derive(Deserialize)]
struct LLMJsonResponse {
    #[serde(rename = "type")]
    commit_type: String,
    title: String,
    body: Option<String>,
}

/// 从 JSON 文本中解析 AI 响应
pub fn parse_suggestion(json_str: &str) -> Result<AiSuggestion, AiError> {
    let json_text = extract_json_from_text(json_str);

    let parsed: LLMJsonResponse = serde_json::from_str(&json_text)
        .map_err(|e| AiError::Parse(format!("无法解析 JSON: {}\n原始响应: {}", e, json_str)))?;

    let commit_type = parse_commit_type(&parsed.commit_type)?;

    Ok(AiSuggestion {
        commit_type,
        title: parsed.title,
        body: parsed.body.filter(|b| !b.trim().is_empty()),
    })
}

/// 从可能包含 Markdown code block 的文本中提取 JSON
fn extract_json_from_text(text: &str) -> String {
    let trimmed = text.trim();

    // 如果包含 ```json ... ``` 代码块，提取内容
    if let Some(start) = trimmed.find("```json") {
        let json_start = start + 7;
        if let Some(end) = trimmed[json_start..].find("```") {
            return trimmed[json_start..json_start + end].trim().to_string();
        }
    }

    // 如果包含 ``` ... ``` 代码块
    if let Some(start) = trimmed.find("```") {
        let json_start = start + 3;
        if let Some(end) = trimmed[json_start..].find("```") {
            return trimmed[json_start..json_start + end].trim().to_string();
        }
    }

    // 尝试找到第一个 { 和最后一个 }
    if let Some(start) = trimmed.find('{')
        && let Some(end) = trimmed.rfind('}')
    {
        return trimmed[start..=end].to_string();
    }

    trimmed.to_string()
}

/// 将字符串解析为 CommitTagType
fn parse_commit_type(s: &str) -> Result<CommitTagType, AiError> {
    let lower = s.trim().to_lowercase();
    CommitTagType::ALL
        .iter()
        .find(|t| t.as_str() == lower)
        .copied()
        .ok_or_else(|| {
            AiError::Parse(format!(
                "未知的 commit 类型: '{}', 合法类型: {}",
                s,
                CommitTagType::ALL
                    .iter()
                    .map(|t| t.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })
}
