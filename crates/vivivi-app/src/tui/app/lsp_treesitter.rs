//! LSP (Language Server Protocol) および Tree-sitter 構文木解析・スマート選択モジュール。

use super::App;
use vivivi_core::CursorPosition;
use vivivi_modes::{SyntaxLang, TreeSitterEngine};
use vivivi_vi::ViMode;

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
                    let uri = format!("file:///{}", path.to_string_lossy().replace('\\', "/"));
                    let text = self.buffer.to_string();
                    let lang_id = match self.syntax_lang {
                        SyntaxLang::Rust => "rust",
                        SyntaxLang::Cpp => "cpp",
                        SyntaxLang::Python => "python",
                        SyntaxLang::JavaScript => "javascript",
                        SyntaxLang::Go => "go",
                        _ => "plaintext",
                    };
                    mgr.notify_did_open(&uri, lang_id, 1, &text);
                }
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
            self.status_message = Some("LSP サーバーを停止しました".to_string());
        } else {
            self.status_message = Some("LSP サーバーは起動していません".to_string());
        }
    }

    /// カーソル位置のホバー情報をリクエストします。
    pub fn request_lsp_hover(&mut self) {
        if let (Some(mgr), Some(path)) = (&mut self.lsp_manager, &self.file_path) {
            let uri = format!("file:///{}", path.to_string_lossy().replace('\\', "/"));
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
        if let (Some(mgr), Some(path)) = (&mut self.lsp_manager, &self.file_path) {
            let uri = format!("file:///{}", path.to_string_lossy().replace('\\', "/"));
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

    /// LSP イベントの定期ポーリング
    pub fn poll_lsp(&mut self) {
        let mut jump_target: Option<(usize, usize)> = None;
        if let Some(ref mut mgr) = self.lsp_manager {
            mgr.poll_events();
            if let Some(hover) = mgr.take_latest_hover() {
                self.lsp_hover_text = Some(hover.contents);
            }
            if let Some(loc) = mgr.latest_definition() {
                let target_line = loc.range.start.line as usize;
                let target_col = loc.range.start.character as usize;
                jump_target = Some((target_line, target_col));
                mgr.clear_latest_definition();
            }
        }
        if let Some((target_line, target_col)) = jump_target {
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
                ts.parse(&text);
            }
            self.treesitter_revision = current_rev;
        }
    }

    /// Tree-sitter 構文木を最新のバッファ内容で更新します。
    pub fn update_treesitter(&mut self) {
        if let Some(ref mut ts) = self.treesitter {
            let text = self.buffer.to_string();
            ts.parse(&text);
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
