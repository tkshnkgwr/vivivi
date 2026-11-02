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
                .bg(app.active_theme.modes.normal.to_ratatui())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(app.active_theme.ui.fg_dimmed.to_ratatui())
                .bg(app.active_theme.ui.bg_statusline.to_ratatui())
        };

        spans.push(Span::styled(label, style));
        spans.push(Span::styled(
            " ",
            Style::default().bg(app.active_theme.ui.bg_panel.to_ratatui()),
        ));
    }

    let tab_line = Line::from(spans);
    let bar_widget = Paragraph::new(tab_line)
        .style(Style::default().bg(app.active_theme.ui.bg_panel.to_ratatui()));
    frame.render_widget(bar_widget, area);
}
