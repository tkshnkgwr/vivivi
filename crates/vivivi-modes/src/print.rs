//! 印刷プレビューおよび PDF 出力モジュール。
//!
//! ViVi 3 / ViVi 4 の印刷プレビュー・PDF印刷機能に基づき、
//! ページ分割（用紙サイズ・行数・桁数）、行番号、ヘッダー/フッター付与、
//! プレビュー整形、および自己完結型の PDF 1.4 出力エンジンを提供します。

use std::fs;
use std::path::Path;

/// 印刷・ページ分割設定。
///
/// 用紙ごとの行数・桁数の上限、行番号の有無、およびヘッダー・フッターの書式を保持します。
///
/// # Examples
/// ```
/// use vivivi_modes::print::PrintSettings;
///
/// let settings = PrintSettings::default();
/// assert_eq!(settings.lines_per_page, 60);
/// assert!(settings.show_line_numbers);
/// ```
#[derive(Debug, Clone)]
pub struct PrintSettings {
    /// 1ページあたりの行数（ヘッダー・フッターを含む、A4縦の標準約60行）
    pub lines_per_page: usize,
    /// 1行あたりの最大文字数（折り返し幅）
    pub columns_per_line: usize,
    /// 行番号を表示するかどうかのフラグ
    pub show_line_numbers: bool,
    /// ヘッダー文字列テンプレート（例: `"%FileName%  [%Date%]"`）
    pub header_format: String,
    /// フッター文字列テンプレート（例: `"- %Page% / %Total% -"`）
    pub footer_format: String,
}

impl Default for PrintSettings {
    /// A4縦向き印刷を想定した標準設定を生成します。
    ///
    /// - 1ページあたり60行、1行あたり80桁
    /// - 行番号表示有効
    /// - ファイル名と日付を含むヘッダー、ページ番号を含むフッター
    fn default() -> Self {
        Self {
            lines_per_page: 60,
            columns_per_line: 80,
            show_line_numbers: true,
            header_format: "%FileName%  [%Date%]".to_string(),
            footer_format: "- %Page% / %Total% -".to_string(),
        }
    }
}

/// 分割された1ページ分の印刷データ。
///
/// ヘッダー、本文行、余白パディング、およびフッターを含む全行が格納されます。
///
/// # Examples
/// ```
/// use vivivi_modes::print::PrintPage;
///
/// let page = PrintPage {
///     page_number: 1,
///     lines: vec!["Header".to_string(), "Line 1".to_string(), "Footer".to_string()],
/// };
/// assert_eq!(page.page_number, 1);
/// assert_eq!(page.lines.len(), 3);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintPage {
    /// ページ番号（1-based）
    pub page_number: usize,
    /// ページを構成する行テキスト一覧（ヘッダー・本文・フッター含む）
    pub lines: Vec<String>,
}

