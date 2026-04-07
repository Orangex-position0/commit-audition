use serde::Deserialize;
use std::path::PathBuf;

/// 用户配置文件
#[derive(Deserialize, Default)]
pub struct AppConfig {
    /// 编辑器配置
    #[serde(default)]
    pub editor: EditorConfig,
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

    let content = std::fs::read_to_string(&path).unwrap_or_default();
    toml::from_str(&content).unwrap_or_default()
}
