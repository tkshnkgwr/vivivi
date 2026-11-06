//! 行トリミング (`:trim`) および外部プロセス連携拡張 (`:r!`, `:[range]!`, `cmdExecDlg`) の統合テスト。

use vivivi_app::tui::app::App;

#[test]
fn test_trim_commands_and_undo() {
    let mut app = App::new(
        "line1  \nline2\t\t\nline3　\nline4\n",
        None,
        Default::default(),
        Default::default(),
    );

    // バッファ全体のトリム (:trim)
    app.execute_ex("trim");
    let content = app.buffer.to_string();
    assert_eq!(content, "line1\nline2\nline3\nline4\n");
    assert!(
        app.status_message
            .as_ref()
            .unwrap()
            .contains("3 行の末尾空白を削除")
    );

    // Undo で直前の空白付き状態に完全復元
    app.feed_vi_char('u');
    assert_eq!(
        app.buffer.to_string(),
        "line1  \nline2\t\t\nline3　\nline4\n"
    );

    // 全角スペースのみトリム (:trim zenkaku)
    app.execute_ex("trim zenkaku");
    assert_eq!(app.buffer.to_string(), "line1  \nline2\t\t\nline3\nline4\n");
}

#[test]
fn test_trim_range_and_half_mode() {
    let mut app = App::new(
        "0   \n1   \n2　\n3   \n",
        None,
        Default::default(),
        Default::default(),
    );

    // 行範囲トリム (:1,2trim)
    app.execute_ex("1,2trim");
    assert_eq!(app.buffer.to_string(), "0\n1\n2　\n3   \n");

    // 半角のみトリム (:trim half)
    app.execute_ex("trim half");
    assert_eq!(app.buffer.to_string(), "0\n1\n2　\n3\n");
}

#[test]
fn test_trim_on_save_setting_toggle() {
    let mut app = App::new("test", None, Default::default(), Default::default());

    // :set trim
    app.execute_ex("set trim");
    assert!(app.config.editing.trim_on_save);
    assert!(app.status_message.as_ref().unwrap().contains("ON"));

    // :set notrim
    app.execute_ex("set notrim");
    assert!(!app.config.editing.trim_on_save);
    assert!(app.status_message.as_ref().unwrap().contains("OFF"));
}

#[test]
fn test_external_command_read_into_buffer() {
    let mut app = App::new("First line\n", None, Default::default(), Default::default());

    // OS 標準シェルでの echo 実行とバッファ挿入 (:r !echo Hello)
    #[cfg(target_os = "windows")]
    let cmd = "r !cmd /c echo Hello World";
    #[cfg(not(target_os = "windows"))]
    let cmd = "r !echo Hello World";

    app.execute_ex(cmd);
    let content = app.buffer.to_string();
    assert!(content.contains("Hello World"));

    // Undo で元に戻るか検証
    app.feed_vi_char('u');
    assert_eq!(app.buffer.to_string(), "First line\n");
}

#[test]
fn test_cmd_exec_dialog_lifecycle() {
    let mut app = App::new("Initial", None, Default::default(), Default::default());

    // :cmdexec でダイアログを起動
    app.execute_ex("cmdexec");
    assert!(app.show_cmd_exec_dialog);
    assert_eq!(app.cmd_exec_mode, 0);

    // Tab キーでモード切替 (0 -> 1 -> 2 -> 0)
    app.handle_key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Tab,
        crossterm::event::KeyModifiers::empty(),
    ));
    assert_eq!(app.cmd_exec_mode, 1);

    app.handle_key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Tab,
        crossterm::event::KeyModifiers::empty(),
    ));
    assert_eq!(app.cmd_exec_mode, 2);

    // 文字入力
    app.handle_key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('d'),
        crossterm::event::KeyModifiers::empty(),
    ));
    assert_eq!(app.cmd_exec_input, "d");

    // Esc で閉じる
    app.handle_key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Esc,
        crossterm::event::KeyModifiers::empty(),
    ));
    assert!(!app.show_cmd_exec_dialog);
}
