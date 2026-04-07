pub mod args;

use args::CliArgs;
use clap::Parser;

/// 解析命令行参数，返回统一配置对象
pub fn parse_args() -> args::CliConfig {
    let args = CliArgs::parse();
    args.into()
}