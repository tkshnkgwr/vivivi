pub mod app;
pub mod event;
pub mod ui;

pub use app::App;

use std::io::stdout;
use std::time::Duration;

use crossterm::{
    event::{self as ct_event, Event as CtEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

/// TUI の実行エントリポイント。
///
/// ターミナルを Raw モードおよび代替スクリーンに切り替え、
/// メインイベントループを実行します。終了時には必ず画面を復元します。
pub fn run(mut app: App) -> anyhow::Result<()> {
    // 1. パニック時でも確実にターミナルを復元するフックを設定
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
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
    while !app.should_quit {
        // 画面描画
        terminal.draw(|frame| {
            ui::render(frame, &mut app);
        })?;

        // イベント待機 (50ms タイムアウトでポーリング)
        if ct_event::poll(tick_rate)?
            && let CtEvent::Key(key) = ct_event::read()?
        {
            event::handle_key_event(&mut app, key);
        }
    }

    // 4. ターミナルの復元
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
