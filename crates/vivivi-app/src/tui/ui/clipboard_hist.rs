//! クリップボード履歴ダイアログ (cbHist) の描画モジュール。
//!
//! ViVi 3 / ViVi 4 の `cbHistDlg` に準拠し、
//! 中央ポップアップウィンドウに過去のコピー・ヤンク履歴を一覧表示します。

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use super::clear_modal_area;
use crate::tui::app::App;

/// 過去のコピー・ヤンク履歴（[`vivivi_vi::ClipboardHistory`]）を中央ポップアップダイアログとして描画します。
///
/// 背景をクリアした上で枠線・タイトルを描画し、カーソル選択中の履歴アイテムをハイライト表示します。
///
/// # 引数
/// - `frame`: Ratatui の描画フレーム
/// - `app`: アプリケーション状態への参照
/// - `area`: 親画面全体の描画領域
pub fn render_clipboard_history(frame: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(60, 60, area);

    // 背景を安全にクリア（全角文字の境界重なり防止）
    clear_modal_area(frame, popup_area);

    let history = app.vi.registers().history();
    let items: Vec<ListItem> = history
        .items()
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = i == app.selected_cb_idx;
            let prefix = if is_selected { "▶ " } else { "  " };

            let content_span = format!("#{:<2} [{}行] {}", item.id, item.line_count, item.preview);

            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Cyan)),
                Span::styled(content_span, style),
            ]))
        })
        .collect();

    let title = format!(" クリップボード履歴 (cbHist) - 全 {} 件 ", history.len());
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(list, popup_area);
}

/// 画面中央に指定パーセンテージの矩形領域を計算します。
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
