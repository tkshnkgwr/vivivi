//! 外部プロセス実行・選択テキストパイプ連携モジュール。
//!
//! ビジュアルモードで選択されたテキストや指定コマンドを OS シェル経由で実行し、
//! 出力を取得・バッファ反映する ViVi 固有機能を提供します。

use std::io::Write;
use std::process::{Command, Stdio};

use super::App;
use vivivi_vi::ViMode;

impl App {
    /// ビジュアルモードで選択されているテキストを文字列として取得します。
    pub fn get_selected_text(&self) -> Option<String> {
        let (anchor, cursor) = match self.mode() {
            ViMode::Visual | ViMode::VisualLine | ViMode::VisualBlock => {
                let anchor = self.cursor.anchor?;
                (anchor, self.cursor.pos)
            }
            _ => return None,
        };

        let (start, end) = if (anchor.line, anchor.col) <= (cursor.line, cursor.col) {
            (anchor, cursor)
        } else {
            (cursor, anchor)
        };

        let mut text = String::new();
        for l in start.line..=end.line {
            let line_str = self.buffer.get_line(l).unwrap_or_default();
            if self.mode() == ViMode::VisualLine {
                text.push_str(&line_str);
                text.push('\n');
            } else if start.line == end.line {
                let start_idx = start.col.min(line_str.chars().count());
                let end_idx = (end.col + 1).min(line_str.chars().count());
                let sub: String = line_str
                    .chars()
                    .skip(start_idx)
                    .take(end_idx.saturating_sub(start_idx))
                    .collect();
                text.push_str(&sub);
            } else if l == start.line {
                let start_idx = start.col.min(line_str.chars().count());
                let sub: String = line_str.chars().skip(start_idx).collect();
                text.push_str(&sub);
                text.push('\n');
            } else if l == end.line {
                let end_idx = (end.col + 1).min(line_str.chars().count());
                let sub: String = line_str.chars().take(end_idx).collect();
                text.push_str(&sub);
            } else {
                text.push_str(&line_str);
                text.push('\n');
            }
        }
        Some(text)
    }

    /// 選択されているテキストをバッファから削除します。
    pub fn delete_selected_text(&mut self) {
        let (anchor, cursor) = match self.mode() {
            ViMode::Visual | ViMode::VisualLine | ViMode::VisualBlock => {
                if let Some(anchor) = self.cursor.anchor {
                    (anchor, self.cursor.pos)
                } else {
                    return;
                }
            }
            _ => return,
        };

        let (start, end) = if (anchor.line, anchor.col) <= (cursor.line, cursor.col) {
            (anchor, cursor)
        } else {
            (cursor, anchor)
        };

        if self.mode() == ViMode::VisualLine {
            let start_offset = self
                .buffer
                .cursor_to_position(vivivi_core::CursorPosition::new(start.line, 0));
            let end_offset = if end.line + 1 < self.buffer.line_count() {
                self.buffer
                    .cursor_to_position(vivivi_core::CursorPosition::new(end.line + 1, 0))
            } else {
                self.buffer.len()
            };
            self.buffer.delete_range(start_offset, end_offset);
            self.cursor.pos = vivivi_core::CursorPosition::new(
                start.line.min(self.buffer.line_count().saturating_sub(1)),
                0,
            );
        } else {
            let start_offset = self.buffer.cursor_to_position(start);
            let end_offset = (self.buffer.cursor_to_position(end) + 1).min(self.buffer.len());
            self.buffer.delete_range(start_offset, end_offset);
            self.cursor.pos = start;
        }
        self.cursor.anchor = None;
        self.vi.set_mode(ViMode::Normal);
    }

    /// 現在のカーソル位置にテキストを直接挿入します。
    pub fn insert_text_at_cursor(&mut self, text: &str) {
        let offset = self.buffer.cursor_to_position(self.cursor.pos);
        self.buffer.insert_text(offset, text);
        let new_offset = offset + text.chars().count();
        self.cursor.pos = self.buffer.position_to_cursor(new_offset);
    }

    /// クリップボードへテキストを設定します。
    pub fn set_clipboard_text(&mut self, text: &str) {
        self.vi.registers_mut().set(
            Some('+'),
            vivivi_vi::RegisterContent::CharacterWise(text.to_string()),
        );
    }

    /// クリップボードからテキストを取得します。
    pub fn get_clipboard_text(&mut self) -> Option<String> {
        self.vi
            .registers_mut()
            .get(Some('+'))
            .map(|c| c.as_str().to_string())
    }

    /// 選択テキスト自体をシェルコマンドとして実行し、標準出力を返します。
    pub fn execute_selected_text(&mut self) -> Result<String, String> {
        let text = self
            .get_selected_text()
            .ok_or_else(|| "テキストが選択されていません".to_string())?;
        self.execute_shell_command(&text)
    }

    /// 選択テキストを標準入力（stdin）経由で外部コマンドに渡し、その出力を返します。
    pub fn pipe_selected_text(&mut self, cmd: &str) -> Result<String, String> {
        let text = self.get_selected_text().unwrap_or_default();
        self.pipe_command_with_input(cmd, &text)
    }

    /// OS の標準シェルでコマンドを実行します。
    pub fn execute_shell_command(&self, cmd_str: &str) -> Result<String, String> {
        let trimmed = cmd_str.trim();
        if trimmed.is_empty() {
            return Err("コマンドが空です".to_string());
        }
        #[cfg(target_os = "windows")]
        let output = Command::new("cmd")
            .args(["/C", trimmed])
            .output()
            .map_err(|e| format!("コマンド実行失敗: {e}"))?;

        #[cfg(not(target_os = "windows"))]
        let output = Command::new("sh")
            .args(["-c", trimmed])
            .output()
            .map_err(|e| format!("コマンド実行失敗: {e}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if output.status.success() {
            if stdout.is_empty() && !stderr.is_empty() {
                Ok(stderr)
            } else {
                Ok(stdout)
            }
        } else {
            Err(if !stderr.is_empty() {
                stderr
            } else {
                format!("終了コード: {:?}", output.status.code())
            })
        }
    }

    /// 外部コマンドへテキストを入力として流し込み（パイプ）、標準出力を取得します。
    pub fn pipe_command_with_input(
        &self,
        cmd_str: &str,
        input_text: &str,
    ) -> Result<String, String> {
        let trimmed = cmd_str.trim();
        if trimmed.is_empty() {
            return Err("コマンドが空です".to_string());
        }
        #[cfg(target_os = "windows")]
        let mut child = Command::new("cmd")
            .args(["/C", trimmed])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("プロセス起動失敗: {e}"))?;

        #[cfg(not(target_os = "windows"))]
        let mut child = Command::new("sh")
            .args(["-c", trimmed])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("プロセス起動失敗: {e}"))?;

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(input_text.as_bytes());
        }

        let output = child
            .wait_with_output()
            .map_err(|e| format!("コマンド実行待機失敗: {e}"))?;
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if output.status.success() {
            Ok(stdout)
        } else {
            Err(if !stderr.is_empty() {
                stderr
            } else {
                format!("終了コード: {:?}", output.status.code())
            })
        }
    }
}
