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
        app.recorded_macro_events.push(key);
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
    // Ctrl + C: 強制終了
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
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

    // F1: ヘルプを開く
    if key.code == KeyCode::F(1) {
        if app.show_help {
            app.show_help = false;
        } else {
            app.open_help(None);
        }
        return;
    }

    // ヘルプ表示中のキーハンドリング
    if app.show_help {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.show_help = false;
                app.status_message = Some("ヘルプを閉じました".to_string());
            }
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
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.show_preview = false;
                app.status_message = Some("印刷プレビューを閉じました".to_string());
            }
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

    // 2ペインファイラー表示中のキーハンドリング
    if app.show_filer {
        let mut to_open = None;
        if let Some(ref mut filer) = app.filer {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.close_filer();
                    app.status_message = Some("ファイラーを閉じました".to_string());
                    return;
                }
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
            app.close_filer();
            if let Err(e) = app.open_file(path.clone()) {
                app.status_message = Some(format!("ファイル読込エラー: {e}"));
            } else {
                app.status_message = Some(format!("ファイルを開きました: {}", path.display()));
            }
        }
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

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.show_recent_modal = false;
                app.status_message = Some("履歴モーダルを閉じました".to_string());
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.selected_recent_idx = app.selected_recent_idx.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') if total_count > 0 => {
                app.selected_recent_idx = (app.selected_recent_idx + 1).min(total_count - 1);
            }
            KeyCode::Enter if total_count > 0 => {
                if let Some(target_path) = all_entries.get(app.selected_recent_idx).cloned() {
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
        let history_len = app.vi.registers().history().len();
        match key.code {
            KeyCode::Esc => {
                app.show_clipboard_hist = false;
            }
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
        match key.code {
            KeyCode::Esc => {
                app.is_migemo_search = false;
                app.migemo_query.clear();
                app.status_message = Some("Migemo検索をキャンセルしました".to_string());
            }
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
        match key.code {
            KeyCode::Esc | KeyCode::Tab => {
                app.outline_focus = false;
                app.status_message =
                    Some("エディタフォーカス中 (Tab / Ctrl-O: アウトライン選択)".to_string());
            }
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

    // 3. Viモード別のキーハンドリング
    match app.mode() {
        ViMode::Insert => handle_insert_mode(app, key),
        ViMode::Replace => handle_replace_mode(app, key),
        ViMode::CommandLine => handle_command_line_mode(app, key),
        _ => handle_normal_mode(app, key),
    }
}

pub(crate) mod ex;
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
            dispatch_key_event(app, ev);
        }
        app.is_recording_macro = prev_recording;
        app.status_message = Some(format!("マクロ再生完了 ({count} 件のキーイベントを実行)"));
    } else {
        app.status_message = Some("再生可能なキーボードマクロがありません".to_string());
    }
}
