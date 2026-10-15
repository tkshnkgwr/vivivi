//! キーイベント処理およびコマンドディスパッチモジュール。
//!
//! ターミナルから受信したキーストローク（[`KeyEvent`]）を解析し、
//! 現在のモード（Viノーマル、挿入、ビジュアル、罫線、CSV、モーダルダイアログ）に応じた
//! コマンド実行やバッファ編集へディスパッチします。

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use vivivi_modes::KeisenDirection;
use vivivi_vi::ViMode;

use super::app::App;

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

    // F8: アウトライン表示のトグル
    if key.code == KeyCode::F(8) {
        app.toggle_outline();
        return;
    }

    // アウトラインペインにフォーカスがある場合のキーハンドリング
    if app.show_outline && app.outline_focus {
        match key.code {
            KeyCode::Esc => {
                app.outline_focus = false;
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
            KeyCode::Char('h') | KeyCode::Char('k') | KeyCode::Left | KeyCode::Up => {
                if app.windows.len() > 1 {
                    app.active_window = if app.active_window == 0 {
                        app.windows.len() - 1
                    } else {
                        app.active_window - 1
                    };
                    app.cursor = app.windows[app.active_window].cursor.clone();
                    app.scroll = app.windows[app.active_window].scroll;
                }
            }
            KeyCode::Char('l') | KeyCode::Char('j') | KeyCode::Right | KeyCode::Down => {
                app.cycle_window();
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

/// 挿入モードのキーハンドリング
fn handle_insert_mode(app: &mut App, key: KeyEvent) {
    // Ctrl-L: 動的補完（前方候補へ）
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('l') {
        app.trigger_dabbrev(true);
        return;
    }
    // Ctrl-M: 動的補完（前候補へ戻る）
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('m') {
        app.trigger_dabbrev(false);
        return;
    }

    // Ctrl-; または Alt-; または F4: 静的略語展開
    if (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char(';'))
        || (key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Char(';'))
        || key.code == KeyCode::F(4)
    {
        app.trigger_abbrev_expansion();
        return;
    }

    // Ctrl-, または Alt-,: Zen-Coding (Emmet) 展開
    if (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char(','))
        || (key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Char(','))
    {
        app.trigger_zen_coding();
        return;
    }

    // Ctrl-J: 字句編集予測
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('j') {
        app.trigger_lex_edit();
        return;
    }

    // 補完キー以外の操作時は補完セッションをリセット
    app.dabbrev.reset();

    match key.code {
        KeyCode::Esc => {
            app.feed_vi_char('\x1b');
            // ノーマルモード復帰時はカーソルが文字境界を超えないよう左へ1文字補正
            if app.cursor.pos.col > 0 {
                app.cursor.pos.col -= 1;
            }
            app.clamp_cursor();
        }
        KeyCode::Char(c) => {
            app.vi.record_inserted_char(c);
            app.dynamic_macro.record_char(c);

            // 罫線モード有効時は枠線保護編集
            if app.keisen_mode.is_active {
                if !vivivi_modes::KeisenMode::insert_char_protected(
                    &mut app.buffer,
                    &mut app.cursor.pos,
                    c,
                ) {
                    app.status_message =
                        Some("罫線保護: 枠内に余白がないため挿入できません".to_string());
                }
            } else {
                let cur = app.cursor_pos();
                let pos = app.buffer.cursor_to_position(cur);
                app.buffer.insert_char(pos, c);
                app.cursor.pos.col += 1;
            }
        }
        KeyCode::Enter => {
            app.dynamic_macro.record_char('\n');
            let cur = app.cursor_pos();
            let pos = app.buffer.cursor_to_position(cur);
            app.buffer.insert_char(pos, '\n');
            app.cursor.pos.line += 1;
            app.cursor.pos.col = 0;
        }
        KeyCode::Backspace => {
            if app.keisen_mode.is_active {
                vivivi_modes::KeisenMode::delete_char_protected(
                    &mut app.buffer,
                    &mut app.cursor.pos,
                    true,
                );
            } else {
                let cur = app.cursor_pos();
                let pos = app.buffer.cursor_to_position(cur);
                if pos > 0 {
                    app.buffer.delete_text(pos - 1, 1, true);
                    if cur.col > 0 {
                        app.cursor.pos.col -= 1;
                    } else if cur.line > 0 {
                        let prev_line = cur.line - 1;
                        let prev_line_len = app
                            .buffer
                            .get_line(prev_line)
                            .map(|l| l.chars().count())
                            .unwrap_or(0);
                        app.cursor.pos.line = prev_line;
                        app.cursor.pos.col = prev_line_len;
                    }
                }
            }
        }
        KeyCode::Delete => {
            if app.keisen_mode.is_active {
                vivivi_modes::KeisenMode::delete_char_protected(
                    &mut app.buffer,
                    &mut app.cursor.pos,
                    false,
                );
            } else {
                let cur = app.cursor_pos();
                let pos = app.buffer.cursor_to_position(cur);
                if pos < app.buffer.len() {
                    app.buffer.delete_text(pos, 1, true);
                }
            }
        }
        KeyCode::Left => {
            app.cursor.pos.col = app.cursor.pos.col.saturating_sub(1);
        }
        KeyCode::Right => {
            let cur_line = app.cursor.pos.line;
            let line_len = app
                .buffer
                .get_line(cur_line)
                .map(|l| l.chars().count())
                .unwrap_or(0);
            app.cursor.pos.col = (app.cursor.pos.col + 1).min(line_len);
        }
        KeyCode::Up => {
            app.cursor.pos.line = app.cursor.pos.line.saturating_sub(1);
            app.clamp_cursor();
        }
        KeyCode::Down => {
            let total_lines = app.buffer.line_count();
            app.cursor.pos.line = (app.cursor.pos.line + 1).min(total_lines.saturating_sub(1));
            app.clamp_cursor();
        }
        KeyCode::Home => {
            app.cursor.pos.col = 0;
        }
        KeyCode::End => {
            let cur_line = app.cursor.pos.line;
            let line_len = app
                .buffer
                .get_line(cur_line)
                .map(|l| l.chars().count())
                .unwrap_or(0);
            app.cursor.pos.col = line_len;
        }
        KeyCode::PageUp => {
            app.scroll_full_page(false);
        }
        KeyCode::PageDown => {
            app.scroll_full_page(true);
        }
        _ => {}
    }
}

/// 置換（リプレース）モードのキーハンドリング (R)
fn handle_replace_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.feed_vi_char('\x1b');
            // ノーマルモード復帰時はカーソルが文字境界を超えないよう左へ1文字補正
            if app.cursor.pos.col > 0 {
                app.cursor.pos.col -= 1;
            }
            app.clamp_cursor();
        }
        KeyCode::Char(c) => {
            app.vi.record_inserted_char(c);
            app.dynamic_macro.record_char(c);

            let cur = app.cursor_pos();
            let line_len = app
                .buffer
                .get_line(cur.line)
                .map(|l| l.chars().count())
                .unwrap_or(0);

            let pos = app.buffer.cursor_to_position(cur);
            app.buffer.open_undo_block();
            if cur.col < line_len {
                // 既存の文字を上書き
                app.buffer.delete_text(pos, 1, false);
                app.buffer.insert_char(pos, c);
            } else {
                // 行末以降なら追加挿入
                app.buffer.insert_char(pos, c);
            }
            app.buffer.close_undo_block();
            app.cursor.pos.col += 1;
        }
        KeyCode::Enter => {
            app.dynamic_macro.record_char('\n');
            let cur = app.cursor_pos();
            let pos = app.buffer.cursor_to_position(cur);
            app.buffer.insert_char(pos, '\n');
            app.cursor.pos.line += 1;
            app.cursor.pos.col = 0;
        }
        KeyCode::Backspace => {
            // 置換モード中の Backspace は直前の置換を Undo してカーソルを左に戻す
            if app.cursor.pos.col > 0 {
                app.buffer.undo();
                app.cursor.pos.col -= 1;
            }
        }
        KeyCode::Left => {
            app.cursor.pos.col = app.cursor.pos.col.saturating_sub(1);
        }
        KeyCode::Right => {
            let cur_line = app.cursor.pos.line;
            let line_len = app
                .buffer
                .get_line(cur_line)
                .map(|l| l.chars().count())
                .unwrap_or(0);
            app.cursor.pos.col = (app.cursor.pos.col + 1).min(line_len);
        }
        KeyCode::Up => {
            app.cursor.pos.line = app.cursor.pos.line.saturating_sub(1);
            app.clamp_cursor();
        }
        KeyCode::Down => {
            let total_lines = app.buffer.line_count();
            app.cursor.pos.line = (app.cursor.pos.line + 1).min(total_lines.saturating_sub(1));
            app.clamp_cursor();
        }
        KeyCode::Home => {
            app.cursor.pos.col = 0;
        }
        KeyCode::End => {
            let cur_line = app.cursor.pos.line;
            let line_len = app
                .buffer
                .get_line(cur_line)
                .map(|l| l.chars().count())
                .unwrap_or(0);
            app.cursor.pos.col = line_len;
        }
        _ => {}
    }
}

/// コマンドラインモードのキーハンドリング
fn handle_command_line_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.feed_vi_char('\x1b');
        }
        KeyCode::Enter => {
            let cmd_text = app
                .vi
                .parser()
                .command_line_buffer()
                .map(|(_, s)| s.to_string())
                .unwrap_or_default();
            let prefix = app
                .vi
                .parser()
                .command_line_buffer()
                .map(|(p, _)| *p)
                .unwrap_or(':');

            // 通常の / や ? 検索実行時は Migemo 検索状態をクリアして通常検索を優先
            if prefix == '/' || prefix == '?' {
                app.migemo_last_regex = None;
            }

            // コマンド実行前にViEngineをノーマルモードに戻す
            app.feed_vi_char('\n');

            // exコマンドの解釈 (':' の場合のみ実行)
            if prefix == ':' {
                execute_ex_command(app, &cmd_text);
            }
        }
        KeyCode::Backspace => {
            app.feed_vi_char('\x08');
        }
        KeyCode::Char(c) => {
            app.feed_vi_char(c);
        }
        _ => {}
    }
}

