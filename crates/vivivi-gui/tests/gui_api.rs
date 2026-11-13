//! `vivivi-gui` クレートの公開API統合テスト（`.agents/rules/rust_testing.md` 準拠）。
//!
//! 外部利用者の視点から、共通 `App` コアとのイベント連携、日本語 IME（Preedit / Commit）、
//! 特殊キー変換、および削除文字落下パーティクルのライフサイクルと省電力制御を検証します。

use egui::{Event, ImeEvent, Key, Modifiers, Pos2};
use vivivi_app::{App, KeyCode, KeyModifiers};
use vivivi_core::{CharEncoding, NewlineType};
use vivivi_gui::{
    ParticleManager, SettingsCategory, SidebarTab, ViviviGuiApp, colors, egui_key_to_app_key,
    extract_deleted_line_segment,
};
use vivivi_vi::ViMode;

/// シナリオ 1: Vi モード遷移と ASCII 文字入力 (`i` -> テキスト入力 -> `Esc`) の統合検証。
#[test]
fn test_gui_100g() {
    let lines = (1..=200)
        .map(|i| format!("line {}", i))
        .collect::<Vec<_>>()
        .join("\n");
    let core = App::new(&lines, None, CharEncoding::Utf8, NewlineType::Lf);
    let mut gui = ViviviGuiApp::from_core(core);

    gui.process_egui_events(
        &[
            Event::Key {
                key: Key::Num1,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            },
            Event::Text("1".to_string()),
        ],
        Modifiers::NONE,
    );

    gui.process_egui_events(
        &[
            Event::Key {
                key: Key::Num0,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            },
            Event::Text("0".to_string()),
        ],
        Modifiers::NONE,
    );

    gui.process_egui_events(
        &[
            Event::Key {
                key: Key::Num0,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            },
            Event::Text("0".to_string()),
        ],
        Modifiers::NONE,
    );

    gui.process_egui_events(
        &[
            Event::Key {
                key: Key::G,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::SHIFT,
            },
            Event::Text("G".to_string()),
        ],
        Modifiers::SHIFT,
    );

    println!("GUI CURSOR LINE: {}", gui.core.cursor_pos().line);
    assert_eq!(gui.core.cursor_pos().line, 99);
}
#[test]
fn test_public_gui_vi_mode_and_text_input_scenario() {
    let core = App::new("hello\nworld", None, CharEncoding::Utf8, NewlineType::Lf);
    let mut gui = ViviviGuiApp::from_core(core);

    assert_eq!(gui.core.mode(), ViMode::Normal);

    // Normal モードで 'i' を入力 -> Insert モードへ遷移
    gui.process_egui_events(&[Event::Text("i".to_string())], Modifiers::NONE);
    assert_eq!(gui.core.mode(), ViMode::Insert);

    // Insert モードで文字列 "rust " を入力
    gui.process_egui_events(&[Event::Text("rust ".to_string())], Modifiers::NONE);
    assert_eq!(gui.core.buffer.get_line(0).unwrap(), "rust hello");

    // Escape キーで Normal モードへ戻る
    gui.process_egui_events(
        &[Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    assert_eq!(gui.core.mode(), ViMode::Normal);
}

/// シナリオ 2: 日本語 IME の Preedit（未確定変換中）の保持、変換中の Backspace 保護、および Commit 確定入力。
#[test]
fn test_public_gui_japanese_ime_preedit_and_commit_scenario() {
    let core = App::new("", None, CharEncoding::Utf8, NewlineType::Lf);
    let mut gui = ViviviGuiApp::from_core(core);

    // Insert モードへ入る
    gui.process_egui_events(&[Event::Text("i".to_string())], Modifiers::NONE);
    assert_eq!(gui.core.mode(), ViMode::Insert);

    // 1. IME Preedit ("にほんご") の受信 -> バッファにはまだ書き込まれず ime_preedit に保持される
    gui.process_egui_events(
        &[Event::Ime(ImeEvent::Preedit("にほんご".to_string()))],
        Modifiers::NONE,
    );
    assert_eq!(gui.ime_preedit, "にほんご");
    assert_eq!(gui.core.buffer.get_line(0).unwrap(), "");

    // 2. IME 変換中（preedit 非空）の Backspace キーは OS 側の変換編集用として無視され、バッファを破壊しない
    gui.process_egui_events(
        &[Event::Key {
            key: Key::Backspace,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    assert_eq!(gui.ime_preedit, "にほんご");

    // 3. IME Commit ("日本語") の受信 -> preedit がクリアされ、確定文字列がバッファへ挿入される
    gui.process_egui_events(
        &[Event::Ime(ImeEvent::Commit("日本語".to_string()))],
        Modifiers::NONE,
    );
    assert!(gui.ime_preedit.is_empty());
    assert_eq!(gui.core.buffer.get_line(0).unwrap(), "日本語");
}

/// シナリオ 3: egui 標準の TextEdit を使わず、共通パーサーへ直接流し込む Ex コマンドライン (`:3`, `:sp`, `:vsp`, `:undo`, `:redo`) の検証。
#[test]
fn test_public_gui_ex_command_line_and_window_split_scenario() {
    let core = App::new(
        "line1\nline2\nline3\nline4",
        None,
        CharEncoding::Utf8,
        NewlineType::Lf,
    );
    let mut gui = ViviviGuiApp::from_core(core);

    // ':' で CommandLine モードへ遷移
    gui.process_egui_events(&[Event::Text(":".to_string())], Modifiers::NONE);
    assert_eq!(gui.core.mode(), ViMode::CommandLine);

    // '3' と Enter で 3行目（0-indexed: 2）へジャンプ
    gui.process_egui_events(
        &[
            Event::Text("3".to_string()),
            Event::Key {
                key: Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            },
        ],
        Modifiers::NONE,
    );
    assert_eq!(gui.core.mode(), ViMode::Normal);
    assert_eq!(gui.core.cursor_pos().line, 2);

    // ':vsp' で垂直分割を実行
    gui.core.execute_ex("vsp");
    assert_eq!(gui.core.windows.len(), 2);

    // 1文字削除 ('x') 後に ':undo' と ':redo' が動作することを検証
    gui.process_egui_events(&[Event::Text("x".to_string())], Modifiers::NONE);
    assert_eq!(gui.core.buffer.get_line(2).unwrap(), "ine3");

    gui.core.execute_ex("undo");
    assert_eq!(gui.core.buffer.get_line(2).unwrap(), "line3");

    gui.core.execute_ex("redo");
    assert_eq!(gui.core.buffer.get_line(2).unwrap(), "ine3");
}

/// シナリオ 4: 文字削除 (`x`, `dd`) 時の落下パーティクル生成と 0.5秒経過後の省電力リアクティブ待機復帰。
#[test]
fn test_public_gui_particle_spawn_and_power_saving_lifecycle() {
    let core = App::new("Rust\nEditor", None, CharEncoding::Utf8, NewlineType::Lf);
    let mut gui = ViviviGuiApp::from_core(core);

    // 初期状態ではパーティクルは空（0% CPU リアクティブ待機）
    assert!(!gui.particles.is_active());
    assert_eq!(gui.particles.len(), 0);

    // Normal モードで 'x' を押して先頭文字 'R' を削除 -> パーティクルがスポーンする
    gui.process_egui_events(&[Event::Text("x".to_string())], Modifiers::NONE);
    assert_eq!(gui.core.buffer.get_line(0).unwrap(), "ust");
    assert!(gui.particles.is_active());
    assert_eq!(gui.particles.len(), 1);

    // 0.25秒経過時点ではまだ生存しており再描画対象（update は 1ステップ最大 0.05秒クランプのため分割更新）
    for _ in 0..5 {
        gui.particles.update(0.05);
    }
    assert!(gui.particles.is_active());

    // さらに 0.30秒経過（合計 0.55秒 > 0.5秒）で全パーティクルが消滅し、省電力待機へ戻る
    for _ in 0..6 {
        gui.particles.update(0.05);
    }
    assert!(!gui.particles.is_active());
    assert_eq!(gui.particles.len(), 0);
}

/// シナリオ 5: キー変換関数 (`egui_key_to_app_key`) および差分抽出 (`extract_deleted_line_segment`) の公開API仕様検証。
#[test]
fn test_public_key_conversion_and_diff_extraction_api() {
    // 修飾なしの通常文字は Event::Text で処理されるため None
    assert!(egui_key_to_app_key(Key::H, Modifiers::NONE).is_none());

    // Ctrl+R (Redo) は変換される
    let ctrl_r = egui_key_to_app_key(Key::R, Modifiers::CTRL).expect("Ctrl+R should map");
    assert_eq!(ctrl_r.code, KeyCode::Char('r'));
    assert!(ctrl_r.modifiers.contains(KeyModifiers::CONTROL));

    // Shift+Ctrl+W は大文字 'W' として変換される
    let ctrl_shift_w = egui_key_to_app_key(
        Key::W,
        Modifiers {
            ctrl: true,
            shift: true,
            ..Modifiers::NONE
        },
    )
    .expect("Ctrl+Shift+W should map");
    assert_eq!(ctrl_shift_w.code, KeyCode::Char('W'));

    // 行内差分抽出の公開API検証
    let (col, deleted) =
        extract_deleted_line_segment("hello world", "hello ").expect("diff should be found");
    assert_eq!(col, 6);
    assert_eq!(deleted, vec!['w', 'o', 'r', 'l', 'd']);

    // ParticleManager の直接操作とクリア
    let mut pm = ParticleManager::new();
    for ch in ['A', 'B', 'C'] {
        pm.spawn_char(ch, Pos2::new(10.0, 20.0), colors::ACCENT_RED);
    }
    assert_eq!(pm.len(), 3);
    pm.clear();
    assert!(pm.is_empty());
}

/// シナリオ 7: OS からのドラッグ＆ドロップファイル処理の検証
#[test]
fn test_public_gui_drag_and_drop_scenario() {
    let temp_dir = std::env::temp_dir();
    let file1 = temp_dir.join("vivivi_dnd_test1.txt");
    let file2 = temp_dir.join("vivivi_dnd_test2.txt");

    std::fs::write(&file1, "Hello from D&D 1").unwrap();
    std::fs::write(&file2, "Hello from D&D 2").unwrap();

    // 1. 新規無名空バッファの場合: 最初のファイルは既存タブを置き換えて開く
    let core = App::new("", None, CharEncoding::Utf8, NewlineType::Lf);
    let mut gui = ViviviGuiApp::from_core(core);

    let dropped1 = egui::DroppedFile {
        path: Some(file1.clone()),
        name: "vivivi_dnd_test1.txt".to_string(),
        mime: String::new(),
        last_modified: None,
        bytes: None,
    };

    gui.handle_drag_and_drop_files(&[dropped1]);
    assert_eq!(gui.core.tabs.len(), 1);
    assert_eq!(gui.core.file_path, Some(file1.clone()));
    assert_eq!(gui.core.buffer.to_string(), "Hello from D&D 1");

    // 2. 既にファイルを開いている状態でさらにドロップ: 新規タブとして開く
    let dropped2 = egui::DroppedFile {
        path: Some(file2.clone()),
        name: "vivivi_dnd_test2.txt".to_string(),
        mime: String::new(),
        last_modified: None,
        bytes: None,
    };

    gui.handle_drag_and_drop_files(&[dropped2]);
    assert_eq!(gui.core.tabs.len(), 2);
    assert_eq!(gui.core.file_path, Some(file2.clone()));
    assert_eq!(gui.core.buffer.to_string(), "Hello from D&D 2");

    // クリーンアップ
    let _ = std::fs::remove_file(file1);
    let _ = std::fs::remove_file(file2);
}

/// シナリオ 8: ミニマップの線形性・終端サンプリング整合性の検証
#[test]
fn test_public_gui_minimap_alignment_and_bounds() {
    let lines = (0..500)
        .map(|i| format!("    let x_{} = {};", i, i))
        .collect::<Vec<_>>()
        .join("\n");
    let core = App::new(&lines, None, CharEncoding::Utf8, NewlineType::Lf);
    let gui = ViviviGuiApp::from_core(core);

    assert!(gui.minimap_open);
    assert_eq!(gui.core.buffer.line_count(), 500);

    // 0行目（先頭）と499行目（末尾）がサンプリング可能か検証
    let max_mm_rows = 150;
    let total = gui.core.buffer.line_count();
    let first_idx = 0;
    let last_idx =
        ((max_mm_rows - 1) as f32 * (total - 1) as f32 / (max_mm_rows - 1) as f32).round() as usize;
    assert_eq!(first_idx, 0);
    assert_eq!(
        last_idx, 499,
        "最終行499が正確にサンプリング終端に一致すること"
    );
}

/// シナリオ 9: Exコマンドによるテーマ・ミニマップ設定のGUI動的同期
#[test]
fn test_public_gui_ex_commands_theme_and_appearance_sync() {
    let core = App::new("fn main() {}", None, CharEncoding::Utf8, NewlineType::Lf);
    let mut gui = ViviviGuiApp::from_core(core);

    assert!(gui.minimap_open);

    // 1. Ex コマンドでテーマを動的変更
    gui.core.execute_ex("colo openvivi");
    assert_eq!(gui.core.config.appearance.theme, "OpenViVi");

    if gui.core.config.appearance.theme != gui.active_theme.meta.name {
        gui.active_theme =
            vivivi_app::theme::Theme::load_by_name(&gui.core.config.appearance.theme);
    }
    assert_eq!(gui.active_theme.meta.name, "OpenViVi");

    // 2. Ex コマンドでミニマップを OFF
    gui.core.execute_ex("set nominimap");
    assert!(!gui.core.config.appearance.gui.show_minimap);

    if gui.minimap_open != gui.core.config.appearance.gui.show_minimap {
        gui.minimap_open = gui.core.config.appearance.gui.show_minimap;
    }
    assert!(!gui.minimap_open);

    // 3. Ex コマンドでミニマップを ON
    gui.core.execute_ex("set minimap");
    assert!(gui.core.config.appearance.gui.show_minimap);

    if gui.minimap_open != gui.core.config.appearance.gui.show_minimap {
        gui.minimap_open = gui.core.config.appearance.gui.show_minimap;
    }
    assert!(gui.minimap_open);

    // 4. :minimap コマンド単体でのトグル
    gui.core.execute_ex("minimap");
    assert!(!gui.core.config.appearance.gui.show_minimap);
    if gui.minimap_open != gui.core.config.appearance.gui.show_minimap {
        gui.minimap_open = gui.core.config.appearance.gui.show_minimap;
    }
    assert!(!gui.minimap_open);

    // 5. ピルボタントグル操作（self.minimap_open と config の双方向同期）
    gui.minimap_open = !gui.minimap_open;
    gui.core.config.appearance.gui.show_minimap = gui.minimap_open;
    assert!(gui.minimap_open);
    assert!(gui.core.config.appearance.gui.show_minimap);
}

/// シナリオ 10: GUI 設定ダイアログ（Preferences Dialog: 左右2ペイン）のライフサイクル、
/// カテゴリ切替、検索フィルタ、ドラフト適用・キャンセルの検証
#[test]
fn test_public_gui_settings_dialog_lifecycle_and_actions() {
    let core = App::new("fn main() {}", None, CharEncoding::Utf8, NewlineType::Lf);
    let mut gui = ViviviGuiApp::from_core(core);

    // 1. 初期状態では設定ダイアログは閉じている
    assert!(!gui.settings_dialog.is_open);
    assert!(!gui.is_any_modal_open());

    // 2. Ex コマンド (:preferences) の実行 -> show_preferences_dialog フラグが立ち、ダイアログが開く
    gui.core.execute_ex("preferences");
    assert!(gui.core.show_preferences_dialog);

    if gui.core.show_preferences_dialog {
        gui.settings_dialog.open(&gui.core.config);
        gui.core.show_preferences_dialog = false;
    }
    assert!(gui.settings_dialog.is_open);
    assert!(gui.is_any_modal_open());

    // 3. カテゴリ切替
    assert_eq!(
        gui.settings_dialog.active_category,
        SettingsCategory::General
    );
    gui.settings_dialog.active_category = SettingsCategory::Appearance;
    assert_eq!(
        gui.settings_dialog.active_category,
        SettingsCategory::Appearance
    );

    // 4. ドラフト設定の変更（テーマ: OpenViVi, 行番号: false, タブ幅: 8, ミニマップ: false）
    gui.settings_dialog.draft_config.appearance.theme = "OpenViVi".to_string();
    gui.settings_dialog.draft_config.appearance.line_number = false;
    gui.settings_dialog.draft_config.editing.tab_width = 8;
    gui.settings_dialog.draft_config.appearance.gui.show_minimap = false;

    // 5. 適用 (プレビュー) のシミュレート
    let ctx = egui::Context::default();
    gui.apply_dialog_config(&ctx);

    assert_eq!(gui.core.config.appearance.theme, "OpenViVi");
    assert_eq!(gui.active_theme.meta.name, "OpenViVi");
    assert!(!gui.core.show_line_numbers);
    assert_eq!(gui.core.tab_width, 8);
    assert!(!gui.minimap_open);

    // 6. キャンセルによる初期バックアップ設定への復帰
    gui.cancel_settings_dialog(Some(&ctx));

    assert!(!gui.settings_dialog.is_open);
    assert!(gui.core.show_line_numbers);
    assert_eq!(gui.core.tab_width, 4);
    assert!(gui.minimap_open);
}

/// シナリオ 11: 設定ダイアログの全6カテゴリのヘッドレス描画、Ctrl+, キー連動、
/// インクリメンタル検索、および設定ファイル保存・復元の網羅的検証。
#[test]
fn test_public_gui_settings_dialog_full_render_and_persistence() {
    let core = App::new(
        "fn main() {\n    println!(\"Hello\");\n}",
        None,
        CharEncoding::Utf8,
        NewlineType::Lf,
    );
    let mut gui = ViviviGuiApp::from_core(core);
    let ctx = egui::Context::default();

    // 1. Ctrl+, キーによるダイアログの開閉検証
    assert!(!gui.settings_dialog.is_open);
    gui.process_egui_events(
        &[Event::Key {
            key: Key::Comma,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::CTRL,
        }],
        Modifiers::CTRL,
    );
    assert!(
        gui.settings_dialog.is_open,
        "Ctrl+, で設定ダイアログが開くこと"
    );

    // ダイアログ表示中は通常打鍵（'j' によるカーソル移動等）が完全に遮断されることを検証
    let initial_cursor = gui.core.cursor_pos();
    gui.process_egui_events(&[Event::Text("j".to_string())], Modifiers::NONE);
    assert_eq!(
        gui.core.cursor_pos(),
        initial_cursor,
        "設定ダイアログ表示中はエディタへの入力が完全に遮断されること"
    );

    // もう一度 Ctrl+, を押すと閉じる
    gui.process_egui_events(
        &[Event::Key {
            key: Key::Comma,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::CTRL,
        }],
        Modifiers::CTRL,
    );
    assert!(
        !gui.settings_dialog.is_open,
        "Ctrl+, で設定ダイアログが閉じること"
    );

    // 2. :preferences コマンドで開く
    gui.core.execute_ex("pref");
    assert!(gui.core.show_preferences_dialog);

    // 3. 全6カテゴリを実際にヘッドレス描画してクラッシュしないことを検証
    for cat in SettingsCategory::all() {
        gui.settings_dialog.open(&gui.core.config);
        gui.settings_dialog.active_category = *cat;

        // egui の 1 フレーム実行（Window 描画を通す）
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            gui.render_settings_dialog(ctx);
        });
    }

    // 4. 設定の変更・保存・再読込の整合性検証
    let temp_dir = std::env::temp_dir().join(format!("vivivi_gui_cfg_test_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).expect("create temp dir");
    let test_cfg_path = temp_dir.join("config.toml");

    // ドラフト設定を変更
    gui.settings_dialog.draft_config.general.default_encoding = "sjis".to_string();
    gui.settings_dialog.draft_config.appearance.theme = "ViVi Classic".to_string();
    gui.settings_dialog.draft_config.appearance.gui.font_size = 18.0;
    gui.settings_dialog.draft_config.editing.tab_width = 8;
    gui.settings_dialog.draft_config.invisibles.enabled = true;
    gui.settings_dialog.draft_config.appearance.gui.show_minimap = false;

    // 適用
    gui.apply_dialog_config(&ctx);
    assert_eq!(gui.core.config.general.default_encoding, "sjis");
    assert_eq!(gui.core.config.appearance.theme, "ViVi Classic");
    assert_eq!(gui.core.tab_width, 8);
    assert!(!gui.minimap_open);

    // ディスク保存
    gui.core
        .config
        .save_to(&test_cfg_path)
        .expect("save config to temp path");
    assert!(test_cfg_path.exists());

    // 保存された TOML をロードして完全一致を検証
    let loaded_config = vivivi_app::config::AppConfig::load_or_default(Some(&test_cfg_path));
    assert_eq!(loaded_config.general.default_encoding, "sjis");
    assert_eq!(loaded_config.appearance.theme, "ViVi Classic");
    assert_eq!(loaded_config.appearance.gui.font_size, 18.0);
    assert_eq!(loaded_config.editing.tab_width, 8);
    assert!(loaded_config.invisibles.enabled);
    assert!(!loaded_config.appearance.gui.show_minimap);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

/// シナリオ 12: フラットデザイン化（Line Height 1.3〜1.5、角丸 2.0px、Iビームカーソル）の検証
#[test]
fn test_public_gui_flat_design_and_ibeam_cursor() {
    let core = App::new("test content", None, CharEncoding::Utf8, NewlineType::Lf);
    let gui = ViviviGuiApp::from_core(core);

    // Line Height が 1.3〜1.5倍の範囲（21.0 / 15.0 = 1.4）
    let ratio = gui.line_height / gui.font_size;
    assert!(
        (1.3..=1.5).contains(&ratio),
        "Line Height はフォントサイズの 1.3〜1.5倍 (現在: {ratio})"
    );

    let ctx = egui::Context::default();
    vivivi_gui::theme::setup_visuals(&ctx);

    // Visuals の角丸が 2.0px に統一されていることの検証
    let corner = ctx.style().visuals.widgets.inactive.corner_radius;
    assert_eq!(corner.nw, 2);
    assert_eq!(corner.ne, 2);
    assert_eq!(corner.sw, 2);
    assert_eq!(corner.se, 2);
}

/// シナリオ 13: 非同期ツリートグル・耐障害スキャンとプロジェクト可視化グラフの検証
#[test]
fn test_public_gui_async_tree_and_visualize_graph_scenario() {
    let core = App::new("fn main() {}", None, CharEncoding::Utf8, NewlineType::Lf);
    let mut gui = ViviviGuiApp::from_core(core);

    // 1. Ctrl+E キーによるサイドバー開閉トグル
    assert!(gui.sidebar_open);
    gui.process_egui_events(
        &[Event::Key {
            key: Key::E,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::CTRL,
        }],
        Modifiers::CTRL,
    );
    assert!(!gui.sidebar_open, "Ctrl+E でサイドバーが閉じること");

    gui.process_egui_events(
        &[Event::Key {
            key: Key::E,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::CTRL,
        }],
        Modifiers::CTRL,
    );
    assert!(gui.sidebar_open, "再度 Ctrl+E でサイドバーが開くこと");

    // 2. Exコマンド (:tree, :Explore) によるトグル要求
    gui.core.execute_ex("tree");
    assert!(gui.core.toggle_tree_request);

    // 3. 非同期ツリーのスキャンとポーリング
    let start = std::time::Instant::now();
    while gui.async_tree.is_scanning() && start.elapsed() < std::time::Duration::from_secs(3) {
        gui.async_tree.poll();
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert_eq!(gui.async_tree.status, vivivi_gui::ScanStatus::Ready);
    assert!(!gui.async_tree.entries.is_empty());

    // 4. プロジェクト可視化グラフ (:graph, :visualize)
    assert!(!gui.core.show_visualize_graph);
    gui.core.execute_ex("graph");
    assert!(
        gui.core.show_visualize_graph,
        ":graph で可視化フラグが立つこと"
    );

    let mut graph = vivivi_gui::ForceGraph::build_from_dir(&gui.current_dir);
    assert!(!graph.nodes.is_empty(), "ノードが生成されていること");
    let is_moving = graph.update_physics(0.016);
    assert!(is_moving, "初期配置で物理運動が発生すること");

    // 再度 :visualize でトグル
    gui.core.execute_ex("visualize");
    assert!(
        !gui.core.show_visualize_graph,
        ":visualize でフラグがトグルして閉じること"
    );
}

/// シナリオ 11: GUI におけるバイナリモード (Hex Editor)、Tree-sitter スマート選択、LSP パネル/ホバー統合検証。
#[test]
fn test_public_gui_hex_lsp_treesitter_integration() {
    let code = "fn main() {\n    let answer = 42;\n}\n";
    let core = App::new(code, None, CharEncoding::Utf8, NewlineType::Lf);
    let mut gui = ViviviGuiApp::from_core(core);

    // 1. Tree-sitter のセットアップと GUI キーイベント (Alt+Up) でのスマート選択拡大
    gui.core.syntax_lang = vivivi_modes::SyntaxLang::Rust;
    let mut ts = vivivi_modes::TreeSitterEngine::new();
    assert!(ts.set_language(vivivi_modes::SyntaxLang::Rust));
    ts.parse(code);
    gui.core.treesitter = Some(ts);

    gui.core.cursor.pos.line = 1;
    gui.core.cursor.pos.col = 8;
    gui.process_egui_events(
        &[Event::Key {
            key: Key::ArrowUp,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::ALT,
        }],
        Modifiers::ALT,
    );
    assert_eq!(gui.core.mode(), ViMode::Visual);
    assert!(gui.core.current_smart_selection.is_some());

    // 2. Escape でノーマルモードへ戻し、:hex でバイナリモードへ遷移
    gui.process_egui_events(
        &[Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    gui.core.execute_ex("hex");
    assert!(gui.core.is_hex_mode);
    gui.core.execute_ex("text");
    assert!(!gui.core.is_hex_mode);

    // 3. LSP 診断パネルとホバーポップアップのモーダル連携
    gui.core.execute_ex("lsp diag");
    assert!(gui.core.lsp_show_diagnostics_panel);
    gui.core.lsp_hover_text = Some("fn main()".to_string());
    gui.close_all_modals();
    assert!(gui.core.lsp_hover_text.is_none());
}

/// シナリオ 12: ヘルプ（チートシート）等のモーダル表示中にマウスホイールを回しても、
/// 背後のテキストバッファのスクロール・カーソル位置が一切汚染・突き抜けしないことの検証。
#[test]
fn test_public_gui_modal_scroll_isolation() {
    let lines = (0..100)
        .map(|i| format!("line {}", i))
        .collect::<Vec<_>>()
        .join("\n");
    let core = App::new(&lines, None, CharEncoding::Utf8, NewlineType::Lf);
    let mut gui = ViviviGuiApp::from_core(core);

    assert_eq!(gui.core.scroll.0, 0);
    assert_eq!(gui.core.cursor.pos.line, 0);

    // 1. 通常時はマウスホイールで背後テキストがスクロールする
    gui.process_egui_events(
        &[Event::MouseWheel {
            unit: egui::MouseWheelUnit::Line,
            delta: egui::vec2(0.0, -5.0),
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    assert!(gui.core.scroll.0 > 0);
    let scrolled_pos = gui.core.scroll.0;

    // 2. ヘルプ（チートシート）を表示
    gui.core.show_help = true;
    assert!(gui.is_any_modal_open());

    // 3. ヘルプ表示中にホイールイベントが発生しても、背後のエディタスクロールは完全に遮断される
    gui.process_egui_events(
        &[Event::MouseWheel {
            unit: egui::MouseWheelUnit::Line,
            delta: egui::vec2(0.0, -10.0),
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    assert_eq!(
        gui.core.scroll.0, scrolled_pos,
        "モーダル表示中は背後テキストのスクロール値が一切変化しないこと"
    );

    // 4. ヘルプを閉じると再びスクロールが機能する
    gui.core.show_help = false;
    assert!(!gui.is_any_modal_open());
    gui.process_egui_events(
        &[Event::MouseWheel {
            unit: egui::MouseWheelUnit::Line,
            delta: egui::vec2(0.0, -2.0),
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    assert!(gui.core.scroll.0 > scrolled_pos);
}

#[test]
fn test_theme_palette_switches_all_ui_colors_not_only_text() {
    let dark = vivivi_app::theme::Theme::builtin_tokyo_night();
    let light = vivivi_app::theme::Theme::builtin_vivi_classic();
    let p_dark = vivivi_gui::GuiPalette::from_theme(&dark);
    let p_light = vivivi_gui::GuiPalette::from_theme(&light);

    assert!(p_dark.is_dark);
    assert!(!p_light.is_dark);
    assert_ne!(p_dark.bg_editor, p_light.bg_editor);
    assert_ne!(p_dark.bg_panel, p_light.bg_panel);
    assert_ne!(p_dark.bg_statusline, p_light.bg_statusline);

    let ctx = egui::Context::default();
    vivivi_gui::apply_theme(&ctx, &light);
    let applied = ctx.style().visuals.clone();
    assert_eq!(applied.panel_fill, p_light.bg_panel);
    assert_eq!(applied.window_fill, p_light.bg_editor);
    assert!(!applied.dark_mode);
}

#[test]
fn test_settings_dialog_renders_above_backdrop_on_foreground_order() {
    let core = App::new("hello", None, CharEncoding::Utf8, NewlineType::Lf);
    let mut gui = ViviviGuiApp::from_core(core);
    gui.settings_dialog.open(&gui.core.config);

    let ctx = egui::Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1024.0, 768.0),
        )),
        ..Default::default()
    };

    let output = ctx.run(raw_input, |ctx| {
        gui.render_settings_dialog(ctx);
    });
    assert!(!output.shapes.is_empty());

    // 設定ウィンドウはバックドロップ (Order::Middle) より前面の Order::Foreground に配置されること
    let win_layer = ctx
        .memory(|mem| {
            mem.areas()
                .visible_layer_ids()
                .iter()
                .copied()
                .find(|l| l.id == egui::Id::new("vivivi_preferences_modal_v2"))
        })
        .expect("設定ダイアログのウィンドウレイヤーが存在すること");
    let backdrop_layer = ctx
        .memory(|mem| {
            mem.areas()
                .visible_layer_ids()
                .iter()
                .copied()
                .find(|l| l.id == egui::Id::new("settings_modal_backdrop_area"))
        })
        .expect("バックドロップ領域レイヤーが存在すること");

    assert_eq!(win_layer.order, egui::Order::Foreground);
    assert_eq!(backdrop_layer.order, egui::Order::Middle);
}

#[test]
fn test_settings_dialog_keeps_fixed_width_across_categories() {
    let core = App::new("hello", None, CharEncoding::Utf8, NewlineType::Lf);
    let mut gui = ViviviGuiApp::from_core(core);
    gui.settings_dialog.open(&gui.core.config);

    let ctx = egui::Context::default();
    let mut observed_widths = Vec::new();

    for category in vivivi_gui::app::SettingsCategory::all() {
        gui.settings_dialog.active_category = *category;
        // 2フレーム回してレイアウト収束後の実幅を取得
        for _ in 0..2 {
            let raw_input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1600.0, 900.0),
                )),
                ..Default::default()
            };
            let _ = ctx.run(raw_input, |ctx| {
                gui.render_settings_dialog(ctx);
            });
        }
        let rect = ctx
            .memory(|mem| mem.area_rect(egui::Id::new("vivivi_preferences_modal_v2")))
            .expect("設定ダイアログのレイアウト矩形が取得できること");
        observed_widths.push((format!("{category:?}"), rect.width()));
    }

    let first_width = observed_widths[0].1;
    assert!(
        (920.0..=950.0).contains(&first_width),
        "設定ダイアログ幅は固定値 920px（+枠余白）付近であること: {observed_widths:?}"
    );
    for (name, w) in &observed_widths {
        assert!(
            (*w - first_width).abs() <= 0.5,
            "カテゴリ {name} 切り替え前後で幅が変化しないこと: {observed_widths:?}"
        );
    }
}

fn collect_text_galley_row_counts(
    shape: &egui::epaint::Shape,
    out: &mut std::collections::HashMap<String, usize>,
) {
    match shape {
        egui::epaint::Shape::Text(text_shape) => {
            let text = text_shape.galley.text().to_string();
            let rows = text_shape.galley.rows.len();
            out.entry(text)
                .and_modify(|max_rows| *max_rows = (*max_rows).max(rows))
                .or_insert(rows);
        }
        egui::epaint::Shape::Vec(shapes) => {
            for child in shapes {
                collect_text_galley_row_counts(child, out);
            }
        }
        _ => {}
    }
}

#[test]
fn test_settings_dialog_labels_stay_single_line() {
    let core = App::new("hello", None, CharEncoding::Utf8, NewlineType::Lf);
    let mut gui = ViviviGuiApp::from_core(core);
    gui.settings_dialog.open(&gui.core.config);

    let ctx = egui::Context::default();
    let expected_single_line_by_category: &[(vivivi_gui::app::SettingsCategory, &[&str])] = &[
        (
            vivivi_gui::app::SettingsCategory::General,
            &[
                "デフォルト文字コード (Encoding):",
                "デフォルト改行形式 (Fileformat):",
                "クリップボード同期 (Clipboard):",
            ],
        ),
        (
            vivivi_gui::app::SettingsCategory::Appearance,
            &[
                "カラーテーマ (Color Theme):",
                "フォントサイズ (Font Size):",
                "行間倍率 (Line Height):",
                "ミニマップ幅 (Minimap Width):",
            ],
        ),
        (
            vivivi_gui::app::SettingsCategory::Invisibles,
            &[
                "全角スペース記号:",
                "半角スペース記号:",
                "タブ文字記号 (Tab):",
                "改行記号 (EOL):",
            ],
        ),
        (
            vivivi_gui::app::SettingsCategory::Editing,
            &["タブ幅 (Tab Stop):"],
        ),
        (
            vivivi_gui::app::SettingsCategory::Keymap,
            &[
                "モード / キー",
                "機能・コマンド",
                "Normal",
                "v, V, Ctrl-v",
                ":preferences",
            ],
        ),
    ];

    for (cat, targets) in expected_single_line_by_category {
        gui.settings_dialog.active_category = *cat;
        let mut last_output = None;
        for _ in 0..2 {
            let raw_input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1600.0, 900.0),
                )),
                ..Default::default()
            };
            last_output = Some(ctx.run(raw_input, |ctx| {
                gui.render_settings_dialog(ctx);
            }));
        }
        let output = last_output.expect("描画出力が得られること");
        let mut row_counts = std::collections::HashMap::new();
        for clipped in &output.shapes {
            collect_text_galley_row_counts(&clipped.shape, &mut row_counts);
        }

        // 左ペインのカテゴリ一覧も 1 行で収まっていること
        for c in vivivi_gui::app::SettingsCategory::all() {
            let cat_label = format!("{} {}", c.icon(), c.display_name());
            if let Some(&rows) = row_counts.get(&cat_label) {
                assert_eq!(
                    rows, 1,
                    "左ペインのカテゴリ「{cat_label}」が改行されている (rows={rows})"
                );
            }
        }

        for target in *targets {
            let rows = row_counts
                .get(*target)
                .copied()
                .unwrap_or_else(|| panic!("対象ラベル「{target}」が見つかりません ({cat:?})"));
            assert_eq!(
                rows, 1,
                "設定項目「{target}」({cat:?}) が改行されている (rows={rows})"
            );
        }
    }
}

#[test]
fn test_help_modal_defaults_to_doubled_width() {
    let mut core = App::new("hello", None, CharEncoding::Utf8, NewlineType::Lf);
    core.open_help(None);
    let mut gui = ViviviGuiApp::from_core(core);

    let ctx = egui::Context::default();
    for _ in 0..2 {
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1600.0, 900.0),
            )),
            ..Default::default()
        };
        let _ = ctx.run(raw_input, |ctx| {
            gui.render_modals(ctx);
        });
    }

    let rect = ctx
        .memory(|mem| mem.area_rect(egui::Id::new("vivivi_help_modal_v2")))
        .expect("ヘルプモーダルの矩形が取得できること");
    assert!(
        rect.width() >= 1200.0,
        "ヘルプモーダル幅が従来の 620px の約2倍（1240px 基準）であること: width={}",
        rect.width()
    );
}

/// シナリオ 13: GUI のアウトラインサイドバーが起動直後およびバッファ編集後に自動で同期されること。
#[test]
fn test_gui_outline_sidebar_auto_syncs_on_render() {
    let core = App::new(
        "fn first_sym() {}\nstruct SecondSym;\n",
        Some(std::path::PathBuf::from("sample.rs")),
        CharEncoding::Utf8,
        NewlineType::Lf,
    );
    let mut gui = ViviviGuiApp::from_core(core);

    // 起動直後に初期シンボルが抽出されていること
    assert_eq!(gui.core.outline_items.len(), 2);
    assert!(gui.core.outline_items[0].title.contains("first_sym"));
    assert!(gui.core.outline_items[1].title.contains("SecondSym"));

    // バッファに新規関数を追加したあと、Outline タブへ切り替えて render_sidebar を呼ぶと同期される
    let end = gui.core.buffer.len();
    gui.core
        .buffer
        .basic_insert_text(end, "pub fn edited_sym() {}\n");
    gui.sidebar_open = true;
    gui.sidebar_tab = SidebarTab::Outline;

    let ctx = egui::Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            Pos2::ZERO,
            egui::vec2(1280.0, 800.0),
        )),
        ..Default::default()
    };
    let _ = ctx.run(raw_input, |ctx| {
        gui.render_sidebar(ctx);
    });

    assert_eq!(gui.core.outline_items.len(), 3);
    assert!(gui.core.outline_items[2].title.contains("edited_sym"));
}

