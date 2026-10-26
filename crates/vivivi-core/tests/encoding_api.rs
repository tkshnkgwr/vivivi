//! 文字コード・改行判定の公開API統合テスト
//!
//! クレート外部の利用者視点（ブラックボックス）で、UTF-8 BOM、Shift_JIS、
//! 改行コード（CRLF / LF / CR）の自動判定、デコード、エンコード処理を検証します。

use encoding_rs::{EUC_JP, SHIFT_JIS};
use vivivi_core::{
    CharEncoding, NewlineType, decode_bytes, detect_encoding, detect_newline, encode_text,
};

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
fn test_early_detection_sjis_katakana_and_special() {
    // カタカナ「テスト」（テ: 0x83 0x65 -> 第2バイトが 0x40..=0x7E で即確定）
    let (bytes, _, _) = SHIFT_JIS.encode("テスト");
    let (enc, _) = detect_encoding(&bytes);
    assert_eq!(enc, CharEncoding::ShiftJis);

    // ダメ文字「表構十申能」（0x5C を含む Shift_JIS 漢字）
    let (bytes_special, _, _) = SHIFT_JIS.encode("表構十申能");
    let (enc_special, _) = detect_encoding(&bytes_special);
    assert_eq!(enc_special, CharEncoding::ShiftJis);

    // ひらがな「こんにちは」（0x82 0xB1 等）
    let (bytes_hira, _, _) = SHIFT_JIS.encode("こんにちは");
    let (enc_hira, _) = detect_encoding(&bytes_hira);
    assert_eq!(enc_hira, CharEncoding::ShiftJis);
}

#[test]
fn test_early_detection_euc_jp() {
    let (euc_bytes, _, _) = EUC_JP.encode("こんにちは世界\n二行目");
    let (text, enc, nl) = decode_bytes(&euc_bytes);

    assert_eq!(enc, CharEncoding::EucJp);
    assert_eq!(nl, NewlineType::Lf);
    assert_eq!(text, "こんにちは世界\n二行目");

    let re_encoded = encode_text(&text, enc, nl);
    assert_eq!(re_encoded, euc_bytes.as_ref());
}

#[test]
fn test_early_detection_utf8_japanese_and_emoji() {
    let utf8_text = "こんにちは世界🦀\n二行目";
    let (text, enc, nl) = decode_bytes(utf8_text.as_bytes());

    assert_eq!(enc, CharEncoding::Utf8);
    assert_eq!(nl, NewlineType::Lf);
    assert_eq!(text, "こんにちは世界🦀\n二行目");
}

#[test]
fn test_early_detection_newline_quick() {
    // 3連続の CRLF で早期確定
    let crlf_data = b"line1\r\nline2\r\nline3\r\nline4\r\n";
    assert_eq!(detect_newline(crlf_data), NewlineType::Crlf);

    // 3連続の LF で早期確定
    let lf_data = b"line1\nline2\nline3\nline4\n";
    assert_eq!(detect_newline(lf_data), NewlineType::Lf);

    // 3連続の CR で早期確定
    let cr_data = b"line1\rline2\rline3\rline4\r";
    assert_eq!(detect_newline(cr_data), NewlineType::Cr);
}

#[test]
fn test_newline_variations() {
    assert_eq!(detect_newline(b"a\r\nb\r\n"), NewlineType::Crlf);
    assert_eq!(detect_newline(b"a\nb\n"), NewlineType::Lf);
    assert_eq!(detect_newline(b"a\rb\r"), NewlineType::Cr);
}

#[test]
fn test_roundtrip_all_combinations() {
    let original = "日本語テスト\n二行目CRLF\n三行目記号【】";

    for target_enc in [
        CharEncoding::Utf8,
        CharEncoding::Utf8Bom,
        CharEncoding::ShiftJis,
        CharEncoding::EucJp,
    ] {
        for target_nl in [NewlineType::Lf, NewlineType::Crlf, NewlineType::Cr] {
            // エンコード
            let encoded_bytes = encode_text(original, target_enc, target_nl);

            // デコード（早期判定による自動判別）
            let (decoded_text, detected_enc, detected_nl) = decode_bytes(&encoded_bytes);

            assert_eq!(
                decoded_text, original,
                "テキスト不一致: enc={:?}, nl={:?}",
                target_enc, target_nl
            );
            assert_eq!(
                detected_enc, target_enc,
                "エンコーディング判定不一致: enc={:?}, nl={:?}",
                target_enc, target_nl
            );
            assert_eq!(
                detected_nl, target_nl,
                "改行コード判定不一致: enc={:?}, nl={:?}",
                target_enc, target_nl
            );
        }
    }
}

#[test]
fn test_edge_cases() {
    // 1. 純粋な ASCII ファイル
    let ascii_data = b"Hello, World!\nSecond line.\n";
    let (text, enc, nl) = decode_bytes(ascii_data);
    assert_eq!(enc, CharEncoding::Utf8);
    assert_eq!(nl, NewlineType::Lf);
    assert_eq!(text, "Hello, World!\nSecond line.\n");

    // 2. 改行なしの単一行ファイル
    let single_line = b"Single line without newline";
    let (_, enc, nl) = decode_bytes(single_line);
    assert_eq!(enc, CharEncoding::Utf8);
    assert_eq!(nl, NewlineType::Lf); // デフォルト

    // 3. 長い ASCII ヘッダの後に Shift_JIS 日本語が登場するファイル
    let mut header_and_sjis = vec![b'#'; 200];
    header_and_sjis.extend_from_slice(b"\n// Config file\n");
    let (sjis_comment, _, _) = SHIFT_JIS.encode("設定項目 = 値\n");
    header_and_sjis.extend_from_slice(&sjis_comment);

    let (_, enc, nl) = decode_bytes(&header_and_sjis);
    assert_eq!(enc, CharEncoding::ShiftJis);
    assert_eq!(nl, NewlineType::Lf);

    // 4. 改行コード混在（CRLF が多数、LF が少数）
    let mixed_newlines = b"line1\r\nline2\r\nline3\r\nline4\nline5\r\n";
    assert_eq!(detect_newline(mixed_newlines), NewlineType::Crlf);
}