/// exコマンドの実行処理 (:w, :q, :wq, :s, etc.)
fn execute_ex_command(app: &mut App, cmd: &str) {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return;
    }

    match trimmed {
        "w" => {
            if let Err(e) = app.save_file() {
                app.status_message = Some(format!("保存エラー: {e}"));
            }
        }
        "q" | "qa" | "qall" => {
            if app.buffer.is_modified() {
                app.status_message = Some(
                    "E37: 前回の変更が保存されていません (! を追加して :q! で強制終了)".to_string(),
                );
            } else {
                app.should_quit = true;
            }
        }
        "q!" | "qa!" | "qall!" => {
            app.should_quit = true;
        }
        "wq" | "x" => {
            if let Err(e) = app.save_file() {
                app.status_message = Some(format!("保存エラー: {e}"));
            } else {
                app.should_quit = true;
            }
        }
        _ if trimmed.starts_with("w ") => {
            let path_str = trimmed[2..].trim();
            app.file_path = Some(std::path::PathBuf::from(path_str));
            if let Err(e) = app.save_file() {
                app.status_message = Some(format!("保存エラー: {e}"));
            }
        }
        _ if trimmed.starts_with("%s/") || trimmed.starts_with("s/") => {
            app.status_message = Some(format!("置換実行: {trimmed}"));
        }

        // --- 検索ハイライト消去 (:noh, :nohl, :nohlsearch, :set nohls, :set hls) ---
        "noh" | "nohl" | "nohlsearch" => {
            app.clear_search_highlight();
            app.status_message = Some("検索ハイライトを消去しました (:nohlsearch)".to_string());
        }
        "set nohls" | "set nohlsearch" => {
            app.clear_search_highlight();
            app.status_message = Some("検索ハイライト: OFF (:set nohlsearch)".to_string());
        }
        "set hls" | "set hlsearch" => {
            app.is_search_highlight_visible = true;
            app.status_message = Some("検索ハイライト: ON (:set hlsearch)".to_string());
        }
        // --- テキストアライン (:align <target>, :align left/center/right) ---
        "align left" | "left" => {
            app.align_keisen_box(vivivi_modes::BoxAlignment::Left);
        }
        "align center" | "center" => {
            app.align_keisen_box(vivivi_modes::BoxAlignment::Center);
        }
        "align right" | "right" => {
            app.align_keisen_box(vivivi_modes::BoxAlignment::Right);
        }
        _ if trimmed.starts_with("align ") => {
            let target = trimmed[6..].trim();
            match target {
                "left" => app.align_keisen_box(vivivi_modes::BoxAlignment::Left),
                "center" => app.align_keisen_box(vivivi_modes::BoxAlignment::Center),
                "right" => app.align_keisen_box(vivivi_modes::BoxAlignment::Right),
                _ => app.align_selection_or_buffer(target),
            }
        }

        // --- リナンバ (:renumber [start] [step], :renum [start] [step]) ---
        _ if trimmed.starts_with("renumber") || trimmed.starts_with("renum") => {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            let start = parts
                .get(1)
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(1);
            let step = parts
                .get(2)
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(1);
            app.renumber_selection_or_buffer(start, step);
        }

        // --- 行頭行末一括挿入 (:insbeg, :insend, :begend, :insbegend) ---
        _ if trimmed.starts_with("insbeg ") => {
            let beg = &trimmed[7..];
            app.insert_beg_end_in_range(beg, "", true, true);
        }
        _ if trimmed.starts_with("insend ") => {
            let end = &trimmed[7..];
            app.insert_beg_end_in_range("", end, false, true);
        }
        _ if trimmed.starts_with("begend ") || trimmed.starts_with("insbegend ") => {
            let rest = trimmed
                .strip_prefix("begend ")
                .or_else(|| trimmed.strip_prefix("insbegend "))
                .unwrap_or(trimmed)
                .trim();
            if let Some((b, e)) = rest.split_once(' ') {
                app.insert_beg_end_in_range(b.trim(), e.trim(), true, true);
            } else {
                app.insert_beg_end_in_range(rest, rest, true, true);
            }
        }

        // --- 静的略語展開 (:abbrev [key val], :ab [key val]) ---
        _ if trimmed.starts_with("abbrev ") || trimmed.starts_with("ab ") => {
            let rest = trimmed
                .strip_prefix("abbrev ")
                .or_else(|| trimmed.strip_prefix("ab "))
                .unwrap_or(trimmed)
                .trim();
            if let Some((k, v)) = rest.split_once(' ') {
                app.abbrev.register(k.trim(), v.trim());
                app.status_message = Some(format!("略語登録: '{}' -> '{}'", k.trim(), v.trim()));
            } else {
                app.status_message = Some(format!("略語登録数: {} 件", app.abbrev.len()));
            }
        }
        "abbrev" | "ab" => {
            app.status_message = Some(format!(
                "略語登録数: {} 件 (挿入モードで Ctrl-; / Alt-; / F4 で展開)",
                app.abbrev.len()
            ));
        }

        // --- ダイナミックマクロ (:dmacro) ---
        "dmacro" => {
            app.trigger_dynamic_macro();
        }

        // --- 字句編集予測 (:lexedit) ---
        "lexedit" => {
            app.trigger_lex_edit();
        }

        // --- Zen-Coding展開 (:zen [expr], :zencoding [expr]) ---
        _ if trimmed.starts_with("zen ") || trimmed.starts_with("zencoding ") => {
            let query = trimmed
                .strip_prefix("zen ")
                .or_else(|| trimmed.strip_prefix("zencoding "))
                .unwrap_or(trimmed)
                .trim();
            if let Some(expanded) = vivivi_modes::expand_zen_coding(query) {
                let cur = app.cursor_pos();
                let pos = app.buffer.cursor_to_position(cur);
                app.buffer.open_undo_block();
                app.buffer.insert_text(pos, &expanded);
                app.buffer.close_undo_block();
                app.status_message = Some(format!("Zen-Coding展開: '{query}'"));
            } else {
                app.status_message = Some(format!("Zen-Coding構文エラー: '{query}'"));
            }
        }
        "zen" | "zencoding" => {
            app.trigger_zen_coding();
        }

        // --- キーボードマクロ (:macro record / stop / play) ---
        "macro record" => {
            if !app.is_recording_macro {
                app.toggle_macro_recording();
            }
        }
        "macro stop" => {
            if app.is_recording_macro {
                app.toggle_macro_recording();
            }
        }
        "macro play" => {
            play_macro(app);
        }

        // --- AI連携 (:ai [instruction]) ---
        _ if trimmed.starts_with("ai ") || trimmed == "ai" => {
            execute_ai_command(app, trimmed);
        }

        // --- 画面分割 (:split, :vsplit, :close, :only) ---
        "split" | "sp" => {
            app.split_horizontal();
        }
        "vsplit" | "vs" => {
            app.split_vertical();
        }
        "close" | "c" => {
            app.close_window();
        }
        "only" => {
            app.only_window();
        }

        // --- アウトライン (:outline) ---
        "outline" => {
            app.toggle_outline();
        }

        // --- 文字変換 (:zenkaku, :hankaku, :hiragana, :katakana, etc.) ---
        "zenkaku" | "%zenkaku" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToZenkakuAlphanumeric);
        }
        "hankaku" | "%hankaku" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToHankakuAlphanumeric);
        }
        "hiragana" | "%hiragana" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToHiragana);
        }
        "katakana" | "%katakana" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToKatakana);
        }
        "hankata" | "%hankata" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToHankakuKatakana);
        }
        "zenkata" | "%zenkata" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToZenkakuKatakana);
        }
        "toupper" | "%toupper" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToUppercase);
        }
        "tolower" | "%tolower" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToLowercase);
        }

        // --- BOX選択数値集計 (:calc, :sum, :total) ---
        "calc" | "sum" | "total" => {
            app.calculate_current_box_selection();
        }

        // --- クリップボード履歴 (:cbhist, :history) ---
        "cbhist" | "history" | "clipboard" => {
            app.show_clipboard_hist = true;
            app.selected_cb_idx = 0;
            app.status_message =
                Some("クリップボード履歴 (j/k:選択, Enter:貼付, Esc:閉じる)".to_string());
        }

        // --- ヘルプ (:help, :h) ---
        "help" | "h" => {
            app.open_help(None);
        }
        _ if trimmed.starts_with("help ") || trimmed.starts_with("h ") => {
            let topic = trimmed.split_whitespace().nth(1);
            app.open_help(topic);
        }

        // --- 最近使ったファイル & お気に入り (:recent, :mru, :fav, :favorite, :e) ---
        "recent" | "mru" | "favs" => {
            app.show_recent_modal = true;
            app.selected_recent_idx = 0;
            app.status_message = Some(
                "最近使ったファイル (j/k:選択, Enter:開く, f:お気に入り, d:削除, Esc:閉じる)"
                    .to_string(),
            );
        }
        "fav" | "favorite" | "addfav" => {
            app.toggle_favorite_current_file();
        }
        _ if trimmed.starts_with("e ") || trimmed.starts_with("edit ") => {
            let path_str = trimmed
                .strip_prefix("edit ")
                .or_else(|| trimmed.strip_prefix("e "))
                .unwrap_or(trimmed)
                .trim();
            if let Err(e) = app.open_file(std::path::PathBuf::from(path_str)) {
                app.status_message = Some(format!("ファイル読込エラー: {e}"));
            }
        }

        // --- 不可視文字可視化 (:set list / :set nolist / :list / :nolist) ---
        "list" | "set list" => {
            app.list_chars_config.enabled = true;
            app.status_message =
                Some("不可視文字表示 (全角スペース□ / タブ▸--- / 改行$): ON".to_string());
        }
        "nolist" | "set nolist" => {
            app.list_chars_config.enabled = false;
            app.status_message = Some("不可視文字表示: OFF".to_string());
        }

        // --- 変更行ガター強調 (:set gutter / :set nogutter / :gutter / :nogutter) ---
        "gutter" | "set gutter" => {
            app.change_tracker.enabled = true;
            app.status_message =
                Some("変更行ガター強調表示 (黄:未保存, 緑:保存済): ON".to_string());
        }
        "nogutter" | "set nogutter" => {
            app.change_tracker.enabled = false;
            app.status_message = Some("変更行ガター強調表示: OFF".to_string());
        }

        // --- ToDo抽出・一覧プレビュー (:todo, :undone) ---
        "todo" | "undone" => {
            let content = app.buffer.to_string();
            let file_name = app.file_path.as_ref().and_then(|p| p.to_str());
            let todos = vivivi_modes::scan_buffer_todos(&content, file_name);
            if todos.is_empty() {
                app.status_message =
                    Some("バッファ内に ToDo タグは見つかりませんでした".to_string());
            } else {
                let mut lines = vec![
                    format!("# バッファ内 ToDo 一覧 (全 {} 件)", todos.len()),
                    "".to_string(),
                ];
                for t in &todos {
                    lines.push(format!("* 行 {:>4} [{:?}]: {}", t.lineno, t.tag, t.message));
                }
                app.preview_lines = lines;
                app.show_preview = true;
                app.preview_scroll = 0;
                app.status_message = Some(format!(
                    "ToDo: {} 件を抽出しました (Escで閉じる)",
                    todos.len()
                ));
            }
        }

        // --- 差分比較プレビュー (:diff) ---
        "diff" => {
            if let Some(ref path) = app.file_path {
                if let Ok(orig) = std::fs::read_to_string(path) {
                    let cur = app.buffer.to_string();
                    let diff_text = vivivi_modes::create_unified_diff(
                        &orig,
                        &cur,
                        &format!("a/{}", path.display()),
                        &format!("b/{}", path.display()),
                    );
                    app.preview_lines = diff_text.lines().map(|s| s.to_string()).collect();
                    app.show_preview = true;
                    app.preview_scroll = 0;
                    app.status_message = Some("保存前との差分を表示中 (Escで閉じる)".to_string());
                } else {
                    app.status_message = Some("ディスク上の元ファイルが読み込めません".to_string());
                }
            } else {
                app.status_message = Some("ファイル名のないバッファはdiffできません".to_string());
            }
        }

        // --- 差分ログ世代一覧 (:difflog) ---
        "difflog" => {
            if let Some(ref path) = app.file_path {
                let log_res = vivivi_modes::DiffLog::load_from_file(path);
                if let Ok(difflog) = log_res {
                    let mut lines = vec![
                        format!("# 差分ログ世代履歴 (全 {} 世代)", difflog.entries.len()),
                        "".to_string(),
                    ];
                    for e in &difflog.entries {
                        lines.push(format!(
                            "* [世代 #{}] {} - {}",
                            e.generation, e.timestamp, e.comment
                        ));
                    }
                    app.preview_lines = lines;
                    app.show_preview = true;
                    app.preview_scroll = 0;
                    app.status_message = Some(format!(
                        "差分ログ: {} 世代 (Escで閉じる)",
                        difflog.entries.len()
                    ));
                } else {
                    app.status_message = Some("差分ログ (.difflog) が存在しません".to_string());
                }
            } else {
                app.status_message =
                    Some("ファイル名のないバッファはdifflogがありません".to_string());
            }
        }

        // --- インラインファイラー (:filer, :explore) ---
        "filer" | "explore" => {
            let cur_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let filer = vivivi_modes::TwoPaneFiler::new(cur_dir);
            let mut lines = vec![
                format!(
                    "# 2ペイン ファイラープレビュー (作業ディレクトリ: {})",
                    filer.left.current_dir.display()
                ),
                "".to_string(),
                "## ファイル一覧:".to_string(),
            ];
            for ent in &filer.left.entries {
                let prefix = if ent.is_dir { "[DIR] " } else { "      " };
                lines.push(format!("* {prefix}{}", ent.name));
            }
            app.preview_lines = lines;
            app.show_preview = true;
            app.preview_scroll = 0;
            app.status_message = Some(format!(
                "ファイラー: {} 件 (Escで閉じる)",
                filer.left.entries.len()
            ));
        }

        // --- 印刷プレビュー (:preview, :print) ---
        "preview" | "print" => {
            let content = app.buffer.to_string();
            let file_name = app
                .file_path
                .as_ref()
                .and_then(|p| p.to_str())
                .unwrap_or("Untitled");
            let settings = vivivi_modes::PrintSettings::default();
            let pages = vivivi_modes::paginate_text(&content, file_name, "2026-10-14", &settings);
            let preview_text = vivivi_modes::format_preview_text(&pages);
            app.preview_lines = preview_text.lines().map(|s| s.to_string()).collect();
            app.show_preview = true;
            app.preview_scroll = 0;
            app.status_message = Some("印刷プレビュー (Escで閉じる)".to_string());
        }

        _ => {
            app.status_message = Some(format!("未対応のexコマンド: {trimmed}"));
        }
    }
}

