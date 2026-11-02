use ratatui::{Terminal, backend::TestBackend};
use vivivi_app::tui::{App, ui::settings::render_settings_modal};
use vivivi_app::{AppKeyEvent, KeyCode, KeyModifiers};

#[test]
fn test_tui_settings_modal_open_and_close() {
    let mut app = App::new("test content", None, Default::default(), Default::default());

    assert!(!app.show_preferences_dialog);

    // 1. :preferences コマンドで開く
    app.execute_ex("preferences");
    assert!(app.show_preferences_dialog);

    // 2. Esc キーで閉じる
    app.handle_key(AppKeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(!app.show_preferences_dialog);

    // 3. :pref コマンドで開き、q キーで閉じる
    app.execute_ex("pref");
    assert!(app.show_preferences_dialog);
    app.handle_key(AppKeyEvent::char('q'));
    assert!(!app.show_preferences_dialog);
}

#[test]
fn test_tui_settings_navigation_and_toggle() {
    let mut app = App::new("test content", None, Default::default(), Default::default());
    app.show_preferences_dialog = true;

    // 初期状態: カテゴリ 0 (General), 項目 0, カテゴリペインフォーカス
    assert_eq!(app.tui_settings_category, 0);
    assert_eq!(app.tui_settings_item, 0);
    assert!(!app.tui_settings_in_items);

    // 1. 下移動 ('j') でカテゴリ 1 (Appearance) へ
    app.handle_key(AppKeyEvent::char('j'));
    assert_eq!(app.tui_settings_category, 1);
    assert_eq!(app.tui_settings_item, 0);

    // 2. 右移動 ('l') で項目ペインへフォーカス切替
    app.handle_key(AppKeyEvent::char('l'));
    assert!(app.tui_settings_in_items);

    // 3. 項目 0 は「カラーテーマ」、下移動 ('j') で項目 1「行番号表示」へ
    app.handle_key(AppKeyEvent::char('j'));
    assert_eq!(app.tui_settings_item, 1);

    // 4. Space キーで行番号表示をトグル (true -> false)
    assert!(app.show_line_numbers);
    app.handle_key(AppKeyEvent::char(' '));
    assert!(!app.show_line_numbers);
    assert!(!app.config.appearance.line_number);

    // もう一度 Space キーで (false -> true)
    app.handle_key(AppKeyEvent::char(' '));
    assert!(app.show_line_numbers);
    assert!(app.config.appearance.line_number);

    // 5. 左移動 ('h') でカテゴリペインへ復帰
    app.handle_key(AppKeyEvent::char('h'));
    assert!(!app.tui_settings_in_items);

    // 6. カテゴリ 3 (Editing) へ移動
    app.handle_key(AppKeyEvent::char('j'));
    app.handle_key(AppKeyEvent::char('j'));
    assert_eq!(app.tui_settings_category, 3);

    // 7. 項目ペインへ移動してタブ幅を変更 (Space で 4 -> 6)
    app.handle_key(AppKeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    assert_eq!(app.tui_settings_item, 0); // タブ幅
    assert_eq!(app.tab_width, 4);
    app.handle_key(AppKeyEvent::char(' '));
    assert_eq!(app.tab_width, 6);
    assert_eq!(app.config.editing.tab_width, 6);
}

#[test]
fn test_tui_settings_render_and_save() {
    let temp_dir = std::env::temp_dir().join(format!("vivivi_tui_cfg_test_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).expect("create temp dir");
    let test_cfg_path = temp_dir.join("config.toml");

    let mut app = App::new("test content", None, Default::default(), Default::default());
    app.show_preferences_dialog = true;

    // 1. Ratatui TestBackend で実際に描画してもクラッシュしないことを検証
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| {
            render_settings_modal(frame, &app, frame.area());
        })
        .unwrap();

    // 2. 設定を変更して 's' キーで保存
    app.show_line_numbers = false;
    app.tab_width = 8;
    app.sync_runtime_to_config();
    app.config
        .save_to(&test_cfg_path)
        .expect("save config to temp");
    assert!(test_cfg_path.exists());

    // 3. 再読込して値が一致することを検証
    let loaded = vivivi_app::config::AppConfig::load_or_default(Some(&test_cfg_path));
    assert!(!loaded.appearance.line_number);
    assert_eq!(loaded.editing.tab_width, 8);

    let _ = std::fs::remove_dir_all(&temp_dir);
}
