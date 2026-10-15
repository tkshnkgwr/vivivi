//! 文字列変換モジュール（全角/半角、ひらがな/カタカナ、大文字/小文字）。
//! ViVi / OpenViVi 準拠の日本語テキスト変換機能を提供します。

/// サポートする文字列変換の種別。
///
/// 全角/半角、ひらがな/カタカナ、大文字/小文字の相互変換ルールを指定します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::conv::{ConversionType, convert_text};
///
/// let katakana = convert_text("こんにちは", ConversionType::ToKatakana);
/// assert_eq!(katakana, "コンニチハ");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionType {
    /// ASCII英数記号・空白を全角へ変換。
    ToZenkakuAlphanumeric,
    /// 全角英数記号・全角空白を半角ASCIIへ変換。
    ToHankakuAlphanumeric,
    /// 全角カタカナをひらがなへ変換。
    ToHiragana,
    /// ひらがなを全角カタカナへ変換。
    ToKatakana,
    /// 全角カタカナを半角カタカナへ変換。
    ToHankakuKatakana,
    /// 半角カタカナを全角カタカナへ変換（濁点・半濁点の自動結合対応）。
    ToZenkakuKatakana,
    /// アルファベットを大文字 (A-Z) へ変換。
    ToUppercase,
    /// アルファベットを小文字 (a-z) へ変換。
    ToLowercase,
    /// 大文字と小文字を反転（SwapCase）。
    SwapCase,
}

/// 指定した変換ルールに基づいて入力文字列を変換します。
///
/// 全角/半角英数、ひらがな/カタカナ（濁音・半濁音の自動合成を含む）、
/// 大文字/小文字のケース変換などを安全に実行します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::conv::{ConversionType, convert_text};
///
/// // ひらがな -> カタカナ
/// assert_eq!(convert_text("あいうえお", ConversionType::ToKatakana), "アイウエオ");
///
/// // 全角英数 -> 半角
/// assert_eq!(convert_text("ＡＢＣ　１２３", ConversionType::ToHankakuAlphanumeric), "ABC 123");
///
/// // 半角カタカナ -> 全角カタカナ（濁点結合）
/// assert_eq!(convert_text("ｶﾞｷﾞｸﾞ", ConversionType::ToZenkakuKatakana), "ガギグ");
///
/// // 大小文字反転
/// assert_eq!(convert_text("Hello, World!", ConversionType::SwapCase), "hELLO, wORLD!");
/// ```
pub fn convert_text(input: &str, conv_type: ConversionType) -> String {
    match conv_type {
        ConversionType::ToZenkakuAlphanumeric => to_zenkaku_alphanumeric(input),
        ConversionType::ToHankakuAlphanumeric => to_hankaku_alphanumeric(input),
        ConversionType::ToHiragana => to_hiragana(input),
        ConversionType::ToKatakana => to_katakana(input),
        ConversionType::ToHankakuKatakana => to_hankaku_katakana(input),
        ConversionType::ToZenkakuKatakana => to_zenkaku_katakana(input),
        ConversionType::ToUppercase => input.to_uppercase(),
        ConversionType::ToLowercase => input.to_lowercase(),
        ConversionType::SwapCase => swap_case(input),
    }
}

/// ASCII英数記号・空白を全角へ変換（！〜～、および全角スペース）。
fn to_zenkaku_alphanumeric(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            ' ' => '　',
            '!'..='~' => char::from_u32(c as u32 + 0xFEE0).unwrap_or(c),
            _ => c,
        })
        .collect()
}

/// 全角英数記号・全角スペースを半角ASCIIへ変換。
fn to_hankaku_alphanumeric(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            '　' => ' ',
            '！'..='～' => char::from_u32(c as u32 - 0xFEE0).unwrap_or(c),
            _ => c,
        })
        .collect()
}

