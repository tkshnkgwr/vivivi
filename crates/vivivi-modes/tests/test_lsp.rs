//! LSP (Language Server Protocol) 統合テスト (tests/test_lsp.rs)

use serde_json::json;
use vivivi_modes::lsp::{
    CompletionItemKind, DiagnosticSeverity, JsonRpcMessage, LspCompletionItem, LspDiagnostic,
    LspManager, LspRange, RequestId, decode_lsp_payload, encode_lsp_payload,
};

#[test]
fn test_encode_and_decode_lsp_payload() {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "textDocument/hover",
        "params": {
            "textDocument": { "uri": "file:///test.rs" },
            "position": { "line": 10, "character": 5 }
        }
    });

    let encoded = encode_lsp_payload(&payload);
    assert!(encoded.starts_with("Content-Length: "));
    assert!(encoded.contains("\r\n\r\n"));

    // 完全なペイロードのデコード
    let (decoded, consumed) = decode_lsp_payload(&encoded).expect("should decode full payload");
    assert_eq!(decoded["id"], 1);
    assert_eq!(decoded["method"], "textDocument/hover");
    assert_eq!(consumed, encoded.len());

    // 不完全なペイロード (途中で切れている場合) は None
    let incomplete = &encoded[..encoded.len() - 10];
    assert!(decode_lsp_payload(incomplete).is_none());
}

#[test]
fn test_jsonrpc_message_variants() {
    // 1. Request
    let req_json = json!({
        "jsonrpc": "2.0",
        "id": 42,
        "method": "initialize",
        "params": {}
    });
    let msg: JsonRpcMessage = serde_json::from_value(req_json).expect("should parse request");
    match msg {
        JsonRpcMessage::Request { id, method, .. } => {
            assert_eq!(id, RequestId::Number(42));
            assert_eq!(method, "initialize");
        }
        _ => panic!("Expected Request variant"),
    }

    // 2. Notification (idなし)
    let notif_json = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": { "textDocument": { "uri": "file:///main.rs" } }
    });
    let notif_msg: JsonRpcMessage =
        serde_json::from_value(notif_json).expect("should parse notification");
    match notif_msg {
        JsonRpcMessage::Notification { method, .. } => {
            assert_eq!(method, "textDocument/didOpen");
        }
        _ => panic!("Expected Notification variant"),
    }

    // 3. Response
    let resp_json = json!({
        "jsonrpc": "2.0",
        "id": 42,
        "result": { "capabilities": {} }
    });
    let resp_msg: JsonRpcMessage =
        serde_json::from_value(resp_json).expect("should parse response");
    match resp_msg {
        JsonRpcMessage::Response { id, result, .. } => {
            assert_eq!(id, RequestId::Number(42));
            assert!(result.get("capabilities").is_some());
        }
        _ => panic!("Expected Response variant"),
    }
}

#[test]
fn test_lsp_diagnostic_severity_and_range() {
    let diag = LspDiagnostic {
        range: LspRange::single_line(5, 0, 10),
        severity: Some(DiagnosticSeverity::Error),
        message: "mismatched types".to_string(),
        source: Some("rustc".to_string()),
    };

    assert_eq!(diag.range.start.line, 5);
    assert_eq!(diag.range.end.character, 10);
    assert_eq!(diag.severity, Some(DiagnosticSeverity::Error));

    let val = serde_json::to_value(&diag).unwrap();
    assert_eq!(val["severity"], 1); // Error = 1
}

#[test]
fn test_lsp_completion_item() {
    let comp = LspCompletionItem {
        label: "clone()".to_string(),
        kind: Some(CompletionItemKind::Method),
        detail: Some("fn clone(&self) -> Self".to_string()),
        documentation: Some("Returns a copy of the value.".to_string()),
        insert_text: Some("clone()".to_string()),
    };

    let val = serde_json::to_value(&comp).unwrap();
    assert_eq!(val["label"], "clone()");
    assert_eq!(val["kind"], 2); // Method = 2
}

#[test]
fn test_lsp_manager_state_and_diagnostics() {
    let mut manager = LspManager::new();
    assert!(!manager.is_running());
    assert!(manager.server_name().is_none());

    // 未対応拡張子のテスト
    let result = manager.start_server_for_path(std::path::Path::new("test.xyz"));
    assert!(result.is_err());
}
