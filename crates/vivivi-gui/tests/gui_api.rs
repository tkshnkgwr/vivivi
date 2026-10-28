//! `vivivi-gui` クレートの公開API統合テスト（`.agents/rules/rust_testing.md` 準拠）。
//!
//! 外部利用者の視点から、共通 `App` コアとのイベント連携、日本語 IME（Preedit / Commit）、
//! 特殊キー変換、および削除文字落下パーティクルのライフサイクルと省電力制御を検証します。

use egui::{Event, ImeEvent, Key, Modifiers, Pos2};
use vivivi_app::{App, KeyCode, KeyModifiers};
use vivivi_core::{CharEncoding, NewlineType};
use vivivi_gui::{
    ParticleManager, ViviviGuiApp, colors, egui_key_to_app_key, extract_deleted_line_segment,
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
