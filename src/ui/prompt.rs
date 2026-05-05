use crate::logic::ai::provider::AiSuggestion;
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
    Select::new(t!("ui.select_type").as_ref(), CommitTagType::ALL.to_vec())
        .prompt_skippable()
        .ok()?
}

/// 输入 commit title
fn input_title() -> Option<String> {
    Text::new(t!("ui.input_title").as_ref())
        .with_validator(|input: &str| match validate_title(input) {
            Ok(_) => Ok(Validation::Valid),
            Err(TitleError::Empty) => Ok(Validation::Invalid(
                t!("ui.title_empty_err").to_string().into(),
            )),
            Err(TitleError::TooLong { width, max }) => Ok(Validation::Invalid(
                t!("ui.title_too_long_err", width = width, max = max)
                    .to_string()
                    .into(),
            )),
            Err(TitleError::EndsWithPeriod) => Ok(Validation::Invalid(
                t!("ui.title_period_err").to_string().into(),
            )),
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

/// 带 AI 预填充的交互式问答
///
/// AI 生成的 type / title / body 预填为默认值，用户仍可修改
pub fn run_prompt_with_suggestion(suggestion: AiSuggestion) -> Option<CommitMessageEntity> {
    loop {
        let msg = collect_inputs_with_suggestion(&suggestion)?;

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

fn collect_inputs_with_suggestion(suggestion: &AiSuggestion) -> Option<CommitMessageEntity> {
    let commit_tag_type = select_type_with_default(suggestion.commit_type)?;
    let title = input_title_with_default(&suggestion.title)?;
    let body = input_body_with_default(suggestion.body.as_deref())?;
    let issue_num = input_issue()?;

    Some(CommitMessageEntity {
        commit_tag_type,
        title,
        body,
        issue_num,
    })
}

/// 类型选择，AI 建议的类型预选中
fn select_type_with_default(default: CommitTagType) -> Option<CommitTagType> {
    let default_index = CommitTagType::ALL
        .iter()
        .position(|t| *t == default)
        .unwrap_or(0);

    Select::new(t!("ui.select_type").as_ref(), CommitTagType::ALL.to_vec())
        .with_starting_cursor(default_index)
        .prompt_skippable()
        .ok()?
}

/// 标题输入，AI 建议的标题预填
fn input_title_with_default(default: &str) -> Option<String> {
    Text::new(t!("ui.input_title").as_ref())
        .with_default(default)
        .with_validator(|input: &str| match validate_title(input) {
            Ok(_) => Ok(Validation::Valid),
            Err(TitleError::Empty) => Ok(Validation::Invalid(
                t!("ui.title_empty_err").to_string().into(),
            )),
            Err(TitleError::TooLong { width, max }) => Ok(Validation::Invalid(
                t!("ui.title_too_long_err", width = width, max = max)
                    .to_string()
                    .into(),
            )),
            Err(TitleError::EndsWithPeriod) => Ok(Validation::Invalid(
                t!("ui.title_period_err").to_string().into(),
            )),
        })
        .prompt_skippable()
        .ok()?
}

/// 正文输入，AI 建议的正文预填
fn input_body_with_default(default_body: Option<&str>) -> Option<Option<String>> {
    let add_body = Confirm::new(t!("ui.add_body").as_ref())
        .with_default(true)
        .prompt()
        .ok()?;

    if !add_body {
        return Some(None);
    }

    // 如果 AI 生成了 body，使用默认编辑器打开预填内容
    match default_body {
        Some(body) => {
            let result = inquire::Editor::new(t!("ui.body_hint_template").as_ref())
                .with_predefined_text(body)
                .prompt_skippable()
                .ok()??;
            Some(Some(result))
        }
        None => input_body(),
    }
}