/// 指定テキストを設定に基づいてページ分割し、ヘッダー・フッター・行番号付きのページ一覧を生成します。
///
/// 桁数を超える長い行は自動的に折り返され、行番号は継続行としてインデント整列されます。
/// ヘッダー内の `%FileName%`, `%Date%` や、フッター内の `%Page%`, `%Total%` プレースホルダーは自動置換されます。
///
/// # 引数
/// - `text`: 印刷対象のテキスト文字列
/// - `file_name`: ヘッダー等に表示するファイル名
/// - `date_str`: ヘッダー等に表示する日付文字列
/// - `settings`: ページ分割・フォーマット設定への参照
///
/// # 戻り値
/// 分割された各ページデータ（[`PrintPage`]）のベクタ
///
/// # Examples
/// ```
/// use vivivi_modes::print::{paginate_text, PrintSettings};
///
/// let text = "First line\nSecond line";
/// let settings = PrintSettings::default();
/// let pages = paginate_text(text, "sample.rs", "2026-10-15", &settings);
/// assert_eq!(pages.len(), 1);
/// assert_eq!(pages[0].page_number, 1);
/// ```
pub fn paginate_text(
    text: &str,
    file_name: &str,
    date_str: &str,
    settings: &PrintSettings,
) -> Vec<PrintPage> {
    let mut raw_pages: Vec<Vec<String>> = Vec::new();
    let body_capacity = settings.lines_per_page.saturating_sub(4); // ヘッダー2行＋フッター2行

    let mut current_page: Vec<String> = Vec::new();
    let mut current_lineno = 1;

    for line in text.lines() {
        // 折り返し処理（文字表示幅を考慮した分割）
        let max_content_width = if settings.show_line_numbers {
            settings.columns_per_line.saturating_sub(8) // " 123 | "
        } else {
            settings.columns_per_line
        };

        if line.is_empty() {
            let formatted = if settings.show_line_numbers {
                format!("{:4} | ", current_lineno)
            } else {
                String::new()
            };
            current_page.push(formatted);
            current_lineno += 1;
            if current_page.len() >= body_capacity {
                raw_pages.push(current_page);
                current_page = Vec::new();
            }
            continue;
        }

        // 表示幅に応じたチャンク分割（全角2桁、半角1桁を考慮）
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_width = 0;

        for c in line.chars() {
            let char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);
            if current_width + char_width > max_content_width && !current_chunk.is_empty() {
                chunks.push(std::mem::take(&mut current_chunk));
                current_width = 0;
            }
            current_chunk.push(c);
            current_width += char_width;
        }
        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        for (chunk_idx, chunk) in chunks.into_iter().enumerate() {
            let formatted = if settings.show_line_numbers {
                if chunk_idx == 0 {
                    format!("{:4} | {}", current_lineno, chunk)
                } else {
                    format!("     : {}", chunk)
                }
            } else {
                chunk
            };

            current_page.push(formatted);
            if chunk_idx == 0 {
                current_lineno += 1;
            }

            if current_page.len() >= body_capacity {
                raw_pages.push(current_page);
                current_page = Vec::new();
            }
        }
    }

    if !current_page.is_empty() {
        raw_pages.push(current_page);
    }

    if raw_pages.is_empty() {
        raw_pages.push(Vec::new());
    }

    let total_pages = raw_pages.len();
    let mut pages = Vec::with_capacity(total_pages);

    for (p_idx, body) in raw_pages.into_iter().enumerate() {
        let page_num = p_idx + 1;
        let mut page_lines = Vec::new();

        // ヘッダー
        let header = settings
            .header_format
            .replace("%FileName%", file_name)
            .replace("%Date%", date_str);
        page_lines.push(header);
        page_lines.push("-".repeat(settings.columns_per_line));

        // 本文
        let mut count = body.len();
        for l in body {
            page_lines.push(l);
        }

        // 余白パディング
        while count < body_capacity {
            page_lines.push(String::new());
            count += 1;
        }

        // フッター
        let footer = settings
            .footer_format
            .replace("%Page%", &page_num.to_string())
            .replace("%Total%", &total_pages.to_string());
        page_lines.push("-".repeat(settings.columns_per_line));
        page_lines.push(center_text(&footer, settings.columns_per_line));

        pages.push(PrintPage {
            page_number: page_num,
            lines: page_lines,
        });
    }

    pages
}

fn center_text(s: &str, width: usize) -> String {
    use unicode_width::UnicodeWidthStr;
    let len = s.width();
    if len >= width {
        return s.to_string();
    }
    let pad = (width - len) / 2;
    format!("{}{}", " ".repeat(pad), s)
}

/// ページ一覧をエディタ上で閲覧可能な単一のプレビューテキスト文字列に結合します。
///
/// 各ページの境界には `===== PAGE X =====` のような区切り線が挿入されます。
///
/// # Examples
/// ```
/// use vivivi_modes::print::{paginate_text, format_preview_text, PrintSettings};
///
/// let settings = PrintSettings::default();
/// let pages = paginate_text("Hello World", "test.txt", "2026-10-15", &settings);
/// let preview = format_preview_text(&pages);
/// assert!(preview.contains("===== PAGE 1 ====="));
/// assert!(preview.contains("Hello World"));
/// ```
pub fn format_preview_text(pages: &[PrintPage]) -> String {
    let mut out = String::new();
    for p in pages {
        out.push_str(&format!("===== PAGE {} =====\n", p.page_number));
        for l in &p.lines {
            out.push_str(l);
            out.push('\n');
        }
        out.push('\n');
    }
    out
}

