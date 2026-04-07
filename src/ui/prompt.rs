use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use colored::Colorize;
use crate::logic::model::{CommitMessageEntity, CommitTagType};
use crate::logic::rules::{validate_title, TitleError};
use crate::ui::editor::input_body;
use crate::ui::render::render_colored_preview;


/// 交互式问答，逐步收集 commit message 各个部分
pub fn run_prompt() -> Option<CommitMessageEntity> {
    loop {
        let msg = collect_inputs()?;

        println!("\n{}", "── 预览 ──".green().bold());
        println!("{}\n", render_colored_preview(&msg));

        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("确认使用这条 commit message?")
            .default(true)
            .interact()
            .unwrap_or(false);

        if confirmed {
            return Some(msg)
        }

        let redo = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("是否重新编辑?")
            .default(true)
            .interact()
            .unwrap_or(false);

        if !redo {
            return None;
        }
    }
}

/// 交互式问答，逐步收集用户输入 (about commit message)
fn collect_inputs() -> Option<CommitMessageEntity> {
    let commit_tag_type = select_type()?;
    let title = input_title()?;
    let body = input_body()?;
    let issue_num = input_issue()?;

    Some(CommitMessageEntity {
        commit_tag_type,
        title,
        body,
        issue_num,
    })
}

/// 下拉选择 commit tag type
fn select_type() -> Option<CommitTagType> {
    let items: Vec<String> = CommitTagType::ALL
        .iter()
        .map(|t| format!("{} - {}", t.as_str(), t.get_description()))
        .collect();

    let index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择 commit 类型")
        .items(&items)
        .interact()
        .ok()?;

    CommitTagType::ALL.get(index).copied()
}



/// 输入 commit title
fn input_title() -> Option<String> {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("输入 commit 标题 (命令式预期，首字母大写，<= 50 字符)")
        .validate_with(|input: &String| -> Result<(), String> {
            match validate_title(input) {
                Ok(_) => Ok(()),
                Err(TitleError::Empty) => Err("标题不能为空".into()),
                Err(TitleError::TooLong { width, max }) => {
                    Err(format!("标题长度超出限制，当前长度为 {}，最大长度为 {}", width, max))
                }
                Err(TitleError::EndsWithPeriod) => {
                    Err("标题不能以句号结束".into())
                }
            }
        })
        .allow_empty(false)
        .interact_text()
        .ok()
}

/// 输入 issue number
fn input_issue() -> Option<Option<u32>> {
    let add_issue = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("是否添加关联 issue?")
        .default(false)
        .interact()
        .ok()?;

    if !add_issue {
        return Some(None);
    }

    let num: u32 = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("输入 issue 编号")
        .validate_with(|input: &String| -> Result<(), String> {
            input
                .trim()
                .parse::<u32>()
                .map(|_| ())
                .map_err(|_| "请输入有效的正整数".into())
        })
        .interact_text()
        .ok()?
        .trim()
        .parse()
        .ok()?;

    Some(Some(num))
}
