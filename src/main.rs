use std::io::Read;

use commit_audition::cli::parse_args;
use commit_audition::logic::builder::build_message;
use commit_audition::prelude::Colorize;
use commit_audition::ui::prompt::run_prompt;

fn main() {
    let _config = parse_args();

    match run_prompt() {
        Some(msg) => {
            let output = build_message(&msg);
            println!("\n{}", output);
        }
        None => println!("{}", "已取消。".yellow()),
    }
}

/// 读取 commit message 内容
#[allow(dead_code)]
fn read_content(source: &str) -> String {
    if source == "-" {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .expect("无法从 stdin 读取");
        buf
    } else {
        std::fs::read_to_string(source).unwrap_or_else(|e| {
            eprintln!("{}", format!("无法读取文件 {}: {}", source, e).red());
            std::process::exit(1);
        })
    }
}
