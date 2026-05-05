use clap::{Parser, Subcommand};

/// 交互式 commit message 生成工具
#[derive(Parser)]
#[command(name = "commit-audition", version, about)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// 跳过交互，直接输出模板 (预留)
    #[arg(long)]
    pub template: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 校验 commit message 文件
    Validate {
        /// commit message 文件路径
        /// git hook 会传入 .git/COMMIT_EDITMSG
        /// 使用 "-" 表示从 stdin 读取（用于管道/CI 场景）
        #[arg(value_name = "FILE")]
        file: String,
    },
    /// 管理 git hooks
    Hook {
        #[command(subcommand)]
        action: HookAction,
    },
    /// AI 生成 commit message 建议
    Ai,
}

#[derive(Subcommand)]
pub enum HookAction {
    /// 安装 commit-msg hook 到当前仓库
    Install,
    /// 卸载当前仓库的 commit-msg hook
    Uninstall,
}

/// CLI 层输出的统一配置对象
pub enum CliConfig {
    /// 默认模式：启动交互式问答
    Interactive { template: bool },
    /// 校验模式：校验 commit message 文件
    Validate { file: String },
    /// Hook 管理
    Hook(HookAction),
    /// AI 生成模式
    Ai,
}

impl From<CliArgs> for CliConfig {
    fn from(args: CliArgs) -> Self {
        match args.command {
            None => CliConfig::Interactive {
                template: args.template,
            },
            Some(Commands::Validate { file }) => CliConfig::Validate { file },
            Some(Commands::Hook { action }) => CliConfig::Hook(action),
            Some(Commands::Ai) => CliConfig::Ai,
        }
    }
}
