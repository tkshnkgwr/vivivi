//! LSP クライアント・プロセス通信マネージャー
//!
//! 言語サーバープロセス（`rust-analyzer`, `clangd` 等）の起動、
//! 標準入出力（stdio）経由での非同期 JSON-RPC メッセージ送受信を管理します。
//! UIスレッドをブロックしないよう、読み取り・書き込みはバックグラウンドスレッドで実行します。

use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread;

use super::protocol::{JsonRpcMessage, RequestId, decode_lsp_payload, encode_lsp_payload};

/// 実行中の言語サーバーステート
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    /// 起動中 / 初期化ネゴシエーション中
    Initializing,
    /// 準備完了 (Ready)
    Running,
    /// 終了中
    ShuttingDown,
    /// 停止済み
    Stopped,
}

/// 言語サーバークライアント
pub struct LspClient {
    /// 子プロセスのハンドル
    child: Option<Child>,
    /// 子プロセスの stdin 送信ハンドル
    stdin_tx: Sender<String>,
    /// 子プロセスの stdout 受信ハンドル
    rx: Receiver<JsonRpcMessage>,
    /// 次のリクエストID生成カウンタ
    next_id: AtomicU64,
    /// サーバー状態
    state: Arc<Mutex<ServerState>>,
    /// コマンドライン
    command_name: String,
}

impl LspClient {
    /// 言語サーバープロセスを起動してクライアントを生成します。
    pub fn spawn(cmd: &str, args: &[&str]) -> std::io::Result<Self> {
        let mut child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let mut stdin = child.stdin.take().expect("Failed to open child stdin");
        let mut stdout = child.stdout.take().expect("Failed to open child stdout");

        let (stdin_tx, stdin_rx): (Sender<String>, Receiver<String>) = channel();
        let (out_tx, out_rx): (Sender<JsonRpcMessage>, Receiver<JsonRpcMessage>) = channel();

        let state = Arc::new(Mutex::new(ServerState::Initializing));
        let state_clone = Arc::clone(&state);

        // 1. stdin 書き込み専用スレッド
        thread::Builder::new()
            .name("lsp-stdin-writer".to_string())
            .spawn(move || {
                while let Ok(msg) = stdin_rx.recv() {
                    if stdin.write_all(msg.as_bytes()).is_err() || stdin.flush().is_err() {
                        break;
                    }
                }
            })?;

        // 2. stdout 読み出し専用スレッド
        thread::Builder::new()
            .name("lsp-stdout-reader".to_string())
            .spawn(move || {
                let mut buffer = Vec::new();
                let mut temp_buf = [0u8; 4096];

                loop {
                    match stdout.read(&mut temp_buf) {
                        Ok(0) => break, // EOF
                        Ok(n) => {
                            buffer.extend_from_slice(&temp_buf[..n]);
                            // バッファから可能な限りメッセージを抽出
                            while let Ok(raw_str) = std::str::from_utf8(&buffer) {
                                if let Some((val, consumed)) = decode_lsp_payload(raw_str) {
                                    if let Ok(parsed) =
                                        serde_json::from_value::<JsonRpcMessage>(val)
                                    {
                                        let _ = out_tx.send(parsed);
                                    }
                                    buffer.drain(..consumed);
                                } else {
                                    break;
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
                if let Ok(mut s) = state_clone.lock() {
                    *s = ServerState::Stopped;
                }
            })?;

        Ok(Self {
            child: Some(child),
            stdin_tx,
            rx: out_rx,
            next_id: AtomicU64::new(1),
            state,
            command_name: cmd.to_string(),
        })
    }

    /// 現在のサーバー状態を取得します。
    pub fn state(&self) -> ServerState {
        *self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// サーバー状態を設定します。
    pub fn set_state(&self, new_state: ServerState) {
        if let Ok(mut s) = self.state.lock() {
            *s = new_state;
        }
    }

    /// コマンド名を取得します。
    pub fn command_name(&self) -> &str {
        &self.command_name
    }

    /// JSON-RPC リクエストを送信し、割り当てられた RequestId を返します。
    pub fn send_request(&self, method: &str, params: serde_json::Value) -> RequestId {
        let id_num = self.next_id.fetch_add(1, Ordering::SeqCst);
        let id = RequestId::Number(id_num);

        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });

        let payload = encode_lsp_payload(&req);
        let _ = self.stdin_tx.send(payload);
        id
    }

    /// JSON-RPC 通知 (Notification) を送信します（IDなし）。
    pub fn send_notification(&self, method: &str, params: serde_json::Value) {
        let notif = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        let payload = encode_lsp_payload(&notif);
        let _ = self.stdin_tx.send(payload);
    }

    /// 受信したメッセージを非同期にポーリングします。
    pub fn try_recv(&self) -> Option<JsonRpcMessage> {
        self.rx.try_recv().ok()
    }

    /// サーバーを強制終了します。
    pub fn kill(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.set_state(ServerState::Stopped);
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        self.kill();
    }
}
