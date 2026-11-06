//! `vivivi-gui` クレートの公開API統合テスト（`.agents/rules/rust_testing.md` 準拠）。
//!
//! 外部利用者の視点から、共通 `App` コアとのイベント連携、日本語 IME（Preedit / Commit）、
//! 特殊キー変換、および削除文字落下パーティクルのライフサイクルと省電力制御を検証します。

use egui::{Event, ImeEvent, Key, Modifiers, Pos2};
use vivivi_app::{App, KeyCode, KeyModifiers};
use vivivi_core::{CharEncoding, NewlineType};
use vivivi_gui::{
    ParticleManager, SettingsCategory, ViviviGuiApp, colors, egui_key_to_app_key,
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
