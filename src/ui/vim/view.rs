use crate::prelude::{CommitTagType, EditorMode};
use crate::ui::vim::app::{App, Step};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Color, Direction};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use rust_i18n::t;

/// 渲染模块
pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // 步骤栏
            Constraint::Min(1),    // 内容区
            Constraint::Length(2), // 底部提示栏
        ])
        .split(f.area());

    render_step_bar(f, app, chunks[0]);
    render_content(f, app, chunks[1]);
    render_help_bar(f, app, chunks[2]);
}

/// 渲染顶部步骤栏
fn render_step_bar(f: &mut Frame, app: &App, area: Rect) {
    let steps = Step::all();
    let spans: Vec<Span> = steps
        .iter()
        .flat_map(|step| {
            let style = if *step == app.step {
                Style::default().fg(Color::Black).bg(Color::White)
            } else if is_step_completed(app, step) {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Gray)
            };
            vec![Span::styled(format!(" [{}] ", step.label()), style)]
        })
        .collect();

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(Color::DarkGray));
    f.render_widget(paragraph, area);
}

/// 判断步骤是否已完成
fn is_step_completed(app: &App, step: &Step) -> bool {
    match step {
        Step::SelectType => app.type_selected,
        Step::InputTitle => !app.title.trim().is_empty(),
        Step::SelectBody => app.body_selected,
        Step::InputIssue => !app.issue_num.trim().is_empty(),
        Step::Preview => false,
    }
}

/// 根据当前步骤渲染内容区
fn render_content(f: &mut Frame, app: &App, area: Rect) {
    match app.step {
        Step::SelectType => render_select_type(f, app, area),
        Step::InputTitle => render_input_title(f, app, area),
        Step::SelectBody => render_select_body(f, app, area),
        Step::InputIssue => render_input_issue(f, app, area),
        Step::Preview => render_preview(f, app, area),
    }
}

/// 步骤 1：类型选择列表
fn render_select_type(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = CommitTagType::ALL
        .iter()
        .filter(|t| {
            if app.filter_text.is_empty() {
                return true;
            }
            t.as_str().contains(&app.filter_text) || t.get_description().contains(&app.filter_text)
        })
        .enumerate()
        .map(|(i, t)| {
            let prefix = if i == app.selected_type_index {
                "▸ "
            } else {
                "  "
            };
            let content = format!("{}{} - {}", prefix, t.as_str(), t.get_description());
            let style = if i == app.selected_type_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(t!("ui.select_type").to_string())
            .borders(Borders::ALL),
    );

    f.render_widget(list, area);
}

/// 步骤 2：标题输入
fn render_input_title(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(1), // 宽度计数器
        ])
        .split(area);

    let border_style = if app.editing {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let title_text = if app.editing {
        t!("vim.title_editing").to_string()
    } else {
        t!("vim.title_viewing").to_string()
    };

    let input = Paragraph::new(app.title.as_str()).block(
        Block::default()
            .title(title_text)
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    f.render_widget(input, chunks[0]);

    // 宽度计数器
    use unicode_width::UnicodeWidthStr;
    let width = UnicodeWidthStr::width(app.title.as_str());
    let color = if width > 50 { Color::Red } else { Color::White };
    let counter = Paragraph::new(t!("vim.width_counter", width = width).to_string())
        .style(Style::default().fg(color));
    f.render_widget(counter, chunks[1]);
}

/// 步骤 3：正文编辑方式选择
fn render_select_body(f: &mut Frame, app: &App, area: Rect) {
    // 上下切分: 选择列表 (60%) + 预览区 (40%)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // 上方: 编辑方式选择栏
    let items: Vec<ListItem> = EditorMode::ALL
        .iter()
        .filter(|m| {
            if app.filter_text.is_empty() {
                return true;
            }
            m.display_label().contains(&app.filter_text)
        })
        .enumerate()
        .map(|(i, m)| {
            let prefix = if i == app.selected_body_index {
                "▸ "
            } else {
                "  "
            };
            let content = format!("{}{}", prefix, m.display_label());
            let style = if i == app.selected_body_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(t!("vim.select_body_mode").to_string())
            .borders(Borders::ALL),
    );
    f.render_widget(list, chunks[0]);

    // 下方: 正文内容预览
    let preview_content = match &app.body {
        Some(text) if !text.trim().is_empty() => {
            Paragraph::new(text.as_str()).wrap(Wrap { trim: false })
        }
        _ => Paragraph::new(t!("vim.body_not_edited").to_string())
            .style(Style::default().fg(Color::DarkGray)),
    };

    let preview = preview_content.block(
        Block::default()
            .title(t!("vim.body_preview").to_string())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightBlue)),
    );
    f.render_widget(preview, chunks[1]);
}

