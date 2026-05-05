use crate::prelude::{CommitTagType, EditorMode};
use crate::ui::vim::app::{App, Step};
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

/// 事件处理
/// 轮询终端事件（仅响应 Press 事件，忽略 Release/Repeat）
pub fn poll_event() -> Option<KeyEvent> {
    if event::poll(Duration::from_millis(100)).ok()?
        && let Event::Key(key) = event::read().ok()?
        && key.kind == event::KeyEventKind::Press
    {
        return Some(key);
    }
    None
}

/// 处理按键事件，修改 App 状态
///
/// 采用 lazygit 式查看/编辑模式分发：
/// 1. 全局快捷键（Ctrl+S）优先处理
/// 2. 编辑模式（editing=true）：所有字符作为文本输入
/// 3. 查看模式（editing=false）：字符映射为 vim 快捷键
pub fn handle_key(key: KeyEvent, app: &mut App) {
    // 全局快捷键：Ctrl+S 推进步骤并重置编辑状态
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
        app.editing = false;
        app.step = app.step.next();
        return;
    }

    // 编辑模式：处理文本输入、光标移动、退格、确认和取消
    if app.editing {
        match key.code {
            KeyCode::Left if app.cursor > 0 => {
                app.cursor -= 1;
            }
            KeyCode::Right => {
                let len = match app.step {
                    Step::InputTitle => app.title.len(),
                    Step::InputIssue => app.issue_num.len(),
                    _ => 0,
                };
                if app.cursor < len {
                    app.cursor += 1;
                }
            }
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.ai_loading = true;
            }
            KeyCode::Char(c) => handle_char(c, app),
            KeyCode::Backspace => handle_backspace(app),
            KeyCode::Enter => {
                app.editing = false;
                app.cursor = 0;
                app.step = app.step.next();
            }
            KeyCode::Esc => {
                app.editing = false;
                app.cursor = 0;
            }
            _ => {}
        }
        return;
    }

    // 查看模式：vim 快捷键全局生效
    match key.code {
        KeyCode::Enter => handle_enter(app),
        KeyCode::Esc => handle_esc(app),
        KeyCode::Backspace => handle_backspace(app),

        // 方向键：始终作为导航
        KeyCode::Left => {
            app.step = app.step.prev();
        }
        KeyCode::Right => {
            app.step = app.step.next();
        }
        KeyCode::Down => handle_down(app),
        KeyCode::Up => handle_up(app),

        // vim 字符快捷键
        KeyCode::Char(c) => match c {
            'h' => {
                app.step = app.step.prev();
            }
            'l' => {
                app.step = app.step.next();
            }
            'j' => handle_down(app),
            'k' => handle_up(app),
            '/' => handle_search_start(app),
            'y' => handle_confirm(app),
            'n' => handle_reedit(app),
            'q' => handle_quit(app),
            '1'..='5' => {
                let idx = (c as usize) - ('1' as usize);
                app.step = Step::all()[idx];
            }
            _ => {}
        },
        _ => {}
    }
}

fn handle_down(app: &mut App) {
    match app.step {
        Step::SelectType => {
            let max = if app.searching {
                filtered_type_count(app)
            } else {
                CommitTagType::ALL.len()
            };
            if max == 0 {
                return;
            }
            app.selected_type_index = (app.selected_type_index + 1) % max;
        }
        Step::SelectBody => {
            let max = if app.searching {
                filtered_body_count(app)
            } else {
                EditorMode::ALL.len()
            };
            if max == 0 {
                return;
            }
            app.selected_body_index = (app.selected_body_index + 1) % max;
        }
        _ => {}
    }
}

fn handle_up(app: &mut App) {
    match app.step {
        Step::SelectType => {
            let max = if app.searching {
                filtered_type_count(app)
            } else {
                CommitTagType::ALL.len()
            };
            if max == 0 {
                return;
            }
            // 循环：到顶部后跳回底部
            app.selected_type_index = (app.selected_type_index + max - 1) % max;
        }
        Step::SelectBody => {
            let max = if app.searching {
                filtered_body_count(app)
            } else {
                EditorMode::ALL.len()
            };
            if max == 0 {
                return;
            }
            app.selected_body_index = (app.selected_body_index + max - 1) % max;
        }
        _ => {}
    }
}

