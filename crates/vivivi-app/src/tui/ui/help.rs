//! ヘルプビューアの描画モジュール。
//!
//! Markdown形式のヘルプドキュメント（`doc/help/*.md`）を読み取り専用で
//! シンタックス色付け・スクロール対応でモーダル表示します。

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::tui::app::App;

/// ヘルプ画面を描画します。
pub fn render_help_view(frame: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(85, 85, area);

    // 背景をクリア
    frame.render_widget(Clear, popup_area);

    let inner_height = (popup_area.height as usize).saturating_sub(2); // 上下枠線分
    let total_lines = app.help_lines.len();
    let scroll_offset = app.help_scroll.min(total_lines.saturating_sub(1));

    let visible_lines: Vec<Line> = app
        .help_lines
        .iter()
        .skip(scroll_offset)
        .take(inner_height)
        .map(|line| format_markdown_line(line))
        .collect();

    let scroll_pct = ((scroll_offset + 1) * 100)
        .checked_div(total_lines)
        .unwrap_or(100);

    let title = format!(
        " {} ({}% - {}/{}行) [j/k:スクロール, d/u:半画面, q/Esc:閉じる] ",
        app.help_title,
        scroll_pct,
        scroll_offset + 1,
        total_lines
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    let paragraph = Paragraph::new(visible_lines).block(block);
    frame.render_widget(paragraph, popup_area);
}

/// Markdown行を簡易的にパースして Ratatui の Line に変換します。
fn format_markdown_line(line: &str) -> Line<'static> {
    if line.starts_with("# ") {
        Line::from(Span::styled(
            line.to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ))
    } else if line.starts_with("## ") {
        Line::from(Span::styled(
            line.to_string(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
    } else if line.starts_with("### ") {
        Line::from(Span::styled(
            line.to_string(),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ))
    } else if line.starts_with("```") {
        Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::DarkGray),
        ))
    } else if line.starts_with('|') {
        Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::LightBlue),
        ))
    } else if line.trim_start().starts_with('-') || line.trim_start().starts_with('*') {
        Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::White),
        ))
    } else {
        Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::Gray),
        ))
    }
}

/// 指定パーセント幅・高さの中央矩形領域を計算します。
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