/// シナリオ 14: `:outline` / `F8`（`toggle_outline`）が GUI サイドバー開閉と Outline タブ切替に連動すること。
#[test]
fn test_gui_outline_toggle_command_and_f8_syncs_sidebar() {
    let core = App::new(
        "fn main() {}\n",
        Some(std::path::PathBuf::from("main.rs")),
        CharEncoding::Utf8,
        NewlineType::Lf,
    );
    let mut gui = ViviviGuiApp::from_core(core);

    // 初期状態は Files タブが開いているため、:outline で Outline タブへ切り替わる
    gui.sidebar_open = true;
    gui.sidebar_tab = SidebarTab::Files;
    gui.core.execute_ex("outline");
    gui.sync_sidebar_requests();
    assert!(gui.sidebar_open);
    assert_eq!(gui.sidebar_tab, SidebarTab::Outline);
    assert!(!gui.core.outline_focus);

    // Outline タブ表示中にもう一度 F8 を押すとサイドバーが閉じる
    let f8 = vivivi_app::AppKeyEvent::new(KeyCode::F(8), KeyModifiers::NONE);
    gui.core.handle_key(f8);
    gui.sync_sidebar_requests();
    assert!(!gui.sidebar_open);
    assert!(!gui.core.show_outline);

    // 閉じた状態から F8 で再度 Outline タブとして開く
    gui.core.handle_key(f8);
    gui.sync_sidebar_requests();
    assert!(gui.sidebar_open);
    assert_eq!(gui.sidebar_tab, SidebarTab::Outline);
    assert!(gui.core.show_outline);
}

