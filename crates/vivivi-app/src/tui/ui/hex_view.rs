//! バイナリモード (Hex Editor) TUI 描画モジュール
//!
//! 16進ダンプ＋ASCII並行表示によるバイト単位の直感表示グリッドを描画します。
//! アドレスオフセット列、16進ダンプ列（16バイト折返し）、ASCII表示列の
//! 3列同期表示と、カーソル位置の双方向ハイライトを提供します。

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use vivivi_modes::hex::{BYTES_PER_ROW, HexPane, calculate_row_count, format_hex_row};

use crate::tui::app::App;

/// バイナリモード (Hex Editor) のメインビューを描画します。
pub fn render_hex_view(frame: &mut Frame, app: &App, area: Rect) {
    let hex_doc = match &app.hex_doc {
        Some(doc) => doc,
        None => return,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" バイナリモード (Hex Editor) [終了: :hex または :text] ");
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let total_rows = calculate_row_count(hex_doc.len());
    let visible_rows = inner_area.height as usize;

    let cursor_offset = app.hex_cursor.offset;
    let cursor_row = cursor_offset / BYTES_PER_ROW;
    let cursor_col_byte = cursor_offset % BYTES_PER_ROW;

    // スクロール調整（カーソルが行内に収まるようにスクロール追従）
    let scroll_row = if cursor_row < app.hex_scroll {
        cursor_row
    } else if cursor_row >= app.hex_scroll + visible_rows {
        cursor_row.saturating_sub(visible_rows - 1)
    } else {
        app.hex_scroll
    };

    let mut lines = Vec::with_capacity(visible_rows);

    for r in 0..visible_rows {
        let row_idx = scroll_row + r;
        if row_idx >= total_rows {
            lines.push(Line::from("~"));
            continue;
        }

        let row_data = match format_hex_row(hex_doc.data(), row_idx) {
            Some(row) => row,
            None => {
                lines.push(Line::from("~"));
                continue;
            }
        };

        let is_cursor_row = row_idx == cursor_row;
        let mut spans = Vec::new();

        // 1. アドレス列 (8桁HEX + ': ')
        let addr_style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);
        spans.push(Span::styled(
            format!("{:08x}:  ", row_data.offset),
            addr_style,
        ));

        // 2. 16進ダンプ列
        for i in 0..BYTES_PER_ROW {
            if i == 8 {
                spans.push(Span::raw(" "));
            }

            if i < row_data.byte_count {
                let byte_offset = row_data.offset + i;
                let b = hex_doc.data()[byte_offset];
                let is_cursor = is_cursor_row && i == cursor_col_byte;

                let style = if is_cursor {
                    if app.hex_cursor.pane == HexPane::Hex {
                        Style::default()
                            .bg(Color::Yellow)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                            .bg(Color::DarkGray)
                            .fg(Color::White)
                            .add_modifier(Modifier::UNDERLINED)
                    }
                } else if b == 0x00 {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                };

                spans.push(Span::styled(format!("{:02x}", b), style));
            } else {
                spans.push(Span::styled("  ", Style::default().fg(Color::DarkGray)));
            }

            if i < BYTES_PER_ROW - 1 && i != 7 {
                spans.push(Span::raw(" "));
            }
        }

        // 区切り
        spans.push(Span::styled("  │ ", Style::default().fg(Color::DarkGray)));

        // 3. ASCII列
        for i in 0..row_data.byte_count {
            let byte_offset = row_data.offset + i;
            let b = hex_doc.data()[byte_offset];
            let is_cursor = is_cursor_row && i == cursor_col_byte;

            let ch = if (0x20..=0x7E).contains(&b) {
                b as char
            } else {
                '.'
            };

            let style = if is_cursor {
                if app.hex_cursor.pane == HexPane::Ascii {
                    Style::default()
                        .bg(Color::Yellow)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .bg(Color::DarkGray)
                        .fg(Color::White)
                        .add_modifier(Modifier::UNDERLINED)
                }
            } else if (0x20..=0x7E).contains(&b) {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            spans.push(Span::styled(ch.to_string(), style));
        }

        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner_area);
}
