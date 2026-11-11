//! LSP (Language Server Protocol) クライアント統合モジュール
//!
//! Language Server Protocol (LSP v3.17) に準拠した言語サーバー統合機能を提供します。
//! JSON-RPC 2.0 / Content-Length 通信ストリーム、診断情報 (Diagnostics) のリアルタイム収集、
//! ホバー情報 (Hover)、コード補完 (Completion)、定義ジャンプ (Go to Definition) を
//! UI非依存の純粋な非同期クライアントとして提供します。

pub mod client;
pub mod manager;
pub mod protocol;

pub use client::{LspClient, ServerState};
pub use manager::{LspManager, PendingLspRequest};
pub use protocol::{
    CompletionItemKind, DiagnosticSeverity, JsonRpcError, JsonRpcMessage, LspCompletionItem,
    LspDiagnostic, LspHover, LspLocation, LspPosition, LspRange, RequestId, decode_lsp_payload,
    encode_lsp_payload,
};
