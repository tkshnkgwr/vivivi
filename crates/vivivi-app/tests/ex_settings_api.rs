use std::io::Write;
use vivivi_app::tui::App;

#[test]
fn test_ex_colorscheme_commands() {
    let mut app = App::new("test", None, Default::default(), Default::default());

    // 1. クエリ (:colo / :colo? / :colorscheme)
    app.execute_ex("colo?");
    let msg = app.status_message.take().unwrap_or_default();
    assert!(msg.contains("現在のカラーテーマ (colorscheme):"));

    // 2. 有効なテーマへの動的切替 (:colo openvivi)
    app.execute_ex("colo openvivi");
    let msg = app.status_message.take().unwrap_or_default();
    assert!(msg.contains("カラーテーマを変更しました: OpenViVi"));
    assert_eq!(app.config.appearance.theme, "OpenViVi");

    // 3. 別テーマへの切替 (:colorscheme vivi-classic)
    app.execute_ex("colorscheme vivi-classic");
    let msg = app.status_message.take().unwrap_or_default();
    assert!(msg.contains("カラーテーマを変更しました: ViVi Classic"));
    assert_eq!(app.config.appearance.theme, "ViVi Classic");

    // 4. 存在しないテーマのエラーハンドリング (:colo dummy)
    app.execute_ex("colo dummy-nonexistent-theme");
    let msg = app.status_message.take().unwrap_or_default();
    assert!(msg.starts_with("E185: テーマ 'dummy-nonexistent-theme' が見つかりません"));
    // テーマは変更されていないこと
    assert_eq!(app.config.appearance.theme, "ViVi Classic");
}

