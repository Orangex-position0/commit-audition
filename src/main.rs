use std::io::Read;

use commit_audition::cli::parse_args;
use commit_audition::logic::builder::build_message;
use commit_audition::logic::config::load_config;
use commit_audition::prelude::Colorize;
use commit_audition::ui::prompt::run_prompt;
use commit_audition::ui::vim::run_vim_prompt;

rust_i18n::i18n!("locales", fallback = "en");

fn main() {
    let _config = parse_args();
    let app_config = load_config();

    // 根据配置设置语言
    rust_i18n::set_locale(&app_config.language);

    let result = if app_config.vim_mode {
        run_vim_prompt()
    } else {
        run_prompt()
    };

    match result {
        Some(msg) => {
            let output = build_message(&msg);
            println!("\n{}", output);
        }
        None => println!("{}", rust_i18n::t!("ui.cancelled").to_string().yellow()),
    }
}

/// 读取 commit message 内容
#[allow(dead_code)]
fn read_content(source: &str) -> String {
    if source == "-" {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .expect(&rust_i18n::t!("terminal.cannot_read_stdin"));
        buf
    } else {
        std::fs::read_to_string(source).unwrap_or_else(|e| {
            eprintln!(
                "{}",
                rust_i18n::t!(
                    "terminal.cannot_read_file",
                    path = source,
                    error = e.to_string()
                )
                .to_string()
                .red()
            );
            std::process::exit(1);
        })
    }
}
