//! LSP (Language Server Protocol) データ構造および JSON-RPC プロトコル定義
//!
//! Language Server Protocol 仕様 (v3.17) に準拠した基本型（Position, Range, Diagnostic,
//! Completion, Hover, Location 等）と、JSON-RPC 2.0 / Content-Length ストリームの
//! シリアライズ・デシリアライズを提供します。

use serde::{Deserialize, Serialize};

/// LSPにおけるドキュメント内の位置 (0-based)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LspPosition {
    /// 行番号 (0-based)
    pub line: u32,
    /// 列番号 (0-based, UTF-16コードユニット単位)
    pub character: u32,
}

impl LspPosition {
    /// 新しい位置を作成します。
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

/// LSPにおけるテキスト範囲
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LspRange {
    /// 開始位置
    pub start: LspPosition,
    /// 終了位置
    pub end: LspPosition,
}

impl LspRange {
    /// 新しい範囲を作成します。
    pub fn new(start: LspPosition, end: LspPosition) -> Self {
        Self { start, end }
    }

    /// 単一行の範囲を簡単に作成します。
    pub fn single_line(line: u32, start_char: u32, end_char: u32) -> Self {
        Self {
            start: LspPosition::new(line, start_char),
            end: LspPosition::new(line, end_char),
        }
    }
}

/// 診断情報の重大度 (Severity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "u32", into = "u32")]
pub enum DiagnosticSeverity {
    /// エラー (赤)
    Error = 1,
    /// 警告 (黄)
    Warning = 2,
    /// 情報 (青)
    Information = 3,
    /// ヒント (灰)
    Hint = 4,
}

impl From<u32> for DiagnosticSeverity {
    fn from(val: u32) -> Self {
        match val {
            1 => Self::Error,
            2 => Self::Warning,
            3 => Self::Information,
            4 => Self::Hint,
            _ => Self::Information,
        }
    }
}

impl From<DiagnosticSeverity> for u32 {
    fn from(val: DiagnosticSeverity) -> Self {
        val as u32
    }
}

/// 言語サーバーから発行される診断情報 (Diagnostic)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LspDiagnostic {
    /// 該当範囲
    pub range: LspRange,
    /// 重大度
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<DiagnosticSeverity>,
    /// メッセージ本文
    pub message: String,
    /// 診断ソース（例: "rustc", "clippy", "clang"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// ファイル内の位置を表す Location
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LspLocation {
    /// ドキュメントURI (例: "file:///path/to/file.rs")
    pub uri: String,
    /// 該当範囲
    pub range: LspRange,
}

/// 補完候補の種別
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "u32", into = "u32")]
pub enum CompletionItemKind {
    Text = 1,
    Method = 2,
    Function = 3,
    Constructor = 4,
    Field = 5,
    Variable = 6,
    Class = 7,
    Interface = 8,
    Module = 9,
    Property = 10,
    Unit = 11,
    Value = 12,
    Enum = 13,
    Keyword = 14,
    Snippet = 15,
    Color = 16,
    File = 17,
    Reference = 18,
    Folder = 19,
    EnumMember = 20,
    Constant = 21,
    Struct = 22,
    Event = 23,
    Operator = 24,
    TypeParameter = 25,
}

impl From<u32> for CompletionItemKind {
    fn from(val: u32) -> Self {
        match val {
            1 => Self::Text,
            2 => Self::Method,
            3 => Self::Function,
            4 => Self::Constructor,
            5 => Self::Field,
            6 => Self::Variable,
            7 => Self::Class,
            8 => Self::Interface,
            9 => Self::Module,
            10 => Self::Property,
            11 => Self::Unit,
            12 => Self::Value,
            13 => Self::Enum,
            14 => Self::Keyword,
            15 => Self::Snippet,
            16 => Self::Color,
            17 => Self::File,
            18 => Self::Reference,
            19 => Self::Folder,
            20 => Self::EnumMember,
            21 => Self::Constant,
            22 => Self::Struct,
            23 => Self::Event,
            24 => Self::Operator,
            25 => Self::TypeParameter,
            _ => Self::Text,
        }
    }
}

impl From<CompletionItemKind> for u32 {
    fn from(val: CompletionItemKind) -> Self {
        val as u32
    }
}

/// コード補完候補項目
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LspCompletionItem {
    /// 候補ラベル
    pub label: String,
    /// 候補種別
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<CompletionItemKind>,
    /// 詳細情報（型やシグネチャ）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// ドキュメンテーション
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
    /// 挿入テキスト（未指定なら label が使われる）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_text: Option<String>,
}

/// ホバー情報
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LspHover {
    /// ホバー内容（プレーンテキストまたはMarkdown）
    pub contents: String,
    /// 該当範囲
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<LspRange>,
}

/// JSON-RPC 2.0 リクエストID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    /// 数値ID
    Number(u64),
    /// 文字列ID
    String(String),
}

/// JSON-RPC 2.0 受信メッセージ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    /// サーバーからのリクエスト
    Request {
        jsonrpc: String,
        id: RequestId,
        method: String,
        #[serde(default)]
        params: Option<serde_json::Value>,
    },
    /// サーバーからのレスポンス（成功）
    Response {
        jsonrpc: String,
        id: RequestId,
        result: serde_json::Value,
    },
    /// サーバーからのエラーレスポンス
    ErrorResponse {
        jsonrpc: String,
        id: RequestId,
        error: JsonRpcError,
    },
    /// サーバーからの通知 (Notification: idなし)
    Notification {
        jsonrpc: String,
        method: String,
        #[serde(default)]
        params: Option<serde_json::Value>,
    },
}

/// JSON-RPC 2.0 エラーオブジェクト
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

/// メッセージを Content-Length 付き LSP ストリーム形式でエンコードします。
pub fn encode_lsp_payload(payload: &serde_json::Value) -> String {
    let json_text = serde_json::to_string(payload).unwrap_or_default();
    let content_len = json_text.len();
    format!("Content-Length: {}\r\n\r\n{}", content_len, json_text)
}

/// Content-Length ヘッダー付きストリームから 1 つの JSON ペイロードをパースします。
///
/// 成功した場合は `(パース済みValue, 消費バイト数)` を返します。
/// データが未完全な場合は `None` を返します。
pub fn decode_lsp_payload(raw: &str) -> Option<(serde_json::Value, usize)> {
    let header_sep = "\r\n\r\n";
    let sep_pos = raw.find(header_sep)?;

    let header_part = &raw[..sep_pos];
    let body_start = sep_pos + header_sep.len();

    let mut content_length = None;
    for line in header_part.lines() {
        if let Some(len_str) = line.strip_prefix("Content-Length:")
            && let Ok(len) = len_str.trim().parse::<usize>()
        {
            content_length = Some(len);
            break;
        }
    }

    let len = content_length?;
    let remaining_bytes = raw.len() - body_start;
    if remaining_bytes < len {
        // まだデータが最後まで届いていない
        return None;
    }

    let body_bytes = &raw.as_bytes()[body_start..body_start + len];
    let body_str = std::str::from_utf8(body_bytes).ok()?;
    let value: serde_json::Value = serde_json::from_str(body_str).ok()?;

    Some((value, body_start + len))
}
