//! Phase 1〜3 未実装機能（Windowsライク操作、外部プロセス連携、TUIテーマ同期）統合テスト。

use vivivi_app::tui::App;
use vivivi_app::{AppKeyEvent, KeyCode, KeyModifiers};
use vivivi_core::{CharEncoding, CursorPosition, NewlineType};
use vivivi_vi::ViMode;

#[test]
fn test_windows_shift_arrow_selection_and_ctrl_shortcuts() {
    let mut app = App::new(
        "Hello World\nLine 2\nLine 3",
        None,
        CharEncoding::Utf8,
        NewlineType::Lf,
    );

    // 初期状態は Normal モード
    assert_eq!(app.mode(), ViMode::Normal);
    assert_eq!(app.cursor_pos(), CursorPosition::new(0, 0));

    // 1. Shift+Right でビジュアル選択を開始し、カーソルを右へ
    app.handle_key(AppKeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
    assert_eq!(app.mode(), ViMode::Visual);
    assert_eq!(app.cursor.anchor, Some(CursorPosition::new(0, 0)));
    assert_eq!(app.cursor_pos(), CursorPosition::new(0, 1));

    // さらに Shift+Right を2回押下
    app.handle_key(AppKeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
    app.handle_key(AppKeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
    assert_eq!(app.cursor_pos(), CursorPosition::new(0, 3));

    // 選択テキストの検証 ("Hel" または "Hell")
    let sel_text = app.get_selected_text();
    assert!(sel_text.is_some());
    assert_eq!(sel_text.unwrap(), "Hell");

    // 2. Ctrl+C でコピーして Normal モードへ復帰
    app.handle_key(AppKeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
    assert_eq!(app.mode(), ViMode::Normal);
    assert_eq!(app.cursor.anchor, None);
    assert!(app.status_message.as_ref().unwrap().contains("コピー"));

    // 3. Ctrl+A で全選択
    app.handle_key(AppKeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL));
    assert_eq!(app.mode(), ViMode::VisualLine);
    assert_eq!(app.cursor.anchor, Some(CursorPosition::new(0, 0)));
    assert_eq!(app.cursor_pos(), CursorPosition::new(2, 0));

    // 4. Ctrl+Z (Undo) / Ctrl+Y (Redo) のキー入力疎通
    app.vi.set_mode(ViMode::Normal);
    app.handle_key(AppKeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL));
    assert!(app.status_message.as_ref().unwrap().contains("取り消し"));

    app.handle_key(AppKeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL));
    assert!(app.status_message.as_ref().unwrap().contains("やり直し"));
}

#[test]
fn test_external_process_execution_and_pipe() {
    let mut app = App::new(
        "echo 42\ntest data",
        None,
        CharEncoding::Utf8,
        NewlineType::Lf,
    );

    // 1. シェルコマンド直接実行
    let res = app.execute_shell_command("echo hello");
    assert!(res.is_ok());
    assert!(res.unwrap().contains("hello"));

    // 2. 選択テキストの取得テスト
    app.cursor.pos = CursorPosition::new(0, 0);
    app.cursor.anchor = Some(CursorPosition::new(0, 0));
    app.cursor.pos = CursorPosition::new(0, 6);
    app.vi.set_mode(ViMode::Visual);

    let sel = app.get_selected_text();
    assert_eq!(sel.unwrap(), "echo 42");

    // 3. Exコマンド :! のディスパッチ
    vivivi_app::tui::event::ex::execute_ex_command(&mut app, "! echo ex_test");
    assert!(app.status_message.is_some());
    assert!(app.status_message.as_ref().unwrap().contains("ex_test"));
}

#[test]
fn test_tui_theme_synchronization_and_colors() {
    let mut app = App::new(
        "fn main() {\n    println!(\"Hello\");\n}",
        None,
        CharEncoding::Utf8,
        NewlineType::Lf,
    );

    // 初期テーマは tokyo-night (表示名: Tokyo Night)
    assert_eq!(app.active_theme.meta.name, "Tokyo Night");

    // Exコマンドで openvivi に切り替え
    vivivi_app::tui::event::ex::execute_ex_command(&mut app, "colorscheme openvivi");
    assert_eq!(app.config.appearance.theme, "OpenViVi");
    assert_eq!(app.active_theme.meta.name, "OpenViVi");

    // Ratatui カラーへの変換確認
    let kw_color = app.active_theme.syntax.keyword.to_ratatui();
    assert!(matches!(kw_color, ratatui::style::Color::Rgb(..)));

    // vivi-classic に切り替え
    vivivi_app::tui::event::ex::execute_ex_command(&mut app, "colo vivi-classic");
    assert_eq!(app.config.appearance.theme, "ViVi Classic");
    assert_eq!(app.active_theme.meta.name, "ViVi Classic");
}
