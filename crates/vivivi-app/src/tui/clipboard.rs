//! システムクリップボード（OS Clipboard）連携モジュール。
//!
//! `arboard` を使用して、OS のクリップボード（`+` / `*` レジスタ）との
//! テキスト送受信を提供します。

use std::sync::{Arc, Mutex};
use vivivi_vi::ClipboardProvider;

/// OS のクリップボードを操作するプロバイダ。
pub struct SystemClipboard {
    clipboard: Option<arboard::Clipboard>,
}

impl std::fmt::Debug for SystemClipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemClipboard")
            .field("is_initialized", &self.clipboard.is_some())
            .finish()
    }
}

impl SystemClipboard {
    /// 新しい [`SystemClipboard`] を初期化します。
    pub fn new() -> Self {
        let clipboard = arboard::Clipboard::new().ok();
        Self { clipboard }
    }

    /// `Arc<Mutex<Self>>` でラップした共有インスタンスを生成します。
    pub fn new_shared() -> Arc<Mutex<dyn ClipboardProvider>> {
        Arc::new(Mutex::new(Self::new()))
    }
}

impl Default for SystemClipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardProvider for SystemClipboard {
    fn get_text(&mut self) -> Result<String, String> {
        if self.clipboard.is_none() {
            self.clipboard = arboard::Clipboard::new().ok();
        }
        if let Some(ref mut cb) = self.clipboard {
            cb.get_text().map_err(|e| e.to_string())
        } else {
            Err("クリップボードが初期化できませんでした".to_string())
        }
    }

    fn set_text(&mut self, text: String) -> Result<(), String> {
        if self.clipboard.is_none() {
            self.clipboard = arboard::Clipboard::new().ok();
        }
        if let Some(ref mut cb) = self.clipboard {
            cb.set_text(text).map_err(|e| e.to_string())
        } else {
            Err("クリップボードが初期化できませんでした".to_string())
        }
    }
}
