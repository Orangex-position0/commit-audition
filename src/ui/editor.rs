use std::io::Write;
use std::process::Command;

use dialoguer::Editor;
use rust_i18n::t;
use tempfile::NamedTempFile;

use crate::logic::config::load_config;
use crate::prelude::*;

/// 在终端中逐行输入正文，输入空行结束
pub fn edit_terminal_inline() -> Option<Option<String>> {
    println!("{}", t!("ui.body_line_by_line").to_string().yellow());

    let mut lines: Vec<String> = Vec::new();
    loop {
        let line: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(t!("ui.body_line_n", n = lines.len() + 1).to_string())
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
        Err(BodyError::LineTooLong {
            line_number, max, ..
        }) => {
            eprintln!(
                "{}",
                t!("ui.body_line_too_long_err", line = line_number, max = max)
                    .to_string()
                    .red()
            );
            None
        }
    }
}

/// 使用系统默认编辑器
pub fn edit_default_editor() -> Option<Option<String>> {
    let editor_hint = detect_default_editor();

    match editor_hint {
        Some(ref name) => {
            println!(
                "{}",
                t!("terminal.using_editor", name = name).to_string().cyan()
            );
        }
        None => {
            println!("{}", t!("terminal.no_env_editor").to_string().yellow());
        }
    }

    let result = Editor::new()
        .extension("md")
        .edit(&t!("ui.body_hint_template"))
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
                Err(BodyError::LineTooLong {
                    line_number,
                    width,
                    max,
                }) => {
                    eprintln!(
                        "{}",
                        t!(
                            "rules.body_line_too_long",
                            line = line_number,
                            max = max,
                            width = width
                        )
                        .to_string()
                        .red()
                    );
                    None
                }
            }
        }
    }
}

/// 使用自定义编辑器
pub fn edit_custom_editor(command: &str, extension: &str) -> Option<Option<String>> {
    let mut temp = NamedTempFile::new().expect(&t!("terminal.cannot_create_temp"));

    writeln!(temp, "{}", t!("ui.body_hint_template")).ok()?;
    temp.flush().ok()?;

    let temp_path = temp.path().with_extension(extension);
    std::fs::rename(temp.path(), &temp_path).ok()?;

    let parts: Vec<&str> = command.split_whitespace().collect();
    let (cmd, args) = parts.split_first()?;

    let mut full_args: Vec<&str> = args.to_vec();
    let path_str = temp_path.to_str()?;
    full_args.push(path_str);

    let status = Command::new(cmd).args(&full_args).status().ok()?;

    if !status.success() {
        eprintln!(
            "{}",
            t!("terminal.editor_exited_abnormally").to_string().red()
        );
        return Some(None);
    }

    let content = std::fs::read_to_string(&temp_path).ok()?;

    let _ = std::fs::remove_file(&temp_path);

    let body = filter_and_clean(&content);

    if body.is_empty() {
        return Some(None);
    }

    match validate_body(&body) {
        Ok(()) => Some(Some(body)),
        Err(BodyError::LineTooLong {
            line_number,
            width,
            max,
        }) => {
            eprintln!(
                "{}",
                t!(
                    "rules.body_line_too_long",
                    line = line_number,
                    max = max,
                    width = width
                )
                .to_string()
                .red()
            );
            None
        }
    }
}

/// 统一的 body 输入入口
pub fn input_body() -> Option<Option<String>> {
    let add_body = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(t!("ui.add_body").to_string())
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
                Some(cmd) => edit_custom_editor(cmd, &config.editor.extension),
                None => {
                    eprintln!("{}", t!("terminal.no_editor_config").to_string().red());
                    eprintln!(
                        "{}",
                        t!("terminal.editor_config_example").to_string().yellow()
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
        .with_prompt(t!("ui.select_editor_mode").to_string())
        .items(&items)
        .default(0)
        .interact()
        .ok()?;

    EditorMode::ALL.get(index).copied()
}
