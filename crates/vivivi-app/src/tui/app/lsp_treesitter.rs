//! LSP (Language Server Protocol) および Tree-sitter 構文木解析・スマート選択モジュール。

use std::path::{Path, PathBuf};

use super::App;
use vivivi_core::CursorPosition;
use vivivi_modes::{SyntaxLang, TreeSitterEngine};
use vivivi_vi::ViMode;

/// [`SyntaxLang`] を LSP 標準の `languageId` 文字列へ変換します。
fn syntax_lang_to_lsp_id(lang: SyntaxLang) -> &'static str {
    match lang {
        SyntaxLang::Rust => "rust",
        SyntaxLang::Cpp => "cpp",
        SyntaxLang::Python => "python",
        SyntaxLang::JavaScript => "javascript",
        SyntaxLang::Json => "json",
        SyntaxLang::Markdown => "markdown",
        SyntaxLang::Sql => "sql",
        SyntaxLang::PlSql => "plsql",
        SyntaxLang::Java => "java",
        SyntaxLang::Html => "html",
        SyntaxLang::Bash => "shellscript",
        SyntaxLang::Go => "go",
        SyntaxLang::Ruby => "ruby",
        SyntaxLang::Toml => "toml",
        SyntaxLang::PowerShell => "powershell",
        SyntaxLang::Yaml => "yaml",
        SyntaxLang::Xml => "xml",
        SyntaxLang::Batch => "bat",
        SyntaxLang::Hosts | SyntaxLang::Cron | SyntaxLang::PlainText => "plaintext",
    }
}

/// ローカルファイルパスを LSP 用 `file://` URI 文字列に変換します。
fn path_to_file_uri(path: &Path) -> String {
    let norm = path.to_string_lossy().replace('\\', "/");
    if norm.starts_with('/') {
        format!("file://{norm}")
    } else {
        format!("file:///{norm}")
    }
}

/// LSP の `file://` URI をローカルファイルパスに変換します。
fn file_uri_to_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let trimmed = if rest.starts_with('/') && rest.as_bytes().get(2) == Some(&b':') {
        &rest[1..]
    } else {
        rest
    };
    let decoded = decode_percent_uri(trimmed);
    #[cfg(windows)]
    let normalized = decoded.replace('/', "\\");
    #[cfg(not(windows))]
    let normalized = decoded;
    Some(PathBuf::from(normalized))
}

