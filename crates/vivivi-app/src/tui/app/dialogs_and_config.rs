//! ファイラー・PDF出力・設定適用・外部コマンド実行ダイアログ連携モジュール。

use std::path::PathBuf;

use super::{App, current_date_str};
use vivivi_core::{CharEncoding, NewlineType};

impl App {
    /// 2ペインファイラーを開きます (`:filer`)。
    pub fn open_filer(&mut self, path: Option<&std::path::Path>) {
        let target_dir = if let Some(p) = path {
            if p.is_dir() {
                p.to_path_buf()
            } else {
                p.parent()
                    .unwrap_or(std::path::Path::new("."))
                    .to_path_buf()
            }
        } else if let Some(ref fp) = self.file_path {
            fp.parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf()
        } else {
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        };

        let filer = vivivi_modes::TwoPaneFiler::new(&target_dir);
        self.filer = Some(filer);
        self.show_filer = true;
        self.status_message = Some(
            "2ペイン ファイラー起動 (Tab: 左右切替, Enter: 開く/入る, q/Esc: 閉じる)".to_string(),
        );
    }

    /// 2ペインファイラーを閉じます。
    pub fn close_filer(&mut self) {
        self.show_filer = false;
        self.filer = None;
    }

    /// 現在のバッファ内容をA4サイズのPDFファイルとして出力します。
    ///
    /// # 引数
    /// - `target_path`: 保存先ファイルパス（指定なしの場合はバッファファイル名`.pdf`、または `Untitled.pdf`）
    ///
    /// # 戻り値
    /// 成功した場合は保存先 `PathBuf`、失敗時はエラーメッセージ。
    pub fn export_pdf(&mut self, target_path: Option<&str>) -> Result<PathBuf, String> {
        let content = self.buffer.to_string();
        let doc_name = self
            .file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled");

        let out_path = if let Some(p) = target_path.filter(|s| !s.trim().is_empty()) {
            PathBuf::from(p.trim())
        } else if let Some(ref fp) = self.file_path {
            fp.with_extension("pdf")
        } else {
            PathBuf::from("Untitled.pdf")
        };

        let date_str = current_date_str();
        let settings = vivivi_modes::PrintSettings::default();
        let pages = vivivi_modes::paginate_text(&content, doc_name, &date_str, &settings);
        vivivi_modes::export_to_pdf(&pages, &out_path)?;

        let display_path = out_path.display().to_string();
        self.status_message = Some(format!("PDFを出力しました: {display_path}"));
        Ok(out_path)
    }

    /// 指定されたクリップボードプロバイダを注入した [`App`] を初期化します。
    pub fn with_clipboard_provider(
        initial_text: &str,
        file_path: Option<PathBuf>,
        encoding: CharEncoding,
        newline: NewlineType,
        provider: std::sync::Arc<std::sync::Mutex<dyn vivivi_vi::ClipboardProvider>>,
    ) -> Self {
        let mut app = Self::new(initial_text, file_path, encoding, newline);
        app.vi.set_clipboard_provider(provider);
        app
    }

    /// テスト用のインメモリ [`crate::tui::clipboard::MockClipboard`] を注入した [`App`] を初期化します。
    pub fn with_mock_clipboard(
        initial_text: &str,
        file_path: Option<PathBuf>,
        encoding: CharEncoding,
        newline: NewlineType,
    ) -> Self {
        Self::with_clipboard_provider(
            initial_text,
            file_path,
            encoding,
            newline,
            crate::tui::clipboard::MockClipboard::new_shared(),
        )
    }

    /// TUI 設定ダイアログを開き、現在の設定をバックアップに退避します。
    pub fn open_preferences_dialog(&mut self) {
        self.sync_runtime_to_config();
        self.tui_settings_backup = Some(self.config.clone());
        self.show_preferences_dialog = true;
    }

    /// TUI 設定ダイアログをキャンセルして閉じ、開く直前の設定に安全にロールバックします。
    pub fn cancel_preferences_dialog(&mut self) {
        if let Some(backup) = self.tui_settings_backup.take() {
            self.apply_config(&backup);
        }
        self.show_preferences_dialog = false;
        self.status_message = Some("設定変更を破棄して設定画面を閉じました".to_string());
    }

