//! フロントエンド非依存の論理型・共通データ構造モジュール。
//!
//! TUI（ratatui / crossterm）および GUI（egui / eframe）の双方から利用可能な
//! 論理矩形（[`LogicalRect`]）および論理キーイベント（[`AppKeyEvent`]）を提供します。

pub use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// 画面・領域の論理矩形（座標とサイズ）。
///
/// TUI（ratatui）および GUI（egui）の双方で利用可能なフロントエンド非依存の矩形表現です。
///
/// # Examples
/// ```
/// use vivivi_app::LogicalRect;
///
/// let rect = LogicalRect::new(0, 0, 80, 24);
/// assert_eq!(rect.width, 80);
/// assert_eq!(rect.height, 24);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LogicalRect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl LogicalRect {
    /// 新しい [`LogicalRect`] を作成します。
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

impl From<ratatui::layout::Rect> for LogicalRect {
    fn from(r: ratatui::layout::Rect) -> Self {
        Self {
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
        }
    }
}

impl From<LogicalRect> for ratatui::layout::Rect {
    fn from(r: LogicalRect) -> Self {
        Self {
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
        }
    }
}

/// フロントエンド非依存の論理キーイベント表現。
///
/// GUI（egui）やTUIからのキー入力・テキスト入力を統一的に [`App`] のイベントハンドラへ渡すための構造体です。
///
/// # Examples
/// ```
/// use vivivi_app::{AppKeyEvent, KeyCode, KeyModifiers};
///
/// let ev = AppKeyEvent::char('a');
/// assert_eq!(ev.code, KeyCode::Char('a'));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppKeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl AppKeyEvent {
    /// 指定したキーコードと修飾キーから [`AppKeyEvent`] を作成します。
    pub const fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    /// 修飾キーなしの単一文字キーイベントを作成します。
    pub const fn char(c: char) -> Self {
        Self {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
        }
    }

    /// [`KeyEvent`] に変換します。
    pub fn into_key_event(self) -> KeyEvent {
        KeyEvent::new(self.code, self.modifiers)
    }
}

impl From<KeyEvent> for AppKeyEvent {
    fn from(k: KeyEvent) -> Self {
        Self {
            code: k.code,
            modifiers: k.modifiers,
        }
    }
}

impl From<AppKeyEvent> for KeyEvent {
    fn from(k: AppKeyEvent) -> Self {
        KeyEvent::new(k.code, k.modifiers)
    }
}