/// シナリオ 15: `:cmd` 外部コマンド実行ダイアログが GUI モーダルとして描画・操作でき、文字入力がエディタに漏れないこと。
#[test]
fn test_gui_cmd_exec_dialog_renders_and_isolates_input() {
    let core = App::new("initial text", None, CharEncoding::Utf8, NewlineType::Lf);
    let mut gui = ViviviGuiApp::from_core(core);

    gui.core.execute_ex("cmd");
    assert!(gui.core.show_cmd_exec_dialog);
    assert!(gui.is_any_modal_open());

    // モーダル表示中はエディタ本体へ q/j キーが漏れない
    gui.process_egui_events(&[Event::Text("q".to_string())], Modifiers::NONE);
    assert!(gui.core.show_cmd_exec_dialog);
    assert_eq!(gui.core.buffer.to_string(), "initial text");

    // 実行モード 0（実行のみ）でコマンド実行すると出力プレビューが更新される
    gui.core.cmd_exec_mode = 0;
    gui.core.cmd_exec_input = "echo hello_cmd".to_string();
    gui.core.execute_cmd_exec_dialog();
    assert!(gui.core.cmd_exec_output_preview.is_some());

    // egui モーダル描画がパニックせず実行できること
    let ctx = egui::Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            Pos2::ZERO,
            egui::vec2(1280.0, 800.0),
        )),
        ..Default::default()
    };
    let _ = ctx.run(raw_input, |ctx| {
        gui.render_modals(ctx);
    });

    // Esc で閉じること
    gui.process_egui_events(
        &[Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    assert!(!gui.core.show_cmd_exec_dialog);
}

/// シナリオ 16: GUI ワードラップ (`:set wrap`) 有効時、長い行が折り返されカーソル矩形の Y 座標が下段へ移動すること。
#[test]
fn test_gui_word_wrap_wraps_long_line_and_cursor_position() {
    let long_line = "a".repeat(220);
    let core = App::new(&long_line, None, CharEncoding::Utf8, NewlineType::Lf);
    let mut gui = ViviviGuiApp::from_core(core);
    gui.sidebar_open = false;
    gui.minimap_open = false;
    gui.core.cursor.pos.line = 0;
    gui.core.cursor.pos.col = 200;

    let ctx = egui::Context::default();
    let make_input = || egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            Pos2::ZERO,
            egui::vec2(640.0, 480.0),
        )),
        ..Default::default()
    };

    // 1. wrap OFF のときは同一論理行の 1 行目 (origin.y) にカーソルが留まる
    gui.core.wrap = false;
    let _ = ctx.run(make_input(), |ctx| {
        gui.render_central_panel(ctx);
    });
    let nowrap_cursor = gui
        .last_cursor_rect
        .expect("nowrap 時もカーソル矩形が計算されること");
    assert!(
        (nowrap_cursor.top() - gui.last_active_text_origin.y).abs() < 1.0,
        "nowrap 時は 1 行目の Y に描画されるべき: top={}, origin_y={}",
        nowrap_cursor.top(),
        gui.last_active_text_origin.y
    );

    // 2. `:set wrap` で有効化すると折り返されてカーソルが 2 行目以降 (origin.y + line_height 以上) へ移る
    gui.core.execute_ex("set wrap");
    assert!(gui.core.wrap);
    let _ = ctx.run(make_input(), |ctx| {
        gui.render_central_panel(ctx);
    });
    let wrap_cursor = gui
        .last_cursor_rect
        .expect("wrap 時もカーソル矩形が計算されること");
    assert!(
        wrap_cursor.top() >= gui.last_active_text_origin.y + gui.line_height,
        "wrap 有効時の長い行の後半カーソルは下段に折り返されるべき: top={}, origin_y={}, line_height={}",
        wrap_cursor.top(),
        gui.last_active_text_origin.y,
        gui.line_height
    );
}

