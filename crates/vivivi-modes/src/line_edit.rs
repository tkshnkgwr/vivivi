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
            let indent_len = line.chars().take_while(|c| c.is_whitespace()).count();
            let (indent, content) = line.split_at(indent_len);
            result.push(format!("{indent}{beg_str}{content}{end_str}"));
        } else {
            result.push(format!("{beg_str}{line}{end_str}"));
        }
    }

    result
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
}
