//! `vivivi-app` クレートのヘッドレス共通ライブラリ公開API統合テスト（`.agents/rules/rust_testing.md` 準拠）。
//!
//! TUI および GUI フロントエンドの双方から共有される `App`, `LogicalRect`, `AppKeyEvent`,
//! `WindowTreeNode`, および `handle_key_event` / `execute_ex_command` の動作を検証します。

use vivivi_app::{
    App, AppKeyEvent, EditorConfig, KeyCode, KeyModifiers, LogicalRect, SplitAxis, WindowDirection,
    WindowTreeNode, handle_key_event,
};
use vivivi_core::{CharEncoding, NewlineType};
use vivivi_vi::ViMode;

/// ヘッドレス `App` インスタンスに対するキーイベント入力とモード遷移・編集の統合検証。
#[test]
fn test_headless_app_key_event_and_editing() {
    let mut app = App::new(
        "first line\nsecond line",
        None,
        CharEncoding::Utf8,
        NewlineType::Lf,
    );
    assert_eq!(app.mode(), ViMode::Normal);

    // 'j' キーで2行目へ移動
    handle_key_event(&mut app, AppKeyEvent::char('j').into_key_event());
    assert_eq!(app.cursor_pos().line, 1);

    // 'A' キーで行末 Insert モードへ遷移し、'!' を追加して Esc で戻る
    handle_key_event(&mut app, AppKeyEvent::char('A').into_key_event());
    assert_eq!(app.mode(), ViMode::Insert);
    handle_key_event(&mut app, AppKeyEvent::char('!').into_key_event());
    handle_key_event(
        &mut app,
        AppKeyEvent::new(KeyCode::Esc, KeyModifiers::NONE).into_key_event(),
    );

    assert_eq!(app.mode(), ViMode::Normal);
    assert_eq!(app.buffer.get_line(1).unwrap(), "second line!");
}

/// フロントエンド非依存の `LogicalRect` と `WindowTreeNode::compute_logical_rects` による分割レイアウト計算の検証。
#[test]
fn test_headless_window_tree_logical_rects_and_navigation() {
    let mut app = App::new("window test", None, CharEncoding::Utf8, NewlineType::Lf);

    // 垂直分割 (:vsp) を実行
    app.execute_ex("vsp");
    assert_eq!(app.windows.len(), 2);

    // 1000x800 の論理画面矩形を分割計算
    let screen = LogicalRect::new(0, 0, 1000, 800);
    let rects = app.window_tree.compute_logical_rects(screen);
    assert_eq!(rects.len(), 2);
    assert_eq!(rects[0].1.width + rects[1].1.width, 1000);
    assert_eq!(rects[0].1.height, 800);

    // 計算された論理矩形を App にセットし、方向指定ウィンドウ移動 (Left / Right) が機能することを確認
    app.window_rects = rects;
    let initial_win = app.active_window;
    app.navigate_window(WindowDirection::Left);
    assert_eq!(app.active_window, 0);
    app.navigate_window(WindowDirection::Right);
    assert_eq!(app.active_window, 1);
    assert_ne!(initial_win, 0);

    // 単一リーフのツリー構築テスト
    let leaf = WindowTreeNode::new(0);
    let single_rects = leaf.compute_logical_rects(LogicalRect::new(10, 20, 300, 200));
    assert_eq!(single_rects, vec![(0, LogicalRect::new(10, 20, 300, 200))]);
    assert_eq!(SplitAxis::Horizontal, SplitAxis::Horizontal);
}

/// `EditorConfig` とマルチタブ操作（`:tabnew`, `:tabn`, `:tabp`）の公開API検証。
#[test]
fn test_headless_multitab_and_config_api() {
    let cfg = EditorConfig::default();
    assert!(cfg.tab_width >= 1);

    let mut app = App::new("tab1", None, CharEncoding::Utf8, NewlineType::Lf);
    assert_eq!(app.tabs.len(), 1);

    app.execute_ex("tabnew");
    assert_eq!(app.tabs.len(), 2);
    assert_eq!(app.active_tab, 1);

    app.execute_ex("tabp");
    assert_eq!(app.active_tab, 0);

    app.execute_ex("tabn");
    assert_eq!(app.active_tab, 1);
}
