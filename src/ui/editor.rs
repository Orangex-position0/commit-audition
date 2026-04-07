use std::io::Write;
use std::process::Command;

use colored::Colorize;
use dialoguer::{Confirm, Editor, Input, Select};
use dialoguer::theme::ColorfulTheme;
use tempfile::NamedTempFile;

use crate::logic::config::load_config;
use crate::logic::model::EditorMode;
use crate::logic::rules::{validate_body, BodyError};

/// 在终端中逐行输入正文，输入空行结束
fn edit_terminal_inline() -> Option<Option<String>> {
    println!(
        "{}",
        "逐行输入正文，输入空行结束: ".yellow()
    );

    let mut lines: Vec<String> = Vec::new();
    loop {
        let line: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("第 {} 行 (留空结束)", lines.len() + 1))
            .allow_empty(true)
            .interact_text()
            .ok()?;

        if line.trim().is_empty() && !lines.is_empty() {
            break;
        }

        if !line.trim().is_empty() {
            lines.push(line);
        }
    }

    let body = lines.join("\n");

    if body.trim().is_empty() {
        return Some(None);
    }

    match validate_body(&body) {
        Ok(()) => Some(Some(body)),
        Err(BodyError::LineTooLong { line_number, max, .. }) => {
            eprintln!(
                "{}",
                format!(
                    "第 {} 行长度超过 {}，请修改",
                    line_number,
                    max
                ).red()
            );
            None
        }
    }
}

/// 使用系统默认编辑器
fn edit_default_editor() -> Option<Option<String>> {
    let editor_hint = detect_default_editor();

    match editor_hint {
        Some(ref name) => {
            println!(
                "{}",
                format!("将使用编辑器: {}", name).cyan()
            );
        }
        None => {
            println!(
                "{}",
                "未检测到 VISUAL 或 EDITOR 环境变量，将使用系统默认编辑器".yellow()
            );
        }
    }

    let result = Editor::new()
        .extension("md")
        .edit("# 输入正文内容，保存退出即可\n")
        .ok()
        .flatten();

    match result {
        None => Some(None),
        Some(content) => {
            let body = filter_and_clean(&content);
            if body.is_empty() {
                return Some(None);
            }

            match validate_body(&body) {
                Ok(()) => Some(Some(body)),
                Err(BodyError::LineTooLong { line_number, width, max }) => {
                    eprintln!(
                        "{}",
                        format!(
                            "第 {} 行长度超过 {} (当前 {})，请修改",
                            line_number,
                            max,
                            width
                        ).red()
                    );
                    None
                }
            }
        }
    }
}

/// 使用自定义编辑器
fn edit_custom_editor(
    command: &str,
    extension: &str,
) -> Option<Option<String>> {
    let mut temp = NamedTempFile::new().expect("无法创建临时文件");

    writeln!(temp, "# 输入正文内容，保存退出即可").ok()?;
    temp.flush().ok()?;

    let temp_path = temp.path().with_extension(extension);
    std::fs::rename(temp.path(), &temp_path).ok()?;

    let parts: Vec<&str> = command.split_whitespace().collect();
    let (cmd, args) = parts.split_first()?;

    let mut full_args: Vec<&str> = args.to_vec();
    let path_str = temp_path.to_str()?;
    full_args.push(path_str);

    let status = Command::new(cmd)
        .args(&full_args)
        .status()
        .ok()?;

    if !status.success() {
        eprintln!("{}", "编辑器退出异常".red());
        return Some(None);
    }

    let content = std::fs::read_to_string(&temp_path)
        .ok()?;

    let _ = std::fs::remove_file(&temp_path);

    let body = filter_and_clean(&content);

    if body.is_empty() {
        return Some(None);
    }

    match validate_body(&body) {
        Ok(()) => Some(Some(body)),
        Err(BodyError::LineTooLong { line_number, width, max }) => {
            eprintln!(
                "{}",
                format!(
                    "第 {} 行超过 {} 字符（当前 {}）",
                    line_number, max, width
                ).red()
            );
            None
        }
    }
}

/// 统一的 body 输入入口
pub fn input_body() -> Option<Option<String>> {
    let add_body = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("是否添加正文?")
        .default(false)
        .interact()
        .ok()?;

    if !add_body {
        return Some(None);
    }

    let mode = select_editor_mode()?;

    match mode {
        EditorMode::TerminalInline => edit_terminal_inline(),
        EditorMode::DefaultEditor => edit_default_editor(),
        EditorMode::CustomEditor => {
            let config = load_config();
            match &config.editor.command {
                Some(cmd) => edit_custom_editor(
                    cmd,
                    &config.editor.extension,
                ),
                None => {
                    eprintln!(
                        "{}",
                        "配置文件中未指定编辑器命令，请编辑 ~/.commit-audition/config.toml".red()
                    );
                    eprintln!(
                        "{}",
                        "示例:\n[editor]\ncommand = \"code --wait\"".yellow()
                    );
                    None
                }
            }
        }
    }
}

/// 清理编辑器内容：过滤注释行（# 开头）并去除首尾空白
fn filter_and_clean(content: &str) -> String {
    content
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// 检测系统默认编辑器
fn detect_default_editor() -> Option<String> {
    std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .ok()
}

/// 下拉选择编辑方式
fn select_editor_mode() -> Option<EditorMode> {
    let items: Vec<String> = EditorMode::ALL
        .iter()
        .map(|m| m.display_label().to_string())
        .collect();

    let index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择编辑方式")
        .items(&items)
        .default(0)
        .interact()
        .ok()?;

    EditorMode::ALL.get(index).copied()
}
