use rust_i18n::t;

use crate::logic::config::load_config;
use crate::prelude::*;

/// 在终端中逐行输入正文，输入空行结束
pub fn edit_terminal_inline() -> Option<Option<String>> {
    println!("{}", t!("ui.body_line_by_line").to_string().yellow());

    let mut lines: Vec<String> = Vec::new();
    loop {
        let line = Text::new(t!("ui.body_line_n", n = lines.len() + 1).as_ref())
            .prompt_skippable()
            .ok()??;

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
    let msg = t!("ui.body_edit").to_string();
    let mut editor = inquire::Editor::new(&msg);
    editor.file_extension = ".md";

    let content = editor
        .with_predefined_text(t!("ui.body_hint_template").as_ref())
        .prompt_skippable()
        .ok()?;

    match content {
        None => Some(None),
        Some(text) => {
            let body = filter_and_clean(&text);
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
    let parts: Vec<&str> = command.split_whitespace().collect();
    let (cmd, args) = parts.split_first()?;

    let os_args: Vec<&std::ffi::OsStr> = args.iter().map(std::ffi::OsStr::new).collect();

    let msg = t!("ui.body_edit").to_string();
    let ext = format!(".{}", extension);
    let mut editor = inquire::Editor::new(&msg);
    editor.file_extension = &ext;

    let content = editor
        .with_predefined_text(t!("ui.body_hint_template").as_ref())
        .with_editor_command(std::ffi::OsStr::new(cmd))
        .with_args(&os_args)
        .prompt_skippable()
        .ok()?;

    match content {
        None => Some(None),
        Some(text) => {
            let body = filter_and_clean(&text);
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

/// 统一的 body 输入入口
pub fn input_body() -> Option<Option<String>> {
    let add_body = Confirm::new(t!("ui.add_body").as_ref())
        .with_default(false)
        .prompt()
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

/// 下拉选择编辑方式
fn select_editor_mode() -> Option<EditorMode> {
    Select::new(
        t!("ui.select_editor_mode").as_ref(),
        EditorMode::ALL.to_vec(),
    )
    .with_starting_cursor(0)
    .prompt_skippable()
    .ok()?
}
