//! GUI ウィンドウ状態の自動記憶・復元モジュール。
//!
//! エディタ終了時のウィンドウ座標、サイズ、最大化フラグを `%USERPROFILE%\.vivivi\cache\window_state.json`
//! に永続化し、次回起動時に安全ガード（マルチモニタ画面外脱出防止・はみ出し自動補正）付きで復元します。

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 最小許容ウィンドウ幅 (px)
pub const MIN_WINDOW_WIDTH: f32 = 640.0;
/// 最小許容ウィンドウ高さ (px)
pub const MIN_WINDOW_HEIGHT: f32 = 480.0;
/// デフォルトウィンドウ幅 (px)
pub const DEFAULT_WINDOW_WIDTH: f32 = 1024.0;
/// デフォルトウィンドウ高さ (px)
pub const DEFAULT_WINDOW_HEIGHT: f32 = 768.0;

fn default_x() -> f32 {
    100.0
}
fn default_y() -> f32 {
    100.0
}
fn default_width() -> f32 {
    DEFAULT_WINDOW_WIDTH
}
fn default_height() -> f32 {
    DEFAULT_WINDOW_HEIGHT
}

/// ウィンドウ表示状態データ。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowState {
    /// デスクトップ上の表示開始 X 座標 (px)
    #[serde(default = "default_x")]
    pub x: f32,
    /// デスクトップ上の表示開始 Y 座標 (px)
    #[serde(default = "default_y")]
    pub y: f32,
    /// 通常表示時のウィンドウ幅 (px, 最小: 640.0)
    #[serde(default = "default_width")]
    pub width: f32,
    /// 通常表示時のウィンドウ高さ (px, 最小: 480.0)
    #[serde(default = "default_height")]
    pub height: f32,
    /// 前回終了時にウィンドウが最大化されていたか
    #[serde(default)]
    pub is_maximized: bool,
    /// 座標指定が明示的に設定されているか（画面中央配置フラグ用、非永続化）
    #[serde(skip)]
    pub custom_position: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            x: default_x(),
            y: default_y(),
            width: DEFAULT_WINDOW_WIDTH,
            height: DEFAULT_WINDOW_HEIGHT,
            is_maximized: false,
            custom_position: false,
        }
    }
}

/// 矩形領域（テスト・安全ガード計算用）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectBounds {
    pub min_x: f32,
    pub min_y: f32,
    pub width: f32,
    pub height: f32,
}

impl RectBounds {
    pub fn new(min_x: f32, min_y: f32, width: f32, height: f32) -> Self {
        Self {
            min_x,
            min_y,
            width,
            height,
        }
    }

    pub fn max_x(&self) -> f32 {
        self.min_x + self.width
    }

    pub fn max_y(&self) -> f32 {
        self.min_y + self.height
    }

    pub fn area(&self) -> f32 {
        if self.width > 0.0 && self.height > 0.0 {
            self.width * self.height
        } else {
            0.0
        }
    }

    /// 2つの矩形の交差（重複）領域を算出します。
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let left = self.min_x.max(other.min_x);
        let top = self.min_y.max(other.min_y);
        let right = self.max_x().min(other.max_x());
        let bottom = self.max_y().min(other.max_y());

        if right > left && bottom > top {
            Some(Self {
                min_x: left,
                min_y: top,
                width: right - left,
                height: bottom - top,
            })
        } else {
            None
        }
    }
}

impl WindowState {
    /// 新規のウィンドウ状態を座標付きで作成します。
    pub fn with_pos(x: f32, y: f32, width: f32, height: f32, is_maximized: bool) -> Self {
        let mut state = Self {
            x,
            y,
            width: width.max(MIN_WINDOW_WIDTH),
            height: height.max(MIN_WINDOW_HEIGHT),
            is_maximized,
            custom_position: true,
        };
        state.sanitize_size();
        state
    }

    /// サイズの最小・最大健全性を保証します。
    pub fn sanitize_size(&mut self) {
        if self.width < MIN_WINDOW_WIDTH || !self.width.is_finite() {
            self.width = DEFAULT_WINDOW_WIDTH;
        }
        if self.height < MIN_WINDOW_HEIGHT || !self.height.is_finite() {
            self.height = DEFAULT_WINDOW_HEIGHT;
        }
    }

    /// 指定されたデスクトップ（仮想画面またはモニタ）領域に対して安全ガードを判定し、
    /// 完全に画面外、または 80% 以上はみ出している場合にデフォルト中央配置へフォールバックします。
    pub fn apply_safety_guard(&mut self, screen: RectBounds) {
        self.sanitize_size();

        let win_bounds = RectBounds::new(self.x, self.y, self.width, self.height);
        let win_area = win_bounds.area();

        if win_area <= 0.0 {
            self.custom_position = false;
            return;
        }

        match win_bounds.intersection(&screen) {
            Some(inter) => {
                let inter_area = inter.area();
                let overlap_ratio = inter_area / win_area;
                // 画面内に 20% 以上残っている（はみ出しが 80% 未満）場合は有効
                if overlap_ratio >= 0.20 {
                    self.custom_position = true;
                } else {
                    // 80% 以上はみ出しているためフォールバック
                    self.custom_position = false;
                }
            }
            None => {
                // 完全に画面外
                self.custom_position = false;
            }
        }
    }

