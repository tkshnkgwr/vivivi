use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use vivivi_modes::KeisenDirection;
use vivivi_vi::ViMode;

use super::app::App;

/// キーイベントを処理し、App の状態やバッファを更新します。
pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    // 状態メッセージを一旦クリア
    app.status_message = None;

    // 1. グローバルショートカット
    // Ctrl + C: 強制終了
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.should_quit = true;
        return;
    }

    // Shift + F5 または F5: 罫線モードのトグル
    if key.code == KeyCode::F(5)
        || (key.modifiers.contains(KeyModifiers::SHIFT) && key.code == KeyCode::F(5))
    {
        app.keisen_mode.toggle();
        let state_msg = if app.keisen_mode.is_active {
            "罫線モード: ON (Ctrl+矢印で描画、Shift+矢印で消去)"
        } else {
            "罫線モード: OFF"
        };
        app.status_message = Some(state_msg.to_string());
        return;
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
    match key.code {
        KeyCode::Esc => {
            app.feed_vi_char('\x1b');
            // ノーマルモード復帰時はカーソルが文字境界を超えないよう左へ1文字補正
            if app.cursor.pos.col > 0 {
                app.cursor.pos.col -= 1;
            }
        }
        KeyCode::Char(c) => {
            let cur = app.cursor_pos();
            let pos = app.buffer.cursor_to_position(cur);
            app.buffer.insert_char(pos, c);
            app.cursor.pos.col += 1;
        }
        KeyCode::Enter => {
            let cur = app.cursor_pos();
            let pos = app.buffer.cursor_to_position(cur);
            app.buffer.insert_char(pos, '\n');
            app.cursor.pos.line += 1;
            app.cursor.pos.col = 0;
        }
        KeyCode::Backspace => {
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
        "align left" | "left" => {
            app.align_keisen_box(vivivi_modes::BoxAlignment::Left);
        }
        "align center" | "center" => {
            app.align_keisen_box(vivivi_modes::BoxAlignment::Center);
        }
        "align right" | "right" => {
            app.align_keisen_box(vivivi_modes::BoxAlignment::Right);
        }
        _ => {
            app.status_message = Some(format!("未対応のexコマンド: {trimmed}"));
        }
    }
}

/// ノーマルモードのキーハンドリング
fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.feed_vi_char('\x1b');
        }
        KeyCode::Char(c) => {
            // コマンド文字をViEngineに入力
            app.feed_vi_char(c);
        }
        KeyCode::Left => {
            app.feed_vi_char('h');
        }
        KeyCode::Right => {
            app.feed_vi_char('l');
        }
        KeyCode::Up => {
            app.feed_vi_char('k');
        }
        KeyCode::Down => {
            app.feed_vi_char('j');
        }
        _ => {}
    }
}
