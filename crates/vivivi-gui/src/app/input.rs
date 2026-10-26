//! `egui` 入力イベント変換・IME Preedit 保護・削除パーティクル生成ディスパッチモジュール。

use eframe::egui;
use egui::{Event, ImeEvent, Key, Modifiers, Pos2};
use unicode_width::UnicodeWidthChar;
use vivivi_app::{AppKeyEvent, KeyCode, KeyModifiers};
use vivivi_vi::ViMode;

use super::ViviviGuiApp;
use crate::particles::extract_deleted_line_segment;
use crate::theme::colors;

impl ViviviGuiApp {
    /// キーイベントを共通 `App` コアへ転送し、文字削除が発生した場合は
    /// 該当グリフのスクリーン座標から落下アニメーション用パーティクルを生成します（ステップ 4）。
    pub(crate) fn dispatch_key_with_particles(&mut self, key: AppKeyEvent) {
        if !self.particles.enabled {
            self.core.handle_key(key);
            return;
        }

        let prev_len = self.core.buffer.len();
        let prev_line_count = self.core.buffer.line_count();
        let prev_cur_line = self.core.cursor.pos.line;
        let prev_line_str = self.core.buffer.get_line(prev_cur_line).unwrap_or_default();

        self.core.handle_key(key);

        let new_len = self.core.buffer.len();
        if new_len >= prev_len {
            return;
        }

        let vis_row = prev_cur_line.saturating_sub(self.core.scroll.0);
        let row_y = self.last_active_text_origin.y + (vis_row as f32) * self.line_height;
        let scroll_col = self.core.scroll.1;
        let tab_width = self.core.tab_width.max(1);

        if self.core.buffer.line_count() == prev_line_count {
            // 同一行内の文字削除 (x, X, dw, D, Backspace, Delete 等)
            let new_line_str = self.core.buffer.get_line(prev_cur_line).unwrap_or_default();
            if let Some((start_char_idx, deleted_chars)) =
                extract_deleted_line_segment(&prev_line_str, &new_line_str)
            {
                let mut vis_col: usize = 0;
                for (idx, ch) in prev_line_str.chars().enumerate() {
                    let ch_w = if ch == '\t' {
                        tab_width - (vis_col % tab_width)
                    } else {
                        UnicodeWidthChar::width(ch).unwrap_or(1).max(1)
                    };
                    if idx >= start_char_idx && idx < start_char_idx + deleted_chars.len() {
                        let char_x = self.last_active_text_origin.x
                            + (vis_col.saturating_sub(scroll_col) as f32) * self.last_char_width;
                        self.particles
                            .spawn_char(ch, Pos2::new(char_x, row_y), colors::ACCENT_RED);
                    }
                    vis_col += ch_w;
                }
            }
        } else {
            // 行削除 (dd, dj 等) または改行結合時の落下パーティクル生成
            let mut vis_col: usize = 0;
            for ch in prev_line_str.chars().take(64) {
                let ch_w = if ch == '\t' {
                    tab_width - (vis_col % tab_width)
                } else {
                    UnicodeWidthChar::width(ch).unwrap_or(1).max(1)
                };
                let char_x = self.last_active_text_origin.x
                    + (vis_col.saturating_sub(scroll_col) as f32) * self.last_char_width;
                self.particles
                    .spawn_char(ch, Pos2::new(char_x, row_y), colors::ACCENT_RED);
                vis_col += ch_w;
            }
        }
    }

