//! テキストアライン (Text Alignment) モジュール。
//!
//! ViVi 3 / ViVi 4 に準拠し、複数行テキスト内の指定文字（`=`, `:`, `,`, `//` 等）の位置で
//! 縦の列（カラム）を一括垂直整列します。

use unicode_width::UnicodeWidthStr;

/// 複数行のテキストを、指定された区切り文字列の位置で垂直に整列します。
///
/// # 引数
/// - `lines`: 整列対象の行スライス
/// - `target`: 整列の基準とする区切り文字列（例: `"="`, `":"`, `","`, `"//"`）
///
/// # 動作
/// 各行内で最初に出現する `target` を探し、その手前（左辺）の表示文字幅を揃えるように
/// スペースを補填して整列します。`target` を含まない行はそのまま維持されます。
pub fn align_lines(lines: &[&str], target: &str) -> Vec<String> {
    if target.is_empty() || lines.is_empty() {
        return lines.iter().map(|s| s.to_string()).collect();
    }

    // 1. 各行を (左辺, 区切り文字以降) に分割し、左辺の最大表示幅を計算
    let mut parsed: Vec<Option<(String, String)>> = Vec::with_capacity(lines.len());
    let mut max_left_width = 0;

    for &line in lines {
        if let Some(idx) = line.find(target) {
            let left = line[..idx].trim_end();
            let right = line[idx + target.len()..].trim_start();
            let width = UnicodeWidthStr::width(left);
            if width > max_left_width {
                max_left_width = width;
            }
            parsed.push(Some((left.to_string(), right.to_string())));
        } else {
            parsed.push(None);
        }
    }

    // 2. 最大表示幅に合わせてスペースをパディングして整形
    let mut result = Vec::with_capacity(lines.len());
    for (i, item) in parsed.into_iter().enumerate() {
        match item {
            Some((left, right)) => {
                let current_width = UnicodeWidthStr::width(left.as_str());
                let pad_len = max_left_width.saturating_sub(current_width);
                let padding = " ".repeat(pad_len);

                if right.is_empty() {
                    result.push(format!("{left}{padding} {target}"));
                } else {
                    result.push(format!("{left}{padding} {target} {right}"));
                }
            }
            None => {
                result.push(lines[i].to_string());
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_equal_sign() {
        let lines = vec![
            "let a = 1;",
            "let long_variable_name = 200;",
            "let count = 3;",
        ];
        let aligned = align_lines(&lines, "=");
        assert_eq!(
            aligned,
            vec![
                "let a                  = 1;",
                "let long_variable_name = 200;",
                "let count              = 3;",
            ]
        );
    }

    #[test]
    fn test_align_colon_with_zenkaku() {
        let lines = vec!["名前: 田中", "生年月日: 1990/01/01", "住所: 東京都"];
        let aligned = align_lines(&lines, ":");
        assert_eq!(
            aligned,
            vec![
                "名前     : 田中",
                "生年月日 : 1990/01/01",
                "住所     : 東京都",
            ]
        );
    }

    #[test]
    fn test_align_skip_unmatched_lines() {
        let lines = vec!["// コメント行", "val x = 10", "", "val yyyyy = 20"];
        let aligned = align_lines(&lines, "=");
        assert_eq!(
            aligned,
            vec!["// コメント行", "val x     = 10", "", "val yyyyy = 20",]
        );
    }
}
