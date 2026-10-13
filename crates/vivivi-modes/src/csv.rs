//! インラインCSV/TSVモードモジュール。
//!
//! ViVi 4 の「インラインCSV表示」仕様に基づき、CSV (カンマ区切り) や TSV (タブ区切り) の
//! 各列（カラム）の最大文字幅を自動算出し、整然とした表形式ビューを構成します。

use unicode_width::UnicodeWidthStr;

/// フィールド区切り文字の種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Delimiter {
    #[default]
    Comma,
    Tab,
}

impl Delimiter {
    /// 区切り文字を `char` として返します。
    pub fn as_char(&self) -> char {
        match self {
            Self::Comma => ',',
            Self::Tab => '\t',
        }
    }
}

/// CSV/TSVモードの動作オプション。
#[derive(Debug, Clone)]
pub struct CsvOptions {
    /// 区切り文字（カンマまたはタブ）
    pub delimiter: Delimiter,
    /// ヘッダ行判定文字（例: `#` で始まる行をヘッダとして扱う）
    pub header_prefix: Option<char>,
    /// 最小列幅（空セルや短い文字列でも確保する幅）
    pub min_col_width: usize,
    /// 最大列幅（極端に長いセルに対する制限）
    pub max_col_width: Option<usize>,
    /// 表示時に囲み記号（" または '）を非表示にするかどうか
    pub hide_quotes_in_view: bool,
}

impl Default for CsvOptions {
    fn default() -> Self {
        Self {
            delimiter: Delimiter::Comma,
            header_prefix: Some('#'),
            min_col_width: 3,
            max_col_width: None,
            hide_quotes_in_view: true,
        }
    }
}

/// インラインCSVモード管理構造体。
#[derive(Debug, Clone, Default)]
pub struct CsvMode {
    /// オプション設定
    pub options: CsvOptions,
    /// 各列の計算済み表示幅リスト
    pub column_widths: Vec<usize>,
}

impl CsvMode {
    /// 指定したオプションで新しい CsvMode を生成します。
    pub fn new(options: CsvOptions) -> Self {
        Self {
            options,
            column_widths: Vec::new(),
        }
    }

    /// 行イテレータを走査し、各カラムの最大表示幅（全角半角を考慮）を計算します。
    pub fn calculate_widths<'a, I>(&mut self, lines: I)
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut widths = Vec::new();
        let delim = self.options.delimiter.as_char();

        for line in lines {
            let cells = Self::parse_csv_cells(line, delim);
            for (col_idx, cell) in cells.iter().enumerate() {
                let text = if self.options.hide_quotes_in_view {
                    cell.content
                } else {
                    cell.raw
                };
                let cell_width = UnicodeWidthStr::width(text).max(self.options.min_col_width);

                if col_idx >= widths.len() {
                    widths.push(cell_width);
                } else if cell_width > widths[col_idx] {
                    widths[col_idx] = cell_width;
                }
            }
        }

        self.column_widths = widths;
    }

    /// 行文字列をクォートを考慮してセル配列に分割します。
    pub fn parse_csv_cells<'a>(line: &'a str, delim: char) -> Vec<CsvCell<'a>> {
        let mut cells = Vec::new();
        let mut in_quotes = false;
        let mut start = 0;
        let chars: Vec<(usize, char)> = line.char_indices().collect();

        for &(idx, ch) in &chars {
            if ch == '"' || ch == '\'' {
                in_quotes = !in_quotes;
            } else if ch == delim && !in_quotes {
                let raw_cell = &line[start..idx];
                cells.push(CsvCell::from_raw(raw_cell));
                start = idx + ch.len_utf8();
            }
        }

        if start <= line.len() {
            let raw_cell = &line[start..];
            cells.push(CsvCell::from_raw(raw_cell));
        }

        cells
    }

    /// 行を表示用にフォーマットします。
    ///
    /// - `is_active_line == true`（カーソル行）: 編集の直感性を保つため生のテキストをそのまま返します。
    /// - `is_active_line == false`（非アクティブ行）:
    ///   クォートの透過（非表示）と各列のパディングを行い、縦の列が美しく整列したビューを返します。
    pub fn format_line(&self, line: &str, is_active_line: bool) -> String {
        if is_active_line {
            return line.to_string();
        }

        let delim = self.options.delimiter.as_char();
        let cells = Self::parse_csv_cells(line, delim);
        let mut formatted = String::new();

        for (i, cell) in cells.iter().enumerate() {
            if i > 0 {
                formatted.push(delim);
                formatted.push(' ');
            }

            let text = if self.options.hide_quotes_in_view {
                cell.content
            } else {
                cell.raw
            };

            formatted.push_str(text);

            // カラム最大幅に合わせたスペースパディング
            if let Some(&max_width) = self.column_widths.get(i) {
                let current_width = UnicodeWidthStr::width(text);
                if max_width > current_width {
                    formatted.push_str(&" ".repeat(max_width - current_width));
                }
            }
        }

        formatted
    }

    /// 指定列番号のセルが生テキスト内のどの文字範囲 `[start, end)` にあるかを返します。
    pub fn cell_range_in_raw_line(&self, line: &str, target_col: usize) -> Option<(usize, usize)> {
        let delim = self.options.delimiter.as_char();
        let mut current_col = 0;
        let mut start = 0;
        let mut in_quotes = false;

        for (idx, ch) in line.char_indices() {
            if ch == '"' || ch == '\'' {
                in_quotes = !in_quotes;
            } else if ch == delim && !in_quotes {
                if current_col == target_col {
                    return Some((start, idx));
                }
                current_col += 1;
                start = idx + ch.len_utf8();
            }
        }

        if current_col == target_col {
            Some((start, line.len()))
        } else {
            None
        }
    }
}