/// 步骤 4：Issue 输入
fn render_input_issue(f: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.editing {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let title_text = if app.editing {
        t!("vim.issue_editing").to_string()
    } else {
        t!("vim.issue_viewing").to_string()
    };

    let input = Paragraph::new(app.issue_num.as_str()).block(
        Block::default()
            .title(title_text)
            .borders(Borders::ALL)
            .border_style(border_style),
    );
    f.render_widget(input, area);
}

/// 步骤 5：预览确认
fn render_preview(f: &mut Frame, app: &App, area: Rect) {
    let entity = match app.to_entity() {
        Some(e) => e,
        None => {
            let warning = Paragraph::new(t!("vim.preview_title_empty").to_string())
                .style(Style::default().fg(Color::Red))
                .block(
                    Block::default()
                        .title(t!("vim.step_preview").to_string())
                        .borders(Borders::ALL),
                );
            f.render_widget(warning, area);
            return;
        }
    };

    let type_str = format!("{}:", entity.commit_tag_type.as_str());
    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            t!("vim.confirm_hint").to_string(),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                type_str,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}", entity.title),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];
    if let Some(body) = &entity.body {
        lines.push(Line::from(""));
        for line in body.lines() {
            lines.push(Line::from(line.to_string()));
        }
    }

    if let Some(issue) = entity.issue_num {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("#{}", issue),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
    }

    let preview = Paragraph::new(lines)
        .block(
            Block::default()
                .title(t!("vim.step_preview").to_string())
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(preview, area);
}

/// 渲染底部快捷键提示栏
fn render_help_bar(f: &mut Frame, app: &App, area: Rect) {
    let help_text: String = match app.step {
        Step::SelectType | Step::SelectBody => {
            if app.searching {
                t!("vim.search_help", text = app.filter_text).to_string()
            } else {
                t!("vim.help_browse").to_string()
            }
        }
        Step::InputTitle => {
            if app.editing {
                t!("vim.help_title_edit").to_string()
            } else {
                t!("vim.help_title_view").to_string()
            }
        }
        Step::InputIssue => {
            if app.editing {
                t!("vim.help_issue_edit").to_string()
            } else {
                t!("vim.help_issue_view").to_string()
            }
        }
        Step::Preview => t!("vim.help_preview").to_string(),
    };

    let paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::vim::app::App;
    // --- is_step_completed ---

    #[test]
    fn step_completed_type_not_selected() {
        let app = App::new();
        assert!(!is_step_completed(&app, &Step::SelectType));
    }

    #[test]
    fn step_completed_type_selected() {
        let mut app = App::new();
        app.type_selected = true;
        assert!(is_step_completed(&app, &Step::SelectType));
    }

    #[test]
    fn step_completed_title_non_empty() {
        let mut app = App::new();
        app.title = "Hello".to_string();
        assert!(is_step_completed(&app, &Step::InputTitle));
    }

    #[test]
    fn step_completed_title_empty() {
        let app = App::new();
        assert!(!is_step_completed(&app, &Step::InputTitle));
    }

    #[test]
    fn step_completed_title_whitespace_only() {
        let mut app = App::new();
        app.title = "   ".to_string();
        assert!(!is_step_completed(&app, &Step::InputTitle));
    }

    #[test]
    fn step_completed_body_not_selected() {
        let app = App::new();
        assert!(!is_step_completed(&app, &Step::SelectBody));
    }

    #[test]
    fn step_completed_body_selected() {
        let mut app = App::new();
        app.body_selected = true;
        assert!(is_step_completed(&app, &Step::SelectBody));
    }

    #[test]
    fn step_completed_issue_non_empty() {
        let mut app = App::new();
        app.issue_num = "42".to_string();
        assert!(is_step_completed(&app, &Step::InputIssue));
    }

    #[test]
    fn step_completed_issue_empty() {
        let app = App::new();
        assert!(!is_step_completed(&app, &Step::InputIssue));
    }

    #[test]
    fn step_completed_preview_always_false() {
        let mut app = App::new();
        app.step = Step::Preview;
        assert!(!is_step_completed(&app, &Step::Preview));
    }
}
