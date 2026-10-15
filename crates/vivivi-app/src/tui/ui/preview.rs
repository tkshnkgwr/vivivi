//! 印刷プレビューの描画モジュール。
//!
//! ページ分割されたテキスト・Markdown整形プレビューを画面中央のモーダルで表示します。

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::tui::app::App;

/// 印刷プレビュー（ヘッダー・フッター・行番号・ページ区切り付き）画面をモーダルビューとして描画します。
///
/// # 引数
/// - `frame`: Ratatui の描画フレーム
/// - `app`: アプリケーション状態への参照
/// - `area`: 親画面全体の描画領域
pub fn render_preview_view(frame: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(90, 90, area);

    // 背景をクリア
    frame.render_widget(Clear, popup_area);

    let inner_height = (popup_area.height as usize).saturating_sub(2);
    let total_lines = app.preview_lines.len();
    let scroll_offset = app.preview_scroll.min(total_lines.saturating_sub(1));

    let visible_lines: Vec<Line> = app
        .preview_lines
        .iter()
        .skip(scroll_offset)
        .take(inner_height)
        .map(|line| Line::from(line.as_str()))
        .collect();

    let title = format!(
        " 印刷プレビュー ({}行中 {}行〜) [j/k:スクロール, d/u:半画面, q/Esc:閉じる] ",
        total_lines,
        scroll_offset + 1
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    let paragraph = Paragraph::new(visible_lines).block(block);
    frame.render_widget(paragraph, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
