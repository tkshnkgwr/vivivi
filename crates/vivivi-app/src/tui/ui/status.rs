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

/// 画面最下行（ステータスライン、コマンドライン入力行、またはMigemo検索行）を描画します。
///
/// コマンドラインモード時は `:` プロンプトと入力中コマンド、Migemo検索時は `g/` プロンプトを表示します。
/// 通常時は現在のViモード名（色分け表示）、マクロ記録中アイコン、ファイル名、エンコーディング、改行コード、
/// シンタックス言語、カーソル座標、および進行率（%）を表示します。
///
/// # 引数
/// - `frame`: Ratatui の描画フレーム
/// - `app`: アプリケーション状態への参照
/// - `area`: 描画領域の矩形（高さ1行）
pub fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    if app.is_migemo_search {
        // Migemo ローマ字検索入力中の表示 (`g/nihon` 等)
        let cmd_line = Line::from(vec![
            Span::styled(
                "g/",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&app.migemo_query, Style::default().fg(Color::White)),
        ]);
        let paragraph = Paragraph::new(cmd_line);
        frame.render_widget(paragraph, area);
        return;
    }

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
        vivivi_modes::SyntaxLang::Sql => "SQL",
        vivivi_modes::SyntaxLang::PlSql => "PL/SQL",
        vivivi_modes::SyntaxLang::Java => "Java",
        vivivi_modes::SyntaxLang::Html => "HTML",
        vivivi_modes::SyntaxLang::Bash => "Bash",
    };

    let enc_info = format!(
        "[{}/{}/{}]",
        app.encoding.name(),
        app.newline.name(),
        lang_str
    );

    let left_info = format!("{mode_str}{keisen_badge}{csv_badge} {file_name}{modified_badge}");
    let right_info = format!("{enc_info} {}:{} | {percent}% ", cur.line + 1, cur.col + 1);

    // 一時通知メッセージがある場合は上書き表示（エラー時は赤背景で警告強調）
    let (display_left, is_error) = if let Some(msg) = &app.status_message {
        let is_err = msg.contains("未保存")
            || msg.contains("エラー")
            || msg.contains("見つかりません")
            || msg.starts_with('E');
        (msg.clone(), is_err)
    } else {
        (left_info, false)
    };

    let width = area.width as usize;
    let left_w = unicode_width::UnicodeWidthStr::width(display_left.as_str());
    let right_w = unicode_width::UnicodeWidthStr::width(right_info.as_str());
    let spaces_needed = width.saturating_sub(left_w).saturating_sub(right_w);
    let status_line_content = format!(
        "{}{}{}",
        display_left,
        " ".repeat(spaces_needed),
        right_info
    );

    let status_style = if is_error {
        Style::default()
            .bg(Color::Rgb(180, 40, 40))
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .bg(Color::Rgb(40, 44, 52))
            .fg(Color::Rgb(220, 223, 228))
            .add_modifier(Modifier::BOLD)
    };

    let paragraph = Paragraph::new(Line::from(Span::styled(status_line_content, status_style)));
    frame.render_widget(paragraph, area);
}
