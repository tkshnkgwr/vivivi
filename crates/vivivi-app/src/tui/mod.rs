pub mod app;
pub mod event;
pub mod ui;
pub mod window;

pub use app::App;

use std::io::stdout;
use std::time::Duration;

use crossterm::{
    event::{self as ct_event, Event as CtEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

/// ターミナル UI（TUI）アプリケーションを実行するメインエントリポイント。
///
/// ターミナルを Raw モードおよび代替スクリーンバッファへ切り替え、
/// 50ms の周期でイベントポーリングと画面再描画（Ratatui）を行うメインループを開始します。
/// アプリケーション終了時、または万一のパニック発生時でも、確実にターミナル設定
/// （カーソル形状、Raw モード無効化、スクリーン復帰）を元の状態にリストアします。
///
/// # 引数
/// - `app`: 初期化された [`App`] インスタンス
///
/// # Errors
/// ターミナルの Raw モード有効化、代替スクリーン切り替え、描画バックエンドの初期化、
/// またはイベントポーリング中に I/O エラー等が発生した場合に [`anyhow::Error`] を返します。
///
/// # Examples
/// ```rust,no_run
/// use vivivi_app::tui::{run, App};
///
/// let app = App::new("Hello, world!", None, Default::default(), Default::default());
/// // run(app).unwrap(); // ターミナル対話セッションを開始
/// ```
pub fn run(mut app: App) -> anyhow::Result<()> {
    // 1. パニック時でも確実にターミナルを復元するフックを設定
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(
            stdout(),
            LeaveAlternateScreen,
            crossterm::cursor::SetCursorStyle::DefaultUserShape
        );
        original_hook(panic_info);
    }));

    // 2. ターミナルの初期化
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 3. メインイベントループ
    let tick_rate = Duration::from_millis(50);
    let mut last_cursor_mode = None;

    while !app.should_quit {
        // モード連動のカーソル形状変更 (Normal/Visual: Block, Insert: Bar, Replace: Underline)
        let current_mode = app.mode();
        if last_cursor_mode != Some(current_mode) {
            let cursor_style = match current_mode {
                vivivi_vi::ViMode::Insert => crossterm::cursor::SetCursorStyle::SteadyBar,
                vivivi_vi::ViMode::Replace => crossterm::cursor::SetCursorStyle::SteadyUnderScore,
                vivivi_vi::ViMode::CommandLine => crossterm::cursor::SetCursorStyle::SteadyBar,
                _ => crossterm::cursor::SetCursorStyle::SteadyBlock,
            };
            execute!(terminal.backend_mut(), cursor_style)?;
            last_cursor_mode = Some(current_mode);
        }

        // 画面描画
        terminal.draw(|frame| {
            ui::render(frame, &mut app);
        })?;

        // イベント待機 (50ms タイムアウトでポーリング)
        if ct_event::poll(tick_rate)?
            && let CtEvent::Key(key) = ct_event::read()?
            && key.kind != ct_event::KeyEventKind::Release
        {
            event::handle_key_event(&mut app, key);
        }
    }

    // 4. ターミナルの復元（カーソル形状も標準に戻す）
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::cursor::SetCursorStyle::DefaultUserShape
    )?;
    terminal.show_cursor()?;

    Ok(())
}
