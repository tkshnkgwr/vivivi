//! 文字コード・改行コードの自動判別および相互変換モジュール。
//!
//! OpenViVi の `charEncoding.cpp` に基づき、BOM（Byte Order Mark）の判定および
//! UTF-8, Shift_JIS (CP932), EUC-JP, UTF-16 の自動判別と、
//! CRLF / LF / CR 改行コードの正規化・保存時変換を提供します。

use encoding_rs::{EUC_JP, SHIFT_JIS, UTF_8, UTF_16BE, UTF_16LE};

/// テキストファイルの文字エンコーディング種別。
///
/// UTF-8（BOM有無）、Shift_JIS（CP932互換）、EUC-JP、UTF-16（LE/BE）をサポートします。
///
/// # Examples
///
/// ```
/// use vivivi_core::CharEncoding;
///
/// let enc = CharEncoding::Utf8;
/// assert_eq!(enc.name(), "UTF-8");
/// ```
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
    /// ステータスライン等で表示するためのエンコーディング表示名を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::CharEncoding;
    ///
    /// assert_eq!(CharEncoding::ShiftJis.name(), "Shift_JIS");
    /// assert_eq!(CharEncoding::Utf8Bom.name(), "UTF-8 (BOM)");
    /// ```
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

    /// 文字列（例: `"utf-8"`, `"sjis"`, `"euc-jp"`）から文字エンコーディングを判別します。
    ///
    /// 大文字小文字やハイフン・アンダースコアの差異を柔軟に吸収します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::CharEncoding;
    ///
    /// assert_eq!(CharEncoding::parse_enc("utf-8"), Some(CharEncoding::Utf8));
    /// assert_eq!(CharEncoding::parse_enc("sjis"), Some(CharEncoding::ShiftJis));
    /// assert_eq!(CharEncoding::parse_enc("cp932"), Some(CharEncoding::ShiftJis));
    /// assert_eq!(CharEncoding::parse_enc("euc-jp"), Some(CharEncoding::EucJp));
    /// assert_eq!(CharEncoding::parse_enc("unknown"), None);
    /// ```
    pub fn parse_enc(s: &str) -> Option<Self> {
        let clean = s.trim().to_ascii_lowercase().replace(['-', '_'], "");
        match clean.as_str() {
            "utf8" => Some(Self::Utf8),
            "utf8bom" => Some(Self::Utf8Bom),
            "sjis" | "shiftjis" | "cp932" | "windows31j" => Some(Self::ShiftJis),
            "eucjp" | "euc" => Some(Self::EucJp),
            "utf16le" | "utf16" => Some(Self::Utf16Le),
            "utf16be" => Some(Self::Utf16Be),
            _ => None,
        }
    }
}

/// テキストファイルの改行コード種別。
///
/// LF（Unix/macOS）、CRLF（Windows）、CR（旧Mac）を区別します。
///
/// # Examples
///
/// ```
/// use vivivi_core::NewlineType;
///
/// let nl = NewlineType::Crlf;
/// assert_eq!(nl.as_str(), "\r\n");
/// assert_eq!(nl.name(), "CRLF");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NewlineType {
    #[default]
    Lf,
    Crlf,
    Cr,
}