    /// キャッシュファイルのデフォルト保存先パスを取得します。
    ///
    /// 1. `%USERPROFILE%\.vivivi\cache\window_state.json` (Windows)
    /// 2. `~/.vivivi/cache/window_state.json` (Unix)
    pub fn default_cache_path() -> PathBuf {
        let base_dir = if let Ok(userprofile) = std::env::var("USERPROFILE") {
            PathBuf::from(userprofile).join(".vivivi").join("cache")
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".vivivi").join("cache")
        } else {
            PathBuf::from(".vivivi").join("cache")
        };
        base_dir.join("window_state.json")
    }

    /// 指定パスまたはデフォルトパスから JSON データを読み込みます。
    pub fn load_from_file(path: &Path) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(path)?;
        let mut state: WindowState = serde_json::from_str(&content)?;
        state.custom_position = true;
        state.sanitize_size();
        Ok(state)
    }

    /// キャッシュから状態をロードし、現在のモニタ仮想画面情報に基づき安全ガードを適用した上で返します。
    pub fn load_and_validate() -> Self {
        let path = Self::default_cache_path();
        let mut state = if path.exists() {
            Self::load_from_file(&path).unwrap_or_default()
        } else {
            Self::default()
        };

        let screen = get_current_screen_bounds();
        state.apply_safety_guard(screen);
        state
    }

    /// 現在の状態をキャッシュファイルへ JSON 保存します。
    pub fn save_to_file(&self, path: &Path) -> Result<(), anyhow::Error> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json_str = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json_str)?;
        Ok(())
    }

    /// デフォルトパスへ保存します。
    pub fn save(&self) -> Result<(), anyhow::Error> {
        self.save_to_file(&Self::default_cache_path())
    }
}

/// 現在接続されているモニタ全体の仮想画面領域を取得します。
pub fn get_current_screen_bounds() -> RectBounds {
    #[cfg(windows)]
    {
        get_windows_virtual_screen_bounds()
    }
    #[cfg(not(windows))]
    {
        // 非 Windows 環境での安全なフォールバック
        RectBounds::new(0.0, 0.0, 3840.0, 2160.0)
    }
}

#[cfg(windows)]
fn get_windows_virtual_screen_bounds() -> RectBounds {
    unsafe extern "system" {
        fn GetSystemMetrics(nIndex: i32) -> i32;
    }

    unsafe {
        let left = GetSystemMetrics(76) as f32; // SM_XVIRTUALSCREEN
        let top = GetSystemMetrics(77) as f32; // SM_YVIRTUALSCREEN
        let width = GetSystemMetrics(78) as f32; // SM_CXVIRTUALSCREEN
        let height = GetSystemMetrics(79) as f32; // SM_CYVIRTUALSCREEN

        if width > 0.0 && height > 0.0 {
            RectBounds::new(left, top, width, height)
        } else {
            let cx = GetSystemMetrics(0) as f32; // SM_CXSCREEN
            let cy = GetSystemMetrics(1) as f32; // SM_CYSCREEN
            let w = if cx > 0.0 { cx } else { 1920.0 };
            let h = if cy > 0.0 { cy } else { 1080.0 };
            RectBounds::new(0.0, 0.0, w, h)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_bounds_intersection() {
        let r1 = RectBounds::new(0.0, 0.0, 100.0, 100.0);
        let r2 = RectBounds::new(50.0, 50.0, 100.0, 100.0);
        let inter = r1.intersection(&r2).unwrap();
        assert_eq!(inter.min_x, 50.0);
        assert_eq!(inter.min_y, 50.0);
        assert_eq!(inter.width, 50.0);
        assert_eq!(inter.height, 50.0);
        assert_eq!(inter.area(), 2500.0);
    }

    #[test]
    fn test_safety_guard_fully_inside() {
        let screen = RectBounds::new(0.0, 0.0, 1920.0, 1080.0);
        let mut state = WindowState::with_pos(100.0, 100.0, 1024.0, 768.0, false);
        state.apply_safety_guard(screen);
        assert!(state.custom_position);
        assert_eq!(state.x, 100.0);
        assert_eq!(state.y, 100.0);
    }

    #[test]
    fn test_safety_guard_completely_outside() {
        let screen = RectBounds::new(0.0, 0.0, 1920.0, 1080.0);
        // モニタ切断により 2500px の位置に取り残された場合
        let mut state = WindowState::with_pos(2500.0, 100.0, 1024.0, 768.0, false);
        state.apply_safety_guard(screen);
        assert!(!state.custom_position, "完全画面外はフォールバックすること");
    }

    #[test]
    fn test_safety_guard_over_80_percent_outside() {
        let screen = RectBounds::new(0.0, 0.0, 1000.0, 1000.0);
        // 幅1000のウィンドウで x=900（交差は幅100pxのみ、残存面積10%）
        let mut state = WindowState::with_pos(900.0, 100.0, 1000.0, 500.0, false);
        state.apply_safety_guard(screen);
        assert!(
            !state.custom_position,
            "80%以上はみ出した場合はフォールバックすること"
        );
    }

    #[test]
    fn test_safety_guard_under_80_percent_outside() {
        let screen = RectBounds::new(0.0, 0.0, 1000.0, 1000.0);
        // 幅1000のウィンドウで x=700（交差は幅300px、残存面積30%）
        let mut state = WindowState::with_pos(700.0, 100.0, 1000.0, 500.0, false);
        state.apply_safety_guard(screen);
        assert!(
            state.custom_position,
            "画面内に20%以上残っていれば有効とみなすこと"
        );
    }

    #[test]
    fn test_json_serialization_roundtrip() {
        let temp_dir = std::env::temp_dir().join("vivivi_test_window_state");
        let path = temp_dir.join("window_state.json");

        let original = WindowState::with_pos(150.0, 200.0, 1280.0, 800.0, true);
        original.save_to_file(&path).expect("保存成功");

        let loaded = WindowState::load_from_file(&path).expect("読み込み成功");
        assert_eq!(loaded.x, 150.0);
        assert_eq!(loaded.y, 200.0);
        assert_eq!(loaded.width, 1280.0);
        assert_eq!(loaded.height, 800.0);
        assert!(loaded.is_maximized);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