/// CSVセルの情報構造体。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvCell<'a> {
    /// 生のセル文字列（外側のクォートを含む）
    pub raw: &'a str,
    /// クォート記号を除去した表示用文字列
    pub content: &'a str,
    /// 外側がクォート記号で囲まれていたかどうか
    pub is_quoted: bool,
}

impl<'a> CsvCell<'a> {
    /// 生の文字列から `CsvCell` を構築します。
    pub fn from_raw(raw: &'a str) -> Self {
        let trimmed = raw.trim();
        let (content, is_quoted) =
            if (trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2)
                || (trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2)
            {
                (&trimmed[1..trimmed.len() - 1], true)
            } else {
                (raw, false)
            };

        Self {
            raw,
            content,
            is_quoted,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_column_widths_calculation() {
        let mut csv = CsvMode::default();
        let lines = vec!["id,name,score", "1,\"Alice\",100", "20,\"Bob (ボブ)\",85"];

        csv.calculate_widths(lines);

        // 列0: "20" (幅2) -> min_col_width 3 なので 3
        assert_eq!(csv.column_widths[0], 3);
        // 列1: "Bob (ボブ)" -> 'Bob ' (4) + '(' (1) + 'ボブ' (全角2*2=4) + ')' (1) = 10
        assert_eq!(csv.column_widths[1], 10);
        // 列2: "score" (幅5)
        assert_eq!(csv.column_widths[2], 5);
    }

    #[test]
    fn test_tsv_delimiter() {
        let mut tsv = CsvMode::new(CsvOptions {
            delimiter: Delimiter::Tab,
            ..Default::default()
        });

        let lines = vec!["ColA\tColB", "LongerData\t1"];
        tsv.calculate_widths(lines);

        assert_eq!(tsv.column_widths[0], 10); // "LongerData"
        assert_eq!(tsv.column_widths[1], 4); // "ColB"
    }

    #[test]
    fn test_csv_format_line_and_quote_hiding() {
        let mut csv = CsvMode::default();
        let lines = vec!["id,name", "1,\"Alice\""];
        csv.calculate_widths(lines);

        // アクティブ行（カーソル行）は生のまま
        let active = csv.format_line("1,\"Alice\"", true);
        assert_eq!(active, "1,\"Alice\"");

        // 非アクティブ行はクォートが外れ、幅に合わせて整列
        let inactive = csv.format_line("1,\"Alice\"", false);
        // idの最小幅3 -> '1  ', Aliceの幅5 -> 'Alice'
        assert_eq!(inactive, "1  , Alice");
    }

    #[test]
    fn test_cell_range_in_raw_line() {
        let csv = CsvMode::default();
        let line = "10,\"Alice, Smith\",99";

        // 第0列: "10" (0..2)
        assert_eq!(csv.cell_range_in_raw_line(line, 0), Some((0, 2)));
        // 第1列: "\"Alice, Smith\"" (3..17)
        assert_eq!(csv.cell_range_in_raw_line(line, 1), Some((3, 17)));
        // 第2列: "99" (18..20)
        assert_eq!(csv.cell_range_in_raw_line(line, 2), Some((18, 20)));
        // 範囲外列
        assert_eq!(csv.cell_range_in_raw_line(line, 3), None);
    }
}