impl NewlineType {
    /// 文字列としての改行記号（`\n`, `\r\n`, `\r`）を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::NewlineType;
    ///
    /// assert_eq!(NewlineType::Lf.as_str(), "\n");
    /// assert_eq!(NewlineType::Crlf.as_str(), "\r\n");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::Crlf => "\r\n",
            Self::Cr => "\r",
        }
    }

    /// ステータスライン等で表示するための改行コード表示名を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::NewlineType;
    ///
    /// assert_eq!(NewlineType::Lf.name(), "LF");
    /// assert_eq!(NewlineType::Crlf.name(), "CRLF");
    /// assert_eq!(NewlineType::Cr.name(), "CR");
    /// ```
    pub fn name(&self) -> &'static str {
        match self {
            Self::Lf => "LF",
            Self::Crlf => "CRLF",
            Self::Cr => "CR",
        }
    }

    /// 文字列（例: `"unix"`, `"dos"`, `"mac"`, `"lf"`, `"crlf"`）から改行種別を判別します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::NewlineType;
    ///
    /// assert_eq!(NewlineType::parse_ff("unix"), Some(NewlineType::Lf));
    /// assert_eq!(NewlineType::parse_ff("dos"), Some(NewlineType::Crlf));
    /// assert_eq!(NewlineType::parse_ff("mac"), Some(NewlineType::Cr));
    /// assert_eq!(NewlineType::parse_ff("crlf"), Some(NewlineType::Crlf));
    /// assert_eq!(NewlineType::parse_ff("unknown"), None);
    /// ```
    pub fn parse_ff(s: &str) -> Option<Self> {
        let clean = s.trim().to_ascii_lowercase();
        match clean.as_str() {
            "unix" | "lf" => Some(Self::Lf),
            "dos" | "crlf" => Some(Self::Crlf),
            "mac" | "cr" => Some(Self::Cr),
            _ => None,
        }
    }
}

