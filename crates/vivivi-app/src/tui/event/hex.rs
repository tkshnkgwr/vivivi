//! バイナリモード (Hex Editor) のキーイベント処理モジュール。

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::App;

/// バイナリモード (Hex Editor) 中のキーイベントを処理します。
pub(super) fn handle_hex_mode_key(app: &mut App, key: KeyEvent, is_esc: bool) {
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
