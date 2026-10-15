//! アウトライン（シンボル階層ツリー）サイドペイン描画モジュール。

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::tui::app::App;
use vivivi_modes::OutlineKind;

/// 画面左側に表示されるアウトライン（関数・見出し・構造体階層一覧）サイドペインを描画します。
///
/// フォーカス状態に応じて枠線の色を強調表示し、選択中の項目をハイライトします。
///
/// # 引数
/// - `frame`: Ratatui の描画フレーム
/// - `app`: アプリケーション状態への参照
/// - `area`: 描画領域の矩形
pub fn render_outline(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Outline (F2/F8 | Tab/Enter) ")
        .border_style(if app.outline_focus {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    let mut lines = Vec::new();
    for (i, item) in app.outline_items.iter().enumerate() {
        let is_selected = i == app.selected_outline_idx;
        let indent = "  ".repeat(item.level.min(4));
        let prefix = match item.kind {
            OutlineKind::Heading(lvl) => format!("H{lvl} "),
            OutlineKind::Function => "fn ".to_string(),
            OutlineKind::Struct => "st ".to_string(),
            OutlineKind::Enum => "en ".to_string(),
            OutlineKind::Class => "cl ".to_string(),
            OutlineKind::Impl => "im ".to_string(),
            OutlineKind::Trait => "tr ".to_string(),
            OutlineKind::Module => "md ".to_string(),
            OutlineKind::Section => "# ".to_string(),
        };

        let style = if is_selected {
            Style::default()
                .bg(Color::Rgb(45, 65, 105))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            match item.kind {
                OutlineKind::Heading(_) => Style::default().fg(Color::Cyan),
                OutlineKind::Function => Style::default().fg(Color::Green),
                OutlineKind::Struct | OutlineKind::Class => Style::default().fg(Color::Yellow),
                _ => Style::default().fg(Color::Gray),
            }
        };

        let content = format!("{}{}{}", indent, prefix, item.title);
        lines.push(Line::from(Span::styled(content, style)));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            " (シンボルなし)",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
