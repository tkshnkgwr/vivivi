//! 行編集支援 (Line Edit) モジュール。
//!
//! ViVi 4 の `insBegEndLine`（行頭・行末文字列一括挿入）に準拠し、
//! 複数行の先頭／末尾にプレフィックス・サフィックスを一括追加します。

/// 行頭および行末に文字列を一括挿入します。
///
/// 複数行に対する一括コメントアウト（`// `）や、HTMLタグ（`<li>` ... `</li>`）での囲み処理などを高速に行います。
///
/// # Parameters
/// - `lines`: 対象行のスライス。
/// - `beg_str`: 行頭に挿入する文字列（不要なら空文字 `""`）。
/// - `end_str`: 行末に挿入する文字列（不要なら空文字 `""`）。
/// - `keep_indent`: `true` の場合、既存のインデント空白の直後に行頭文字列を挿入します。
/// - `skip_empty`: `true` の場合、空行（空白のみの行）はスキップして何も挿入しません。
///
/// # Examples
///
/// ```
/// use vivivi_modes::line_edit::insert_beg_end_lines;
///
/// let lines = vec!["apple", "banana"];
/// let tagged = insert_beg_end_lines(&lines, "<li>", "</li>", false, false);
/// assert_eq!(tagged[0], "<li>apple</li>");
/// assert_eq!(tagged[1], "<li>banana</li>");
/// ```
pub fn insert_beg_end_lines(
    lines: &[&str],
    beg_str: &str,
    end_str: &str,
    keep_indent: bool,
    skip_empty: bool,
) -> Vec<String> {
    let mut result = Vec::with_capacity(lines.len());

    for &line in lines {
        if skip_empty && line.trim().is_empty() {
            result.push(line.to_string());
            continue;
        }

        if keep_indent {
            let indent_byte_len: usize = line
                .chars()
                .take_while(|c| c.is_whitespace())
                .map(|c| c.len_utf8())
                .sum();
            let (indent, content) = line.split_at(indent_byte_len);
            result.push(format!("{indent}{beg_str}{content}{end_str}"));
        } else {
            result.push(format!("{beg_str}{line}{end_str}"));
        }
    }

    result
}

use std::cmp::Ordering;

/// 行ソートコマンド（`:sort`）の動作オプション。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SortOptions {
    /// 逆順ソート（`:sort!` または `:sort r`）
    pub reverse: bool,
    /// 大文字・小文字を区別しない（`:sort i`）
    pub ignore_case: bool,
    /// 重複行を除去する（`:sort u`）
    pub unique: bool,
    /// 行内の数値を数値順で比較する（`:sort n`）
    pub numeric: bool,
}

/// `:sort` に続くフラグ文字列（例: `"!"`, `"! iu"`, `"n r"`）を解析して [`SortOptions`] を構築します。
pub fn parse_sort_flags(args: &str) -> Result<SortOptions, String> {
    let mut opts = SortOptions::default();
    for ch in args.chars() {
        if ch.is_whitespace() {
            continue;
        }
        match ch {
            '!' | 'r' => opts.reverse = true,
            'i' => opts.ignore_case = true,
            'u' => opts.unique = true,
            'n' => opts.numeric = true,
            other => return Err(format!("不明な sort フラグ: '{other}'")),
        }
    }
    Ok(opts)
}

fn extract_leading_number(s: &str) -> Option<f64> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        let is_signed_start =
            (b == b'-' || b == b'+') && i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit();
        if b.is_ascii_digit() || is_signed_start {
            let start = i;
            if b == b'-' || b == b'+' {
                i += 1;
            }
            let mut seen_dot = false;
            while i < bytes.len() {
                if bytes[i].is_ascii_digit() {
                    i += 1;
                } else if bytes[i] == b'.'
                    && !seen_dot
                    && i + 1 < bytes.len()
                    && bytes[i + 1].is_ascii_digit()
                {
                    seen_dot = true;
                    i += 1;
                } else {
                    break;
                }
            }
            if let Ok(val) = s[start..i].parse::<f64>()
                && val.is_finite()
            {
                return Some(val);
            }
        }
        i += 1;
    }
    None
}

fn compare_lines(a: &str, b: &str, options: SortOptions) -> Ordering {
    let base = if options.numeric {
        match (extract_leading_number(a), extract_leading_number(b)) {
            (Some(na), Some(nb)) => na.total_cmp(&nb),
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        }
    } else {
        Ordering::Equal
    };

    let ord = if base != Ordering::Equal {
        base
    } else if options.ignore_case {
        a.to_lowercase().cmp(&b.to_lowercase())
    } else {
        a.cmp(b)
    };

    if options.reverse { ord.reverse() } else { ord }
}

/// 指定された行スライスを [`SortOptions`] に従ってソートします。
///
/// # Examples
///
/// ```
/// use vivivi_modes::line_edit::{SortOptions, sort_lines};
///
/// let lines = vec!["cherry", "apple", "banana"];
/// let sorted = sort_lines(&lines, SortOptions::default());
/// assert_eq!(sorted, vec!["apple", "banana", "cherry"]);
/// ```
pub fn sort_lines(lines: &[&str], options: SortOptions) -> Vec<String> {
    let mut sorted: Vec<String> = lines.iter().map(|s| (*s).to_string()).collect();
    sorted.sort_by(|a, b| compare_lines(a, b, options));
    if options.unique {
        sorted.dedup_by(|a, b| {
            if options.ignore_case {
                a.to_lowercase() == b.to_lowercase()
            } else {
                a == b
            }
        });
    }
    sorted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_beg_end_comment_out() {
        let lines = vec!["fn hello() {", "    println!(\"hi\");", "}"];
        // インデントを保持して行頭に // を挿入
        let edited = insert_beg_end_lines(&lines, "// ", "", true, true);
        assert_eq!(
            edited,
            vec!["// fn hello() {", "    // println!(\"hi\");", "// }",]
        );
    }

    #[test]
    fn test_insert_beg_end_html_tag() {
        let lines = vec!["Apple", "Banana", "", "Cherry"];
        // 空行をスキップして <li> ... </li> で囲む
        let edited = insert_beg_end_lines(&lines, "<li>", "</li>", false, true);
        assert_eq!(
            edited,
            vec!["<li>Apple</li>", "<li>Banana</li>", "", "<li>Cherry</li>",]
        );
    }

    #[test]
    fn test_insert_beg_end_zenkaku_space_indent() {
        let lines = vec!["　　全角インデント", "　テスト"];
        let edited = insert_beg_end_lines(&lines, "/* ", " */", true, false);
        assert_eq!(edited, vec!["　　/* 全角インデント */", "　/* テスト */"]);
    }

    #[test]
    fn test_sort_lines_variants() {
        let lines = vec!["banana", "Apple", "cherry", "apple"];
        assert_eq!(
            sort_lines(&lines, SortOptions::default()),
            vec!["Apple", "apple", "banana", "cherry"]
        );
        assert_eq!(
            sort_lines(
                &lines,
                SortOptions {
                    reverse: true,
                    ..Default::default()
                }
            ),
            vec!["cherry", "banana", "apple", "Apple"]
        );
        assert_eq!(
            sort_lines(
                &lines,
                SortOptions {
                    ignore_case: true,
                    unique: true,
                    ..Default::default()
                }
            ),
            vec!["Apple", "banana", "cherry"]
        );

        let nums = vec!["item 20", "item 3", "item -5", "item 10"];
        assert_eq!(
            sort_lines(
                &nums,
                SortOptions {
                    numeric: true,
                    ..Default::default()
                }
            ),
            vec!["item -5", "item 3", "item 10", "item 20"]
        );
    }
}