/// 生バイト列から文字コードを自動判別します。
///
/// 1. BOM（Byte Order Mark）を先頭で即時判定（UTF-8 BOM, UTF-16LE, UTF-16BE）。
/// 2. 特有文字（排他的バイトパターン）の早期検出（Shift_JIS特有の第2バイト、EUC-JPかな領域、UTF-8 3/4バイト列）。
/// 3. 特有文字が検出されなかった場合（ASCII英数のみ等）は、統計スコアリングおよびUTF-8妥当性検証で決定します。
///
/// # Examples
///
/// ```
/// use vivivi_core::{CharEncoding, detect_encoding};
///
/// let (enc, _) = detect_encoding(b"Hello, World!");
/// assert_eq!(enc, CharEncoding::Utf8);
///
/// let (bom_enc, _) = detect_encoding(&[0xef, 0xbb, 0xbf, b'A']);
/// assert_eq!(bom_enc, CharEncoding::Utf8Bom);
/// ```
pub fn detect_encoding(bytes: &[u8]) -> (CharEncoding, &'static encoding_rs::Encoding) {
    if bytes.is_empty() {
        return (CharEncoding::Utf8, UTF_8);
    }

    // 1. BOM（Byte Order Mark）判定（先頭数バイトで即座に確定）
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return (CharEncoding::Utf8Bom, UTF_8);
    }
    if bytes.starts_with(&[0xff, 0xfe]) {
        return (CharEncoding::Utf16Le, UTF_16LE);
    }
    if bytes.starts_with(&[0xfe, 0xff]) {
        return (CharEncoding::Utf16Be, UTF_16BE);
    }

    // UTF-16 BOMなし検出（0x00 が1バイトおきに頻出する場合）
    if bytes.len() >= 4 {
        let sample = &bytes[..bytes.len().min(512)];
        let zeros_even = sample.iter().step_by(2).filter(|&&b| b == 0).count();
        let zeros_odd = sample
            .iter()
            .skip(1)
            .step_by(2)
            .filter(|&&b| b == 0)
            .count();
        if zeros_even > sample.len() / 4 && zeros_odd == 0 {
            return (CharEncoding::Utf16Be, UTF_16BE);
        } else if zeros_odd > sample.len() / 4 && zeros_even == 0 {
            return (CharEncoding::Utf16Le, UTF_16LE);
        }
    }

    // 最大 64KB までをサンプリング
    let scan_len = bytes.len().min(65536);
    let sample = &bytes[..scan_len];

    // 2. UTF-8 妥当性判定
    //    UTF-8 として合法であれば即座に UTF-8 確定
    //    （Shift_JIS / EUC-JP の日本語テキストが UTF-8 として妥当になることは構造上あり得ない）
    if std::str::from_utf8(sample).is_ok() {
        return (CharEncoding::Utf8, UTF_8);
    }

    // 3. Shift_JIS vs EUC-JP の早期判定（特有文字・排他バイトが出た瞬間に即時確定）
    let mut sjis_score = 0usize;
    let mut euc_score = 0usize;

    let mut i = 0;
    while i < sample.len() {
        let b1 = sample[i];

        // --- A. EUC-JP 特有確定パターン（Shift_JISより先に判定） ---
        // EUC-JP ひらがな領域: 0xA4 + 0xA1..=0xF3 (ぁ〜ん)
        // EUC-JP カタカナ領域: 0xA5 + 0xA1..=0xF6 (ァ〜ヶ)
        // Shift_JIS では半角カナ（0xA1..=0xDF）の連続になり、通常文章ではあり得ないため即確定
        if (b1 == 0xa4 || b1 == 0xa5) && i + 1 < sample.len() {
            let b2 = sample[i + 1];
            if (b1 == 0xa4 && matches!(b2, 0xa1..=0xf3))
                || (b1 == 0xa5 && matches!(b2, 0xa1..=0xf6))
            {
                return (CharEncoding::EucJp, EUC_JP);
            }
        }

        // EUC-JP 3バイト文字 (補助漢字 0x8F + 0xA1..=0xFE + 0xA1..=0xFE)
        if b1 == 0x8f && i + 2 < sample.len() {
            let b2 = sample[i + 1];
            let b3 = sample[i + 2];
            if matches!(b2, 0xa1..=0xfe) && matches!(b3, 0xa1..=0xfe) {
                euc_score += 3;
                i += 3;
                continue;
            }
        }

        // EUC-JP 半角カナ (0x8E + 0xA1..=0xDF)
        if b1 == 0x8e && i + 1 < sample.len() {
            let b2 = sample[i + 1];
            if matches!(b2, 0xa1..=0xdf) {
                euc_score += 2;
                i += 2;
                continue;
            }
        }

        // --- B. Shift_JIS 特有確定パターン ---
        if (matches!(b1, 0x81..=0x9f | 0xe0..=0xfc)) && i + 1 < sample.len() {
            let b2 = sample[i + 1];

            // 決定打1: 0x81..=0x9F (0x8E, 0x8F除く) は EUC-JP には存在しないコード領域
            // (0x82 はひらがな、0x83 はカタカナ、0x88..=0x98 はJIS第1水準漢字)
            // これが出た時点で即座に Shift_JIS 確定
            if matches!(b1, 0x81..=0x8d | 0x90..=0x9f)
                && matches!(b2, 0x40..=0x7e | 0x80..=0xfc)
                && b2 != 0x7f
            {
                return (CharEncoding::ShiftJis, SHIFT_JIS);
            }

            // 決定打2: 第2バイトが 0x40..=0x7E (ASCII英数記号領域、0x7F除く)
            // EUC-JP のマルチバイト後続バイト（0xA1..=0xFE）には絶対にあり得ないため Shift_JIS 確定
            if matches!(b2, 0x40..=0x7e) && b2 != 0x7f {
                return (CharEncoding::ShiftJis, SHIFT_JIS);
            }

            // 決定打3: 0x8E / 0x8F で第2バイトが EUC-JP 半角カナ範囲外
            if matches!(b1, 0x8e | 0x8f) && matches!(b2, 0x40..=0x7e | 0xe0..=0xfc) && b2 != 0x7f {
                return (CharEncoding::ShiftJis, SHIFT_JIS);
            }

            // 注意: b1 が 0xE0..=0xFC かつ b2 が 0xA1..=0xFE の場合、
            // EUC-JP（第2水準漢字等: 0xD0..=0xF4 + 0xA1..=0xFE）と Shift_JIS でバイト範囲が重複する！
            // そのため、ここで即座に Shift_JIS スコアに加算して continue してしまうと
            // EUC-JP の判定がスキップされてしまう。
            // したがって重複領域では双方にスコアを付けるか、重複しない場合のみ SJIS スコアを加算する。
            if matches!(b2, 0x40..=0x7e | 0x80..=0xa0) && b2 != 0x7f {
                sjis_score += 2;
                i += 2;
                continue;
            } else if matches!(b2, 0xa1..=0xfc) {
                // 重複領域: SJIS (0xE0..=0xFC + 0xA1..=0xFC) かつ EUC-JP (0xE0..=0xFC + 0xA1..=0xFE)
                sjis_score += 1;
                euc_score += 1;
                i += 2;
                continue;
            }
        }

        // 一般的な EUC-JP 2バイト文字 (0xA1..=0xFE + 0xA1..=0xFE)
        if matches!(b1, 0xa1..=0xfe) && i + 1 < sample.len() {
            let b2 = sample[i + 1];
            if matches!(b2, 0xa1..=0xfe) {
                euc_score += 2;
                i += 2;
                continue;
            }
        }

        i += 1;
    }

    // 4. 特有文字で早期確定しなかった場合の判定
    // 確証が持てない場合は encoding_rs によるデコード妥当性（エラーの有無）を検証
    if euc_score > sjis_score && euc_score > 0 {
        let (_, _, euc_err) = EUC_JP.decode(sample);
        if !euc_err {
            return (CharEncoding::EucJp, EUC_JP);
        }
    }

    if sjis_score > euc_score && sjis_score > 0 {
        let (_, _, sjis_err) = SHIFT_JIS.decode(sample);
        if !sjis_err {
            return (CharEncoding::ShiftJis, SHIFT_JIS);
        }
    }

    // スコア同点または一方がエラーだった場合: エラーのない方を優先
    let (_, _, euc_err) = EUC_JP.decode(sample);
    let (_, _, sjis_err) = SHIFT_JIS.decode(sample);

    if !euc_err && sjis_err {
        (CharEncoding::EucJp, EUC_JP)
    } else if !sjis_err && euc_err {
        (CharEncoding::ShiftJis, SHIFT_JIS)
    } else if euc_score > sjis_score && euc_score > 0 {
        (CharEncoding::EucJp, EUC_JP)
    } else if sjis_score > 0 {
        (CharEncoding::ShiftJis, SHIFT_JIS)
    } else {
        (CharEncoding::Utf8, UTF_8)
    }
}

