//! 文字コード・改行判定の公開API統合テスト
//!
//! クレート外部の利用者視点（ブラックボックス）で、UTF-8 BOM、Shift_JIS、
//! 改行コード（CRLF / LF / CR）の自動判定、デコード、エンコード処理を検証します。

use encoding_rs::SHIFT_JIS;
use vivivi_core::{CharEncoding, NewlineType, decode_bytes, detect_newline, encode_text};

#[test]
fn test_utf8_bom_detection_and_decoding() {
    let mut raw = vec![0xef, 0xbb, 0xbf];
    raw.extend_from_slice("こんにちは\r\n世界".as_bytes());

    let (text, enc, nl) = decode_bytes(&raw);
    assert_eq!(enc, CharEncoding::Utf8Bom);
    assert_eq!(nl, NewlineType::Crlf);
    assert_eq!(text, "こんにちは\n世界");

    let re_encoded = encode_text(&text, enc, nl);
    assert_eq!(re_encoded, raw);
}

#[test]
fn test_sjis_detection_and_encoding() {
    // "テスト文書\r\n二行目" を Shift_JIS でエンコード
    let (sjis_bytes, _, _) = SHIFT_JIS.encode("テスト文書\r\n二行目");
    let (text, enc, nl) = decode_bytes(&sjis_bytes);

    assert_eq!(enc, CharEncoding::ShiftJis);
    assert_eq!(nl, NewlineType::Crlf);
    assert_eq!(text, "テスト文書\n二行目");

    let encoded = encode_text(&text, enc, nl);
    assert_eq!(encoded, sjis_bytes.as_ref());
}

#[test]
fn test_newline_variations() {
    assert_eq!(detect_newline(b"a\r\nb\r\n"), NewlineType::Crlf);
    assert_eq!(detect_newline(b"a\nb\n"), NewlineType::Lf);
    assert_eq!(detect_newline(b"a\rb\r"), NewlineType::Cr);
}