/// ノーマルモードのキーハンドリング
fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    // Ctrl キー組み合わせの処理
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('r') => {
                app.g_pending = false;
                app.feed_vi_char('\x12'); // Redo
                return;
            }
            KeyCode::Char('v') => {
                app.g_pending = false;
                app.feed_vi_char('\x16'); // VisualBlock
                return;
            }
            KeyCode::Char('d') => {
                app.g_pending = false;
                app.scroll_half_page(true); // 半画面スクロールダウン
                return;
            }
            KeyCode::Char('u') => {
                app.g_pending = false;
                app.scroll_half_page(false); // 半画面スクロールアップ
                return;
            }
            KeyCode::Char('f') => {
                app.g_pending = false;
                app.scroll_full_page(true); // 1画面スクロールダウン (PageDown)
                return;
            }
            KeyCode::Char('b') => {
                app.g_pending = false;
                app.scroll_full_page(false); // 1画面スクロールアップ (PageUp)
                return;
            }
            KeyCode::Char('a') => {
                app.g_pending = false;
                app.feed_vi_char('\x01'); // インクリメント
                return;
            }
            KeyCode::Char('x') => {
                app.g_pending = false;
                app.feed_vi_char('\x18'); // デクリメント
                return;
            }
            KeyCode::Char(']') => {
                app.g_pending = false;
                app.feed_vi_char('\x1d'); // タグジャンプ
                return;
            }
            KeyCode::Char('t') => {
                app.g_pending = false;
                app.feed_vi_char('\x14'); // タグ復帰
                return;
            }
            KeyCode::Char('j') => {
                app.trigger_lex_edit();
                return;
            }
            _ => {}
        }
    }

    match key.code {
        KeyCode::Esc => {
            app.g_pending = false;
            app.clear_search_highlight();
            app.feed_vi_char('\x1b');
        }
        KeyCode::Char('/') if app.g_pending => {
            // g/ で Migemo ローマ字日本語検索を開始
            app.g_pending = false;
            app.is_migemo_search = true;
            app.migemo_query.clear();
            app.status_message =
                Some("g/ (ローマ字入力で日本語検索 - Enterで確定, Escでキャンセル)".to_string());
        }
        KeyCode::Char('g') => {
            if app.g_pending {
                // gg (先頭行ジャンプ)
                app.g_pending = false;
                app.feed_vi_char('g');
            } else {
                app.g_pending = true;
                app.feed_vi_char('g');
            }
        }
        KeyCode::Char('n') if app.migemo_last_regex.is_some() => {
            app.g_pending = false;
            app.jump_next_migemo(true);
        }
        KeyCode::Char('N') if app.migemo_last_regex.is_some() => {
            app.g_pending = false;
            app.jump_next_migemo(false);
        }
        KeyCode::Char(c) => {
            app.g_pending = false;
            // コマンド文字をViEngineに入力
            app.feed_vi_char(c);

            // 'q' を押してマクロ記録対象レジスタ待ちになった場合、案内を表示（終了コマンド :q との混同を防ぐ）
            if c == 'q' && app.vi.parser().is_waiting_for_macro_record() {
                app.status_message = Some(
                    "マクロ記録: レジスタ (a-z) を指定してください (エディタ終了は :q です)"
                        .to_string(),
                );
            }
        }
        KeyCode::Left => {
            app.g_pending = false;
            app.feed_vi_char('h');
        }
        KeyCode::Right => {
            app.g_pending = false;
            app.feed_vi_char('l');
        }
        KeyCode::Up => {
            app.g_pending = false;
            app.feed_vi_char('k');
        }
        KeyCode::Down => {
            app.g_pending = false;
            app.feed_vi_char('j');
        }
        KeyCode::Home => {
            app.g_pending = false;
            app.feed_vi_char('0');
        }
        KeyCode::End => {
            app.g_pending = false;
            app.feed_vi_char('$');
        }
        KeyCode::PageDown => {
            app.g_pending = false;
            app.scroll_full_page(true);
        }
        KeyCode::PageUp => {
            app.g_pending = false;
            app.scroll_full_page(false);
        }
        _ => {
            app.g_pending = false;
        }
    }
}

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

/// AI連携exコマンド (:ai [instruction]) の実行
fn execute_ai_command(app: &mut App, cmd: &str) {
    let instruction = if cmd == "ai" { "help" } else { cmd[3..].trim() };

    match instruction {
        "help" => {
            app.status_message = Some(
                "AI連携: :ai <指示> (例: :ai doc, :ai refactor, :ai commit, :ai inline)"
                    .to_string(),
            );
        }
        "commit" => {
            app.status_message =
                Some("AI連携 [コミット支援]: git diff からメッセージ生成準備".to_string());
        }
        _ => {
            let (start, end) = app.get_target_line_range();
            app.status_message = Some(format!(
                "AI連携 [指示受付]: '{instruction}' (対象行: {}〜{} 行)",
                start + 1,
                end + 1
            ));
        }
    }
}
