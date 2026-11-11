//! キーイベント処理およびコマンドディスパッチモジュール。
//!
//! ターミナルから受信したキーストローク（[`KeyEvent`]）を解析し、
//! 現在のモード（Viノーマル、挿入、ビジュアル、罫線、CSV、モーダルダイアログ）に応じた
//! コマンド実行やバッファ編集へディスパッチします。

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use vivivi_modes::KeisenDirection;
use vivivi_vi::ViMode;

use super::app::App;
use super::window::WindowDirection;
pub mod windows_keys;

/// キーイベントを処理し、アプリケーション状態やテキストバッファを更新します。
///
/// Windows Terminal / Crossterm におけるキー解放（Release）イベントを自動無視して二重入力を防ぎ、
/// キーボードマクロ記録中であればイベントをキューに記録した上で、モード別のディスパッチ処理を実行します。
///
/// # 引数
/// - `app`: アプリケーション状態への可変参照
/// - `key`: 受信したキーイベント
///
/// # Panics
/// パニックは発生しません。不正なキー入力は無視されるかエラーメッセージとしてステータスラインに表示されます。
///
/// # Examples
/// ```rust,ignore
/// use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
/// use vivivi_app::tui::event::handle_key_event;
/// use vivivi_app::tui::App;
///
/// let mut app = App::new("", None, Default::default(), Default::default());
/// let key = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
/// handle_key_event(&mut app, key);
/// ```
pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    // Windows Terminal / Crossterm におけるキー解放 (Release) イベントを無視し、二重入力を防止
    if key.kind == KeyEventKind::Release {
        return;
    }

    // 状態メッセージを一旦クリア
    app.status_message = None;

    // キーボードマクロ記録トグル/再生キーかどうかの判定
    let is_macro_toggle = (key
        .modifiers
        .contains(KeyModifiers::CONTROL | KeyModifiers::ALT)
        && (key.code == KeyCode::Char('r') || key.code == KeyCode::Char('R')))
        || key.code == KeyCode::F(9);
    let is_macro_play = (key
        .modifiers
        .contains(KeyModifiers::CONTROL | KeyModifiers::ALT)
        && (key.code == KeyCode::Char('p') || key.code == KeyCode::Char('P')))
        || key.code == KeyCode::F(10);

    // マクロ記録中であれば、記録トグル/再生キー以外のキーストロークをバッファに保存
    if app.is_recording_macro && !is_macro_toggle && !is_macro_play {
        app.recorded_macro_events.push(key.into());
    }

    dispatch_key_event(app, key);

    // クラッシュリカバリ: 20打鍵ごと、または Esc キーで編集モードを抜けた際に自動スワップ保存
    app.key_stroke_count += 1;
    if app.key_stroke_count >= 20 || key.code == KeyCode::Esc {
        app.key_stroke_count = 0;
        app.save_swap_file();
    }
}