/// URI 内の基本的な `%XX` パーセントエンコードをデコードします。
fn decode_percent_uri(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let (Some(hi), Some(lo)) = (
                (bytes[i + 1] as char).to_digit(16),
                (bytes[i + 2] as char).to_digit(16),
            )
        {
            out.push(((hi << 4) | lo) as u8);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

impl App {
    /// LSP サーバーを開始します。
    pub fn start_lsp(&mut self, custom_cmd: Option<&str>) {
        let mut mgr = self.lsp_manager.take().unwrap_or_default();
        let res = if let Some(cmd) = custom_cmd {
            mgr.start_server(cmd, &[])
        } else if let Some(ref path) = self.file_path {
            mgr.start_server_for_path(path)
        } else {
            Err("ファイルパスが未設定のため、LSPサーバーを特定できません (:lsp start <cmd> で指定してください)".to_string())
        };

        match res {
            Ok(()) => {
                let name = mgr.server_name().unwrap_or("LSP").to_string();
                if let Some(ref path) = self.file_path {
                    let uri = path_to_file_uri(path);
                    let text = self.buffer.to_string();
                    let lang_id = syntax_lang_to_lsp_id(self.syntax_lang);
                    mgr.notify_did_open(&uri, lang_id, 1, &text);
                    self.lsp_doc_version = 1;
                    self.lsp_synced_revision = self.buffer.revision();
                } else {
                    self.lsp_doc_version = 0;
                    self.lsp_synced_revision = 0;
                }
                self.lsp_completions.clear();
                self.lsp_manager = Some(mgr);
                self.status_message = Some(format!("LSP サーバー '{name}' を起動しました"));
            }
            Err(e) => {
                self.status_message = Some(format!("LSP 起動エラー: {e}"));
            }
        }
    }

    /// LSP サーバーを停止します。
    pub fn stop_lsp(&mut self) {
        if let Some(mut mgr) = self.lsp_manager.take() {
            mgr.stop_server();
            self.lsp_doc_version = 0;
            self.lsp_synced_revision = 0;
            self.lsp_completions.clear();
            self.status_message = Some("LSP サーバーを停止しました".to_string());
        } else {
            self.status_message = Some("LSP サーバーは起動していません".to_string());
        }
    }

    /// バッファ変更がある場合に LSP へ `textDocument/didChange` を送信して同期します。
    pub fn sync_lsp_if_dirty(&mut self) {
        let current_rev = self.buffer.revision();
        if current_rev == self.lsp_synced_revision {
            return;
        }
        if let (Some(mgr), Some(path)) = (&mut self.lsp_manager, &self.file_path) {
            let uri = path_to_file_uri(path);
            let text = self.buffer.to_string();
            self.lsp_doc_version = self.lsp_doc_version.saturating_add(1).max(1);
            mgr.notify_did_change(&uri, self.lsp_doc_version, &text);
            self.lsp_synced_revision = current_rev;
        }
    }

    /// カーソル位置のホバー情報をリクエストします。
    pub fn request_lsp_hover(&mut self) {
        self.sync_lsp_if_dirty();
        if let (Some(mgr), Some(path)) = (&mut self.lsp_manager, &self.file_path) {
            let uri = path_to_file_uri(path);
            mgr.request_hover(
                &uri,
                self.cursor.pos.line as u32,
                self.cursor.pos.col as u32,
            );
            self.lsp_show_hover = true;
            self.status_message = Some("LSP ホバー情報を要求中...".to_string());
        } else {
            self.status_message = Some("LSP サーバーが起動していません (:lsp start)".to_string());
        }
    }

    /// カーソル位置のシンボル定義ジャンプをリクエストします。
    pub fn request_lsp_definition(&mut self) {
        self.sync_lsp_if_dirty();
        if let (Some(mgr), Some(path)) = (&mut self.lsp_manager, &self.file_path) {
            let uri = path_to_file_uri(path);
            mgr.request_definition(
                &uri,
                self.cursor.pos.line as u32,
                self.cursor.pos.col as u32,
            );
            self.status_message = Some("LSP 定義ジャンプを要求中...".to_string());
        } else {
            self.status_message = Some("LSP サーバーが起動していません (:lsp start)".to_string());
        }
    }

    /// カーソル位置のコード補完候補 (`textDocument/completion`) をリクエストします。
    pub fn request_lsp_completion(&mut self) {
        self.sync_lsp_if_dirty();
        if let (Some(mgr), Some(path)) = (&mut self.lsp_manager, &self.file_path) {
            let uri = path_to_file_uri(path);
            mgr.request_completion(
                &uri,
                self.cursor.pos.line as u32,
                self.cursor.pos.col as u32,
            );
            self.status_message = Some("LSP 補完候補を要求中...".to_string());
        } else {
            self.status_message = Some("LSP サーバーが起動していません (:lsp start)".to_string());
        }
    }

    /// LSP イベントの定期ポーリング
    pub fn poll_lsp(&mut self) {
        self.sync_lsp_if_dirty();

        let mut jump_target: Option<(Option<PathBuf>, usize, usize)> = None;
        let mut completions_received = Vec::new();
        if let Some(ref mut mgr) = self.lsp_manager {
            mgr.poll_events();
            if let Some(hover) = mgr.take_latest_hover() {
                self.lsp_hover_text = Some(hover.contents);
            }
            completions_received = mgr.take_latest_completions();
            if let Some(loc) = mgr.latest_definition() {
                let target_path = file_uri_to_path(&loc.uri);
                let target_line = loc.range.start.line as usize;
                let target_col = loc.range.start.character as usize;
                jump_target = Some((target_path, target_line, target_col));
                mgr.clear_latest_definition();
            }
        }

        if !completions_received.is_empty() {
            self.lsp_completions = completions_received
                .iter()
                .map(|c| c.label.clone())
                .collect();
            let mut lines = vec![format!(
                "=== LSP 補完候補 ({}件) ===",
                completions_received.len()
            )];
            for item in &completions_received {
                if let Some(ref detail) = item.detail {
                    lines.push(format!("{}  -  {}", item.label, detail));
                } else {
                    lines.push(item.label.clone());
                }
            }
            self.preview_lines = lines;
            self.preview_scroll = 0;
            self.show_preview = true;
            self.status_message = Some(format!(
                "LSP 補完候補を {} 件取得しました",
                completions_received.len()
            ));
        }

        if let Some((target_path, target_line, target_col)) = jump_target {
            if let Some(path) = target_path {
                let is_different = self.file_path.as_ref().is_none_or(|cur| cur != &path);
                if is_different && path.exists() {
                    let _ = self.open_file(path);
                }
            }
            self.cursor.pos.line = target_line.min(self.buffer.line_count().saturating_sub(1));
            self.cursor.pos.col = target_col;
            self.clamp_cursor();
            self.status_message = Some(format!(
                "定義へジャンプしました: 行 {}, 列 {}",
                target_line + 1,
                target_col + 1
            ));
        }
    }

    /// Tree-sitter 構文木を現在の言語設定および最新バッファ内容で再初期化します。
    pub fn reinit_treesitter(&mut self) {
        let mut ts_engine = TreeSitterEngine::new();
        if ts_engine.set_language(self.syntax_lang) {
            let text = self.buffer.to_string();
            ts_engine.parse(&text);
            self.treesitter = Some(ts_engine);
        } else {
            self.treesitter = None;
        }
        self.treesitter_lang = self.syntax_lang;
        self.treesitter_revision = self.buffer.revision();
    }

    /// バッファ編集または言語変更があった場合のみ Tree-sitter 構文木を再同期します。
    pub fn sync_treesitter_if_dirty(&mut self) {
        if self.treesitter_lang != self.syntax_lang {
            self.reinit_treesitter();
            return;
        }
        let current_rev = self.buffer.revision();
        if self.treesitter_revision != current_rev {
            if let Some(ref mut ts) = self.treesitter {
                let text = self.buffer.to_string();
                ts.parse_incremental(&text);
            }
            self.treesitter_revision = current_rev;
        }
    }

    /// Tree-sitter 構文木を最新のバッファ内容で更新します。
    pub fn update_treesitter(&mut self) {
        if let Some(ref mut ts) = self.treesitter {
            let text = self.buffer.to_string();
            ts.parse_incremental(&text);
        }
        self.treesitter_lang = self.syntax_lang;
        self.treesitter_revision = self.buffer.revision();
    }

    /// 構文木ノードに基づくスマート選択拡大（Expand Selection）を実行します。
    pub fn expand_smart_selection(&mut self) {
        self.update_treesitter();
        if let Some(ref mut ts) = self.treesitter {
            let line = self.cursor.pos.line;
            let col = self.cursor.pos.col;
            if let Some(new_range) =
                ts.smart_expand_selection(line, col, self.current_smart_selection)
            {
                self.current_smart_selection = Some(new_range);
                self.cursor.anchor = Some(CursorPosition::new(
                    new_range.start_line,
                    new_range.start_col,
                ));
                self.cursor.pos =
                    CursorPosition::new(new_range.end_line, new_range.end_col.saturating_sub(1));
                self.vi.set_mode(ViMode::Visual);
                self.status_message = Some(format!(
                    "スマート選択 (拡大): 行 {}:{} 〜 {}:{}",
                    new_range.start_line + 1,
                    new_range.start_col + 1,
                    new_range.end_line + 1,
                    new_range.end_col
                ));
            }
        } else {
            self.status_message =
                Some("現在のファイルは Tree-sitter 構文木解析に対応していません".to_string());
        }
    }

    /// 構文木ノードに基づくスマート選択縮小（Shrink Selection）を実行します。
    pub fn shrink_smart_selection(&mut self) {
        if let Some(ref mut ts) = self.treesitter {
            if let Some(prev_range) = ts.smart_shrink_selection() {
                self.current_smart_selection = Some(prev_range);
                self.cursor.anchor = Some(CursorPosition::new(
                    prev_range.start_line,
                    prev_range.start_col,
                ));
                self.cursor.pos =
                    CursorPosition::new(prev_range.end_line, prev_range.end_col.saturating_sub(1));
                self.status_message = Some(format!(
                    "スマート選択 (縮小): 行 {}:{} 〜 {}:{}",
                    prev_range.start_line + 1,
                    prev_range.start_col + 1,
                    prev_range.end_line + 1,
                    prev_range.end_col
                ));
            } else {
                self.current_smart_selection = None;
                self.cursor.anchor = None;
                self.vi.set_mode(ViMode::Normal);
                self.status_message = Some("スマート選択を解除しました".to_string());
            }
        }
    }
}
