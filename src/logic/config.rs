use serde::Deserialize;
use std::path::PathBuf;

/// 用户配置文件
#[derive(Deserialize)]
pub struct AppConfig {
    /// 语言设置，默认 "en"
    #[serde(default = "default_language")]
    pub language: String,

    /// 编辑器配置
    #[serde(default)]
    pub editor: EditorConfig,

    /// 是否启用 vim 模式
    #[serde(default)]
    pub vim_mode: bool,

    /// AI 配置 (可选)
    pub ai: Option<AiConfig>,
}

fn default_language() -> String {
    "en".to_string()
}

/// 编辑器配置
#[derive(Deserialize)]
pub struct EditorConfig {
    /// 编辑器命令
    #[serde(default)]
    pub command: Option<String>,
    /// 临时文件扩展名
    #[serde(default = "default_extension")]
    pub extension: String,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            command: None,
            extension: "md".to_string(),
        }
    }
}

fn default_extension() -> String {
    "md".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: default_language(),
            editor: EditorConfig::default(),
            vim_mode: false,
            ai: None,
        }
    }
}

/// 配置文件路径
fn config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".commit-audition").join("config.toml"))
}

/// 读取配置文件
pub fn load_config() -> AppConfig {
    let path = match config_path() {
        None => return AppConfig::default(),
        Some(p) => p,
    };

    if !path.exists() {
        return AppConfig::default();
    }

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("警告: 无法读取配置文件 {}: {}", path.display(), e);
            return AppConfig::default();
        }
    };

    match toml::from_str(&content) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("警告: 配置文件解析失败: {}", e);
            AppConfig::default()
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct AiConfig {
    /// LLM 提供者名称
    /// - "claude" → ClaudeProvider（Claude Messages API 独立实现）
    /// - "openai" / "ollama" / "deepseek" / 其他 → OpenAiCompatibleProvider
    pub provider: String,

    /// API key（Claude 必填，OpenAI 兼容的 provider 按需填写，Ollama 不需要）
    pub api_key: Option<String>,

    /// 自定义 API 端点（覆盖 provider 的默认 endpoint）
    pub endpoint: Option<String>,

    /// 模型名称（可选，每个 provider 有默认值）
    pub model: Option<String>,

    /// 自定义 prompt 文件路径
    pub prompt_file: Option<String>,
}
