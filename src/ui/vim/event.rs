use std::time::Duration;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crate::prelude::{CommitTagType, EditorMode};
use crate::ui::vim::app::{App, Step};

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

/// 处理按键事件，修改 APP 状态
pub fn handle_key(key: KeyEvent, app: &mut App) {
    // 全局快捷键
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
        app.step = app.step.next();
        return;
    }

    match key.code {
        // 全局导航
        KeyCode::Char('h') | KeyCode::Left => {
            app.step = app.step.prev();
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.step = app.step.next();
        }

        // 各步骤分别处理
        KeyCode::Char('j') | KeyCode::Down => handle_down(app),
        KeyCode::Char('k') | KeyCode::Up => handle_up(app),
        KeyCode::Enter => handle_enter(app),
        KeyCode::Esc => handle_esc(app),
        KeyCode::Char('/') => handle_search_start(app),
        KeyCode::Char('y') => handle_confirm(app),
        KeyCode::Char('n') => handle_reedit(app),
        KeyCode::Char('q') => handle_quit(app),
        KeyCode::Backspace => handle_backspace(app),
        KeyCode::Char(c) => handle_char(c, app),
        _ => {}
    }
}

fn handle_down(app: &mut App) {
    match app.step {
        Step::SelectType => {
            let max = if app.searching { filtered_type_count(app) } else { 7 };
            if app.selected_type_index < max.saturating_sub(1) {
                app.selected_type_index += 1;
            }
        }
        Step::SelectBody => {
            let max = if app.searching { filtered_body_count(app) } else { 3 };
            if app.selected_body_index < max.saturating_sub(1) {
                app.selected_body_index += 1;
            }
        }
        _ => {}
    }
}

fn handle_up(app: &mut App) {
    match app.step {
        Step::SelectType => {
            if app.selected_type_index > 0 {
                app.selected_type_index -= 1;
            }
        }
        Step::SelectBody => {
            if app.selected_body_index > 0 {
                app.selected_body_index -= 1;
            }
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
            app.step = app.step.next();
        }
        Step::SelectBody => {
            app.body_selected = true;
            app.searching = false;
            app.filter_text.clear();
            let mode = EditorMode::ALL.get(app.selected_body_index).copied();
            app.pending_editor = mode;
            // 不在此处推进步骤，由事件循环在编辑器完成后处理
        }
        Step::InputIssue => {
            app.step = app.step.next();
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
        Step::SelectType | Step::SelectBody => {
            if app.searching {
                app.filter_text.push(c);
            }
        }
        Step::InputTitle => {
            app.title.push(c);
        }
        Step::InputIssue => {
            app.issue_num.push(c);
        }
        _ => {}
    }
}

fn handle_backspace(app: &mut App) {
    match app.step {
        Step::SelectType | Step::SelectBody => {
            if app.searching {
                app.filter_text.pop();
            }
        }
        Step::InputTitle => {
            app.title.pop();
        }
        Step::InputIssue => {
            app.issue_num.pop();
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

/// q 在非输入步骤直接退出（输入步骤中 q 正常输入字符）
fn handle_quit(app: &mut App) {
    match app.step {
        Step::SelectType | Step::SelectBody | Step::Preview => {
            app.confirmed = false;
            app.quit = true;
        }
        _ => {}
    }
}

fn filtered_type_count(app: &App) -> usize {
    if app.filter_text.is_empty() {
        return 7;
    }
    CommitTagType::ALL
        .iter()
        .filter(|t| {
            t.as_str().contains(&app.filter_text)
                || t.get_description().contains(&app.filter_text)
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
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use super::*;

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
        app.step = Step::InputTitle;
        handle_key(key(KeyCode::Char('h')), &mut app);
        assert_eq!(app.step, Step::SelectType);
        handle_key(key(KeyCode::Char('l')), &mut app);
        assert_eq!(app.step, Step::InputTitle);
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
    fn handle_quit_ignored_in_input_steps() {
        let mut app = App::new();
        app.step = Step::InputTitle;
        handle_quit(&mut app);
        assert!(!app.quit);

        app.step = Step::InputIssue;
        handle_quit(&mut app);
        assert!(!app.quit);
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
    fn handle_enter_input_issue_advances() {
        let mut app = App::new();
        app.step = Step::InputIssue;
        handle_enter(&mut app);
        assert_eq!(app.step, Step::Preview);
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
    fn handle_down_clamps_at_bottom() {
        let mut app = App::new();
        app.step = Step::SelectType;
        for _ in 0..20 {
            handle_down(&mut app);
        }
        assert_eq!(app.selected_type_index, 6);
    }

    #[test]
    fn handle_up_clamps_at_top() {
        let mut app = App::new();
        app.step = Step::SelectType;
        app.selected_type_index = 5;
        for _ in 0..20 {
            handle_up(&mut app);
        }
        assert_eq!(app.selected_type_index, 0);
    }

    #[test]
    fn handle_down_body_clamps_at_bottom() {
        let mut app = App::new();
        app.step = Step::SelectBody;
        for _ in 0..20 {
            handle_down(&mut app);
        }
        assert_eq!(app.selected_body_index, 2);
    }

    #[test]
    fn handle_up_body_clamps_at_top() {
        let mut app = App::new();
        app.step = Step::SelectBody;
        app.selected_body_index = 2;
        for _ in 0..20 {
            handle_up(&mut app);
        }
        assert_eq!(app.selected_body_index, 0);
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
        handle_backspace(&mut app);
        assert_eq!(app.title, "H");
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
        let mut app = App::new();
        app.filter_text = "Terminal".to_string();
        assert_eq!(filtered_body_count(&app), 1);
    }
}