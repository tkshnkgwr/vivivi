//! 最近使ったファイル (Recent Files) & お気に入り (Favorites) モーダル描画モジュール。
//!
//! 中央ポップアップダイアログにお気に入りファイル（★マーク付き）および
//! 最近開いたファイルの履歴を一覧表示し、Enterで即座に切り替え可能にします。

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem},
};

use crate::tui::app::App;

/// 最近開いたファイル（MRU）およびお気に入りファイルの選択ダイアログを描画します。
///
/// お気に入りファイルには★マークを付与して先頭に配置し、最近使ったファイルを一覧表示します。
///
/// # 引数
/// - `frame`: Ratatui の描画フレーム
/// - `app`: アプリケーション状態への参照
/// - `area`: 親画面全体の描画領域
pub fn render_recent_modal(frame: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(70, 70, area);

    // 背景をクリアして透過を防ぐ
    frame.render_widget(Clear, popup_area);

    let mut all_entries = Vec::new();

    // 1. お気に入りファイル
    for fav in &app.recent_manager.favorite_files {
        all_entries.push((fav.clone(), true));
    }

    // 2. 最近使ったファイル（お気に入りに入っていないもの）
    for rec in &app.recent_manager.recent_files {
        if !app.recent_manager.favorite_files.contains(rec) {
            all_entries.push((rec.clone(), false));
        }
    }

    let total_count = all_entries.len();
    let selected_idx = app.selected_recent_idx.min(total_count.saturating_sub(1));

    let items: Vec<ListItem> = if all_entries.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "  履歴・お気に入りファイルはありません",
            Style::default().fg(Color::DarkGray),
        )))]
    } else {
        all_entries
            .iter()
            .enumerate()
            .map(|(i, (path, is_fav))| {
                let is_selected = i == selected_idx;
                let sel_marker = if is_selected { "▶ " } else { "  " };

                let fav_marker = if *is_fav { "★ " } else { "  " };
                let fav_style = if *is_fav {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                let path_str = path.display().to_string();

                let path_style = if is_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .bg(Color::Rgb(40, 45, 55))
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(sel_marker, Style::default().fg(Color::Yellow)),
                    Span::styled(fav_marker, fav_style),
                    Span::styled(path_str, path_style),
                ]))
            })
            .collect()
    };

    let title = format!(
        " 最近使ったファイル & お気に入り - 全 {} 件 [Enter:開く f:お気に入り d:削除 Esc:閉じる] ",
        total_count
    );

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Green)),
    );

    frame.render_widget(list, popup_area);
}

/// 画面中央に指定パーセンテージの矩形領域を計算
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