    /// `egui` の入力イベント列（`Event::Key`, `Event::Text`, `Event::Ime`, `Event::Paste`）を解析し、
    /// 共通 [`vivivi_app::App`] コア層の入力ハンドラへ流し込みます。
    ///
    /// 日本語 IME の未確定文字列（`ImeEvent::Preedit`）が存在する間は、OS 側の変換操作と競合する
    /// `Backspace` / `Enter` / `Escape` / 矢印キーを自動的にブロックして背後バッファを保護します。
    ///
    /// # Examples
    /// ```
    /// use egui::{Event, Modifiers};
    /// use vivivi_app::App;
    /// use vivivi_core::{CharEncoding, NewlineType};
    /// use vivivi_gui::ViviviGuiApp;
    /// use vivivi_vi::ViMode;
    ///
    /// let core = App::new("abc", None, CharEncoding::Utf8, NewlineType::Lf);
    /// let mut gui = ViviviGuiApp::from_core(core);
    /// gui.process_egui_events(&[Event::Text("i".to_string())], Modifiers::NONE);
    /// assert_eq!(gui.core.mode(), ViMode::Insert);
    /// ```
    pub fn process_egui_events(&mut self, events: &[Event], current_modifiers: Modifiers) {
        for event in events {
            match event {
                // 1. 日本語 IME イベントの処理
                Event::Ime(ime_event) => match ime_event {
                    ImeEvent::Enabled => {
                        self.ime_enabled = true;
                    }
                    ImeEvent::Preedit(text) => {
                        if text != "\n" && text != "\r" {
                            self.ime_enabled = true;
                            self.ime_preedit = text.clone();
                        }
                    }
                    ImeEvent::Commit(text) => {
                        self.ime_preedit.clear();
                        self.ime_enabled = false;
                        for ch in text.chars() {
                            if ch != '\r' && ch != '\n' {
                                self.dispatch_key_with_particles(AppKeyEvent::char(ch));
                            }
                        }
                    }
                    ImeEvent::Disabled => {
                        self.ime_preedit.clear();
                        self.ime_enabled = false;
                    }
                },

                // 2. 確定テキスト入力イベント（通常キー打鍵）
                Event::Text(text) => {
                    // IME 変換中（未確定文字列が存在する間）または Ctrl/Alt 押下時は Text イベントを無視
                    let is_ctrl_or_alt = current_modifiers.ctrl
                        || current_modifiers.alt
                        || current_modifiers.command;
                    if !self.ime_preedit.is_empty() || is_ctrl_or_alt {
                        continue;
                    }

                    let shift_mod = if current_modifiers.shift {
                        KeyModifiers::SHIFT
                    } else {
                        KeyModifiers::NONE
                    };

                    for ch in text.chars() {
                        if ch != '\r' && ch != '\n' && ch != '\t' && !ch.is_control() {
                            let mods = if ch.is_ascii_alphabetic() {
                                shift_mod
                            } else {
                                KeyModifiers::NONE
                            };
                            self.dispatch_key_with_particles(AppKeyEvent::new(
                                KeyCode::Char(ch),
                                mods,
                            ));
                        }
                    }
                }

                // 3. ペースト（Ctrl+V 等）イベント
                Event::Paste(text) => {
                    if self.core.mode() == ViMode::Normal && !self.core.is_migemo_search {
                        // Vi ノーマルモードでの Ctrl+V は矩形ビジュアル選択（VisualBlock）を起動
                        self.dispatch_key_with_particles(AppKeyEvent::new(
                            KeyCode::Char('v'),
                            KeyModifiers::CONTROL,
                        ));
                    } else {
                        for ch in text.chars() {
                            if ch == '\n' {
                                self.dispatch_key_with_particles(AppKeyEvent::new(
                                    KeyCode::Enter,
                                    KeyModifiers::NONE,
                                ));
                            } else if ch != '\r' {
                                self.dispatch_key_with_particles(AppKeyEvent::char(ch));
                            }
                        }
                    }
                }

                // 4. 物理・論理キー押下イベント（特殊キーおよび Ctrl/Alt ショートカット）
                Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    // IME 未確定文字列の変換中は、OS 側の IME 操作と競合する編集キーを無視
                    if !self.ime_preedit.is_empty()
                        && matches!(
                            key,
                            Key::Backspace
                                | Key::Enter
                                | Key::Escape
                                | Key::Space
                                | Key::ArrowLeft
                                | Key::ArrowRight
                                | Key::ArrowUp
                                | Key::ArrowDown
                        )
                    {
                        continue;
                    }

                    if let Some(app_key) = egui_key_to_app_key(*key, *modifiers) {
                        self.dispatch_key_with_particles(app_key);
                    }
                }

                // 5. マウスホイールによるスクロール
                Event::MouseWheel { delta, .. } => {
                    if delta.y < -0.1 {
                        let lines = ((-delta.y).ceil() as usize).clamp(1, 5);
                        let max_line = self.core.buffer.line_count().saturating_sub(1);
                        self.core.scroll.0 = (self.core.scroll.0 + lines).min(max_line);
                        if self.core.cursor.pos.line < self.core.scroll.0 {
                            self.core.cursor.pos.line = self.core.scroll.0;
                            self.core.clamp_cursor();
                        }
                    } else if delta.y > 0.1 {
                        let lines = (delta.y.ceil() as usize).clamp(1, 5);
                        self.core.scroll.0 = self.core.scroll.0.saturating_sub(lines);
                    }
                }

                _ => {}
            }
        }

        // ノーマルモードへ復帰した際は IME の未確定状態を確実にクリア
        if self.core.mode() == ViMode::Normal && !self.core.is_migemo_search {
            self.ime_preedit.clear();
            self.ime_enabled = false;
        }
    }
}

