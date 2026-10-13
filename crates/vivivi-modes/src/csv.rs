//! Inline CSV/TSV mode inspired by ViVi 4.x specifications.

use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Delimiter {
    #[default]
    Comma,
    Tab,
}

impl Delimiter {
    pub fn as_char(&self) -> char {
        match self {
            Self::Comma => ',',
            Self::Tab => '\t',
        }
    }
}

#[derive(Debug, Clone)]
pub struct CsvOptions {
    pub delimiter: Delimiter,
    pub header_prefix: Option<char>,
    pub min_col_width: usize,
    pub max_col_width: Option<usize>,
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

#[derive(Debug, Clone, Default)]
pub struct CsvMode {
    pub options: CsvOptions,
    pub column_widths: Vec<usize>,
}

impl CsvMode {
    pub fn new(options: CsvOptions) -> Self {
        Self {
            options,
            column_widths: Vec::new(),
        }
    }

    /// Calculates column widths based on text lines.
    pub fn calculate_widths<'a, I>(&mut self, lines: I)
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut widths = Vec::new();
        let delim = self.options.delimiter.as_char();

        for line in lines {
            let cells = line.split(delim);
            for (col_idx, cell) in cells.enumerate() {
                let text = cell.trim_matches(|c| c == '"' || c == '\'');
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
}