/// 内部キーイベントディスパッチ処理
fn dispatch_key_event(app: &mut App, key: KeyEvent) {
    // 1. グローバルショートカット
    // Ctrl + C: ビジュアルモード中はコピー（ヤンク）、それ以外は終了
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        let mode = app.mode();
        if mode == ViMode::Visual || mode == ViMode::VisualLine || mode == ViMode::VisualBlock {
            if let Some(text) = app.get_selected_text() {
                app.set_clipboard_text(&text);
                app.status_message = Some(format!("{} 文字をコピーしました", text.chars().count()));
            }
            app.cursor.anchor = None;
            app.vi.set_mode(ViMode::Normal);
            return;
        }
        app.should_quit = true;
        return;
    }

    // F9 または Ctrl+Alt+R: キーボードマクロ記録トグル
    if (key
        .modifiers
        .contains(KeyModifiers::CONTROL | KeyModifiers::ALT)
        && (key.code == KeyCode::Char('r') || key.code == KeyCode::Char('R')))
        || key.code == KeyCode::F(9)
    {
        app.toggle_macro_recording();
        return;
    }

    // F10 または Ctrl+Alt+P: キーボードマクロ再生
    if (key
        .modifiers
        .contains(KeyModifiers::CONTROL | KeyModifiers::ALT)
        && (key.code == KeyCode::Char('p') || key.code == KeyCode::Char('P')))
        || key.code == KeyCode::F(10)
    {
        play_macro(app);
        return;
    }

    // Alt-M または Ctrl+Alt+M: ダイナミックマクロ（増井式反復実行）
    if (key.modifiers.contains(KeyModifiers::ALT)
        || key
            .modifiers
            .contains(KeyModifiers::CONTROL | KeyModifiers::ALT))
        && (key.code == KeyCode::Char('m') || key.code == KeyCode::Char('M'))
    {
        app.trigger_dynamic_macro();
        return;
    }

    // Alt + Up / Alt + Down / Alt + '+' / Alt + '-': Tree-sitter スマート構文木選択 (拡大・縮小)
    if key.modifiers.contains(KeyModifiers::ALT) {
        match key.code {
            KeyCode::Up | KeyCode::Char('+') | KeyCode::Char('=') => {
                app.expand_smart_selection();
                return;
            }
            KeyCode::Down | KeyCode::Char('-') => {
                app.shrink_smart_selection();
                return;
            }
            _ => {}
        }
    }

    // F1: ヘルプを開く
    if key.code == KeyCode::F(1) {
        if app.show_help {
            app.show_help = false;
        } else {
            app.open_help(None);
        }
        return;
    }

    // Esc または Ctrl+[ (Vim互換Escキー) の判定
    let is_esc = key.code == KeyCode::Esc
        || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('['));

    // ヘルプ表示中のキーハンドリング
    if app.show_help {
        if is_esc || key.code == KeyCode::Char('q') {
            app.show_help = false;
            app.status_message = Some("ヘルプを閉じました".to_string());
            return;
        }
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                app.help_scroll += 1;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.help_scroll = app.help_scroll.saturating_sub(1);
            }
            KeyCode::PageDown | KeyCode::Char('d') => {
                app.help_scroll += 10;
            }
            KeyCode::PageUp | KeyCode::Char('u') => {
                app.help_scroll = app.help_scroll.saturating_sub(10);
            }
            KeyCode::Home | KeyCode::Char('g') => {
                app.help_scroll = 0;
            }
            KeyCode::End | KeyCode::Char('G') => {
                app.help_scroll = app.help_lines.len().saturating_sub(1);
            }
            _ => {}
        }
        return;
    }

    // 印刷プレビュー表示中のキーハンドリング
    if app.show_preview {
        if is_esc || key.code == KeyCode::Char('q') {
            app.show_preview = false;
            app.status_message = Some("印刷プレビューを閉じました".to_string());
            return;
        }
        match key.code {
            KeyCode::Char('p') | KeyCode::Char('P') => {
                if let Err(e) = app.export_pdf(None) {
                    app.status_message = Some(format!("PDF出力エラー: {e}"));
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.preview_scroll += 1;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.preview_scroll = app.preview_scroll.saturating_sub(1);
            }
            KeyCode::PageDown | KeyCode::Char('d') => {
                app.preview_scroll += 10;
            }
            KeyCode::PageUp | KeyCode::Char('u') => {
                app.preview_scroll = app.preview_scroll.saturating_sub(10);
            }
            _ => {}
        }
        return;
    }

    // 外部コマンド実行ダイアログ (cmdExecDlg) 表示中のキーハンドリング
    if app.show_cmd_exec_dialog {
        if is_esc {
            app.close_cmd_exec_dialog();
            return;
        }

        match key.code {
            KeyCode::Tab => {
                app.cmd_exec_mode = (app.cmd_exec_mode + 1) % 3;
            }
            KeyCode::Backspace => {
                app.cmd_exec_input.pop();
            }
            KeyCode::Enter => {
                let cmd = app.cmd_exec_input.trim().to_string();
                if !cmd.is_empty() {
                    match app.cmd_exec_mode {
                        0 => {
                            // 実行のみ (結果プレビュー)
                            match app.execute_shell_command(&cmd) {
                                Ok(out) => {
                                    app.cmd_exec_output_preview = Some(out);
                                    app.status_message = Some(format!("!{cmd} ➔ 実行完了"));
                                }
                                Err(e) => {
                                    app.cmd_exec_output_preview = Some(format!("エラー: {e}"));
                                    app.status_message = Some(format!("コマンド実行エラー: {e}"));
                                }
                            }
                        }
                        1 => {
                            // バッファ挿入 (:r !)
                            match app.read_command_into_buffer(&cmd, None) {
                                Ok(count) => {
                                    app.status_message =
                                        Some(format!(":r !{cmd} ➔ {count} 行を挿入しました"));
                                    app.close_cmd_exec_dialog();
                                }
                                Err(e) => {
                                    app.cmd_exec_output_preview = Some(format!("エラー: {e}"));
                                    app.status_message = Some(format!("コマンド読込エラー: {e}"));
                                }
                            }
                        }
                        2 => {
                            // 選択行置換 (:[range]!)
                            let (start, end) = match app.cursor.anchor {
                                Some(anchor) => (
                                    anchor.line.min(app.cursor.pos.line),
                                    anchor.line.max(app.cursor.pos.line),
                                ),
                                None => (app.cursor.pos.line, app.cursor.pos.line),
                            };
                            match app.filter_range_with_command(&cmd, start, end) {
                                Ok(count) => {
                                    app.status_message =
                                        Some(format!(":[range]!{cmd} ➔ {count} 行に置換しました"));
                                    app.close_cmd_exec_dialog();
                                }
                                Err(e) => {
                                    app.cmd_exec_output_preview = Some(format!("エラー: {e}"));
                                    app.status_message = Some(format!("置換エラー: {e}"));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            KeyCode::Char(c) => {
                app.cmd_exec_input.push(c);
            }
            _ => {}
        }
        return;
    }

    // TUI 設定ダイアログ表示中のキーハンドリング
    if app.show_preferences_dialog {
        if is_esc || key.code == KeyCode::Char('q') {
            app.cancel_preferences_dialog();

            return;
        }

        match key.code {
            KeyCode::Char('s') => {
                app.sync_runtime_to_config();
                let path = crate::config::AppConfig::default_config_path();
                match app.config.save_to(&path) {
                    Ok(()) => {
                        app.status_message =
                            Some(format!("設定を保存しました: {}", path.display()));
                    }
                    Err(e) => {
                        app.status_message = Some(format!("設定の保存に失敗しました: {e}"));
                    }
                }
                app.tui_settings_backup = None;
                app.show_preferences_dialog = false;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                app.tui_settings_in_items = false;
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => {
                app.tui_settings_in_items = true;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !app.tui_settings_in_items {
                    app.tui_settings_category = (app.tui_settings_category + 1).min(5);
                    app.tui_settings_item = 0;
                } else {
                    let max_items = crate::tui::ui::settings::get_category_items(
                        app,
                        app.tui_settings_category,
                    )
                    .len();
                    if max_items > 0 {
                        app.tui_settings_item = (app.tui_settings_item + 1).min(max_items - 1);
                    }
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !app.tui_settings_in_items {
                    app.tui_settings_category = app.tui_settings_category.saturating_sub(1);
                    app.tui_settings_item = 0;
                } else {
                    app.tui_settings_item = app.tui_settings_item.saturating_sub(1);
                }
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                if !app.tui_settings_in_items {
                    app.tui_settings_in_items = true;
                } else {
                    toggle_tui_settings_item(app);
                }
            }
            _ => {}
        }
        return;
    }

    // 2ペインファイラー表示中のキーハンドリング
    if app.show_filer {
        if is_esc || key.code == KeyCode::Char('q') {
            app.close_filer();
            app.status_message = Some("ファイラーを閉じました".to_string());
            return;
        }
        let mut to_open = None;
        if let Some(ref mut filer) = app.filer {
            match key.code {
                KeyCode::Tab => {
                    filer.switch_pane();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    filer.active_pane_mut().cursor_down();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    filer.active_pane_mut().cursor_up();
                }
                KeyCode::PageDown => {
                    for _ in 0..10 {
                        filer.active_pane_mut().cursor_down();
                    }
                }
                KeyCode::PageUp => {
                    for _ in 0..10 {
                        filer.active_pane_mut().cursor_up();
                    }
                }
                KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    for _ in 0..10 {
                        filer.active_pane_mut().cursor_down();
                    }
                }
                KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    for _ in 0..10 {
                        filer.active_pane_mut().cursor_up();
                    }
                }
                KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    for _ in 0..10 {
                        filer.active_pane_mut().cursor_down();
                    }
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    for _ in 0..10 {
                        filer.active_pane_mut().cursor_up();
                    }
                }
                KeyCode::Home | KeyCode::Char('g') => {
                    filer.active_pane_mut().cursor = 0;
                }
                KeyCode::End | KeyCode::Char('G') => {
                    let total = filer.active_pane().entries.len();
                    filer.active_pane_mut().cursor = total.saturating_sub(1);
                }
                KeyCode::Char(' ') => {
                    filer.active_pane_mut().toggle_select();
                }
                KeyCode::Backspace | KeyCode::Char('h') | KeyCode::Left => {
                    filer.active_pane_mut().up_dir();
                }
                KeyCode::Enter | KeyCode::Right => match filer.active_pane_mut().enter() {
                    Ok(Some(path)) => {
                        to_open = Some(path);
                    }
                    Ok(None) => {}
                    Err(e) => {
                        app.status_message = Some(format!("エラー: {e}"));
                    }
                },
                KeyCode::Char('c') => match filer.copy_to_other() {
                    Ok(cnt) => {
                        app.status_message =
                            Some(format!("{cnt} 件のファイルを反対画面へコピーしました"));
                    }
                    Err(e) => {
                        app.status_message = Some(format!("コピー失敗: {e}"));
                    }
                },
                KeyCode::Char('m') => match filer.move_to_other() {
                    Ok(cnt) => {
                        app.status_message =
                            Some(format!("{cnt} 件のファイルを反対画面へ移動しました"));
                    }
                    Err(e) => {
                        app.status_message = Some(format!("移動失敗: {e}"));
                    }
                },
                KeyCode::Char('d') | KeyCode::Delete | KeyCode::Char('x') => {
                    match filer.delete_selected() {
                        Ok(0) => {
                            app.status_message = Some(
                                "削除対象がありません（親ディレクトリ「..」は削除できません）"
                                    .to_string(),
                            );
                        }
                        Ok(cnt) => {
                            app.status_message = Some(format!("{cnt} 件のファイルを削除しました"));
                        }
                        Err(e) => {
                            app.status_message = Some(format!("削除失敗: {e}"));
                        }
                    }
                }
                _ => {}
            }
        } else {
            app.show_filer = false;
            return;
        }

        if let Some(path) = to_open {
            if app.buffer.is_modified() {
                app.status_message = Some(
                    "E37: 前回の変更が保存されていません。先に保存するか新しいタブ (:tabnew) で開いてください"
                        .to_string(),
                );
                return;
            }
            app.close_filer();
            if let Err(e) = app.open_file(path.clone()) {
                app.status_message = Some(format!("ファイル読込エラー: {e}"));
            } else {
                app.status_message = Some(format!("ファイルを開きました: {}", path.display()));
            }
        }
        return;
    }

    // LSP ホバー表示中のキーハンドリング
    if app.lsp_show_hover && (is_esc || key.code == KeyCode::Char('q')) {
        app.lsp_show_hover = false;
        app.lsp_hover_text = None;
        return;
    }

    // LSP 診断情報パネル表示中のキーハンドリング
    if app.lsp_show_diagnostics_panel && (is_esc || key.code == KeyCode::Char('q')) {
        app.lsp_show_diagnostics_panel = false;
        return;
    }

    // バイナリモード (Hex Editor) のキーハンドリング
    if app.is_hex_mode {
        handle_hex_mode_key(app, key, is_esc);
        return;
    }

    // Shift + F5 または F5: 罫線モードのトグル
    if key.code == KeyCode::F(5)
        || (key.modifiers.contains(KeyModifiers::SHIFT) && key.code == KeyCode::F(5))
    {
        app.keisen_mode.toggle();
        let state_msg = if app.keisen_mode.is_active {
            "罫線保護編集モード: ON (文字入力時に枠線を保護・維持 / Ctrl+矢印で描画)"
        } else {
            "罫線保護編集モード: OFF"
        };
        app.status_message = Some(state_msg.to_string());
        return;
    }

    // F6: クリップボード履歴ダイアログのトグル
    if key.code == KeyCode::F(6) {
        app.show_clipboard_hist = !app.show_clipboard_hist;
        app.selected_cb_idx = 0;
        return;
    }

    // F7: 最近使ったファイル & お気に入りダイアログのトグル
    if key.code == KeyCode::F(7) {
        app.show_recent_modal = !app.show_recent_modal;
        app.selected_recent_idx = 0;
        return;
    }

    // 最近使ったファイル & お気に入りモーダル表示中のキーハンドリング
    if app.show_recent_modal {
        let mut all_entries = Vec::new();
        for fav in &app.recent_manager.favorite_files {
            all_entries.push(fav.clone());
        }
        for rec in &app.recent_manager.recent_files {
            if !app.recent_manager.favorite_files.contains(rec) {
                all_entries.push(rec.clone());
            }
        }
        let total_count = all_entries.len();

        if is_esc || key.code == KeyCode::Char('q') {
            app.show_recent_modal = false;
            app.status_message = Some("履歴モーダルを閉じました".to_string());
            return;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                app.selected_recent_idx = app.selected_recent_idx.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') if total_count > 0 => {
                app.selected_recent_idx = (app.selected_recent_idx + 1).min(total_count - 1);
            }
            KeyCode::Enter if total_count > 0 => {
                if let Some(target_path) = all_entries.get(app.selected_recent_idx).cloned() {
                    if app.buffer.is_modified() {
                        app.status_message = Some(
                            "E37: 前回の変更が保存されていません。先に保存するか新しいタブ (:tabnew) で開いてください"
                                .to_string(),
                        );
                        return;
                    }
                    app.show_recent_modal = false;
                    if let Err(e) = app.open_file(target_path) {
                        app.status_message = Some(format!("ファイル読込エラー: {e}"));
                    }
                }
            }
            KeyCode::Char('f') if total_count > 0 => {
                if let Some(target_path) = all_entries.get(app.selected_recent_idx).cloned() {
                    let is_fav = app.recent_manager.toggle_favorite(target_path.clone());
                    let p = crate::tui::app::get_recent_file_path();
                    let _ = app.recent_manager.save_to_file(&p);
                    if is_fav {
                        app.status_message = Some(format!("お気に入りに追加: {:?}", target_path));
                    } else {
                        app.status_message = Some(format!("お気に入りを解除: {:?}", target_path));
                    }
                }
            }
            KeyCode::Char('d') | KeyCode::Delete if total_count > 0 => {
                if let Some(target_path) = all_entries.get(app.selected_recent_idx).cloned() {
                    app.recent_manager.remove_recent(&target_path);
                    if app.recent_manager.is_favorite(&target_path) {
                        app.recent_manager.toggle_favorite(target_path.clone());
                    }
                    let p = crate::tui::app::get_recent_file_path();
                    let _ = app.recent_manager.save_to_file(&p);
                    app.status_message = Some(format!("履歴から削除: {:?}", target_path));
                }
            }
            _ => {}
        }
        return;
    }

    // クリップボード履歴ダイアログ表示中のキーハンドリング
    if app.show_clipboard_hist {
        if is_esc || key.code == KeyCode::Char('q') {
            app.show_clipboard_hist = false;
            return;
        }
        let history_len = app.vi.registers().history().len();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                app.selected_cb_idx = app.selected_cb_idx.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') if history_len > 0 => {
                app.selected_cb_idx = (app.selected_cb_idx + 1).min(history_len - 1);
            }
            KeyCode::Enter if history_len > 0 => {
                app.paste_clipboard_item(app.selected_cb_idx);
            }
            _ => {}
        }
        return;
    }

    // Migemoローマ字検索プロンプト入力中
    if app.is_migemo_search {
        if is_esc {
            app.is_migemo_search = false;
            app.migemo_query.clear();
            app.status_message = Some("Migemo検索をキャンセルしました".to_string());
            return;
        }
        match key.code {
            KeyCode::Enter => {
                app.is_migemo_search = false;
                let query = std::mem::take(&mut app.migemo_query);
                app.execute_migemo_search(&query);
            }
            KeyCode::Backspace => {
                app.migemo_query.pop();
            }
            KeyCode::Char(c) => {
                app.migemo_query.push(c);
            }
            _ => {}
        }
        return;
    }

    // F2 / F8: アウトライン表示のトグル
    if key.code == KeyCode::F(2) || key.code == KeyCode::F(8) {
        app.toggle_outline();
        return;
    }

    // アウトラインペインにフォーカスがある場合のキーハンドリング
    if app.show_outline && app.outline_focus {
        if is_esc || key.code == KeyCode::Tab {
            app.outline_focus = false;
            app.status_message =
                Some("エディタフォーカス中 (Tab / Ctrl-O: アウトライン選択)".to_string());
            return;
        }
        match key.code {
            KeyCode::Char('q') => {
                app.toggle_outline();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.selected_outline_idx = app.selected_outline_idx.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !app.outline_items.is_empty() {
                    app.selected_outline_idx =
                        (app.selected_outline_idx + 1).min(app.outline_items.len() - 1);
                }
            }
            KeyCode::Enter => {
                app.jump_to_selected_outline();
            }
            _ => {}
        }
        return;
    }

    // アウトライン表示中にエディタ側で Tab を押した場合にアウトラインへフォーカス移行
    if app.show_outline
        && !app.outline_focus
        && app.mode() == ViMode::Normal
        && key.code == KeyCode::Tab
    {
        app.outline_focus = true;
        app.status_message =
            Some("アウトライン選択中 (j/k:選択, Enter:ジャンプ, Tab/Esc:エディタ復帰)".to_string());
        return;
    }

    // Ctrl-W ウィンドウプレフィックス処理
    if app.ctrl_w_pending {
        app.ctrl_w_pending = false;
        match key.code {
            KeyCode::Char('w') | KeyCode::Char('W') => {
                app.cycle_window();
            }
            KeyCode::Char('s') => {
                app.split_horizontal();
            }
            KeyCode::Char('v') => {
                app.split_vertical();
            }
            KeyCode::Char('c') | KeyCode::Char('q') => {
                app.close_window();
            }
            KeyCode::Char('o') => {
                app.only_window();
            }
            KeyCode::Char('h') | KeyCode::Left => {
                app.navigate_window(WindowDirection::Left);
            }
            KeyCode::Char('j') | KeyCode::Down => {
                app.navigate_window(WindowDirection::Down);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.navigate_window(WindowDirection::Up);
            }
            KeyCode::Char('l') | KeyCode::Right => {
                app.navigate_window(WindowDirection::Right);
            }
            _ => {}
        }
        return;
    }

    // ノーマルモードでの Ctrl-W 開始および Ctrl-O アウトラインフォーカス
    if app.mode() == ViMode::Normal && key.modifiers.contains(KeyModifiers::CONTROL) {
        if key.code == KeyCode::Char('w') {
            app.ctrl_w_pending = true;
            app.status_message =
                Some("Ctrl-W: [w:巡回, s:水平分割, v:垂直分割, c:閉じる, o:単一化]".to_string());
            return;
        }
        if key.code == KeyCode::Char('o') && app.show_outline {
            app.outline_focus = true;
            app.status_message =
                Some("アウトラインペイン選択中 (Enter:ジャンプ, Esc:エディタ復帰)".to_string());
            return;
        }
    }

    // 2. 罫線モード有効時の操作
    // Ctrl+矢印 / Alt+矢印 で描画、Shift+矢印 で消去
    // Ctrl+hjkl で描画、Shift+hjkl で消去
    // ※修飾キーなしの通常の矢印キー（十字キー）や素のhjklは通常のカーソル移動を行う
    if app.keisen_mode.is_active {
        let is_shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let is_alt = key.modifiers.contains(KeyModifiers::ALT);

        if is_ctrl || is_shift || is_alt {
            let is_erase = is_shift;

            let dir = match key.code {
                KeyCode::Left => Some(KeisenDirection::Left),
                KeyCode::Right => Some(KeisenDirection::Right),
                KeyCode::Up => Some(KeisenDirection::Up),
                KeyCode::Down => Some(KeisenDirection::Down),
                KeyCode::Char('h') | KeyCode::Char('H') => Some(KeisenDirection::Left),
                KeyCode::Char('l') | KeyCode::Char('L') => Some(KeisenDirection::Right),
                KeyCode::Char('k') | KeyCode::Char('K') => Some(KeisenDirection::Up),
                KeyCode::Char('j') | KeyCode::Char('J') => Some(KeisenDirection::Down),
                _ => None,
            };

            if let Some(dir) = dir {
                app.step_keisen(dir, is_erase);
                return;
            }
        }
    }

    // 2.8 Windows ライク操作体系 (Shift+矢印選択, Ctrl+C/V/X/Z/Y/A/S)
    let app_key_event: crate::AppKeyEvent = key.into();
    if windows_keys::handle_windows_shortcuts(app, &app_key_event) {
        return;
    }

    // 3. Viモード別のキーハンドリング
    match app.mode() {
        ViMode::Insert => handle_insert_mode(app, key),
        ViMode::Replace => handle_replace_mode(app, key),
        ViMode::CommandLine => handle_command_line_mode(app, key),
        _ => handle_normal_mode(app, key),
    }
}

pub mod ex;
pub use ex::execute_ex_command;
mod modes;

pub(crate) use modes::{
    handle_command_line_mode, handle_insert_mode, handle_normal_mode, handle_replace_mode,
};

/// 直近に記録されたキーボードマクロのキーストローク列を再生・実行します。
///
/// 再生中は一時的にマクロの二重記録を無効化し、記録されたイベントを順番に
/// キーディスパッチャへ投入します。再生が完了した際、またはマクロが存在しない場合は
/// ステータスラインにメッセージを表示します。
///
/// # 引数
/// - `app`: アプリケーション状態への可変参照
///
/// # Examples
/// ```rust,ignore
/// use vivivi_app::tui::event::play_macro;
/// use vivivi_app::tui::App;
///
/// let mut app = App::new("", None, Default::default(), Default::default());
/// play_macro(&mut app);
/// ```
pub fn play_macro(app: &mut App) {
    if let Some(events) = app.last_macro_events.clone() {
        let count = events.len();
        let prev_recording = app.is_recording_macro;
        app.is_recording_macro = false;
        for ev in events {
            dispatch_key_event(app, ev.into());
        }
        app.is_recording_macro = prev_recording;
        app.status_message = Some(format!("マクロ再生完了 ({count} 件のキーイベントを実行)"));
    } else {
        app.status_message = Some("再生可能なキーボードマクロがありません".to_string());
    }
}

/// TUI 設定ダイアログで Space または Enter が押されたときの項目トグル処理
fn toggle_tui_settings_item(app: &mut App) {
    let cat = app.tui_settings_category;
    let item = app.tui_settings_item;

    match cat {
        0 => match item {
            // 文字エンコーディング
            0 => {
                app.config.general.default_encoding =
                    match app.config.general.default_encoding.as_str() {
                        "utf-8" => "sjis".to_string(),
                        "sjis" => "euc-jp".to_string(),
                        _ => "utf-8".to_string(),
                    };
            }
            // 改行形式
            1 => {
                app.config.general.default_fileformat =
                    if app.config.general.default_fileformat == "dos" {
                        "unix".to_string()
                    } else {
                        "dos".to_string()
                    };
            }
            // クリップボード
            2 => {
                app.config.general.clipboard = match app.config.general.clipboard.as_str() {
                    "none" => "unnamed".to_string(),
                    "unnamed" => "unnamedplus".to_string(),
                    _ => "none".to_string(),
                };
                let opt = vivivi_vi::ClipboardOption::parse(&app.config.general.clipboard);
                app.vi.set_clipboard_option(opt);
            }
            // スワップ
            3 => {
                app.config.general.crash_recovery = !app.config.general.crash_recovery;
            }
            // 更新チェック
            4 => {
                app.config.general.check_update = !app.config.general.check_update;
            }
            _ => {}
        },
        1 => match item {
            // カラーテーマ
            0 => {
                let themes = crate::theme::Theme::list_available_themes();
                let cur = app.config.appearance.theme.to_lowercase().replace(' ', "-");
                let mut next_theme = "tokyo-night".to_string();
                if let Some(pos) = themes
                    .iter()
                    .position(|t| t == &cur || t.replace('-', " ") == cur)
                {
                    next_theme = themes[(pos + 1) % themes.len()].clone();
                }
                let loaded = crate::theme::Theme::load_by_name(&next_theme);
                app.config.appearance.theme = loaded.meta.name.clone();
                app.active_theme = loaded;
            }
            // 行番号
            1 => {
                app.show_line_numbers = !app.show_line_numbers;
                app.config.appearance.line_number = app.show_line_numbers;
            }
            // 相対行番号
            2 => {
                app.relative_number = !app.relative_number;
                app.config.appearance.relative_number = app.relative_number;
            }
            // 折り返し
            3 => {
                app.wrap = !app.wrap;
                app.config.appearance.wrap = app.wrap;
            }
            // ガター強調
            4 => {
                app.change_tracker.enabled = !app.change_tracker.enabled;
                app.config.appearance.gutter_markers = app.change_tracker.enabled;
            }
            // ミニマップ
            5 => {
                app.config.appearance.gui.show_minimap = !app.config.appearance.gui.show_minimap;
            }
            _ => {}
        },
        2 => match item {
            // 不可視文字
            0 => {
                app.list_chars_config.enabled = !app.list_chars_config.enabled;
                app.config.invisibles.enabled = app.list_chars_config.enabled;
            }
            // 全角スペース
            1 => {
                if app.config.invisibles.zenkaku_space == "□" {
                    app.config.invisibles.zenkaku_space = "　".to_string();
                } else {
                    app.config.invisibles.zenkaku_space = "□".to_string();
                }
                app.list_chars_config.zenkaku_space =
                    app.config.invisibles.zenkaku_space.clone().into();
            }
            // 半角スペース
            2 => {
                if app.config.invisibles.space == '·' {
                    app.config.invisibles.space = ' ';
                } else {
                    app.config.invisibles.space = '·';
                }
                app.list_chars_config.space = app.config.invisibles.space;
            }
            // タブ
            3 => {
                if app.config.invisibles.tab == ">---" {
                    app.config.invisibles.tab = "▸---".to_string();
                } else {
                    app.config.invisibles.tab = ">---".to_string();
                }
            }
            // 改行
            4 => {
                if app.config.invisibles.eol == '↵' {
                    app.config.invisibles.eol = '$';
                } else {
                    app.config.invisibles.eol = '↵';
                }
                app.list_chars_config.eol = app.config.invisibles.eol;
            }
            _ => {}
        },
        3 => match item {
            // タブ幅
            0 => {
                app.tab_width = if app.tab_width >= 16 {
                    2
                } else {
                    app.tab_width + 2
                };
                app.config.editing.tab_width = app.tab_width;
            }
            // スペース展開
            1 => {
                app.config.editing.expand_tab = !app.config.editing.expand_tab;
            }
            // 罫線保護
            2 => {
                app.config.editing.keisen_protect = !app.config.editing.keisen_protect;
            }
            // 括弧強調
            3 => {
                app.config.editing.match_paren = !app.config.editing.match_paren;
            }
            // dabbrev大文字無視
            4 => {
                app.config.editing.dabbrev_ignorecase = !app.config.editing.dabbrev_ignorecase;
            }
            // ignorecase
            5 => {
                app.config.editing.ignorecase = !app.config.editing.ignorecase;
            }
            // smartcase
            6 => {
                app.config.editing.smartcase = !app.config.editing.smartcase;
            }
            _ => {}
        },
        4 => match item {
            0 => {
                app.config.appearance.gui.enable_gravity_drop =
                    !app.config.appearance.gui.enable_gravity_drop;
            }
            1 => {
                app.config.appearance.gui.reactive_idle_sleep =
                    !app.config.appearance.gui.reactive_idle_sleep;
            }
            _ => {}
        },
        _ => {}
    }
}

/// バイナリモード (Hex Editor) 中のキーイベントを処理します。
fn handle_hex_mode_key(app: &mut App, key: KeyEvent, is_esc: bool) {
    if is_esc {
        app.hex_cursor.nibble = vivivi_modes::hex::Nibble::High;
        return;
    }

    let doc_len = app.hex_doc.as_ref().map(|d| d.len()).unwrap_or(0);
    if doc_len == 0 {
        return;
    }

    match key.code {
        // ペイン切り替え (Hex ⇄ ASCII)
        KeyCode::Tab => {
            app.hex_cursor.toggle_pane();
        }
        // カーソル移動 (左)
        KeyCode::Left | KeyCode::Char('h') => {
            app.hex_cursor.offset = app.hex_cursor.offset.saturating_sub(1);
            app.hex_cursor.nibble = vivivi_modes::hex::Nibble::High;
        }
        // カーソル移動 (右)
        KeyCode::Right | KeyCode::Char('l') => {
            if app.hex_cursor.offset + 1 < doc_len {
                app.hex_cursor.offset += 1;
            }
            app.hex_cursor.nibble = vivivi_modes::hex::Nibble::High;
        }
        // カーソル移動 (上: 16バイト戻る)
        KeyCode::Up | KeyCode::Char('k') => {
            app.hex_cursor.offset = app.hex_cursor.offset.saturating_sub(16);
            app.hex_cursor.nibble = vivivi_modes::hex::Nibble::High;
        }
        // カーソル移動 (下: 16バイト進む)
        KeyCode::Down | KeyCode::Char('j') => {
            if app.hex_cursor.offset + 16 < doc_len {
                app.hex_cursor.offset += 16;
            } else {
                app.hex_cursor.offset = doc_len.saturating_sub(1);
            }
            app.hex_cursor.nibble = vivivi_modes::hex::Nibble::High;
        }
        // 行頭移動 (0 / ^ / Home)
        KeyCode::Home | KeyCode::Char('0') | KeyCode::Char('^') => {
            app.hex_cursor.offset = (app.hex_cursor.offset / 16) * 16;
            app.hex_cursor.nibble = vivivi_modes::hex::Nibble::High;
        }
        // 行末移動 ($ / End)
        KeyCode::End | KeyCode::Char('$') => {
            let row_start = (app.hex_cursor.offset / 16) * 16;
            app.hex_cursor.offset = (row_start + 15).min(doc_len.saturating_sub(1));
            app.hex_cursor.nibble = vivivi_modes::hex::Nibble::High;
        }
        // 先頭移動 (g)
        KeyCode::Char('g') => {
            app.hex_cursor.offset = 0;
            app.hex_cursor.nibble = vivivi_modes::hex::Nibble::High;
        }
        // 末尾移動 (G)
        KeyCode::Char('G') => {
            app.hex_cursor.offset = doc_len.saturating_sub(1);
            app.hex_cursor.nibble = vivivi_modes::hex::Nibble::High;
        }
        // スクロール (PageDown / PageUp)
        KeyCode::PageDown => {
            app.hex_cursor.offset =
                (app.hex_cursor.offset + 16 * 10).min(doc_len.saturating_sub(1));
        }
        KeyCode::PageUp => {
            app.hex_cursor.offset = app.hex_cursor.offset.saturating_sub(16 * 10);
        }
        // Undo (u)
        KeyCode::Char('u') if key.modifiers == KeyModifiers::NONE => {
            if let Some(ref mut doc) = app.hex_doc
                && doc.undo()
            {
                app.status_message = Some("Undo 完了".to_string());
            }
        }
        // Redo (Ctrl-r)
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(ref mut doc) = app.hex_doc
                && doc.redo()
            {
                app.status_message = Some("Redo 完了".to_string());
            }
        }
        // 削除 (x)
        KeyCode::Char('x') if key.modifiers == KeyModifiers::NONE => {
            if let Some(ref mut doc) = app.hex_doc {
                doc.delete_bytes(app.hex_cursor.offset, 1);
                if app.hex_cursor.offset >= doc.len() && !doc.is_empty() {
                    app.hex_cursor.offset = doc.len() - 1;
                }
            }
        }
        // 編集入力 (文字入力)
        KeyCode::Char(ch) => match app.hex_cursor.pane {
            vivivi_modes::hex::HexPane::Hex => {
                if let Some(digit) = ch.to_digit(16) {
                    let hex_val = digit as u8;
                    let offset = app.hex_cursor.offset;
                    let nibble = app.hex_cursor.nibble;
                    if let Some(ref mut doc) = app.hex_doc {
                        doc.set_nibble(offset, nibble, hex_val);
                        match nibble {
                            vivivi_modes::hex::Nibble::High => {
                                app.hex_cursor.nibble = vivivi_modes::hex::Nibble::Low;
                            }
                            vivivi_modes::hex::Nibble::Low => {
                                app.hex_cursor.nibble = vivivi_modes::hex::Nibble::High;
                                if app.hex_cursor.offset + 1 < doc.len() {
                                    app.hex_cursor.offset += 1;
                                }
                            }
                        }
                    }
                }
            }
            vivivi_modes::hex::HexPane::Ascii => {
                if ch.is_ascii() {
                    let byte_val = ch as u8;
                    let offset = app.hex_cursor.offset;
                    if let Some(ref mut doc) = app.hex_doc {
                        doc.set_byte(offset, byte_val);
                        if app.hex_cursor.offset + 1 < doc.len() {
                            app.hex_cursor.offset += 1;
                        }
                    }
                }
            }
        },
        _ => {}
    }
}
