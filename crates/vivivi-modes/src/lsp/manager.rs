//! LSP 高レベルマネージャー (LspManager)
//!
//! 開いているドキュメント（URI）とアクティブな言語サーバーセッションを紐付け、
//! 診断情報（Diagnostics）の自動蓄積、ホバー、補完、定義ジャンプ等の
//! リクエスト送受信とキャッシュ管理を一括して担います。

use std::collections::HashMap;
use std::path::Path;

use super::client::{LspClient, ServerState};
use super::protocol::{
    JsonRpcMessage, LspCompletionItem, LspDiagnostic, LspHover, LspLocation, LspPosition, LspRange,
    RequestId,
};

/// 処理待ちリクエストの種別
#[derive(Debug, Clone)]
pub enum PendingLspRequest {
    Initialize,
    Hover { uri: String, position: LspPosition },
    Completion { uri: String, position: LspPosition },
    Definition { uri: String, position: LspPosition },
}

/// LSP マネージャー
///
/// # Examples
/// ```
/// use vivivi_modes::lsp::LspManager;
///
/// let mgr = LspManager::new();
/// assert!(!mgr.is_running());
/// ```
pub struct LspManager {
    /// 起動中の LSP クライアント
    client: Option<LspClient>,
    /// URI ごとの最新診断情報キャッシュ
    diagnostics: HashMap<String, Vec<LspDiagnostic>>,
    /// 保留中のリクエスト追跡 (RequestId -> PendingLspRequest)
    pending_requests: HashMap<RequestId, PendingLspRequest>,
    /// 直近に取得したホバー情報キャッシュ
    latest_hover: Option<LspHover>,
    /// 直近に取得した補完候補キャッシュ
    latest_completions: Vec<LspCompletionItem>,
    /// 直近に取得した定義位置
    latest_definition: Option<LspLocation>,
}

impl LspManager {
    /// 新規 LSP マネージャーを作成します。
    pub fn new() -> Self {
        Self {
            client: None,
            diagnostics: HashMap::new(),
            pending_requests: HashMap::new(),
            latest_hover: None,
            latest_completions: Vec::new(),
            latest_definition: None,
        }
    }

    /// サーバーが起動中かどうかを判定します。
    pub fn is_running(&self) -> bool {
        self.client
            .as_ref()
            .map(|c| c.state() == ServerState::Running || c.state() == ServerState::Initializing)
            .unwrap_or(false)
    }

    /// 言語サーバーコマンド名を取得します。
    pub fn server_name(&self) -> Option<&str> {
        self.client.as_ref().map(|c| c.command_name())
    }

    /// ファイルパスまたは言語種別に応じたデフォルトの言語サーバーを起動します。
    pub fn start_server_for_path(&mut self, path: &Path) -> Result<(), String> {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let (cmd, args): (&str, &[&str]) = match ext {
            "rs" => ("rust-analyzer", &[]),
            "c" | "cpp" | "h" | "hpp" => ("clangd", &[]),
            "py" => ("pyright-langserver", &["--stdio"]),
            "go" => ("gopls", &[]),
            "js" | "ts" | "jsx" | "tsx" => ("typescript-language-server", &["--stdio"]),
            _ => return Err(format!("未対応のファイル拡張子です: .{}", ext)),
        };

        self.start_server(cmd, args)
    }

    /// 指定したコマンドと引数で言語サーバーを起動し、初期化ネゴシエーションを行います。
    pub fn start_server(&mut self, cmd: &str, args: &[&str]) -> Result<(), String> {
        self.stop_server();

        let client = LspClient::spawn(cmd, args)
            .map_err(|e| format!("LSPサーバー起動失敗 ({}): {}", cmd, e))?;

        // 1. initialize リクエスト送信
        let init_params = serde_json::json!({
            "processId": std::process::id(),
            "rootUri": serde_json::Value::Null,
            "capabilities": {
                "textDocument": {
                    "hover": { "contentFormat": ["markdown", "plaintext"] },
                    "completion": { "completionItem": { "snippetSupport": false } },
                    "definition": { "linkSupport": false },
                    "publishDiagnostics": { "relatedInformation": true }
                }
            }
        });

        let req_id = client.send_request("initialize", init_params);
        self.pending_requests
            .insert(req_id, PendingLspRequest::Initialize);

        self.client = Some(client);
        Ok(())
    }