/// 全角カタカナをひらがなへ変換（ァ..ン、ヴ等）。
fn to_hiragana(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'ァ'..='ン' => char::from_u32(c as u32 - 0x60).unwrap_or(c),
            'ヴ' => 'ゔ',
            'ヵ' => 'か',
            'ヶ' => 'け',
            _ => c,
        })
        .collect()
}

/// ひらがなを全角カタカナへ変換（ぁ..ん、ゔ等）。
fn to_katakana(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'ぁ'..='ん' => char::from_u32(c as u32 + 0x60).unwrap_or(c),
            'ゔ' => 'ヴ',
            _ => c,
        })
        .collect()
}

/// 全角カタカナを半角カタカナへ変換。
fn to_hankaku_katakana(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            // 濁音
            'ガ' => result.push_str("ｶﾞ"),
            'ギ' => result.push_str("ｷﾞ"),
            'グ' => result.push_str("ｸﾞ"),
            'ゲ' => result.push_str("ｹﾞ"),
            'ゴ' => result.push_str("ｺﾞ"),
            'ザ' => result.push_str("ｻﾞ"),
            'ジ' => result.push_str("ｼﾞ"),
            'ズ' => result.push_str("ｽﾞ"),
            'ゼ' => result.push_str("ｾﾞ"),
            'ゾ' => result.push_str("ｿﾞ"),
            'ダ' => result.push_str("ﾀﾞ"),
            'ヂ' => result.push_str("ﾁﾞ"),
            'ヅ' => result.push_str("ﾂﾞ"),
            'デ' => result.push_str("ﾃﾞ"),
            'ド' => result.push_str("ﾄﾞ"),
            'バ' => result.push_str("ﾊﾞ"),
            'ビ' => result.push_str("ﾋﾞ"),
            'ブ' => result.push_str("ﾌﾞ"),
            'ベ' => result.push_str("ﾍﾞ"),
            'ボ' => result.push_str("ﾎﾞ"),
            'ヴ' => result.push_str("ｳﾞ"),

            // 半濁音
            'パ' => result.push_str("ﾊﾟ"),
            'ピ' => result.push_str("ﾋﾟ"),
            'プ' => result.push_str("ﾌﾟ"),
            'ペ' => result.push_str("ﾍﾟ"),
            'ポ' => result.push_str("ﾎﾟ"),

            // 通常カタカナ
            'ア' => result.push('ｱ'),
            'イ' => result.push('ｲ'),
            'ウ' => result.push('ｳ'),
            'エ' => result.push('ｴ'),
            'オ' => result.push('ｵ'),
            'カ' => result.push('ｶ'),
            'キ' => result.push('ｷ'),
            'ク' => result.push('ｸ'),
            'ケ' => result.push('ｹ'),
            'コ' => result.push('ｺ'),
            'サ' => result.push('ｻ'),
            'シ' => result.push('ｼ'),
            'ス' => result.push('ｽ'),
            'セ' => result.push('ｾ'),
            'ソ' => result.push('ｿ'),
            'タ' => result.push('ﾀ'),
            'チ' => result.push('ﾁ'),
            'ツ' => result.push('ﾂ'),
            'テ' => result.push('ﾃ'),
            'ト' => result.push('ﾄ'),
            'ナ' => result.push('ﾅ'),
            'ニ' => result.push('ﾆ'),
            'ヌ' => result.push('ﾇ'),
            'ネ' => result.push('ﾈ'),
            'ノ' => result.push('ﾉ'),
            'ハ' => result.push('ﾊ'),
            'ヒ' => result.push('ﾋ'),
            'フ' => result.push('ﾌ'),
            'ヘ' => result.push('ﾍ'),
            'ホ' => result.push('ﾎ'),
            'マ' => result.push('ﾏ'),
            'ミ' => result.push('ﾐ'),
            'ム' => result.push('ﾑ'),
            'メ' => result.push('ﾒ'),
            'モ' => result.push('ﾓ'),
            'ヤ' => result.push('ﾔ'),
            'ユ' => result.push('ﾕ'),
            'ヨ' => result.push('ﾖ'),
            'ラ' => result.push('ﾗ'),
            'リ' => result.push('ﾘ'),
            'ル' => result.push('ﾙ'),
            'レ' => result.push('ﾚ'),
            'ロ' => result.push('ﾛ'),
            'ワ' => result.push('ﾜ'),
            'ヲ' => result.push('ｦ'),
            'ン' => result.push('ﾝ'),

            // 拗音・促音（小文字カタカナ）
            'ァ' => result.push('ｧ'),
            'ィ' => result.push('ｨ'),
            'ゥ' => result.push('ｩ'),
            'ェ' => result.push('ｪ'),
            'ォ' => result.push('ｫ'),
            'ッ' => result.push('ｯ'),
            'ャ' => result.push('ｬ'),
            'ュ' => result.push('ｭ'),
            'ョ' => result.push('ｮ'),

            // 記号
            'ー' => result.push('ｰ'),
            '・' => result.push('･'),
            '「' => result.push('｢'),
            '」' => result.push('｣'),
            '、' => result.push('､'),
            '。' => result.push('｡'),

            _ => result.push(c),
        }
    }
    result
}

