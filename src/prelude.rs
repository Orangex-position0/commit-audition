// 外部 crate 工具
pub use colored::Colorize;
pub use dialoguer::theme::ColorfulTheme;
pub use dialoguer::{Confirm, Input, Select};

// Logic 层 - 领域模型
pub use crate::logic::model::{CommitMessageEntity, CommitTagType, EditorMode};

// Logic 层 - 校验规则
pub use crate::logic::rules::{BodyError, TitleError, validate_body, validate_title};
