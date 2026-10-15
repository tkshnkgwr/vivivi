//! TUI 画面描画モジュール。
//!
//! 各領域（エディタ、アウトライン、ステータスバー）のレイアウト計算と描画ディスパッチを担当します。

pub mod clipboard_hist;
pub mod editor;
pub mod help;
pub mod outline;
pub mod preview;
pub mod recent;
pub mod status;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position, Rect},
    widgets::{Block, Borders},
};

use super::app::App;
use super::window::SplitAxis;
use vivivi_vi::ViMode;

/// ターミナル画面全体の描画を実行するメインディスパッチ関数。
///
/// 画面全体をエディタ領域、アウトラインサイドペイン、ステータスライン/コマンドラインに分割し、
/// 複数ウィンドウ分割（水平・垂直）や各種モーダルダイアログ（クリップボード履歴、最近使ったファイル、ヘルプ、印刷プレビュー）を重ねてレンダリングします。
/// また、アクティブウィンドウ内のカーソル座標に基づいてターミナルの物理カーソル位置を設定します。
///
/// # 引数
/// - `frame`: Ratatui の描画フレーム
/// - `app`: アプリケーション状態への可変参照
///
/// # Examples
/// ```rust,ignore
/// // Ratatui の terminal.draw 内から呼び出されます
/// terminal.draw(|frame| {
///     vivivi_app::tui::ui::render(frame, &mut app);
/// })?;
/// ```
pub fn render(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // 画面をエディタ領域（上部）とステータスバー／コマンドライン（最下部）に分割
    let chunks = Layout::vertical([
        Constraint::Min(1),    // エディタ領域
        Constraint::Length(1), // ステータスライン / コマンドライン
    ])
    .split(size);

    let main_area = chunks[0];
    let status_area = chunks[1];

    // アウトライン表示が有効な場合は画面左側にサイドペインを配置
    let (outline_area, editor_container) = if app.show_outline {
        let h_chunks = Layout::horizontal([
            Constraint::Length(30), // アウトラインサイドペイン
            Constraint::Min(1),     // エディタメイン領域
        ])
        .split(main_area);
        (Some(h_chunks[0]), h_chunks[1])
    } else {
        (None, main_area)
    };

    // 画面分割 (Split / Vsplit) のレイアウト計算
    let window_count = app.windows.len();
    let window_areas = if window_count <= 1 {
        vec![editor_container]
    } else {
        let constraints = vec![Constraint::Ratio(1, window_count as u32); window_count];
        match app.split_axis {
            SplitAxis::Horizontal => Layout::vertical(constraints)
                .split(editor_container)
                .to_vec(),
            SplitAxis::Vertical => Layout::horizontal(constraints)
                .split(editor_container)
                .to_vec(),
        }
    };

    let active_idx = app.active_window.min(window_areas.len().saturating_sub(1));
    let active_editor_area = window_areas[active_idx];
    let inner_active_editor_area = if window_count > 1 {
        let block = Block::default().borders(Borders::ALL);
        block.inner(active_editor_area)
    } else {
        active_editor_area
    };

    // アクティブウィンドウに合わせてスクロール調整
    let line_number_width = 6; // " 123 ▌ " の幅
    let text_view_width =
        (inner_active_editor_area.width as usize).saturating_sub(line_number_width);
    let text_view_height = inner_active_editor_area.height as usize;
    app.adjust_scroll(text_view_height, text_view_width);

    // 1. 各エディタウィンドウの描画
    for (i, &area) in window_areas.iter().enumerate() {
        let is_active = i == active_idx;
        editor::render_window(frame, app, area, i, is_active);
    }

    // 2. アウトラインペインの描画
    if let Some(o_area) = outline_area {
        outline::render_outline(frame, app, o_area);
    }

    // 3. 最下行（ステータスバーまたはコマンドライン）の描画
    status::render_status_bar(frame, app, status_area);

    // 4. クリップボード履歴ポップアップの描画（表示中のみ最前面に重ねて描画）
    if app.show_clipboard_hist {
        clipboard_hist::render_clipboard_history(frame, app, size);
    }

    // 5. ヘルプビューアの描画（表示中のみ最前面に重ねて描画）
    if app.show_help {
        help::render_help_view(frame, app, size);
    }

    // 6. 印刷プレビューの描画（表示中のみ最前面に重ねて描画）
    if app.show_preview {
        preview::render_preview_view(frame, app, size);
    }

    // 7. 最近使ったファイル & お気に入りモーダルの描画（表示中のみ最前面に重ねて描画）
    if app.show_recent_modal {
        recent::render_recent_modal(frame, app, size);
    }

    // 8. カーソルの画面配置

    set_terminal_cursor(
        frame,
        app,
        inner_active_editor_area,
        status_area,
        line_number_width,
    );
}

/// ターミナルカーソルの座標設定
fn set_terminal_cursor(
    frame: &mut Frame,
    app: &App,
    editor_area: Rect,
    status_area: Rect,
    line_number_width: usize,
) {
    if app.is_migemo_search {
        // Migemo 検索入力プロンプト (g/<query>)
        let prompt_len = 2 + app.migemo_query.chars().count(); // "g/" + query
        let cursor_x = (status_area.x as usize + prompt_len).min(status_area.width as usize - 1);
        frame.set_cursor_position(Position::new(cursor_x as u16, status_area.y));
        return;
    }

    if app.mode() == ViMode::CommandLine {
        // コマンドラインのカーソル位置（全角文字幅考慮）
        let cmd_buf = app
            .vi
            .parser()
            .command_line_buffer()
            .map(|(_, s)| s)
            .unwrap_or("");
        let mut cmd_width = 1; // ':' の1マス
        for ch in cmd_buf.chars() {
            cmd_width += unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
        }
        let cursor_x = (status_area.x as usize + cmd_width).min(status_area.width as usize - 1);
        frame.set_cursor_position(Position::new(cursor_x as u16, status_area.y));
        return;
    }

    let cur = app.cursor_pos();
    let screen_row = cur.line.saturating_sub(app.scroll.0);

    // カレント行の0〜cur.col文字までの画面表示桁幅（全角2, 半角1, タブ幅）を計算
    let cur_line = app.buffer.get_line(cur.line).unwrap_or_default();
    let scroll_x = app.scroll.1;
    let mut visual_col = 0;

    for (i, ch) in cur_line.chars().enumerate() {
        if i >= cur.col {
            break;
        }
        if i >= scroll_x {
            if ch == '\t' {
                visual_col += 4 - (visual_col % 4);
            } else {
                visual_col += unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
            }
        }
    }

    if screen_row < editor_area.height as usize {
        let cursor_x = editor_area.x as usize + line_number_width + 1 + visual_col;
        let cursor_y = editor_area.y as usize + screen_row;

        if cursor_x < (editor_area.x + editor_area.width) as usize {
            frame.set_cursor_position(Position::new(cursor_x as u16, cursor_y as u16));
        }
    }
}