fn handle_enter(app: &mut App) {
    match app.step {
        Step::SelectType => {
            app.type_selected = true;
            app.searching = false;
            app.filter_text.clear();
            app.step = app.step.next();
        }
        Step::InputTitle => {
            app.editing = true;
            app.cursor = app.title.len();
        }
        Step::SelectBody => {
            app.body_selected = true;
            app.searching = false;
            app.filter_text.clear();
            let mode = EditorMode::ALL.get(app.selected_body_index).copied();
            app.pending_editor = mode;
        }
        Step::InputIssue => {
            app.editing = true;
            app.cursor = app.issue_num.len();
        }
        Step::Preview => {
            app.confirmed = true;
            app.quit = true;
        }
    }
}

/// Esc 层级递退：搜索模式 → 清空输入 → 退出
fn handle_esc(app: &mut App) {
    match app.step {
        Step::SelectType | Step::SelectBody => {
            if app.searching {
                app.searching = false;
                app.filter_text.clear();
            } else {
                app.confirmed = false;
                app.quit = true;
            }
        }
        Step::InputTitle => {
            if !app.title.is_empty() {
                app.title.clear();
            } else {
                app.confirmed = false;
                app.quit = true;
            }
        }
        Step::InputIssue => {
            if !app.issue_num.is_empty() {
                app.issue_num.clear();
            } else {
                app.confirmed = false;
                app.quit = true;
            }
        }
        Step::Preview => {
            app.confirmed = false;
            app.quit = true;
        }
    }
}

fn handle_search_start(app: &mut App) {
    match app.step {
        Step::SelectType | Step::SelectBody => {
            app.searching = true;
            app.filter_text.clear();
        }
        _ => {}
    }
}

fn handle_char(c: char, app: &mut App) {
    match app.step {
        Step::SelectType | Step::SelectBody if app.searching => {
            app.filter_text.push(c);
        }
        Step::InputTitle => {
            app.title.insert(app.cursor, c);
            app.cursor += 1;
        }
        Step::InputIssue => {
            app.issue_num.insert(app.cursor, c);
            app.cursor += 1;
        }
        _ => {}
    }
}

fn handle_backspace(app: &mut App) {
    match app.step {
        Step::SelectType | Step::SelectBody if app.searching => {
            app.filter_text.pop();
        }
        Step::InputTitle if app.cursor > 0 => {
            app.title.remove(app.cursor - 1);
            app.cursor -= 1;
        }
        Step::InputIssue if app.cursor > 0 => {
            app.issue_num.remove(app.cursor - 1);
            app.cursor -= 1;
        }
        _ => {}
    }
}

fn handle_confirm(app: &mut App) {
    if app.step == Step::Preview {
        app.confirmed = true;
        app.quit = true;
    }
}

fn handle_reedit(app: &mut App) {
    if app.step == Step::Preview {
        app.step = Step::SelectType;
    }
}

/// q 退出软件
///
/// 在 lazygit 模式下仅从查看模式调用（编辑模式中 q 作为字符输入），
/// 因此无需按步骤区分。
fn handle_quit(app: &mut App) {
    app.confirmed = false;
    app.quit = true;
}

fn filtered_type_count(app: &App) -> usize {
    if app.filter_text.is_empty() {
        return 7;
    }
    CommitTagType::ALL
        .iter()
        .filter(|t| {
            t.as_str().contains(&app.filter_text) || t.get_description().contains(&app.filter_text)
        })
        .count()
}

