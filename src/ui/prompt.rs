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

        let confirmed = Confirm::new(t!("ui.confirm_commit").as_ref())
            .with_default(true)
            .prompt()
            .unwrap_or(false);

        if confirmed {
            return Some(msg);
        }

        let redo = Confirm::new(t!("ui.re_edit").as_ref())
            .with_default(true)
            .prompt()
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
   Select::new(
        t!("ui.select_type").as_ref(),
        CommitTagType::ALL.to_vec(),
    )
   .prompt_skippable()
   .ok()?
}

/// 输入 commit title
fn input_title() -> Option<String> {
    Text::new(t!("ui.input_title").as_ref())
        .with_validator(|input: &str| {
            match validate_title(input) {
                Ok(_) => Ok(Validation::Valid),
                Err(TitleError::Empty) => Ok(Validation::Invalid(t!("ui.title_empty_err").to_string().into())),
                Err(TitleError::TooLong { width, max }) => Ok(Validation::Invalid(
                    t!("ui.title_too_long_err", width = width, max = max).to_string().into(),
                )),
                Err(TitleError::EndsWithPeriod) => {
                    Ok(Validation::Invalid(t!("ui.title_period_err").to_string().into()))
                }
            }
        })
        .prompt_skippable()
        .ok()?
}

/// 输入 issue number
fn input_issue() -> Option<Option<u32>> {
    let add_issue = Confirm::new(t!("ui.add_issue").as_ref())
        .with_default(false)
        .prompt()
        .ok()?;

    if !add_issue {
        return Some(None);
    }

    let num_str = Text::new(t!("ui.input_issue").as_ref())
        .with_validator(|input: &str| {
            input
                .trim()
                .parse::<u32>()
                .map(|_| Validation::Valid)
                .map_err(|_| t!("ui.issue_invalid").to_string().into())
        })
        .prompt_skippable()
        .ok()??;

    let num = num_str.trim().parse().ok()?;

    Some(Some(num))
}
