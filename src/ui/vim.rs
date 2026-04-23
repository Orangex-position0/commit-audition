mod app;
mod event;
mod view;

use std::io::stdout;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::logic::config::load_config;
use crate::prelude::{CommitMessageEntity, EditorMode};
use crate::ui::editor;

/// vim mode 的主入口
pub fn run_vim_prompt() -> Option<CommitMessageEntity> {
    // 初始化终端
    enable_raw_mode().expect("无法启用原始模式");
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .expect("无法切换到备选屏幕");
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("无法创建终端");

    let mut app = app::App::new();

    // 事件循环
    loop {
        terminal.draw(|f| view::render(f, &app)).expect("渲染失败");

        if let Some(key) = event::poll_event() {
            event::handle_key(key, &mut app);
        }

        // 处理待执行的编辑器调用
        if let Some(editor_mode) = app.pending_editor.take() {
            // 挂起 TUI
            disable_raw_mode().ok();
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )
            .ok();
            terminal.show_cursor().ok();

            // 调用编辑器
            let result = match editor_mode {
                EditorMode::TerminalInline => editor::edit_terminal_inline(),
                EditorMode::DefaultEditor => editor::edit_default_editor(),
                EditorMode::CustomEditor => {
                    let config = load_config();
                    match &config.editor.command {
                        Some(cmd) => {
                            editor::edit_custom_editor(cmd, &config.editor.extension)
                        }
                        None => {
                            eprintln!("配置文件中未指定编辑器命令，请编辑 ~/.commit-audition/config.toml");
                            None
                        }
                    }
                }
            };

            // 保存编辑结果
            match result {
                Some(Some(body)) => app.body = Some(body),
                _ => app.body = None,
            }

            // 恢复 TUI
            let mut new_stdout = std::io::stdout();
            enable_raw_mode().expect("无法启用原始模式");
            execute!(new_stdout, EnterAlternateScreen, EnableMouseCapture)
                .expect("无法切换到备选屏幕");
            let backend = CrosstermBackend::new(new_stdout);
            terminal = Terminal::new(backend).expect("无法创建终端");

            // 进入下一步
            app.step = app.step.next();
        }

        if app.quit {
            break;
        }
    }

    // 恢复终端
    disable_raw_mode().expect("无法禁用原始模式");
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .expect("无法恢复终端");
    terminal.show_cursor().expect("无法显示光标");

    // 返回结果
    if app.confirmed {
        app.to_entity()
    } else {
        None
    }
}
