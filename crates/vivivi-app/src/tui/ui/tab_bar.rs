//! マルチタブバー（Tab Bar）描画コンポーネント。
//!
//! 複数タブが開かれている場合、画面最上段に各タブのインデックス番号、ファイル名、
//! および未保存マーカー `[+]` を表示します。

use crate::tui::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

/// タブバーを描画します。
pub fn render_tab_bar(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans = Vec::new();

    for (i, tab) in app.tabs.iter().enumerate() {
        let is_active = i == app.active_tab;
        let tab_num = i + 1;
        let title = tab.display_title();
        let label = format!(" {tab_num}: {title} ");

        let style = if is_active {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray).bg(Color::Rgb(40, 44, 52))
        };

        spans.push(Span::styled(label, style));
        spans.push(Span::styled(
            " ",
            Style::default().bg(Color::Rgb(25, 28, 34)),
        ));
    }

    let tab_line = Line::from(spans);
    let bar_widget = Paragraph::new(tab_line).style(Style::default().bg(Color::Rgb(25, 28, 34)));
    frame.render_widget(bar_widget, area);
}