    /// 言語サーバーを安全に停止します。
    pub fn stop_server(&mut self) {
        if let Some(mut client) = self.client.take() {
            client.kill();
        }
        self.pending_requests.clear();
    }

    /// ファイルオープン通知 (textDocument/didOpen) を送信します。
    pub fn notify_did_open(&mut self, uri: &str, language_id: &str, version: i32, text: &str) {
        if let Some(client) = &self.client {
            let params = serde_json::json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": language_id,
                    "version": version,
                    "text": text
                }
            });
            client.send_notification("textDocument/didOpen", params);
        }
    }

    /// ファイル変更通知 (textDocument/didChange) をフル同期で送信します。
    pub fn notify_did_change(&mut self, uri: &str, version: i32, text: &str) {
        if let Some(client) = &self.client {
            let params = serde_json::json!({
                "textDocument": {
                    "uri": uri,
                    "version": version
                },
                "contentChanges": [
                    { "text": text }
                ]
            });
            client.send_notification("textDocument/didChange", params);
        }
    }

    /// ファイルクローズ通知 (textDocument/didClose) を送信します。
    pub fn notify_did_close(&mut self, uri: &str) {
        if let Some(client) = &self.client {
            let params = serde_json::json!({
                "textDocument": {
                    "uri": uri
                }
            });
            client.send_notification("textDocument/didClose", params);
        }
    }

    /// ホバー情報 (textDocument/hover) をリクエストします。
    pub fn request_hover(&mut self, uri: &str, line: u32, character: u32) {
        if let Some(client) = &self.client {
            let position = LspPosition::new(line, character);
            let params = serde_json::json!({
                "textDocument": { "uri": uri },
                "position": position
            });
            let req_id = client.send_request("textDocument/hover", params);
            self.pending_requests.insert(
                req_id,
                PendingLspRequest::Hover {
                    uri: uri.to_string(),
                    position,
                },
            );
        }
    }

    /// コード補完 (textDocument/completion) をリクエストします。
    pub fn request_completion(&mut self, uri: &str, line: u32, character: u32) {
        if let Some(client) = &self.client {
            let position = LspPosition::new(line, character);
            let params = serde_json::json!({
                "textDocument": { "uri": uri },
                "position": position
            });
            let req_id = client.send_request("textDocument/completion", params);
            self.pending_requests.insert(
                req_id,
                PendingLspRequest::Completion {
                    uri: uri.to_string(),
                    position,
                },
            );
        }
    }

    /// 定義ジャンプ (textDocument/definition) をリクエストします。
    pub fn request_definition(&mut self, uri: &str, line: u32, character: u32) {
        if let Some(client) = &self.client {
            let position = LspPosition::new(line, character);
            let params = serde_json::json!({
                "textDocument": { "uri": uri },
                "position": position
            });
            let req_id = client.send_request("textDocument/definition", params);
            self.pending_requests.insert(
                req_id,
                PendingLspRequest::Definition {
                    uri: uri.to_string(),
                    position,
                },
            );
        }
    }

    /// 指定した URI の診断情報リストを取得します。
    pub fn get_diagnostics(&self, uri: &str) -> Option<&[LspDiagnostic]> {
        self.diagnostics.get(uri).map(|v| v.as_slice())
    }

    /// 直近のホバー情報を取得します。
    pub fn latest_hover(&self) -> Option<&LspHover> {
        self.latest_hover.as_ref()
    }

    /// 直近のホバー情報を取得し、内部状態をクリアします。
    pub fn take_latest_hover(&mut self) -> Option<LspHover> {
        self.latest_hover.take()
    }

    /// 直近のホバー情報をクリアします。
    pub fn clear_latest_hover(&mut self) {
        self.latest_hover = None;
    }

    /// 直近の補完候補一覧を取得します。
    pub fn latest_completions(&self) -> &[LspCompletionItem] {
        &self.latest_completions
    }

    /// 直近の定義ジャンプ位置を取得します。
    pub fn latest_definition(&self) -> Option<&LspLocation> {
        self.latest_definition.as_ref()
    }

    /// 定義ジャンプ位置をクリアします。
    pub fn clear_latest_definition(&mut self) {
        self.latest_definition = None;
    }

    /// 受信キューをポーリングし、サーバーからのレスポンスや通知を処理します。
    /// （UIの毎フレームまたはイベントループ時に呼び出します）
    pub fn poll_events(&mut self) {
        let mut messages = Vec::new();
        if let Some(client) = &self.client {
            while let Some(msg) = client.try_recv() {
                messages.push(msg);
            }
        }

        for msg in messages {
            match msg {
                JsonRpcMessage::Notification { method, params, .. } => {
                    if method == "textDocument/publishDiagnostics"
                        && let Some(params) = params
                    {
                        self.handle_publish_diagnostics(params);
                    }
                }
                JsonRpcMessage::Response { id, result, .. } => {
                    if let Some(pending) = self.pending_requests.remove(&id) {
                        match pending {
                            PendingLspRequest::Initialize => {
                                if let Some(client) = &self.client {
                                    client.set_state(ServerState::Running);
                                    client.send_notification("initialized", serde_json::json!({}));
                                }
                            }
                            PendingLspRequest::Hover { .. } => {
                                self.handle_hover_response(result);
                            }
                            PendingLspRequest::Completion { .. } => {
                                self.handle_completion_response(result);
                            }
                            PendingLspRequest::Definition { .. } => {
                                self.handle_definition_response(result);
                            }
                        }
                    }
                }
                JsonRpcMessage::ErrorResponse { id, error, .. } => {
                    self.pending_requests.remove(&id);
                    eprintln!("LSP Error: code={}, msg={}", error.code, error.message);
                }
                _ => {}
            }
        }
    }

    fn handle_publish_diagnostics(&mut self, params: serde_json::Value) {
        let uri = match params.get("uri").and_then(|u| u.as_str()) {
            Some(u) => u.to_string(),
            None => return,
        };

        if let Some(diags_val) = params.get("diagnostics").and_then(|d| d.as_array()) {
            let mut diags = Vec::new();
            for d in diags_val {
                if let Ok(diag) = serde_json::from_value::<LspDiagnostic>(d.clone()) {
                    diags.push(diag);
                }
            }
            self.diagnostics.insert(uri, diags);
        }
    }

    fn handle_hover_response(&mut self, result: serde_json::Value) {
        if result.is_null() {
            self.latest_hover = None;
            return;
        }

        let contents = if let Some(contents_obj) = result.get("contents") {
            if let Some(s) = contents_obj.as_str() {
                s.to_string()
            } else if let Some(value) = contents_obj.get("value").and_then(|v| v.as_str()) {
                value.to_string()
            } else {
                contents_obj.to_string()
            }
        } else {
            result.to_string()
        };

        let range = result
            .get("range")
            .and_then(|r| serde_json::from_value::<LspRange>(r.clone()).ok());

        self.latest_hover = Some(LspHover { contents, range });
    }

    fn handle_completion_response(&mut self, result: serde_json::Value) {
        self.latest_completions.clear();
        let items_val = if let Some(items) = result.get("items").and_then(|i| i.as_array()) {
            items
        } else if let Some(items) = result.as_array() {
            items
        } else {
            return;
        };

        for item in items_val {
            if let Ok(comp) = serde_json::from_value::<LspCompletionItem>(item.clone()) {
                self.latest_completions.push(comp);
            }
        }
    }

    fn handle_definition_response(&mut self, result: serde_json::Value) {
        if result.is_null() {
            self.latest_definition = None;
            return;
        }

        if let Ok(loc) = serde_json::from_value::<LspLocation>(result.clone()) {
            self.latest_definition = Some(loc);
        } else if let Some(locs) = result.as_array()
            && let Some(first) = locs.first()
            && let Ok(loc) = serde_json::from_value::<LspLocation>(first.clone())
        {
            self.latest_definition = Some(loc);
        }
    }
}

impl Default for LspManager {
    fn default() -> Self {
        Self::new()
    }
}