    /// 外部設定構造体 [`crate::config::AppConfig`] を現在の実行時状態に適用します。
    pub fn apply_config(&mut self, cfg: &crate::config::AppConfig) {
        self.show_line_numbers = cfg.appearance.line_number;
        self.relative_number = cfg.appearance.relative_number;
        self.tab_width = cfg.editing.tab_width;
        self.wrap = cfg.appearance.wrap;
        self.change_tracker.enabled = cfg.appearance.gutter_markers;
        self.list_chars_config.enabled = cfg.invisibles.enabled;
        self.list_chars_config.zenkaku_space = cfg.invisibles.zenkaku_space.clone().into();
        self.list_chars_config.space = cfg.invisibles.space;
        self.list_chars_config.eol = cfg.invisibles.eol;
        let cb_opt = vivivi_vi::ClipboardOption::parse(&cfg.general.clipboard);
        self.vi.set_clipboard_option(cb_opt);
        self.vi.ignorecase = cfg.editing.ignorecase;
        self.vi.smartcase = cfg.editing.smartcase;
        self.config = cfg.clone();
    }

    /// 現在のエディタ実行時状態（行番号、折り返し、不可視文字など）を設定構造体に同期させます。
    pub fn sync_runtime_to_config(&mut self) {
        self.config.appearance.line_number = self.show_line_numbers;
        self.config.appearance.relative_number = self.relative_number;
        self.config.editing.tab_width = self.tab_width;
        self.config.editing.ignorecase = self.vi.ignorecase;
        self.config.editing.smartcase = self.vi.smartcase;
        self.config.appearance.wrap = self.wrap;
        self.config.appearance.gutter_markers = self.change_tracker.enabled;
        self.config.invisibles.enabled = self.list_chars_config.enabled;
        self.config.invisibles.zenkaku_space = self.list_chars_config.zenkaku_space.to_string();
        self.config.invisibles.space = self.list_chars_config.space;
        self.config.invisibles.eol = self.list_chars_config.eol;
        self.config.general.clipboard = match self.vi.clipboard_option() {
            vivivi_vi::ClipboardOption::None => "none".to_string(),
            vivivi_vi::ClipboardOption::Unnamed => "unnamed".to_string(),
            vivivi_vi::ClipboardOption::UnnamedPlus => "unnamedplus".to_string(),
        };
    }

    /// 外部コマンド実行ダイアログ (cmdExecDlg) を開きます。
    pub fn open_cmd_exec_dialog(&mut self) {
        self.show_cmd_exec_dialog = true;
        self.cmd_exec_output_preview = None;
    }

    /// 外部コマンド実行ダイアログ (cmdExecDlg) を閉じます。
    pub fn close_cmd_exec_dialog(&mut self) {
        self.show_cmd_exec_dialog = false;
        self.cmd_exec_output_preview = None;
    }

    /// 外部コマンド実行ダイアログの入力コマンドと選択モードを実行します。
    pub fn execute_cmd_exec_dialog(&mut self) {
        let cmd = self.cmd_exec_input.trim().to_string();
        if cmd.is_empty() {
            return;
        }
        match self.cmd_exec_mode {
            0 => {
                // 実行のみ (結果プレビュー)
                match self.execute_shell_command(&cmd) {
                    Ok(out) => {
                        self.cmd_exec_output_preview = Some(out);
                        self.status_message = Some(format!("!{cmd} ➔ 実行完了"));
                    }
                    Err(e) => {
                        self.cmd_exec_output_preview = Some(format!("エラー: {e}"));
                        self.status_message = Some(format!("コマンド実行エラー: {e}"));
                    }
                }
            }
            1 => {
                // バッファ挿入 (:r !)
                match self.read_command_into_buffer(&cmd, None) {
                    Ok(count) => {
                        self.status_message = Some(format!(":r !{cmd} ➔ {count} 行を挿入しました"));
                        self.close_cmd_exec_dialog();
                    }
                    Err(e) => {
                        self.cmd_exec_output_preview = Some(format!("エラー: {e}"));
                        self.status_message = Some(format!("コマンド読込エラー: {e}"));
                    }
                }
            }
            2 => {
                // 選択行置換 (:[range]!)
                let (start, end) = match self.cursor.anchor {
                    Some(anchor) => (
                        anchor.line.min(self.cursor.pos.line),
                        anchor.line.max(self.cursor.pos.line),
                    ),
                    None => (self.cursor.pos.line, self.cursor.pos.line),
                };
                match self.filter_range_with_command(&cmd, start, end) {
                    Ok(count) => {
                        self.status_message =
                            Some(format!(":[range]!{cmd} ➔ {count} 行に置換しました"));
                        self.close_cmd_exec_dialog();
                    }
                    Err(e) => {
                        self.cmd_exec_output_preview = Some(format!("エラー: {e}"));
                        self.status_message = Some(format!("置換エラー: {e}"));
                    }
                }
            }
            _ => {}
        }
    }
}