/// [`egui::Key`] と [`egui::Modifiers`] を `vivivi-app` 共通の [`AppKeyEvent`] へ変換します。
///
/// 通常の印字可能文字（Ctrl/Alt なし）は `egui::Event::Text` 側で正確に処理されるため、
/// ここでは特殊制御キー（`Esc`, `Enter`, `Backspace`, `Tab`, 矢印キー, ファンクションキー等）および
/// `Ctrl` / `Alt` 修飾付きのショートカットキーのみを変換します。
///
/// # Examples
/// ```
/// use egui::{Key, Modifiers};
/// use vivivi_app::{KeyCode, KeyModifiers};
/// use vivivi_gui::egui_key_to_app_key;
///
/// // Escape などの特殊キーは常に変換される
/// let esc = egui_key_to_app_key(Key::Escape, Modifiers::NONE).unwrap();
/// assert_eq!(esc.code, KeyCode::Esc);
///
/// // 修飾なしの文字キーは Event::Text 側で処理するため None を返す
/// assert!(egui_key_to_app_key(Key::A, Modifiers::NONE).is_none());
///
/// // Ctrl+W などのショートカットは変換される
/// let ctrl_w = egui_key_to_app_key(Key::W, Modifiers::CTRL).unwrap();
/// assert_eq!(ctrl_w.code, KeyCode::Char('w'));
/// assert!(ctrl_w.modifiers.contains(KeyModifiers::CONTROL));
/// ```
pub fn egui_key_to_app_key(key: Key, modifiers: Modifiers) -> Option<AppKeyEvent> {
    let mut mods = KeyModifiers::NONE;
    if modifiers.shift {
        mods |= KeyModifiers::SHIFT;
    }
    if modifiers.ctrl || modifiers.command || modifiers.mac_cmd {
        mods |= KeyModifiers::CONTROL;
    }
    if modifiers.alt {
        mods |= KeyModifiers::ALT;
    }

    // 1. 特殊キー・ナビゲーションキー・ファンクションキーは常に Event::Key から変換
    let special_code = match key {
        Key::Escape => Some(KeyCode::Esc),
        Key::Enter => Some(KeyCode::Enter),
        Key::Backspace => Some(KeyCode::Backspace),
        Key::Delete => Some(KeyCode::Delete),
        Key::Tab => Some(KeyCode::Tab),
        Key::Insert => Some(KeyCode::Insert),
        Key::Home => Some(KeyCode::Home),
        Key::End => Some(KeyCode::End),
        Key::PageUp => Some(KeyCode::PageUp),
        Key::PageDown => Some(KeyCode::PageDown),
        Key::ArrowLeft => Some(KeyCode::Left),
        Key::ArrowRight => Some(KeyCode::Right),
        Key::ArrowUp => Some(KeyCode::Up),
        Key::ArrowDown => Some(KeyCode::Down),
        Key::F1 => Some(KeyCode::F(1)),
        Key::F2 => Some(KeyCode::F(2)),
        Key::F3 => Some(KeyCode::F(3)),
        Key::F4 => Some(KeyCode::F(4)),
        Key::F5 => Some(KeyCode::F(5)),
        Key::F6 => Some(KeyCode::F(6)),
        Key::F7 => Some(KeyCode::F(7)),
        Key::F8 => Some(KeyCode::F(8)),
        Key::F9 => Some(KeyCode::F(9)),
        Key::F10 => Some(KeyCode::F(10)),
        Key::F11 => Some(KeyCode::F(11)),
        Key::F12 => Some(KeyCode::F(12)),
        _ => None,
    };

    if let Some(code) = special_code {
        return Some(AppKeyEvent::new(code, mods));
    }

    // 2. Ctrl または Alt が押されている場合のみ、文字キーをショートカットとして変換
    if !mods.contains(KeyModifiers::CONTROL) && !mods.contains(KeyModifiers::ALT) {
        return None;
    }

    let ch = match key {
        Key::A => 'a',
        Key::B => 'b',
        Key::C => 'c',
        Key::D => 'd',
        Key::E => 'e',
        Key::F => 'f',
        Key::G => 'g',
        Key::H => 'h',
        Key::I => 'i',
        Key::J => 'j',
        Key::K => 'k',
        Key::L => 'l',
        Key::M => 'm',
        Key::N => 'n',
        Key::O => 'o',
        Key::P => 'p',
        Key::Q => 'q',
        Key::R => 'r',
        Key::S => 's',
        Key::T => 't',
        Key::U => 'u',
        Key::V => 'v',
        Key::W => 'w',
        Key::X => 'x',
        Key::Y => 'y',
        Key::Z => 'z',
        Key::Semicolon => ';',
        Key::Comma => ',',
        Key::Period => '.',
        Key::Slash => '/',
        Key::OpenBracket => '[',
        Key::CloseBracket => ']',
        Key::Minus => '-',
        Key::Equals => '=',
        _ => return None,
    };

    let final_ch = if mods.contains(KeyModifiers::SHIFT) {
        ch.to_ascii_uppercase()
    } else {
        ch
    };

    Some(AppKeyEvent::new(KeyCode::Char(final_ch), mods))
}
