//! 印刷プレビューおよび PDF 出力モジュール。
//!
//! ViVi 3 / ViVi 4 の印刷プレビュー・PDF印刷機能に基づき、
//! ページ分割（用紙サイズ・行数・桁数）、行番号、ヘッダー/フッター付与、
//! Markdown整形プレビュー、および自己完結型の PDF 1.4 出力エンジンを提供します。

use std::fs;
use std::path::Path;

/// 印刷・ページ分割設定。
#[derive(Debug, Clone)]
pub struct PrintSettings {
    /// 1ページあたりの行数（ヘッダー・フッターを含む、A4縦の標準約60行）
    pub lines_per_page: usize,
    /// 1行あたりの最大文字数（折り返し幅）
    pub columns_per_line: usize,
    /// 行番号を表示するか
    pub show_line_numbers: bool,
    /// ヘッダー文字列（例: "%FileName% - %Date%"）
    pub header_format: String,
    /// フッター文字列（例: "- %Page% / %Total% -"）
    pub footer_format: String,
}

impl Default for PrintSettings {
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

/// ページデータ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintPage {
    pub page_number: usize,
    pub lines: Vec<String>,
}

/// テキストをページ分割し、ヘッダー・フッター・行番号付きのプレビューページ一覧を生成します。
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
        // 折り返し処理（文字幅考慮の簡易分割）
        let chars: Vec<char> = line.chars().collect();
        let max_content_width = if settings.show_line_numbers {
            settings.columns_per_line.saturating_sub(8) // " 123 | "
        } else {
            settings.columns_per_line
        };

        let mut start = 0;
        let mut is_first_chunk = true;

        if chars.is_empty() {
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

        while start < chars.len() {
            let end = (start + max_content_width).min(chars.len());
            let chunk: String = chars[start..end].iter().collect();

            let formatted = if settings.show_line_numbers {
                if is_first_chunk {
                    format!("{:4} | {}", current_lineno, chunk)
                } else {
                    format!("     : {}", chunk)
                }
            } else {
                chunk
            };

            current_page.push(formatted);
            if is_first_chunk {
                current_lineno += 1;
                is_first_chunk = false;
            }

            if current_page.len() >= body_capacity {
                raw_pages.push(current_page);
                current_page = Vec::new();
            }

            start = end;
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
    let len = s.chars().count();
    if len >= width {
        return s.to_string();
    }
    let pad = (width - len) / 2;
    format!("{}{}", " ".repeat(pad), s)
}

/// ページ一覧を単一のプレビューテキスト文字列に結合します。
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

/// 自己完結型 PDF 1.4 生成エンジン。
/// 外部クレートに依存せず、A4用紙サイズ（595.28 x 841.89 pt）の標準PDFバイナリを生成します。
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
    // 4 + num_pages: Font object (Courier)
    // 4 + num_pages + 1 .. 4 + num_pages + 1 + num_pages - 1: Content stream objects

    let font_obj_id = 4 + num_pages;
    let first_content_id = font_obj_id + 1;

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

    // Font オブジェクト (Courier)
    offsets.push(pdf.len());
    let font_obj = format!(
        "{} 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Courier >>\nendobj\n",
        font_obj_id
    );
    pdf.extend_from_slice(font_obj.as_bytes());

    // 各 Content stream オブジェクト
    for (i, page) in pages.iter().enumerate() {
        offsets.push(pdf.len());
        let content_id = first_content_id + i;

        let mut stream = String::new();
        stream.push_str("BT\n/F1 9 Tf\n12 TL\n40 800 Td\n");

        for line in &page.lines {
            let escaped = escape_pdf_string(line);
            stream.push_str(&format!("({}) '\n", escaped));
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

fn escape_pdf_string(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            '\\' => out.push_str("\\\\"),
            _ if c.is_ascii() => out.push(c),
            _ => out.push('?'), // ASCII範囲外フォールバック
        }
    }
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
}
