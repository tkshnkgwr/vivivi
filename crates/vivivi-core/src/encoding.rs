//! 文字コード・改行コードの自動判別および相互変換モジュール。
//!
//! OpenViVi の `charEncoding.cpp` に基づき、BOM（Byte Order Mark）の判定および
//! UTF-8, Shift_JIS (CP932), EUC-JP, UTF-16 の自動判別と、
//! CRLF / LF / CR 改行コードの正規化・保存時変換を提供します。

use encoding_rs::{EUC_JP, SHIFT_JIS, UTF_8, UTF_16BE, UTF_16LE};

/// ファイルの文字エンコーディング種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CharEncoding {
    #[default]
    Utf8,
    Utf8Bom,
    ShiftJis,
    EucJp,
    Utf16Le,
    Utf16Be,
}

impl CharEncoding {
    /// 表示用・UI用の名称を返します。
    pub fn name(&self) -> &'static str {
        match self {
            Self::Utf8 => "UTF-8",
            Self::Utf8Bom => "UTF-8 (BOM)",
            Self::ShiftJis => "Shift_JIS",
            Self::EucJp => "EUC-JP",
            Self::Utf16Le => "UTF-16LE",
            Self::Utf16Be => "UTF-16BE",
        }
    }
}

/// ファイルの改行コード種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NewlineType {
    #[default]
    Lf,
    Crlf,
    Cr,
}

impl NewlineType {
    /// 文字列としての改行記号を返します。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::Crlf => "\r\n",
            Self::Cr => "\r",
        }
    }

    /// 表示用・UI用の名称を返します。
    pub fn name(&self) -> &'static str {
        match self {
            Self::Lf => "LF",
            Self::Crlf => "CRLF",
            Self::Cr => "CR",
        }
    }
}

/// 生バイト列から文字コードを自動判別します。
pub fn detect_encoding(bytes: &[u8]) -> (CharEncoding, &'static encoding_rs::Encoding) {
    if bytes.is_empty() {
        return (CharEncoding::Utf8, UTF_8);
    }

    // 1. BOM（Byte Order Mark）判定
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return (CharEncoding::Utf8Bom, UTF_8);
    }
    if bytes.starts_with(&[0xff, 0xfe]) {
        return (CharEncoding::Utf16Le, UTF_16LE);
    }
    if bytes.starts_with(&[0xfe, 0xff]) {
        return (CharEncoding::Utf16Be, UTF_16BE);
    }

    // 2. UTF-8 検証（エラーがなければ UTF-8 として扱う）
    if std::str::from_utf8(bytes).is_ok() {
        return (CharEncoding::Utf8, UTF_8);
    }

    // 3. 日本語文字コード（Shift_JIS vs EUC-JP）の統計的・パターン判定
    let mut sjis_score = 0usize;
    let mut euc_score = 0usize;

    let mut i = 0;
    while i < bytes.len() {
        let b1 = bytes[i];

        // Shift_JIS (0x81..=0x9F, 0xE0..=0xFC) + 2nd (0x40..=0x7E, 0x80..=0xFC)
        if (matches!(b1, 0x81..=0x9f | 0xe0..=0xfc)) && i + 1 < bytes.len() {
            let b2 = bytes[i + 1];
            if matches!(b2, 0x40..=0x7e | 0x80..=0xfc) {
                sjis_score += 2;
                i += 2;
                continue;
            }
        }

        // EUC-JP (0xA1..=0xFE) + 2nd (0xA1..=0xFE)
        if matches!(b1, 0xa1..=0xfe) && i + 1 < bytes.len() {
            let b2 = bytes[i + 1];
            if matches!(b2, 0xa1..=0xfe) {
                euc_score += 2;
                i += 2;
                continue;
            }
        }

        i += 1;
    }

    if euc_score > sjis_score && euc_score > 0 {
        (CharEncoding::EucJp, EUC_JP)
    } else if sjis_score > 0 {
        (CharEncoding::ShiftJis, SHIFT_JIS)
    } else {
        // デフォルトは UTF-8
        (CharEncoding::Utf8, UTF_8)
    }
}

/// テキストから改行コードを自動判別します。
pub fn detect_newline(bytes: &[u8]) -> NewlineType {
    let mut crlf_count = 0usize;
    let mut lf_count = 0usize;
    let mut cr_count = 0usize;

    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\r' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                crlf_count += 1;
                i += 2;
                continue;
            } else {
                cr_count += 1;
            }
        } else if bytes[i] == b'\n' {
            lf_count += 1;
        }
        i += 1;
    }

    if crlf_count >= lf_count && crlf_count >= cr_count && crlf_count > 0 {
        NewlineType::Crlf
    } else if cr_count > lf_count && cr_count > crlf_count {
        NewlineType::Cr
    } else {
        NewlineType::Lf
    }
}

/// 生バイト列から指定/自動判別されたエンコーディングで文字列へデコードし、
/// バッファ内部用に改行を `\n` に正規化します。
pub fn decode_bytes(bytes: &[u8]) -> (String, CharEncoding, NewlineType) {
    let (encoding, enc_type) = detect_encoding(bytes);
    let newline = detect_newline(bytes);

    // BOM を除去したスライスを用意
    let body = match encoding {
        CharEncoding::Utf8Bom => &bytes[3..],
        CharEncoding::Utf16Le | CharEncoding::Utf16Be => &bytes[2..],
        _ => bytes,
    };

    let (decoded, _, _) = enc_type.decode(body);

    // 改行を \n に正規化
    let normalized = decoded.replace("\r\n", "\n").replace('\r', "\n");

    (normalized, encoding, newline)
}

/// バッファ内の正規化テキスト（改行 `\n`）を指定の文字コードと改行コードでバイト列にエンコードします。
pub fn encode_text(text: &str, encoding: CharEncoding, newline: NewlineType) -> Vec<u8> {
    // 1. 改行コードを目的の形式に置換
    let target_newline_text = match newline {
        NewlineType::Lf => text.to_string(),
        NewlineType::Crlf => text.replace('\n', "\r\n"),
        NewlineType::Cr => text.replace('\n', "\r"),
    };

    // 2. エンコーディング変換
    let mut output = Vec::new();

    // BOM 付与
    match encoding {
        CharEncoding::Utf8Bom => {
            output.extend_from_slice(&[0xef, 0xbb, 0xbf]);
            output.extend_from_slice(target_newline_text.as_bytes());
        }
        CharEncoding::Utf16Le => {
            output.extend_from_slice(&[0xff, 0xfe]);
            for ch in target_newline_text.encode_utf16() {
                output.extend_from_slice(&ch.to_le_bytes());
            }
        }
        CharEncoding::Utf16Be => {
            output.extend_from_slice(&[0xfe, 0xff]);
            for ch in target_newline_text.encode_utf16() {
                output.extend_from_slice(&ch.to_be_bytes());
            }
        }
        CharEncoding::ShiftJis => {
            let (encoded, _, _) = SHIFT_JIS.encode(&target_newline_text);
            output.extend_from_slice(&encoded);
        }
        CharEncoding::EucJp => {
            let (encoded, _, _) = EUC_JP.encode(&target_newline_text);
            output.extend_from_slice(&encoded);
        }
        CharEncoding::Utf8 => {
            output.extend_from_slice(target_newline_text.as_bytes());
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // "こんにちは\n" を Shift_JIS でエンコード
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
}
