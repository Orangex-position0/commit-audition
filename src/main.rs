use commit_audition::cli::args::CliConfig;
use commit_audition::cli::parse_args;
use commit_audition::logic::ai;
use commit_audition::logic::ai::provider::AiSuggestion;
use commit_audition::logic::builder::build_message;
use commit_audition::logic::config::{AppConfig, load_config};
use commit_audition::prelude::Colorize;
use commit_audition::ui::prompt::{run_prompt, run_prompt_with_suggestion};
use commit_audition::ui::vim::{run_vim_prompt, run_vim_prompt_with_suggestion};
use rust_i18n::t;
use std::io::{Read, Write};
use std::time::Duration;

rust_i18n::i18n!("locales", fallback = "en");

#[tokio::main]
async fn main() {
    let config = parse_args();
    let app_config = load_config();

    // 根据配置设置语言
    rust_i18n::set_locale(&app_config.language);

    match config {
        CliConfig::Ai => {
            run_ai_mode(&app_config).await;
            return;
        }
        CliConfig::Validate { file } => {
            let _content = read_content(&file);
            return;
        }
        CliConfig::Hook(action) => {
            use commit_audition::cli::args::HookAction;
            let result = match action {
                HookAction::Install => commit_audition::logic::hook::install_hook(),
                HookAction::Uninstall => commit_audition::logic::hook::uninstall_hook(),
            };
            match result {
                Ok(msg) => println!("{}", msg),
                Err(e) => eprintln!("{}", Colorize::red(e.as_str())),
            }
            return;
        }
        CliConfig::Interactive { .. } => {}
    }

    run_interactive_mode(&app_config);
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

async fn run_ai_mode(app_config: &AppConfig) {
    let ai_config = match &app_config.ai {
        Some(c) => c,
        None => {
            eprintln!("{}", Colorize::red(t!("ai.no_config").as_ref()));
            std::process::exit(1);
        }
    };

    let provider = match ai::create_provider(ai_config) {
        Ok(p) => p,
        Err(e) => {
            eprintln!(
                "{}",
                Colorize::red(format!("{}{}", t!("ai.config_error"), e).as_str())
            );
            std::process::exit(1);
        }
    };

    println!("{}", Colorize::cyan(t!("ai.analyzing").as_ref()));

    let provider_ref = provider.as_ref();
    let config_clone = ai_config.clone();

    // 启动一个后台任务打印进度点
    let result = {
        let mut interval = tokio::time::interval(Duration::from_secs(3));
        loop {
            tokio::select! {
                r = ai::generate_suggestion(provider_ref, config_clone.clone()) => {
                    break r;
                }
                _= interval.tick() => {
                    print!(".");
                    std::io::stdout().flush().ok();
                }
            }
        }
    };

    println!();

    match result {
        Ok(suggestion) => {
            println!("{}", Colorize::green(t!("ai.generating_done").as_ref()));
            run_interactive_with_suggestion(app_config, suggestion);
        }
        Err(e) => {
            eprintln!(
                "{}",
                Colorize::yellow(format!("{}{}", t!("ai.generation_failed"), e).as_str())
            );
            eprintln!("{}", Colorize::yellow(t!("ai.fallback_manual").as_ref()));
            run_interactive_mode(app_config);
        }
    }
}

/// 带有 AI 预填充的交互模式
fn run_interactive_with_suggestion(app_config: &AppConfig, suggestion: AiSuggestion) {
    let result = if app_config.vim_mode {
        run_vim_prompt_with_suggestion(suggestion)
    } else {
        run_prompt_with_suggestion(suggestion)
    };

    match result {
        Some(msg) => {
            let output = build_message(&msg);
            println!("\n{}", output);
        }
        None => println!("{}", Colorize::yellow("已取消")),
    }
}

/// 普通 (非 AI) 交互模式
fn run_interactive_mode(app_config: &AppConfig) {
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
        None => println!("{}", Colorize::yellow("已取消")),
    }
}