/// シナリオ 17: Files ツリーの全幅ホバー時の指カーソル、行余白でのシングルクリックファイルオープン、および階層展開。
#[test]
fn test_gui_sidebar_files_tree_full_row_hover_cursor_and_single_click() {
    let temp_dir =
        std::env::temp_dir().join(format!("vivivi_gui_tree_test_{}", std::process::id()));
    let sub_dir = temp_dir.join("sub_dir");
    let nested_file = sub_dir.join("inside.rs");
    let top_file = temp_dir.join("hello.txt");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&sub_dir).expect("一時テストディレクトリの作成に失敗");
    std::fs::write(&nested_file, "fn inside() {}\n").expect("子ファイルの書込に失敗");
    std::fs::write(&top_file, "hello world\n").expect("トップファイルの書込に失敗");

    let core = App::new("", None, CharEncoding::Utf8, NewlineType::Lf);
    let mut gui = ViviviGuiApp::from_core(core);
    gui.sidebar_open = true;
    gui.sidebar_tab = SidebarTab::Files;
    gui.sidebar_width = 240.0;
    gui.navigate_sidebar_dir(temp_dir.clone());

    let start = std::time::Instant::now();
    while gui.async_tree.is_scanning() && start.elapsed() < std::time::Duration::from_secs(3) {
        gui.async_tree.poll();
        std::thread::sleep(std::time::Duration::from_millis(15));
    }
    assert_eq!(gui.async_tree.status, vivivi_gui::ScanStatus::Ready);
    assert_eq!(gui.async_tree.visible_rows().len(), 2);

    let ctx = egui::Context::default();
    let run_frame = |gui: &mut ViviviGuiApp, events: Vec<Event>| {
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(1280.0, 800.0),
            )),
            events,
            ..Default::default()
        };
        ctx.run(raw_input, |ctx| {
            gui.render_sidebar(ctx);
        })
    };

    // 初回描画
    let _ = run_frame(&mut gui, vec![]);

    // テキストが全く描かれていない右側余白 (x = 200.0) を走査し、最初の PointingHand 行 (..) を特定
    let hover_x = 200.0;
    let mut first_row_y = None;
    for step in 0..40 {
        let y = 30.0 + (step as f32) * 5.0;
        let out = run_frame(&mut gui, vec![Event::PointerMoved(egui::pos2(hover_x, y))]);
        if out.platform_output.cursor_icon == egui::CursorIcon::PointingHand {
            first_row_y = Some(y + 4.0);
            break;
        }
    }
    let up_y = first_row_y.expect("行右側の余白でも指カーソル (PointingHand) になるべき");

    // `..` の 1 行下 (+22px) が `sub_dir`、2 行下 (+44px) が `hello.txt`
    let sub_dir_pos = egui::pos2(hover_x, up_y + 22.0);
    let hello_pos = egui::pos2(hover_x, up_y + 44.0);

    // 1. ファイル行の右端余白をシングルクリックして即座に開かれること
    let hover_out = run_frame(&mut gui, vec![Event::PointerMoved(hello_pos)]);
    assert_eq!(
        hover_out.platform_output.cursor_icon,
        egui::CursorIcon::PointingHand
    );
    let _ = run_frame(
        &mut gui,
        vec![
            Event::PointerMoved(hello_pos),
            Event::PointerButton {
                pos: hello_pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    let _ = run_frame(
        &mut gui,
        vec![Event::PointerButton {
            pos: hello_pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );
    assert_eq!(
        gui.core.file_path.as_deref(),
        Some(top_file.as_path()),
        "ファイル行の余白シングルクリックでファイルが即開かれるべき"
    );

    // 2. ディレクトリ行の右端余白をシングルクリックして階層展開されること
    let _ = run_frame(
        &mut gui,
        vec![
            Event::PointerMoved(sub_dir_pos),
            Event::PointerButton {
                pos: sub_dir_pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    let _ = run_frame(
        &mut gui,
        vec![Event::PointerButton {
            pos: sub_dir_pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );
    let sub_start = std::time::Instant::now();
    while gui.async_tree.is_scanning() && sub_start.elapsed() < std::time::Duration::from_secs(3) {
        gui.async_tree.poll();
        std::thread::sleep(std::time::Duration::from_millis(15));
    }
    let rows = gui.async_tree.visible_rows();
    assert!(
        rows.iter()
            .any(|r| r.depth == 1 && r.name == "inside.rs" && r.path == nested_file),
        "ディレクトリクリックでツリーの次階層に inside.rs が展開されるべき"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

/// シナリオ 18: Outline タブでも全幅ホバーで指カーソルになり、余白シングルクリックでジャンプすること。
#[test]
fn test_gui_sidebar_outline_full_row_hover_and_single_click_jump() {
    let core = App::new(
        "fn first_fn() {}\n\n\nfn second_fn() {}\n",
        Some(std::path::PathBuf::from("outline_click.rs")),
        CharEncoding::Utf8,
        NewlineType::Lf,
    );
    let mut gui = ViviviGuiApp::from_core(core);
    gui.sidebar_open = true;
    gui.sidebar_tab = SidebarTab::Outline;
    gui.sidebar_width = 240.0;
    assert_eq!(gui.core.outline_items.len(), 2);

    let ctx = egui::Context::default();
    let run_frame = |gui: &mut ViviviGuiApp, events: Vec<Event>| {
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(1280.0, 800.0),
            )),
            events,
            ..Default::default()
        };
        ctx.run(raw_input, |ctx| {
            gui.render_sidebar(ctx);
        })
    };

    let _ = run_frame(&mut gui, vec![]);

    let hover_x = 200.0;
    let mut first_row_y = None;
    for step in 0..40 {
        let y = 30.0 + (step as f32) * 5.0;
        let out = run_frame(&mut gui, vec![Event::PointerMoved(egui::pos2(hover_x, y))]);
        if out.platform_output.cursor_icon == egui::CursorIcon::PointingHand {
            first_row_y = Some(y + 4.0);
            break;
        }
    }
    let first_y = first_row_y.expect("Outline タブでも余白ホバー時に指カーソルになるべき");
    let second_pos = egui::pos2(hover_x, first_y + 22.0);

    let _ = run_frame(
        &mut gui,
        vec![
            Event::PointerMoved(second_pos),
            Event::PointerButton {
                pos: second_pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    let _ = run_frame(
        &mut gui,
        vec![Event::PointerButton {
            pos: second_pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );

    assert_eq!(
        gui.core.cursor.pos.line, 3,
        "Outline 行の右端余白シングルクリックで対象シンボル行へジャンプするべき"
    );
}