fn filtered_body_count(app: &App) -> usize {
    if app.filter_text.is_empty() {
        return 3;
    }
    EditorMode::ALL
        .iter()
        .filter(|m| m.display_label().contains(&app.filter_text))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn key_ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    // --- handle_key 分发 ---

    #[test]
    fn handle_key_h_l_navigation() {
        let mut app = App::new();
        app.step = Step::SelectType;
        handle_key(key(KeyCode::Char('l')), &mut app);
        assert_eq!(app.step, Step::InputTitle);
        handle_key(key(KeyCode::Char('h')), &mut app);
        assert_eq!(app.step, Step::SelectType);
        handle_key(key(KeyCode::Char('h')), &mut app);
        assert_eq!(app.step, Step::Preview);
    }

    #[test]
    fn handle_key_arrow_keys_navigation() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        handle_key(key(KeyCode::Left), &mut app);
        assert_eq!(app.step, Step::SelectType);
        handle_key(key(KeyCode::Right), &mut app);
        assert_eq!(app.step, Step::InputTitle);
    }

    #[test]
    fn handle_key_ctrl_s_advances_step() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        handle_key(key_ctrl(KeyCode::Char('s')), &mut app);
        assert_eq!(app.step, Step::SelectBody);
    }

    #[test]
    fn handle_key_ctrl_c_does_nothing() {
        let mut app = App::new();
        app.step = Step::SelectType;
        handle_key(key_ctrl(KeyCode::Char('c')), &mut app);
        assert!(!app.quit);
    }

    #[test]
    fn handle_key_j_k_dispatches_to_handle_down_up() {
        let mut app = App::new();
        app.step = Step::SelectType;
        handle_key(key(KeyCode::Char('j')), &mut app);
        assert_eq!(app.selected_type_index, 1);
        handle_key(key(KeyCode::Char('k')), &mut app);
        assert_eq!(app.selected_type_index, 0);
    }

    #[test]
    fn handle_key_n_y_q_act_as_vim_in_preview() {
        let mut app = App::new();
        app.step = Step::Preview;

        // n 在预览步骤触发重新编辑
        handle_key(key(KeyCode::Char('n')), &mut app);
        assert_eq!(app.step, Step::SelectType, "n 在预览步骤应回到第一步");

        // y 在预览步骤触发确认
        app.step = Step::Preview;
        handle_key(key(KeyCode::Char('y')), &mut app);
        assert!(app.confirmed, "y 在预览步骤应确认");
        assert!(app.quit);
    }

    // --- lazygit 模式：查看/编辑模式 ---

    #[test]
    fn handle_enter_enters_edit_mode_for_title() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        handle_key(key(KeyCode::Enter), &mut app);
        assert!(app.editing);
        assert_eq!(app.step, Step::InputTitle);
    }

    #[test]
    fn handle_enter_enters_edit_mode_for_issue() {
        let mut app = App::new();
        app.step = Step::InputIssue;
        handle_key(key(KeyCode::Enter), &mut app);
        assert!(app.editing);
        assert_eq!(app.step, Step::InputIssue);
    }

    #[test]
    fn edit_mode_enter_exits_and_advances() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        app.editing = true;
        handle_key(key(KeyCode::Enter), &mut app);
        assert!(!app.editing);
        assert_eq!(app.step, Step::SelectBody);
    }

    #[test]
    fn edit_mode_esc_exits_without_advancing() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        app.editing = true;
        handle_key(key(KeyCode::Esc), &mut app);
        assert!(!app.editing);
        assert_eq!(app.step, Step::InputTitle);
    }

    // --- 光标移动 ---

    #[test]
    fn edit_mode_left_moves_cursor() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        app.editing = true;
        app.title = "abc".to_string();
        app.cursor = 2;
        handle_key(key(KeyCode::Left), &mut app);
        assert_eq!(app.cursor, 1);
    }

    #[test]
    fn edit_mode_right_moves_cursor() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        app.editing = true;
        app.title = "abc".to_string();
        app.cursor = 1;
        handle_key(key(KeyCode::Right), &mut app);
        assert_eq!(app.cursor, 2);
    }

    #[test]
    fn edit_mode_left_at_zero_no_op() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        app.editing = true;
        app.cursor = 0;
        handle_key(key(KeyCode::Left), &mut app);
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn edit_mode_right_at_end_no_op() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        app.editing = true;
        app.title = "abc".to_string();
        app.cursor = 3;
        handle_key(key(KeyCode::Right), &mut app);
        assert_eq!(app.cursor, 3);
    }

    #[test]
    fn edit_mode_char_inserts_at_cursor() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        app.editing = true;
        app.title = "ac".to_string();
        app.cursor = 1;
        handle_key(key(KeyCode::Char('b')), &mut app);
        assert_eq!(app.title, "abc");
        assert_eq!(app.cursor, 2);
    }

    #[test]
    fn edit_mode_backspace_deletes_before_cursor() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        app.editing = true;
        app.title = "abc".to_string();
        app.cursor = 2;
        handle_key(key(KeyCode::Backspace), &mut app);
        assert_eq!(app.title, "ac");
        assert_eq!(app.cursor, 1);
    }

    #[test]
    fn edit_mode_backspace_at_zero_no_op() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        app.editing = true;
        app.title = "abc".to_string();
        app.cursor = 0;
        handle_key(key(KeyCode::Backspace), &mut app);
        assert_eq!(app.title, "abc");
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn edit_mode_esc_resets_cursor() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        app.editing = true;
        app.title = "abc".to_string();
        app.cursor = 2;
        handle_key(key(KeyCode::Esc), &mut app);
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn edit_mode_enter_resets_cursor() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        app.editing = true;
        app.title = "abc".to_string();
        app.cursor = 2;
        handle_key(key(KeyCode::Enter), &mut app);
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn edit_mode_all_chars_are_text_input() {
        let vim_keys = ['h', 'j', 'k', 'l', 'y', 'n', 'q', '/'];
        for c in vim_keys {
            let mut app = App::new();
            app.step = Step::InputTitle;
            app.editing = true;
            handle_key(key(KeyCode::Char(c)), &mut app);
            assert_eq!(app.title, c.to_string(), "字符 '{c}' 应被输入到标题");
            assert!(!app.quit, "字符 '{c}' 不应触发退出");
            assert!(app.editing, "字符 '{c}' 不应退出编辑模式");
        }
    }

    #[test]
    fn edit_mode_q_types_char_not_quit() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        app.editing = true;
        handle_key(key(KeyCode::Char('q')), &mut app);
        assert_eq!(app.title, "q");
        assert!(!app.quit);
    }

    #[test]
    fn view_mode_h_navigates_prev() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        app.editing = false;
        handle_key(key(KeyCode::Char('h')), &mut app);
        assert_eq!(app.step, Step::SelectType);
    }

    #[test]
    fn view_mode_q_quits() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        app.editing = false;
        handle_key(key(KeyCode::Char('q')), &mut app);
        assert!(app.quit);
    }

    #[test]
    fn view_mode_n_does_nothing_in_input_steps() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        app.editing = false;
        handle_key(key(KeyCode::Char('n')), &mut app);
        assert!(!app.quit);
        assert_eq!(app.step, Step::InputTitle);
        assert!(app.title.is_empty());
    }

    #[test]
    fn ctrl_s_resets_editing_and_advances() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        app.editing = true;
        handle_key(key_ctrl(KeyCode::Char('s')), &mut app);
        assert!(!app.editing);
        assert_eq!(app.step, Step::SelectBody);
    }

    // --- handle_confirm / handle_quit ---

    #[test]
    fn handle_confirm_sets_confirmed_and_quit() {
        let mut app = App::new();
        app.step = Step::Preview;
        handle_confirm(&mut app);
        assert!(app.confirmed);
        assert!(app.quit);
    }

    #[test]
    fn handle_confirm_ignored_outside_preview() {
        let mut app = App::new();
        app.step = Step::SelectType;
        handle_confirm(&mut app);
        assert!(!app.confirmed);
        assert!(!app.quit);
    }

    #[test]
    fn handle_quit_sets_quit_without_confirm() {
        let mut app = App::new();
        app.step = Step::Preview;
        handle_quit(&mut app);
        assert!(!app.confirmed);
        assert!(app.quit);
    }

    #[test]
    fn handle_quit_works_from_select_type() {
        let mut app = App::new();
        app.step = Step::SelectType;
        handle_quit(&mut app);
        assert!(app.quit);
    }

    #[test]
    fn handle_quit_works_from_select_body() {
        let mut app = App::new();
        app.step = Step::SelectBody;
        handle_quit(&mut app);
        assert!(app.quit);
    }

    #[test]
    fn handle_quit_works_from_all_steps() {
        let mut app = App::new();
        for step in Step::all() {
            app.step = step;
            app.quit = false;
            app.confirmed = false;
            handle_quit(&mut app);
            assert!(app.quit, "handle_quit 应从 {step:?} 退出");
            assert!(!app.confirmed);
        }
    }

    // --- handle_enter ---

    #[test]
    fn handle_enter_select_type_advances_and_sets_flag() {
        let mut app = App::new();
        app.step = Step::SelectType;
        app.searching = true;
        app.filter_text = "feat".to_string();
        handle_enter(&mut app);
        assert!(app.type_selected);
        assert!(!app.searching);
        assert!(app.filter_text.is_empty());
        assert_eq!(app.step, Step::InputTitle);
    }

    #[test]
    fn handle_enter_select_body_sets_pending_editor() {
        let mut app = App::new();
        app.step = Step::SelectBody;
        app.selected_body_index = 1;
        handle_enter(&mut app);
        assert!(app.body_selected);
        assert_eq!(app.pending_editor, Some(EditorMode::DefaultEditor));
        assert_eq!(app.step, Step::SelectBody);
    }

    #[test]
    fn handle_enter_preview_sets_quit() {
        let mut app = App::new();
        app.step = Step::Preview;
        handle_enter(&mut app);
        assert!(app.quit);
    }

    // --- handle_down / handle_up ---

    #[test]
    fn handle_down_wraps_around_in_select_type() {
        let mut app = App::new();
        app.step = Step::SelectType;
        // 移到最后一项
        for _ in 0..CommitTagType::ALL.len() - 1 {
            handle_down(&mut app);
        }
        assert_eq!(app.selected_type_index, CommitTagType::ALL.len() - 1);
        // 再按一次，循环回顶部
        handle_down(&mut app);
        assert_eq!(app.selected_type_index, 0);
    }

    #[test]
    fn handle_up_wraps_around_in_select_type() {
        let mut app = App::new();
        app.step = Step::SelectType;
        // 在顶部按一次，循环到底部
        handle_up(&mut app);
        assert_eq!(app.selected_type_index, CommitTagType::ALL.len() - 1);
    }

    #[test]
    fn handle_down_wraps_around_in_select_body() {
        let mut app = App::new();
        app.step = Step::SelectBody;
        for _ in 0..EditorMode::ALL.len() - 1 {
            handle_down(&mut app);
        }
        assert_eq!(app.selected_body_index, EditorMode::ALL.len() - 1);
        // 再按一次，循环回顶部
        handle_down(&mut app);
        assert_eq!(app.selected_body_index, 0);
    }

    #[test]
    fn handle_up_wraps_around_in_select_body() {
        let mut app = App::new();
        app.step = Step::SelectBody;
        // 在顶部按一次，循环到底部
        handle_up(&mut app);
        assert_eq!(app.selected_body_index, EditorMode::ALL.len() - 1);
    }

    #[test]
    fn handle_down_ignored_in_input_steps() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        let original_step = app.step;
        handle_down(&mut app);
        assert_eq!(app.step, original_step);
    }

    // --- handle_char / handle_backspace ---

    #[test]
    fn handle_char_appends_to_title() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        handle_char('H', &mut app);
        handle_char('i', &mut app);
        assert_eq!(app.title, "Hi");
    }

    #[test]
    fn handle_char_appends_to_issue() {
        let mut app = App::new();
        app.step = Step::InputIssue;
        handle_char('4', &mut app);
        handle_char('2', &mut app);
        assert_eq!(app.issue_num, "42");
    }

    #[test]
    fn handle_char_appends_to_filter_when_searching() {
        let mut app = App::new();
        app.step = Step::SelectType;
        app.searching = true;
        handle_char('f', &mut app);
        handle_char('e', &mut app);
        assert_eq!(app.filter_text, "fe");
    }

    #[test]
    fn handle_char_ignored_in_list_without_search() {
        let mut app = App::new();
        app.step = Step::SelectType;
        handle_char('x', &mut app);
        assert!(app.filter_text.is_empty());
    }

    #[test]
    fn handle_backspace_removes_from_title() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        app.title = "Hi".to_string();
        app.cursor = 2;
        handle_backspace(&mut app);
        assert_eq!(app.title, "H");
        assert_eq!(app.cursor, 1);
    }

    #[test]
    fn handle_backspace_removes_from_filter() {
        let mut app = App::new();
        app.step = Step::SelectType;
        app.searching = true;
        app.filter_text = "abc".to_string();
        handle_backspace(&mut app);
        assert_eq!(app.filter_text, "ab");
    }

    // --- handle_esc ---

    #[test]
    fn handle_esc_clears_title() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        app.title = "Hello".to_string();
        handle_esc(&mut app);
        assert!(app.title.is_empty());
        assert!(!app.quit);
    }

    #[test]
    fn handle_esc_quits_when_title_empty() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        handle_esc(&mut app);
        assert!(app.quit);
        assert!(!app.confirmed);
    }

    #[test]
    fn handle_esc_clears_issue() {
        let mut app = App::new();
        app.step = Step::InputIssue;
        app.issue_num = "42".to_string();
        handle_esc(&mut app);
        assert!(app.issue_num.is_empty());
        assert!(!app.quit);
    }

    #[test]
    fn handle_esc_quits_when_issue_empty() {
        let mut app = App::new();
        app.step = Step::InputIssue;
        handle_esc(&mut app);
        assert!(app.quit);
    }

    #[test]
    fn handle_esc_exits_preview() {
        let mut app = App::new();
        app.step = Step::Preview;
        handle_esc(&mut app);
        assert!(app.quit);
        assert!(!app.confirmed);
    }

    #[test]
    fn handle_esc_clears_search_in_list() {
        let mut app = App::new();
        app.step = Step::SelectType;
        app.searching = true;
        app.filter_text = "feat".to_string();
        handle_esc(&mut app);
        assert!(!app.searching);
        assert!(app.filter_text.is_empty());
        assert!(!app.quit);
    }

    #[test]
    fn handle_esc_quits_from_list_without_search() {
        let mut app = App::new();
        app.step = Step::SelectType;
        handle_esc(&mut app);
        assert!(app.quit);
    }

    // --- handle_search_start ---

    #[test]
    fn handle_search_start_in_select_type() {
        let mut app = App::new();
        app.step = Step::SelectType;
        handle_search_start(&mut app);
        assert!(app.searching);
        assert!(app.filter_text.is_empty());
    }

    #[test]
    fn handle_search_start_ignored_in_input_steps() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        handle_search_start(&mut app);
        assert!(!app.searching);
    }

    // --- handle_reedit ---

    #[test]
    fn handle_reedit_goes_to_first_step() {
        let mut app = App::new();
        app.step = Step::Preview;
        handle_reedit(&mut app);
        assert_eq!(app.step, Step::SelectType);
    }

    #[test]
    fn handle_reedit_ignored_outside_preview() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        handle_reedit(&mut app);
        assert_eq!(app.step, Step::InputTitle);
    }

    // --- 过滤计数 ---

    #[test]
    fn filtered_type_count_no_filter() {
        let app = App::new();
        assert_eq!(filtered_type_count(&app), 7);
    }

    #[test]
    fn filtered_type_count_with_filter() {
        let mut app = App::new();
        app.filter_text = "feat".to_string();
        assert!(filtered_type_count(&app) >= 1);
    }

    #[test]
    fn filtered_type_count_no_match() {
        let mut app = App::new();
        app.filter_text = "zzzzz".to_string();
        assert_eq!(filtered_type_count(&app), 0);
    }

    #[test]
    fn filtered_body_count_no_filter() {
        let app = App::new();
        assert_eq!(filtered_body_count(&app), 3);
    }

    #[test]
    fn filtered_body_count_with_filter() {
        rust_i18n::set_locale("en");
        let mut app = App::new();
        app.filter_text = "Terminal".to_string();
        assert_eq!(filtered_body_count(&app), 1);
    }

    #[test]
    fn handle_down_wraps_with_search_filter() {
        let mut app = App::new();
        app.step = Step::SelectType;
        app.searching = true;
        app.filter_text = "feat".to_string();
        let count = filtered_type_count(&app);
        if count > 1 {
            for _ in 0..count - 1 {
                handle_down(&mut app);
            }
            handle_down(&mut app);
            assert_eq!(app.selected_type_index, 0);
        }
    }

    #[test]
    fn handle_down_empty_search_result_no_panic() {
        let mut app = App::new();
        app.step = Step::SelectType;
        app.searching = true;
        app.filter_text = "zzzzz".to_string();
        handle_down(&mut app);
        assert_eq!(app.selected_type_index, 0);
    }

    // --- 数字键 1-5 快速跳转步骤 ---

    #[test]
    fn number_keys_jump_to_step() {
        let cases = [
            ('1', Step::SelectType),
            ('2', Step::InputTitle),
            ('3', Step::SelectBody),
            ('4', Step::InputIssue),
            ('5', Step::Preview),
        ];
        for (ch, expected_step) in cases {
            let mut app = App::new();
            app.step = Step::Preview;
            handle_key(key(KeyCode::Char(ch)), &mut app);
            assert_eq!(
                app.step, expected_step,
                "按 '{ch}' 应跳转到 {expected_step:?}"
            );
        }
    }

    #[test]
    fn number_keys_in_edit_mode_types_text() {
        // 编辑模式下数字应作为文本输入
        let mut app = App::new();
        app.step = Step::InputTitle;
        app.editing = true;
        handle_key(key(KeyCode::Char('1')), &mut app);
        assert_eq!(app.title, "1");
        assert_eq!(app.step, Step::InputTitle);
    }

    #[test]
    fn number_key_6_and_0_do_nothing() {
        let mut app = App::new();
        app.step = Step::SelectType;
        handle_key(key(KeyCode::Char('0')), &mut app);
        assert_eq!(app.step, Step::SelectType);
        handle_key(key(KeyCode::Char('6')), &mut app);
        assert_eq!(app.step, Step::SelectType);
    }
}