/// テキストから改行コードを自動判別します。
///
/// CRLF、LF、CR を順次走査し、同一の改行形式が3回出現した時点で即座に早期リターンします。
/// 改行の出現が少ない場合は最大 8KB までのサンプリングで最多の形式を判定します。
///
/// # Examples
///
/// ```
/// use vivivi_core::{NewlineType, detect_newline};
///
/// assert_eq!(detect_newline(b"line1\r\nline2\r\n"), NewlineType::Crlf);
/// assert_eq!(detect_newline(b"line1\nline2\n"), NewlineType::Lf);
/// ```
pub fn detect_newline(bytes: &[u8]) -> NewlineType {
    let mut crlf_count = 0usize;
    let mut lf_count = 0usize;
    let mut cr_count = 0usize;

    // 最大 8KB をサンプリング走査（早期確定時は数行で即時 return）
    let scan_len = bytes.len().min(8192);
    let sample = &bytes[..scan_len];

    let mut i = 0;
    while i < sample.len() {
        if sample[i] == b'\r' {
            let is_crlf = if i + 1 < sample.len() {
                sample[i + 1] == b'\n'
            } else {
                bytes.get(i + 1) == Some(&b'\n')
            };
            if is_crlf {
                crlf_count += 1;
                if crlf_count >= 3 && lf_count == 0 && cr_count == 0 {
                    return NewlineType::Crlf;
                }
                i += 2;
                continue;
            } else {
                cr_count += 1;
                if cr_count >= 3 && crlf_count == 0 && lf_count == 0 {
                    return NewlineType::Cr;
                }
            }
        } else if sample[i] == b'\n' {
            lf_count += 1;
            if lf_count >= 3 && crlf_count == 0 && cr_count == 0 {
                return NewlineType::Lf;
            }
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
///
/// UTF-8 BOM や UTF-16 の BOM は自動的に除去され、判別された [`CharEncoding`] および
/// [`NewlineType`] とともに返されます。
///
/// # Examples
///
/// ```
/// use vivivi_core::{CharEncoding, NewlineType, decode_bytes};
///
/// let input = b"Hello\r\nWorld";
/// let (text, enc, nl) = decode_bytes(input);
/// assert_eq!(text, "Hello\nWorld");
/// assert_eq!(enc, CharEncoding::Utf8);
/// assert_eq!(nl, NewlineType::Crlf);
/// ```
pub fn decode_bytes(bytes: &[u8]) -> (String, CharEncoding, NewlineType) {
    let (encoding, enc_type) = detect_encoding(bytes);
    let mut newline = detect_newline(bytes);

    // BOM を除去したスライスを用意（BOMが存在する場合のみスキップ）
    let body = match encoding {
        CharEncoding::Utf8Bom => {
            if bytes.starts_with(b"\xEF\xBB\xBF") {
                &bytes[3..]
            } else {
                bytes
            }
        }
        CharEncoding::Utf16Le => {
            if bytes.starts_with(b"\xFF\xFE") {
                &bytes[2..]
            } else {
                bytes
            }
        }
        CharEncoding::Utf16Be => {
            if bytes.starts_with(b"\xFE\xFF") {
                &bytes[2..]
            } else {
                bytes
            }
        }
        _ => bytes,
    };

    let (decoded, _, _) = enc_type.decode(body);

    // UTF-16 はバイト走査で改行判定が誤るため、デコード後のテキストから再検出
    if matches!(encoding, CharEncoding::Utf16Le | CharEncoding::Utf16Be) {
        newline = detect_newline(decoded.as_bytes());
    }

    // 改行を \n に正規化
    let normalized = decoded.replace("\r\n", "\n").replace('\r', "\n");

    (normalized, encoding, newline)
}

/// 指定された文字エンコーディングで生バイト列をデコードし、改行を `\n` に正規化します。
///
/// # Examples
///
/// ```
/// use vivivi_core::{CharEncoding, NewlineType, decode_bytes_with_encoding};
///
/// let (text, enc, nl) = decode_bytes_with_encoding(b"Hello\n", CharEncoding::Utf8);
/// assert_eq!(text, "Hello\n");
/// assert_eq!(enc, CharEncoding::Utf8);
/// ```
pub fn decode_bytes_with_encoding(
    bytes: &[u8],
    encoding: CharEncoding,
) -> (String, CharEncoding, NewlineType) {
    let enc_type: &'static encoding_rs::Encoding = match encoding {
        CharEncoding::Utf8 | CharEncoding::Utf8Bom => encoding_rs::UTF_8,
        CharEncoding::ShiftJis => encoding_rs::SHIFT_JIS,
        CharEncoding::EucJp => encoding_rs::EUC_JP,
        CharEncoding::Utf16Le => encoding_rs::UTF_16LE,
        CharEncoding::Utf16Be => encoding_rs::UTF_16BE,
    };
    let mut newline = detect_newline(bytes);

    let body = match encoding {
        CharEncoding::Utf8Bom => {
            if bytes.starts_with(b"\xEF\xBB\xBF") {
                &bytes[3..]
            } else {
                bytes
            }
        }
        CharEncoding::Utf16Le => {
            if bytes.starts_with(b"\xFF\xFE") {
                &bytes[2..]
            } else {
                bytes
            }
        }
        CharEncoding::Utf16Be => {
            if bytes.starts_with(b"\xFE\xFF") {
                &bytes[2..]
            } else {
                bytes
            }
        }
        _ => bytes,
    };

    let (decoded, _, _) = enc_type.decode(body);

    // UTF-16 はバイト走査で改行判定が誤るため、デコード後のテキストから再検出
    if matches!(encoding, CharEncoding::Utf16Le | CharEncoding::Utf16Be) {
        newline = detect_newline(decoded.as_bytes());
    }

    let normalized = decoded.replace("\r\n", "\n").replace('\r', "\n");
    (normalized, encoding, newline)
}

/// バッファ内の正規化テキスト（改行 `\n`）を指定の文字コードと改行コードでバイト列にエンコードします。
///
/// 目的の改行形式への置換、BOM の付与、指定エンコーディングへの変換を一括して実行します。
///
/// # Examples
///
/// ```
/// use vivivi_core::{CharEncoding, NewlineType, encode_text};
///
/// let bytes = encode_text("A\nB", CharEncoding::Utf8, NewlineType::Crlf);
/// assert_eq!(bytes, b"A\r\nB");
/// ```
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
