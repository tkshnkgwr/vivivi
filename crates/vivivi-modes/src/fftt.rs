//! ViVi 3 / ViVi 4 準拠の f/F/t/T 全角文字自動マッチ (`fFtT.txt` 連携) モジュール。
//!
//! 半角ASCII（英数字・記号・空白）入力時、対応する全角文字（全角英数・日本語括弧・句読点等）にも
//! 自動的に透過マッチさせます。

/// 与えられた検索文字 `target` と候補文字 `candidate` が一致するか判定します。
///
/// 半角文字が指定された場合、対応する全角文字（全角英数字、全角スペース、日本語括弧・句読点等）も
/// 一致として判定します（OpenViVi の `fFtT.txt` 準拠）。
///
/// # Examples
///
/// ```
/// use vivivi_modes::fftt::char_matches_fftt;
///
/// assert!(char_matches_fftt('a', 'ａ'));
/// assert!(char_matches_fftt('(', '（'));
/// assert!(char_matches_fftt('(', '「'));
/// assert!(char_matches_fftt(' ', '　'));
/// assert!(!char_matches_fftt('a', 'b'));
/// ```
pub fn char_matches_fftt(target: char, candidate: char) -> bool {
    if target == candidate {
        return true;
    }

    // 全角英数（ASCII 0x21..=0x7E に対する 0xFF01..=0xFF5E）の判定
    if target.is_ascii_graphic() {
        let full_width_code = (target as u32) + 0xFEE0;
        if candidate as u32 == full_width_code {
            return true;
        }
    }

    // 空白（半角スペースと全角スペース）
    if target == ' ' && candidate == '\u{3000}' {
        return true;
    }

    // 日本語記号の特殊マッピング（句読点、日本語引用符・括弧等）
    match target {
        '(' => matches!(candidate, '（' | '「' | '『' | '【' | '〈' | '《'),
        ')' => matches!(candidate, '）' | '」' | '』' | '】' | '〉' | '》'),
        '[' => matches!(candidate, '［' | '【' | '〔'),
        ']' => matches!(candidate, '］' | '】' | '〕'),
        '{' => matches!(candidate, '｛'),
        '}' => matches!(candidate, '｝'),
        '<' => matches!(candidate, '＜' | '〈' | '《'),
        '>' => matches!(candidate, '＞' | '〉' | '》'),
        '"' => matches!(candidate, '“' | '”' | '″'),
        '\'' => matches!(candidate, '‘' | '’' | '′'),
        ',' => matches!(candidate, '、' | '，'),
        '.' => matches!(candidate, '。' | '．'),
        '-' => matches!(candidate, 'ー' | '−' | '―' | '‐'),
        '~' => matches!(candidate, '〜' | '～'),
        '\\' => matches!(candidate, '￥' | '＼'),
        _ => false,
    }
}

/// 指定行内において、現在列 `col` から右（前方）に向かって `target`（全角含む）を探索します。
///
/// # Parameters
/// - `line`: 対象行文字列。
/// - `col`: 探索起点のカラム位置（0-indexed）。
/// - `target`: 検索する文字（半角指定時は全角もヒット）。
/// - `till`: `true`（`t` コマンド）なら一致文字の1文字手前、`false`（`f` コマンド）なら一致文字そのものの位置を返します。
/// - `count`: 何番目の一致を探すか（通常は 1）。
///
/// # Examples
///
/// ```
/// use vivivi_modes::fftt::find_char_forward;
///
/// let line = "hello (world)";
/// // 'o' (col 4) から右の '(' を探す
/// assert_eq!(find_char_forward(line, 4, '(', false, 1), Some(6));
/// // 't' コマンド（1文字手前）
/// assert_eq!(find_char_forward(line, 4, '(', true, 1), Some(5));
/// ```
pub fn find_char_forward(
    line: &str,
    col: usize,
    target: char,
    till: bool,
    count: usize,
) -> Option<usize> {
    let chars: Vec<char> = line.chars().collect();
    let mut found_count = 0;

    for (i, &ch) in chars.iter().enumerate().skip(col + 1) {
        if char_matches_fftt(target, ch) {
            found_count += 1;
            if found_count == count {
                return if till {
                    if i > 0 { Some(i - 1) } else { None }
                } else {
                    Some(i)
                };
            }
        }
    }
    None
}

/// 指定行内において、現在列 `col` から左（後方）に向かって `target`（全角含む）を探索します。
///
/// # Parameters
/// - `line`: 対象行文字列。
/// - `col`: 探索起点のカラム位置（0-indexed）。
/// - `target`: 検索する文字（半角指定時は全角もヒット）。
/// - `till`: `true`（`T` コマンド）なら一致文字の1文字直後、`false`（`F` コマンド）なら一致文字そのものの位置を返します。
/// - `count`: 何番目の一致を探すか（通常は 1）。
///
/// # Examples
///
/// ```
/// use vivivi_modes::fftt::find_char_backward;
///
/// let line = "hello (world)";
/// // ')' (col 12) から左の '(' を探す
/// assert_eq!(find_char_backward(line, 12, '(', false, 1), Some(6));
/// ```
pub fn find_char_backward(
    line: &str,
    col: usize,
    target: char,
    till: bool,
    count: usize,
) -> Option<usize> {
    let chars: Vec<char> = line.chars().collect();
    if col == 0 || chars.is_empty() {
        return None;
    }

    let mut found_count = 0;
    for (i, &ch) in chars.iter().enumerate().take(col).rev() {
        if char_matches_fftt(target, ch) {
            found_count += 1;
            if found_count == count {
                return if till { Some(i + 1) } else { Some(i) };
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_matches_fftt() {
        // 半角英数と全角英数
        assert!(char_matches_fftt('a', 'ａ'));
        assert!(char_matches_fftt('1', '１'));
        assert!(char_matches_fftt('Z', 'Ｚ'));

        // 空白
        assert!(char_matches_fftt(' ', '　'));

        // 日本語括弧・句読点
        assert!(char_matches_fftt('(', '（'));
        assert!(char_matches_fftt('(', '「'));
        assert!(char_matches_fftt(')', '）'));
        assert!(char_matches_fftt(',', '、'));
        assert!(char_matches_fftt('.', '。'));
    }

    #[test]
    fn test_find_forward_and_backward() {
        let line = "Hello (世界) 123 １２３ 「テスト」";
        // '1' で検索 -> 最初の一致は半角 '1' (col 11)
        assert_eq!(find_char_forward(line, 0, '1', false, 1), Some(11));
        // 2回目の '1' -> 全角 '１' (col 15)
        assert_eq!(find_char_forward(line, 0, '1', false, 2), Some(15));

        // '(' で検索 -> 最初の一致は半角 '(' (col 6)
        assert_eq!(find_char_forward(line, 0, '(', false, 1), Some(6));
        // '(' で2回目の一致 -> 全角 '「' (col 19)
        assert_eq!(find_char_forward(line, 0, '(', false, 2), Some(19));

        // 後方検索
        assert_eq!(find_char_backward(line, 22, '(', false, 1), Some(19));
        assert_eq!(find_char_backward(line, 19, '(', false, 1), Some(6));
    }
}
