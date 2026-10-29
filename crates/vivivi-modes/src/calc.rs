//! BOX（矩形）選択範囲の数値集計（合計・平均・標準偏差等）モジュール。
//!
//! ViVi 3 / ViVi 4 の「計算」機能（`calc.html`）に準拠し、
//! 矩形選択（VisualBlock）された複数行の文字列から数値を自動抽出し、
//! 合計、平均、標準偏差、最小値、最大値を高速計算します。

/// BOX選択範囲の数値計算結果。
///
/// 抽出された有効な数値の件数、合計、平均、標準偏差、最小値、最大値を保持します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::calc::calculate_box_lines;
///
/// let lines = vec!["10".to_string(), "20".to_string(), "30".to_string()];
/// let result = calculate_box_lines(&lines).unwrap();
/// assert_eq!(result.count, 3);
/// assert_eq!(result.sum, 60.0);
/// assert_eq!(result.average, 20.0);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct BoxCalculationResult {
    /// 抽出された有効な数値の個数
    pub count: usize,
    /// 合計値
    pub sum: f64,
    /// 平均値
    pub average: f64,
    /// 母標準偏差
    pub std_dev: f64,
    /// 最小値
    pub min: f64,
    /// 最大値
    pub max: f64,
}

impl BoxCalculationResult {
    /// ステータスバーやダイアログ用のフォーマット済みサマリー文字列を生成します。
    ///
    /// 整数値は小数点なし、小数は第2位まで整形して出力します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::calc::calculate_box_lines;
    ///
    /// let lines = vec!["10".to_string(), "20".to_string()];
    /// let result = calculate_box_lines(&lines).unwrap();
    /// let summary = result.formatted_summary();
    /// assert!(summary.contains("合計: 30"));
    /// assert!(summary.contains("平均: 15"));
    /// ```
    pub fn formatted_summary(&self) -> String {
        // 整数値かどうか判定して表示桁を最適化
        let fmt_num = |val: f64| -> String {
            if (val - val.round()).abs() < 1e-9 {
                format!("{}", val.round() as i64)
            } else {
                format!("{:.2}", val)
            }
        };

        format!(
            "合計: {}  平均: {}  標準偏差: {} (N={}, 最小: {}, 最大: {})",
            fmt_num(self.sum),
            fmt_num(self.average),
            fmt_num(self.std_dev),
            self.count,
            fmt_num(self.min),
            fmt_num(self.max),
        )
    }
}

use vivivi_core::TextBuffer;

/// 矩形行スライスから数値を抽出し、統計計算を実行します。
///
/// カンマ区切り（`1,234.50` など）は自動的に除去されて数値として解釈されます。
/// 有効な数値が1つも存在しない場合は `None` を返します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::calc::calculate_box_lines;
///
/// let lines = vec![" 1,000 ".to_string(), " 2,500 ".to_string()];
/// let result = calculate_box_lines(&lines).unwrap();
/// assert_eq!(result.sum, 3500.0);
/// assert_eq!(result.count, 2);
/// ```
pub fn calculate_box_lines(lines: &[String]) -> Option<BoxCalculationResult> {
    let mut numbers = Vec::new();
    for line in lines {
        let cleaned = normalize_calc_line(line);
        if cleaned.trim().is_empty() {
            continue;
        }

        // 行全体が数値か、または空白区切り等の先頭/各トークンから数値を抽出
        for token in cleaned.split_whitespace() {
            if let Ok(num) = token.parse::<f64>()
                && num.is_finite()
            {
                numbers.push(num);
            }
        }
    }

    if numbers.is_empty() {
        return None;
    }

    let count = numbers.len();
    let sum: f64 = numbers.iter().sum();
    let average = sum / (count as f64);

    let variance = numbers
        .iter()
        .map(|&x| {
            let diff = x - average;
            diff * diff
        })
        .sum::<f64>()
        / (count as f64);

    let std_dev = variance.sqrt();
    let min = numbers.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = numbers.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    Some(BoxCalculationResult {
        count,
        sum,
        average,
        std_dev,
        min,
        max,
    })
}

