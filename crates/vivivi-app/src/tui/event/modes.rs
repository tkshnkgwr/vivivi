use super::ex::execute_ex_command;
use super::*;

/// 挿入モードのキーハンドリング
pub(crate) fn handle_insert_mode(app: &mut App, key: KeyEvent) {
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

    // Ctrl-[ または Esc: ノーマルモードへ脱出
    let is_esc = key.code == KeyCode::Esc
        || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('['));
    if is_esc {
        app.feed_vi_char('\x1b');
        // ノーマルモード復帰時はカーソルが文字境界を超えないよう左へ1文字補正
        if app.cursor.pos.col > 0 {
            app.cursor.pos.col -= 1;
        }
        app.clamp_cursor();
        return;
    }

    match key.code {
        KeyCode::Insert => {
            // Vim互換: 挿入モード中に Insert キーを押すとリプレース（上書き）モードへトグル切り替え
            app.vi.set_mode(vivivi_vi::ViMode::Replace);
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
        KeyCode::Tab => {
            let c = '\t';
            app.vi.record_inserted_char(c);
            app.dynamic_macro.record_char(c);

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
            app.vi.record_inserted_backspace();
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
pub(crate) fn handle_replace_mode(app: &mut App, key: KeyEvent) {
    let is_esc = key.code == KeyCode::Esc
        || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('['));
    if is_esc {
        app.feed_vi_char('\x1b');
        // ノーマルモード復帰時はカーソルが文字境界を超えないよう左へ1文字補正
        if app.cursor.pos.col > 0 {
            app.cursor.pos.col -= 1;
        }
        app.clamp_cursor();
        return;
    }

    match key.code {
        KeyCode::Insert => {
            // Vim互換: リプレースモード中に Insert キーを押すと挿入モードへトグル復帰
            app.vi.set_mode(vivivi_vi::ViMode::Insert);
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
        KeyCode::Tab => {
            let c = '\t';
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
                app.buffer.delete_text(pos, 1, false);
                app.buffer.insert_char(pos, c);
            } else {
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
pub(crate) fn handle_command_line_mode(app: &mut App, key: KeyEvent) {
    let is_esc = key.code == KeyCode::Esc
        || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('['));
    if is_esc {
        app.feed_vi_char('\x1b');
        return;
    }

    // Ctrl+u (行クリア)
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('u') {
        let len = app
            .vi
            .parser()
            .command_line_buffer()
            .map(|(_, s)| s.chars().count())
            .unwrap_or(0);
        for _ in 0..len {
            app.feed_vi_char('\x08');
        }
        return;
    }

    // Ctrl+w (単語削除)
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('w') {
        let buf = app
            .vi
            .parser()
            .command_line_buffer()
            .map(|(_, s)| s.to_string())
            .unwrap_or_default();
        let trimmed_end = buf.trim_end();
        let spaces_len = buf.len() - trimmed_end.len();
        let last_word_len = trimmed_end
            .split_whitespace()
            .last()
            .map(|w| w.chars().count())
            .unwrap_or(0);
        let del_count = spaces_len + last_word_len;
        for _ in 0..del_count {
            app.feed_vi_char('\x08');
        }
        return;
    }

    // Ctrl+h / Backspace
    if (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('h'))
        || key.code == KeyCode::Backspace
    {
        app.feed_vi_char('\x08');
        return;
    }

    match key.code {
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
        KeyCode::Char(c) => {
            app.feed_vi_char(c);
        }
        _ => {}
    }
}

/// ノーマルモードのキーハンドリング
pub(crate) fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    let is_esc = key.code == KeyCode::Esc
        || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('['));
    if is_esc {
        app.g_pending = false;
        let now = std::time::Instant::now();
        let is_double_esc = if let Some(last_time) = app.last_esc_time {
            now.duration_since(last_time).as_millis() <= 500
        } else {
            false
        };

        if is_double_esc {
            app.clear_search_highlight();
            app.status_message = Some("検索ハイライトを消去しました (:nohlsearch)".to_string());
            app.last_esc_time = None;
        } else {
            app.last_esc_time = Some(now);
        }

        app.feed_vi_char('\x1b');
        return;
    }

    app.last_esc_time = None;

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
        KeyCode::Insert => {
            // Vim互換: ノーマルモードで Insert キーを押すと挿入モードへ遷移
            app.g_pending = false;
            app.feed_vi_char('i');
        }
        KeyCode::Char('/') if app.g_pending => {
            // g/ で Migemo ローマ字日本語検索を開始
            app.g_pending = false;
            app.feed_vi_char('\x1b'); // ViEngineのg待ち状態を安全に解除
            app.is_migemo_search = true;
            app.migemo_query.clear();
            app.status_message =
                Some("g/ (ローマ字入力で日本語検索 - Enterで確定, Escでキャンセル)".to_string());
        }
        KeyCode::Char('t') if app.g_pending => {
            // gt: 次のタブへ切り替え
            app.g_pending = false;
            app.feed_vi_char('\x1b');
            app.next_tab();
        }
        KeyCode::Char('T') if app.g_pending => {
            // gT: 前のタブへ切り替え
            app.g_pending = false;
            app.feed_vi_char('\x1b');
            app.prev_tab();
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
