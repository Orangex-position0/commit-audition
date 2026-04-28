pub mod app;
mod event;
mod view;

use std::io::stdout;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use rust_i18n::t;

use crate::logic::config::load_config;
use crate::prelude::{CommitMessageEntity, EditorMode};
use crate::ui::editor;

/// vim mode 的主入口
pub fn run_vim_prompt() -> Option<CommitMessageEntity> {
    // 初始化终端
    enable_raw_mode().expect(&t!("terminal.raw_mode_enable"));
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .expect(&t!("terminal.alt_screen_enter"));
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect(&t!("terminal.terminal_create"));

    let mut app = app::App::new();

    // 事件循环
    loop {
        terminal
            .draw(|f| view::render(f, &app))
            .expect(&t!("terminal.render_failed"));

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
                        Some(cmd) => editor::edit_custom_editor(cmd, &config.editor.extension),
                        None => {
                            eprintln!("{}", t!("terminal.no_editor_config"));
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
            enable_raw_mode().expect(&t!("terminal.raw_mode_enable"));
            execute!(new_stdout, EnterAlternateScreen, EnableMouseCapture)
                .expect(&t!("terminal.alt_screen_enter"));
            let backend = CrosstermBackend::new(new_stdout);
            terminal = Terminal::new(backend).expect(&t!("terminal.terminal_create"));

            // 进入下一步
            app.step = app.step.next();
        }

        if app.quit {
            break;
        }
    }

    // 恢复终端
    disable_raw_mode().expect(&t!("terminal.raw_mode_disable"));
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .expect(&t!("terminal.alt_screen_leave"));
    terminal.show_cursor().expect(&t!("terminal.cursor_show"));

    // 返回结果
    if app.confirmed { app.to_entity() } else { None }
}