/// ページデータを自己完結型 PDF 1.4 ファイルとして出力・保存します。
///
/// 外部Cライブラリや重厚なクレートに依存せず、A4用紙サイズ（595.28 x 841.89 pt）の
/// 標準PDFバイナリを直接エンコードして書き出します。
/// 日本語（CJK）およびASCII混在環境に完全対応するため、Adobe 標準 CIDFont（`HeiseiKakuGo-W5` / `UniJIS-UTF16-H`）
/// を採用し、半角英数字（CID 1..230）を幅500、全角文字を幅1000の等幅グリッドとして出力します。
///
/// # 引数
/// - `pages`: 出力対象のページデータ一覧
/// - `output_path`: 出力先PDFファイルのパス
///
/// # Errors
/// 出力先ディレクトリの作成失敗やファイル書き込み権限不足など、I/Oエラーが発生した場合にエラー文字列を返します。
///
/// # Examples
/// ```rust,no_run
/// use vivivi_modes::print::{paginate_text, export_to_pdf, PrintSettings};
///
/// let settings = PrintSettings::default();
/// let pages = paginate_text("Code content", "main.rs", "2026-10-15", &settings);
/// export_to_pdf(&pages, "output.pdf").unwrap();
/// ```
pub fn export_to_pdf<P: AsRef<Path>>(pages: &[PrintPage], output_path: P) -> Result<(), String> {
    let mut pdf = Vec::new();
    let mut offsets = Vec::new();

    // PDF ヘッダー
    pdf.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");

    let num_pages = pages.len();
    // オブジェクト番号:
    // 1: Catalog
    // 2: Outlines
    // 3: Pages
    // 4 .. 4 + num_pages - 1: Page objects
    // 4 + num_pages: Font object (Type0 HeiseiKakuGo-W5)
    // 5 + num_pages: CIDFontType0 object
    // 6 + num_pages: FontDescriptor object
    // 7 + num_pages .. 7 + num_pages + num_pages - 1: Content stream objects

    let font_obj_id = 4 + num_pages;
    let cid_font_id = font_obj_id + 1;
    let font_desc_id = font_obj_id + 2;
    let first_content_id = font_obj_id + 3;

    // 1 0 obj: Catalog
    offsets.push(pdf.len());
    pdf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 3 0 R >>\nendobj\n");

    // 2 0 obj: Outlines
    offsets.push(pdf.len());
    pdf.extend_from_slice(b"2 0 obj\n<< /Type /Outlines /Count 0 >>\nendobj\n");

    // 3 0 obj: Pages
    offsets.push(pdf.len());
    let mut kids_str = String::new();
    for i in 0..num_pages {
        kids_str.push_str(&format!("{} 0 R ", 4 + i));
    }
    let pages_obj = format!(
        "3 0 obj\n<< /Type /Pages /Count {} /Kids [ {} ] >>\nendobj\n",
        num_pages, kids_str
    );
    pdf.extend_from_slice(pages_obj.as_bytes());

    // 4.. 各 Page オブジェクト
    for i in 0..num_pages {
        offsets.push(pdf.len());
        let content_id = first_content_id + i;
        let page_obj = format!(
            "{} 0 obj\n<< /Type /Page /Parent 3 0 R /MediaBox [0 0 595.28 841.89] /Contents {} 0 R /Resources << /Font << /F1 {} 0 R >> >> >>\nendobj\n",
            4 + i,
            content_id,
            font_obj_id
        );
        pdf.extend_from_slice(page_obj.as_bytes());
    }

    // Type0 Font オブジェクト (HeiseiKakuGo-W5 / UniJIS-UTF16-H)
    offsets.push(pdf.len());
    let font_obj = format!(
        "{} 0 obj\n<< /Type /Font /Subtype /Type0 /BaseFont /HeiseiKakuGo-W5-UniJIS-UTF16-H /Encoding /UniJIS-UTF16-H /DescendantFonts [ {} 0 R ] >>\nendobj\n",
        font_obj_id, cid_font_id
    );
    pdf.extend_from_slice(font_obj.as_bytes());

    // CIDFontType0 オブジェクト（半角=500、全角=1000）
    offsets.push(pdf.len());
    let cid_font_obj = format!(
        "{} 0 obj\n<< /Type /Font /Subtype /CIDFontType0 /BaseFont /HeiseiKakuGo-W5 /CIDSystemInfo << /Registry (Adobe) /Ordering (Japan1) /Supplement 4 >> /FontDescriptor {} 0 R /DW 1000 /W [ 1 230 500 ] >>\nendobj\n",
        cid_font_id, font_desc_id
    );
    pdf.extend_from_slice(cid_font_obj.as_bytes());

    // FontDescriptor オブジェクト
    offsets.push(pdf.len());
    let desc_obj = format!(
        "{} 0 obj\n<< /Type /FontDescriptor /FontName /HeiseiKakuGo-W5 /Flags 4 /FontBBox [-100 -200 1000 900] /ItalicAngle 0 /Ascent 880 /Descent -120 /CapHeight 700 /StemV 80 >>\nendobj\n",
        font_desc_id
    );
    pdf.extend_from_slice(desc_obj.as_bytes());

    // 各 Content stream オブジェクト
    for (i, page) in pages.iter().enumerate() {
        offsets.push(pdf.len());
        let content_id = first_content_id + i;

        let mut stream = String::new();
        stream.push_str("BT\n/F1 9 Tf\n12 TL\n40 800 Td\n");

        for line in &page.lines {
            stream.push_str(&encode_pdf_hex_line(line));
        }
        stream.push_str("ET\n");

        let stream_bytes = stream.as_bytes();
        let content_obj = format!(
            "{} 0 obj\n<< /Length {} >>\nstream\n",
            content_id,
            stream_bytes.len()
        );
        pdf.extend_from_slice(content_obj.as_bytes());
        pdf.extend_from_slice(stream_bytes);
        pdf.extend_from_slice(b"\nendstream\nendobj\n");
    }

    // xref テーブル
    let xref_offset = pdf.len();
    let total_objs = offsets.len() + 1; // 0番を含む
    pdf.extend_from_slice(format!("xref\n0 {}\n", total_objs).as_bytes());
    pdf.extend_from_slice(b"0000000000 65535 f \n");
    for off in &offsets {
        pdf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }

    // trailer
    let trailer = format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
        total_objs, xref_offset
    );
    pdf.extend_from_slice(trailer.as_bytes());

    if let Some(parent) = output_path.as_ref().parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(output_path, pdf).map_err(|e| e.to_string())?;

    Ok(())
}