/// 半角カタカナを全角カタカナへ変換（濁点・半濁点の自動結合）。
fn to_zenkaku_katakana(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut result = String::with_capacity(chars.len());
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        let next = if i + 1 < chars.len() {
            Some(chars[i + 1])
        } else {
            None
        };

        // 濁点・半濁点との結合判定
        if let Some(n) = next {
            if n == 'ﾞ' {
                let combined = match c {
                    'ｶ' => Some('ガ'),
                    'ｷ' => Some('ギ'),
                    'ｸ' => Some('グ'),
                    'ｹ' => Some('ゲ'),
                    'ｺ' => Some('ゴ'),
                    'ｻ' => Some('ザ'),
                    'ｼ' => Some('ジ'),
                    'ｽ' => Some('ズ'),
                    'ｾ' => Some('ゼ'),
                    'ｿ' => Some('ゾ'),
                    'ﾀ' => Some('ダ'),
                    'ﾁ' => Some('ヂ'),
                    'ﾂ' => Some('ヅ'),
                    'ﾃ' => Some('デ'),
                    'ﾄ' => Some('ド'),
                    'ﾊ' => Some('バ'),
                    'ﾋ' => Some('ビ'),
                    'ﾌ' => Some('ブ'),
                    'ﾍ' => Some('ベ'),
                    'ﾎ' => Some('ボ'),
                    'ｳ' => Some('ヴ'),
                    _ => None,
                };
                if let Some(zen) = combined {
                    result.push(zen);
                    i += 2;
                    continue;
                }
            } else if n == 'ﾟ' {
                let combined = match c {
                    'ﾊ' => Some('パ'),
                    'ﾋ' => Some('ピ'),
                    'ﾌ' => Some('プ'),
                    'ﾍ' => Some('ペ'),
                    'ﾎ' => Some('ポ'),
                    _ => None,
                };
                if let Some(zen) = combined {
                    result.push(zen);
                    i += 2;
                    continue;
                }
            }
        }

        // 単一文字マッピング
        let single = match c {
            'ｱ' => 'ア',
            'ｲ' => 'イ',
            'ｳ' => 'ウ',
            'ｴ' => 'エ',
            'ｵ' => 'オ',
            'ｶ' => 'カ',
            'ｷ' => 'キ',
            'ｸ' => 'ク',
            'ｹ' => 'ケ',
            'ｺ' => 'コ',
            'ｻ' => 'サ',
            'ｼ' => 'シ',
            'ｽ' => 'ス',
            'ｾ' => 'セ',
            'ｿ' => 'ソ',
            'ﾀ' => 'タ',
            'ﾁ' => 'チ',
            'ﾂ' => 'ツ',
            'ﾃ' => 'テ',
            'ﾄ' => 'ト',
            'ﾅ' => 'ナ',
            'ﾆ' => 'ニ',
            'ﾇ' => 'ヌ',
            'ﾈ' => 'ネ',
            'ノ' => 'ノ',
            'ﾊ' => 'ハ',
            'ﾋ' => 'ヒ',
            'ﾌ' => 'フ',
            'ﾍ' => 'ヘ',
            'ﾎ' => 'ホ',
            'ﾏ' => 'マ',
            'ﾐ' => 'ミ',
            'ﾑ' => 'ム',
            'ﾒ' => 'メ',
            'ﾓ' => 'モ',
            'ﾔ' => 'ヤ',
            'ﾕ' => 'ユ',
            'ﾖ' => 'ヨ',
            'ﾗ' => 'ラ',
            'ﾘ' => 'リ',
            'ﾙ' => 'ル',
            'ﾚ' => 'レ',
            'ﾛ' => 'ロ',
            'ﾜ' => 'ワ',
            'ｦ' => 'ヲ',
            'ﾝ' => 'ン',
            'ｧ' => 'ァ',
            'ｨ' => 'ィ',
            'ｩ' => 'ゥ',
            'ｪ' => 'ェ',
            'ｫ' => 'ォ',
            'ｯ' => 'ッ',
            'ｬ' => 'ャ',
            'ｭ' => 'ュ',
            'ｮ' => 'ョ',
            'ｰ' => 'ー',
            '･' => '・',
            '｢' => '「',
            '｣' => '」',
            '､' => '、',
            '｡' => '。',
            'ﾞ' => '゛',
            'ﾟ' => '゜',
            _ => c,
        };
        result.push(single);
        i += 1;
    }

    result
}