#[test]
fn test_ex_mkvivivirc_and_source_commands() {
    let temp_dir = std::env::temp_dir().join(format!("vivivi_test_cfg_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).expect("create test temp dir");
    let config_file = temp_dir.join("test_config.toml");
    let config_file_str = config_file.to_string_lossy().to_string();

    let mut app = App::new("test", None, Default::default(), Default::default());

    // 設定を一時変更
    app.execute_ex("nonumber");
    app.execute_ex("wrap");
    app.execute_ex("set ts=6");
    app.execute_ex("colo openvivi");

    // :mkvivivirc <path> で保存
    app.execute_ex(&format!("mkvivivirc {config_file_str}"));
    let msg = app.status_message.take().unwrap_or_default();
    assert!(msg.contains("設定ファイルを保存しました:"));
    assert!(config_file.exists());

    // 別の App インスタンスを初期化して、保存した設定を :source で復元
    let mut app2 = App::new("test2", None, Default::default(), Default::default());
    assert!(app2.show_line_numbers);
    assert!(!app2.wrap);

    app2.execute_ex(&format!("source {config_file_str}"));
    let msg = app2.status_message.take().unwrap_or_default();
    assert!(msg.contains("設定ファイルを再読込しました:"));
    assert!(!app2.show_line_numbers);
    assert!(app2.wrap);
    assert_eq!(app2.tab_width, 6);
    assert_eq!(app2.config.appearance.theme, "OpenViVi");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_ex_source_script_commands() {
    let temp_dir = std::env::temp_dir().join(format!("vivivi_test_scr_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).expect("create test temp dir");
    let script_file = temp_dir.join("init.vivivirc");
    {
        let mut file = std::fs::File::create(&script_file).expect("create script file");
        writeln!(file, "# Ex script test").unwrap();
        writeln!(file, "set number").unwrap();
        writeln!(file, "\" comment line").unwrap();
        writeln!(file, "set wrap").unwrap();
        writeln!(file, "set ts=3").unwrap();
        writeln!(file, "set list").unwrap();
    }
    let script_file_str = script_file.to_string_lossy().to_string();

    let mut app = App::new("test", None, Default::default(), Default::default());
    app.show_line_numbers = false;
    app.wrap = false;
    app.tab_width = 8;
    app.list_chars_config.enabled = false;

    app.execute_ex(&format!("source {script_file_str}"));
    let msg = app.status_message.take().unwrap_or_default();
    assert!(msg.contains("設定スクリプトを実行しました (4 行):"));
    assert!(app.show_line_numbers);
    assert!(app.wrap);
    assert_eq!(app.tab_width, 3);
    assert!(app.list_chars_config.enabled);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_ex_extended_set_commands() {
    let mut app = App::new("test", None, Default::default(), Default::default());

    // 1. ミニマップ切替 (:set minimap / :set nominimap / :set minimap? / :minimap / :set minimap!)
    app.execute_ex("set nominimap");
    assert!(!app.config.appearance.gui.show_minimap);
    app.execute_ex("set minimap?");
    let msg = app.status_message.take().unwrap_or_default();
    assert!(msg.contains("nominimap"));

    app.execute_ex("set minimap");
    assert!(app.config.appearance.gui.show_minimap);
    app.execute_ex("set minimap?");
    let msg = app.status_message.take().unwrap_or_default();
    assert!(msg.contains("minimap (ミニマップ表示: ON)"));

    // :minimap コマンド単体でのトグル動作
    app.execute_ex("minimap");
    assert!(!app.config.appearance.gui.show_minimap);
    app.execute_ex("minimap");
    assert!(app.config.appearance.gui.show_minimap);

    // :set minimap! による反転動作
    app.execute_ex("set minimap!");
    assert!(!app.config.appearance.gui.show_minimap);
    app.execute_ex("set minimap!");
    assert!(app.config.appearance.gui.show_minimap);

    // 2. 文字削除重力落下アニメーション切替 (:set particles / :set noparticles / :set particles?)
    app.execute_ex("set noparticles");
    assert!(!app.config.appearance.gui.enable_gravity_drop);
    app.execute_ex("set particles?");
    let msg = app.status_message.take().unwrap_or_default();
    assert!(msg.contains("noparticles"));

    app.execute_ex("set particles");
    assert!(app.config.appearance.gui.enable_gravity_drop);

    // 3. 背景明暗切替 (:set bg=dark / :set bg=light / :set bg?)
    app.execute_ex("set bg=light");
    assert_eq!(app.config.appearance.theme, "vivivi-light");
    app.execute_ex("set bg?");
    let msg = app.status_message.take().unwrap_or_default();
    assert!(msg.contains("background=light"));

    app.execute_ex("set bg=dark");
    assert_eq!(app.config.appearance.theme, "tokyo-night");
    app.execute_ex("set bg?");
    let msg = app.status_message.take().unwrap_or_default();
    assert!(msg.contains("background=dark"));

    // 4. 不可視文字記号設定 (:set listchars=... / :set listchars?)
    app.execute_ex("set listchars=zenkaku:全,space:·,eol:¶");
    assert_eq!(app.list_chars_config.zenkaku_space, "全");
    assert_eq!(app.list_chars_config.space, '·');
    assert_eq!(app.list_chars_config.eol, '¶');

    app.execute_ex("set listchars?");
    let msg = app.status_message.take().unwrap_or_default();
    assert!(msg.contains("listchars=zenkaku:全"));

    // 5. 大文字小文字無視・スマートケース切替 (:set ic / :set noic / :set sc / :set nosc)
    assert!(app.config.editing.ignorecase);
    assert!(app.vi.ignorecase);
    assert!(app.config.editing.smartcase);
    assert!(app.vi.smartcase);

    app.execute_ex("set noic");
    assert!(!app.config.editing.ignorecase);
    assert!(!app.vi.ignorecase);
    app.execute_ex("set ic?");
    let msg = app.status_message.take().unwrap_or_default();
    assert!(msg.contains("noignorecase"));

    app.execute_ex("set ic");
    assert!(app.config.editing.ignorecase);
    assert!(app.vi.ignorecase);

    app.execute_ex("set nosc");
    assert!(!app.config.editing.smartcase);
    assert!(!app.vi.smartcase);
    app.execute_ex("set sc?");
    let msg = app.status_message.take().unwrap_or_default();
    assert!(msg.contains("nosmartcase"));

    app.execute_ex("set sc");
    assert!(app.config.editing.smartcase);
    assert!(app.vi.smartcase);

    // 6. 全オプション表示 (:set all)
    app.execute_ex("set all");
    let msg = app.status_message.take().unwrap_or_default();
    assert!(msg.contains("ts="));
    assert!(msg.contains("theme="));
    assert!(msg.contains("fenc="));
    assert!(msg.contains("ic"));
    assert!(msg.contains("sc"));
}