/// 文字列を行単位の PDF 16進テキスト（UTF-16BE `<HEX> '\n`）としてエンコードします。
fn encode_pdf_hex_line(s: &str) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(s.len() * 4 + 4);
    out.push('<');
    for u in s.encode_utf16() {
        let _ = write!(out, "{:04x}", u);
    }
    out.push_str("> '\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paginate_and_preview() {
        let text = "Line 1\nLine 2\nLine 3\n";
        let settings = PrintSettings {
            lines_per_page: 10,
            columns_per_line: 40,
            show_line_numbers: true,
            header_format: "TestDoc".into(),
            footer_format: "%Page%".into(),
        };

        let pages = paginate_text(text, "test.txt", "2026-10-14", &settings);
        assert_eq!(pages.len(), 1);
        let preview = format_preview_text(&pages);
        assert!(preview.contains("TestDoc"));
        assert!(preview.contains("Line 1"));
    }

    #[test]
    fn test_paginate_japanese_width() {
        let text = "こんにちは世界\nHello World\n";
        let settings = PrintSettings {
            lines_per_page: 10,
            columns_per_line: 20,
            show_line_numbers: false,
            header_format: "TestJP".into(),
            footer_format: "%Page%".into(),
        };

        let pages = paginate_text(text, "test_jp.txt", "2026-10-19", &settings);
        assert_eq!(pages.len(), 1);
        let preview = format_preview_text(&pages);
        assert!(preview.contains("こんにちは世界"));
        assert!(preview.contains("Hello World"));
    }

    #[test]
    fn test_export_to_pdf_japanese() {
        let temp_dir = std::env::temp_dir();
        let pdf_path = temp_dir.join("vivivi_test_jp_out.pdf");

        let settings = PrintSettings::default();
        let text =
            "fn main() {\n    // 日本語PDF出力の検証\n    println!(\"こんにちは世界！\");\n}\n";
        let pages = paginate_text(text, "main.rs", "2026-10-19", &settings);

        export_to_pdf(&pages, &pdf_path).unwrap();
        assert!(pdf_path.exists());

        let bytes = std::fs::read(&pdf_path).unwrap();
        assert!(bytes.starts_with(b"%PDF-1.4"));
        let pdf_str = String::from_utf8_lossy(&bytes);
        assert!(pdf_str.contains("/BaseFont /HeiseiKakuGo-W5"));
        assert!(pdf_str.contains("/Encoding /UniJIS-UTF16-H"));

        let _ = std::fs::remove_file(&pdf_path);
    }
}
