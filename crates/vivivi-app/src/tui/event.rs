use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use vivivi_modes::KeisenDirection;
use vivivi_vi::ViMode;

use super::app::App;

/// キーイベントを処理し、App の状態やバッファを更新します。
pub fn handle_key_event(app: &mut App, key: KeyEvent) {
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

    // 2. 罫線モード有効時の矢印キー操作 (Ctrl+矢印で描画、Shift+矢印で消去)
    if app.keisen_mode.is_active {
        let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let is_shift = key.modifiers.contains(KeyModifiers::SHIFT);

        if is_ctrl || is_shift {
            let is_erase = is_shift;
            let dir = match key.code {
                KeyCode::Left => Some(KeisenDirection::Left),
                KeyCode::Right => Some(KeisenDirection::Right),
                KeyCode::Up => Some(KeisenDirection::Up),
                KeyCode::Down => Some(KeisenDirection::Down),
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
        }
        KeyCode::Down => {
            let total_lines = app.buffer.line_count();
            app.cursor.pos.line = (app.cursor.pos.line + 1).min(total_lines.saturating_sub(1));
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
            // コマンド実行前にViEngineをノーマルモードに戻す
            app.feed_vi_char('\n');

            // exコマンドの解釈
            execute_ex_command(app, &cmd_text);
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
        "q" => {
            if app.buffer.is_modified() {
                app.status_message =
                    Some("未保存の変更があります (! を追加して :q! で終了)".to_string());
            } else {
                app.should_quit = true;
            }
        }
        "q!" => {
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

        _ => {
            app.status_message = Some(format!("未対応のexコマンド: {trimmed}"));
        }
    }
}

/// ノーマルモードのキーハンドリング
fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    // Ctrl-J: 字句編集予測
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('j') {
        app.trigger_lex_edit();
        return;
    }

    match key.code {
        KeyCode::Esc => {
            app.g_pending = false;
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
        _ => {
            app.g_pending = false;
        }
    }
}

/// 記録されたキーボードマクロを再生・実行します。
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