/// 大文字と小文字を相互反転します。
fn swap_case(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_lowercase() {
                c.to_uppercase().collect::<String>()
            } else if c.is_uppercase() {
                c.to_lowercase().collect::<String>()
            } else {
                c.to_string()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zenkaku_hankaku_alphanumeric() {
        let ascii = "Hello, World! 123 #$%";
        let zenkaku = convert_text(ascii, ConversionType::ToZenkakuAlphanumeric);
        assert_eq!(zenkaku, "Ｈｅｌｌｏ，　Ｗｏｒｌｄ！　１２３　＃＄％");

        let back = convert_text(&zenkaku, ConversionType::ToHankakuAlphanumeric);
        assert_eq!(back, ascii);
    }

    #[test]
    fn test_hiragana_katakana() {
        let hira = "こんにちは、ゔぃゔぃゔぃ！";
        let kata = convert_text(hira, ConversionType::ToKatakana);
        assert_eq!(kata, "コンニチハ、ヴィヴィヴィ！");

        let back = convert_text(&kata, ConversionType::ToHiragana);
        assert_eq!(back, hira);
    }

    #[test]
    fn test_zenkaku_hankaku_katakana() {
        let zen = "ヴィヴィヴィ ガッコー パソコン";
        let han = convert_text(zen, ConversionType::ToHankakuKatakana);
        assert_eq!(han, "ｳﾞｨｳﾞｨｳﾞｨ ｶﾞｯｺｰ ﾊﾟｿｺﾝ");

        let back = convert_text(&han, ConversionType::ToZenkakuKatakana);
        assert_eq!(back, zen);
    }

    #[test]
    fn test_case_conversions() {
        let text = "ViVi Editor 2026";
        assert_eq!(
            convert_text(text, ConversionType::ToUppercase),
            "VIVI EDITOR 2026"
        );
        assert_eq!(
            convert_text(text, ConversionType::ToLowercase),
            "vivi editor 2026"
        );
        assert_eq!(
            convert_text(text, ConversionType::SwapCase),
            "vIvI eDITOR 2026"
        );
    }
}
