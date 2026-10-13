//! ステータスバーおよびコマンドライン描画モジュール。

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::tui::app::App;
use vivivi_vi::ViMode;

/// 最下行（ステータスバーまたはコマンドライン）の描画
pub fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    if app.mode() == ViMode::CommandLine {
        // コマンドライン入力中の表示 (`:w` 等)
        let cmd_str = app
            .vi
            .parser()
            .command_line_buffer()
            .map(|(prefix, buf)| format!("{prefix}{buf}"))
            .unwrap_or_else(|| ":".to_string());
        let cmd_line = Line::from(Span::styled(
            cmd_str,
            Style::default().fg(Color::White).bg(Color::Reset),
        ));
        let paragraph = Paragraph::new(cmd_line);
        frame.render_widget(paragraph, area);
        return;
    }

    // ステータスバー表示
    let mode_str = match app.mode() {
        ViMode::Normal => "-- NORMAL --",
        ViMode::Insert => "-- INSERT --",
        ViMode::Replace => "-- REPLACE --",
        ViMode::Visual => "-- VISUAL --",
        ViMode::VisualLine => "-- VISUAL LINE --",
        ViMode::VisualBlock => "-- VISUAL BLOCK --",
        ViMode::CommandLine => "",
    };

    let keisen_badge = if app.keisen_mode.is_active {
        " [KEISEN]"
    } else {
        ""
    };

    let csv_badge = if app.is_csv_active { " [CSV]" } else { "" };

    let modified_badge = if app.buffer.is_modified() { " [+]" } else { "" };

    let file_name = app
        .file_path
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("[No Name]");

    let cur = app.cursor_pos();
    let total_lines = app.buffer.line_count();
    let percent = ((cur.line + 1) * 100)
        .checked_div(total_lines)
        .unwrap_or(100);

    let lang_str = match app.syntax_lang {
        vivivi_modes::SyntaxLang::PlainText => "Text",
        vivivi_modes::SyntaxLang::Rust => "Rust",
        vivivi_modes::SyntaxLang::Cpp => "C/C++",
        vivivi_modes::SyntaxLang::Python => "Python",
        vivivi_modes::SyntaxLang::JavaScript => "JS/TS",
        vivivi_modes::SyntaxLang::Json => "JSON",
        vivivi_modes::SyntaxLang::Markdown => "Markdown",
    };

    let enc_info = format!(
        "[{}/{}/{}]",
        app.encoding.name(),
        app.newline.name(),
        lang_str
    );

    let left_info = format!("{mode_str}{keisen_badge}{csv_badge} {file_name}{modified_badge}");
    let right_info = format!("{enc_info} {}:{} | {percent}% ", cur.line + 1, cur.col + 1);

    // 一時通知メッセージがある場合は上書き表示
    let display_left = if let Some(msg) = &app.status_message {
        msg.clone()
    } else {
        left_info
    };

    let width = area.width as usize;
    let spaces_needed = width
        .saturating_sub(display_left.chars().count())
        .saturating_sub(right_info.chars().count());
    let status_line_content = format!(
        "{}{}{}",
        display_left,
        " ".repeat(spaces_needed),
        right_info
    );

    let status_style = Style::default()
        .bg(Color::Rgb(40, 44, 52))
        .fg(Color::Rgb(220, 223, 228))
        .add_modifier(Modifier::BOLD);

    let paragraph = Paragraph::new(Line::from(Span::styled(status_line_content, status_style)));
    frame.render_widget(paragraph, area);
}
