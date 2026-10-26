//! TUI 画面描画モジュール。
//!
//! 各領域（エディタ、アウトライン、ステータスバー）のレイアウト計算と描画ディスパッチを担当します。

pub mod clipboard_hist;
pub mod editor;
pub mod filer;
pub mod help;
pub mod outline;
pub mod preview;
pub mod recent;
pub mod status;
pub mod tab_bar;

use unicode_width::UnicodeWidthStr;

/// ポップアップモーダルを描画する領域を背景から安全にクリアします。
///
/// 単純に `Clear` を適用するだけでは、モーダルの左境界（`area.x - 1`）に全角文字
/// （表示幅2の日本語文字等）が存在する場合、その文字の右半分がモーダルの枠線に
/// はみ出して重なり、文字化けや表示乱れを起こします（Windows Terminal / ConPTYの仕様）。
/// 本関数は、モーダル領域のクリアに加えて境界線と衝突する全角文字を安全にブランク化します。
pub fn clear_modal_area(frame: &mut Frame, area: Rect) {
    // 1. 通常の領域クリア
    frame.render_widget(ratatui::widgets::Clear, area);

    // 2. 左境界に跨がる全角文字の除去
    let buf = frame.buffer_mut();
    if area.x > 0 {
        let left_x = area.x - 1;
        for y in area.top()..area.bottom() {
            if left_x < buf.area.width && y < buf.area.height {
                let cell = &buf[(left_x, y)];
                if cell.symbol().width() > 1 {
                    buf[(left_x, y)].set_symbol(" ");
                }
            }
        }
    }

    // 3. 右境界に跨がる全角文字の除去（右枠線の直後セルが全角後続の場合等）
    let right_x = area.x + area.width;
    if right_x < buf.area.width {
        for y in area.top()..area.bottom() {
            if y < buf.area.height {
                let cell = &buf[(right_x, y)];
                if cell.symbol().width() > 1 {
                    buf[(right_x, y)].set_symbol(" ");
                }
            }
        }
    }
}

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position, Rect},
    widgets::{Block, Borders},
};

use super::app::App;
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

    // 複数タブが開かれている場合は最上部にタブバーを配置
    let (tab_bar_area, main_area) = if app.tabs.len() > 1 {
        let v = Layout::vertical([
            Constraint::Length(1), // タブバー
            Constraint::Min(1),    // エディタ領域
        ])
        .split(chunks[0]);
        (Some(v[0]), v[1])
    } else {
        (None, chunks[0])
    };

    let status_area = chunks[1];

    if let Some(tb_area) = tab_bar_area {
        tab_bar::render_tab_bar(frame, app, tb_area);
    }

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

    // 画面分割 (Split / Vsplit) の階層的レイアウト計算 (Window Layout Tree)
    let window_count = app.windows.len();
    let window_rects = if window_count <= 1 {
        vec![(0, editor_container)]
    } else {
        app.window_tree.compute_rects(editor_container)
    };
    app.window_rects = window_rects
        .iter()
        .map(|&(idx, r)| (idx, crate::types::LogicalRect::from(r)))
        .collect();

    let active_idx = app.active_window;
    let active_editor_area = window_rects
        .iter()
        .find(|(i, _)| *i == active_idx)
        .map(|(_, r)| *r)
        .unwrap_or(editor_container);

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
    for &(win_idx, area) in &window_rects {
        let is_active = win_idx == active_idx;
        editor::render_window(frame, app, area, win_idx, is_active);
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

    // 8. 2ペインファイラーの描画（表示中のみ最前面に重ねて描画）
    if app.show_filer {
        filer::render_filer(frame, app, size);
    }

    // 9. カーソルの画面配置

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

    // カレント行のカーソル表示カラム位置（全角2, 半角1, タブ幅）を計算
    let cur_visual_col = app.visual_col_at(cur.line, cur.col);

    if app.wrap {
        let content_width = (editor_area.width as usize)
            .saturating_sub(line_number_width + 1)
            .max(1);
        let wrapped_row = cur_visual_col / content_width;
        let col_in_wrapped_row = cur_visual_col % content_width;
        let target_screen_row = screen_row + wrapped_row;

        if target_screen_row < editor_area.height as usize {
            let cursor_x = editor_area.x as usize + line_number_width + 1 + col_in_wrapped_row;
            let cursor_y = editor_area.y as usize + target_screen_row;

            if cursor_x < (editor_area.x + editor_area.width) as usize {
                frame.set_cursor_position(Position::new(cursor_x as u16, cursor_y as u16));
            }
        }
    } else {
        let scroll_x = app.scroll.1;
        if screen_row < editor_area.height as usize && cur_visual_col >= scroll_x {
            let relative_col = cur_visual_col - scroll_x;
            let cursor_x = editor_area.x as usize + line_number_width + 1 + relative_col;
            let cursor_y = editor_area.y as usize + screen_row;

            if cursor_x < (editor_area.x + editor_area.width) as usize {
                frame.set_cursor_position(Position::new(cursor_x as u16, cursor_y as u16));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_clear_modal_area_wide_char_boundary() {
        let backend = TestBackend::new(40, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                // (5, 5) に全角文字 "編" を配置（5と6を占有）
                f.buffer_mut()
                    .set_string(5, 5, "編", ratatui::style::Style::default());
                assert_eq!(f.buffer_mut()[(5, 5)].symbol(), "編");

                // x=6 を左端とするモーダル領域をクリア
                let popup = Rect::new(6, 2, 20, 10);
                clear_modal_area(f, popup);

                // 左境界 (5, 5) の全角文字が " " にクリアされ、枠線との重なりが防止されていること
                assert_eq!(f.buffer_mut()[(5, 5)].symbol(), " ");
                // モーダル内部 (6, 5) も空白になっていること
                assert_eq!(f.buffer_mut()[(6, 5)].symbol(), " ");
            })
            .unwrap();
    }
}