/// 指定したバッファと矩形選択範囲（行・カラム）から数値を集計します。
///
/// バッファの指定矩形範囲をスライスとして切り出し、[`calculate_box_lines`] で集計します。
/// 範囲内に数値が存在しない場合は `None` を返します。
///
/// # Examples
///
/// ```
/// use vivivi_core::TextBuffer;
/// use vivivi_modes::calc::calculate_box_selection;
///
/// let buffer = TextBuffer::from_text("item:  100\nitem:  200\n");
/// let result = calculate_box_selection(&buffer, 0, 1, 6, 10).unwrap();
/// assert_eq!(result.sum, 300.0);
/// ```
pub fn calculate_box_selection(
    buffer: &TextBuffer,
    min_line: usize,
    max_line: usize,
    min_col: usize,
    max_col: usize,
) -> Option<BoxCalculationResult> {
    let mut block_lines = Vec::new();

    for line in min_line..=max_line {
        if line >= buffer.line_count() {
            continue;
        }
        let line_str = buffer.line_text(line).unwrap_or_default();
        let chars: Vec<char> = line_str.chars().collect();

        let start = min_col.min(chars.len());
        let end = (max_col + 1).min(chars.len());

        if start < end {
            let slice: String = chars[start..end].iter().collect();
            block_lines.push(slice);
        }
    }

    calculate_box_lines(&block_lines)
}

fn normalize_calc_line(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();

    let mut i = 0;
    while i < len {
        let ch = chars[i];
        if ch == ',' {
            let prev_is_digit = i > 0 && chars[i - 1].is_ascii_digit();
            let next_3_digits = i + 3 < len
                && chars[i + 1].is_ascii_digit()
                && chars[i + 2].is_ascii_digit()
                && chars[i + 3].is_ascii_digit();
            let next_4_is_not_digit = i + 4 >= len
                || !chars[i + 4].is_ascii_digit()
                || chars[i + 4] == ','
                || chars[i + 4] == '.';

            if prev_is_digit && next_3_digits && next_4_is_not_digit {
                // 3桁区切りのカンマは除去
            } else {
                // セパレータカンマは空白に置換
                out.push(' ');
            }
        } else {
            out.push(ch);
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_calculation_basic() {
        let lines = vec![
            "  10  ".to_string(),
            "  20  ".to_string(),
            "  30  ".to_string(),
        ];
        let result = calculate_box_lines(&lines).expect("should calculate");
        assert_eq!(result.count, 3);
        assert_eq!(result.sum, 60.0);
        assert_eq!(result.average, 20.0);
        assert!((result.std_dev - 8.1649).abs() < 1e-4);
        assert_eq!(result.min, 10.0);
        assert_eq!(result.max, 30.0);
    }

    #[test]
    fn test_box_calculation_with_commas_and_floats() {
        let lines = vec![
            "1,234.50".to_string(),
            "2,000.00".to_string(),
            "-234.50".to_string(),
        ];
        let result = calculate_box_lines(&lines).expect("should calculate");
        assert_eq!(result.count, 3);
        assert_eq!(result.sum, 3000.0);
        assert_eq!(result.average, 1000.0);
        assert_eq!(result.min, -234.50);
        assert_eq!(result.max, 2000.0);
    }

    #[test]
    fn test_calculate_box_selection_from_buffer() {
        let text = "Item A:   100 yen\nItem B:   250 yen\nItem C:   150 yen";
        let buffer = TextBuffer::from_text(text);
        // カラム 10..=12 に "100", "250", "150" がある
        let result = calculate_box_selection(&buffer, 0, 2, 10, 13).expect("should find numbers");
        assert_eq!(result.count, 3);
        assert_eq!(result.sum, 500.0);
        assert_eq!(result.average, 166.66666666666666);
    }

    #[test]
    fn test_box_calculation_comma_separated_numbers() {
        // カンマ区切りの数値リストが結合されず別々の数値として集計されること
        let lines = vec!["10,20,30".to_string()];
        let result = calculate_box_lines(&lines).expect("should calculate");
        assert_eq!(result.count, 3);
        assert_eq!(result.sum, 60.0);
    }
}
