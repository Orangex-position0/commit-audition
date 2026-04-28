use crate::prelude::*;
use crate::ui::editor::input_body;
use crate::ui::render::render_colored_preview;
use rust_i18n::t;

/// 交互式问答，逐步收集 commit message 各个部分
pub fn run_prompt() -> Option<CommitMessageEntity> {
    loop {
        let msg = collect_inputs()?;

        println!("\n{}", t!("ui.preview").to_string().green().bold());
        println!("{}\n", render_colored_preview(&msg));

        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(t!("ui.confirm_commit").to_string())
            .default(true)
            .interact()
            .unwrap_or(false);

        if confirmed {
            return Some(msg);
        }

        let redo = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(t!("ui.re_edit").to_string())
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
        .with_prompt(t!("ui.select_type").to_string())
        .items(&items)
        .interact()
        .ok()?;

    CommitTagType::ALL.get(index).copied()
}

/// 输入 commit title
fn input_title() -> Option<String> {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt(t!("ui.input_title").to_string())
        .validate_with(|input: &String| -> Result<(), String> {
            match validate_title(input) {
                Ok(_) => Ok(()),
                Err(TitleError::Empty) => Err(t!("ui.title_empty_err").to_string()),
                Err(TitleError::TooLong { width, max }) => {
                    Err(t!("ui.title_too_long_err", width = width, max = max).to_string())
                }
                Err(TitleError::EndsWithPeriod) => Err(t!("ui.title_period_err").to_string()),
            }
        })
        .allow_empty(false)
        .interact_text()
        .ok()
}

/// 输入 issue number
fn input_issue() -> Option<Option<u32>> {
    let add_issue = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(t!("ui.add_issue").to_string())
        .default(false)
        .interact()
        .ok()?;

    if !add_issue {
        return Some(None);
    }

    let num: u32 = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(t!("ui.input_issue").to_string())
        .validate_with(|input: &String| -> Result<(), String> {
            input
                .trim()
                .parse::<u32>()
                .map(|_| ())
                .map_err(|_| t!("ui.issue_invalid").to_string())
        })
        .interact_text()
        .ok()?
        .trim()
        .parse()
        .ok()?;

    Some(Some(num))
}
