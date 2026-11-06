//! Windows ライク操作体系（Shift+矢印選択、Ctrl+C/V/X/Z/Y/A/S）ディスパッチャ。
//!
//! ViVi 伝統の「モード管理を維持しつつ Windows 標準のキーバインドを直感的に併用できる」操作系を提供します。

use crate::tui::App;
use crate::{AppKeyEvent, KeyCode, KeyModifiers};
use vivivi_vi::ViMode;

/// Windows ライクなキー操作を判定・ディスパッチします。
/// 処理された場合は `true` を返します。
pub fn handle_windows_shortcuts(app: &mut App, key: &AppKeyEvent) -> bool {
    let has_shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let has_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    // 1. Shift + 矢印キー（Left/Right/Up/Down）による選択開始 & 拡張
    if has_shift && !has_ctrl {
        match key.code {
            KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => {
                let mode = app.mode();
                if mode != ViMode::Visual
                    && mode != ViMode::VisualLine
                    && mode != ViMode::VisualBlock
                {
                    // ビジュアルモード（文字選択）を開始
                    let anchor = app.cursor.pos;
                    app.vi.set_mode(ViMode::Visual);
                    app.cursor.anchor = Some(anchor);
                }
                // 矢印方向へカーソル移動
                match key.code {
                    KeyCode::Left => {
                        app.cursor.pos.col = app.cursor.pos.col.saturating_sub(1);
                    }
                    KeyCode::Right => {
                        let line_len = app
                            .buffer
                            .get_line(app.cursor.pos.line)
                            .map(|l| l.chars().count())
                            .unwrap_or(0);
                        app.cursor.pos.col = (app.cursor.pos.col + 1).min(line_len);
                    }
                    KeyCode::Up => {
                        app.cursor.pos.line = app.cursor.pos.line.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        app.cursor.pos.line = (app.cursor.pos.line + 1)
                            .min(app.buffer.line_count().saturating_sub(1));
                    }
                    _ => {}
                }
                return true;
            }
            _ => {}
        }
    }

    // 2. Ctrl ショートカット群
    if has_ctrl && !has_shift {
        match key.code {
            // Ctrl+A: 全選択
            KeyCode::Char('a') | KeyCode::Char('A') if app.mode() != ViMode::CommandLine => {
                let total_lines = app.buffer.line_count();
                if total_lines > 0 {
                    app.vi.set_mode(ViMode::VisualLine);
                    app.cursor.anchor = Some(vivivi_core::CursorPosition::new(0, 0));
                    app.cursor.pos =
                        vivivi_core::CursorPosition::new(total_lines.saturating_sub(1), 0);
                    app.status_message = Some("-- 全選択 --".to_string());
                    return true;
                }
            }
            // Ctrl+C: 選択範囲のコピー（ヤンク）
            KeyCode::Char('c') | KeyCode::Char('C') => {
                let mode = app.mode();
                if mode == ViMode::Visual
                    || mode == ViMode::VisualLine
                    || mode == ViMode::VisualBlock
                {
                    if let Some(text) = app.get_selected_text() {
                        app.set_clipboard_text(&text);
                        app.status_message =
                            Some(format!("{} 文字をコピーしました", text.chars().count()));
                    }
                    app.cursor.anchor = None;
                    app.vi.set_mode(ViMode::Normal);
                    return true;
                }
            }
            // Ctrl+X: 選択範囲の切り取り（カット）
            KeyCode::Char('x') | KeyCode::Char('X') => {
                let mode = app.mode();
                if mode == ViMode::Visual
                    || mode == ViMode::VisualLine
                    || mode == ViMode::VisualBlock
                {
                    if let Some(text) = app.get_selected_text() {
                        app.set_clipboard_text(&text);
                        app.delete_selected_text();
                        app.status_message =
                            Some(format!("{} 文字を切り取りました", text.chars().count()));
                    }
                    return true;
                }
            }
            // Ctrl+V: クリップボードからの貼り付け（ペースト）
            KeyCode::Char('v') | KeyCode::Char('V') => {
                let is_windows_preset = app.config.keymap.preset.eq_ignore_ascii_case("windows");
                let is_insert = app.mode() == ViMode::Insert;
                if (is_insert || is_windows_preset)
                    && let Some(clip_text) = app.get_clipboard_text()
                {
                    app.insert_text_at_cursor(&clip_text);
                    app.status_message = Some(format!(
                        "{} 文字を貼り付けました",
                        clip_text.chars().count()
                    ));
                    return true;
                }
            }
            // Ctrl+Z: アンドゥ（取り消し）
            KeyCode::Char('z') | KeyCode::Char('Z') if app.mode() != ViMode::CommandLine => {
                app.buffer.undo();
                app.status_message = Some("取り消し (Undo)".to_string());
                return true;
            }
            // Ctrl+Y: リドゥ（やり直し）
            KeyCode::Char('y') | KeyCode::Char('Y') if app.mode() != ViMode::CommandLine => {
                app.buffer.redo();
                app.status_message = Some("やり直し (Redo)".to_string());
                return true;
            }
            // Ctrl+S: 上書き保存 (:w)
            KeyCode::Char('s') | KeyCode::Char('S') if app.mode() != ViMode::CommandLine => {
                match app.save_file() {
                    Ok(()) => {
                        let fname = app
                            .file_path
                            .as_ref()
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|| "[無題]".to_string());
                        app.status_message = Some(format!("\"{fname}\" を保存しました"));
                    }
                    Err(e) => {
                        app.status_message = Some(format!("E212: 保存失敗: {e}"));
                    }
                }
                return true;
            }
            // Ctrl+,: 設定画面の起動 (:preferences)
            KeyCode::Char(',') if app.mode() != ViMode::CommandLine => {
                app.open_preferences_dialog();
                app.status_message = Some("設定画面を開きます".to_string());
                return true;
            }
            _ => {}
        }
    }

    false
}
