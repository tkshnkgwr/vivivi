//! 設定ファイル (`config.toml` / `vivivirc`) の読み込み・永続化モジュール。
//!
//! `%USERPROFILE%\.vivivi\config.toml` や `~/.config/vivivi/config.toml` から
//! エディタの初期設定（タブ幅、行番号、不可視文字、エンコーディング、クリップボード同期等）をロードします。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// エディタ基本設定。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EditorConfig {
    /// タブ幅（半角スペース換算、デフォルト: 4）
    #[serde(default = "default_tab_width")]
    pub tab_width: usize,
    /// 行番号を表示するかどうか（デフォルト: true）
    #[serde(default = "default_true")]
    pub line_number: bool,
    /// 相対行番号を表示するかどうか（デフォルト: false）
    #[serde(default)]
    pub relative_number: bool,
    /// 新規作成時のデフォルト文字エンコーディング（デフォルト: "utf-8"）
    #[serde(default = "default_encoding")]
    pub default_encoding: String,
    /// 新規作成時のデフォルト改行形式（"unix" または "dos"、デフォルト: "unix"）
    #[serde(default = "default_fileformat")]
    pub default_fileformat: String,
    /// クリップボード同期設定（"none", "unnamed", "unnamedplus"、デフォルト: "none"）
    #[serde(default = "default_clipboard")]
    pub clipboard: String,
    /// 検索時大文字小文字無視（デフォルト: true）
    #[serde(default = "default_true")]
    pub ignorecase: bool,
    /// 大文字が含まれる場合は区別（デフォルト: true）
    #[serde(default = "default_true")]
    pub smartcase: bool,
}

fn default_tab_width() -> usize {
    4
}
fn default_true() -> bool {
    true
}
fn default_encoding() -> String {
    "utf-8".to_string()
}
fn default_fileformat() -> String {
    "unix".to_string()
}
fn default_clipboard() -> String {
    "none".to_string()
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_width: 4,
            line_number: true,
            relative_number: false,
            default_encoding: "utf-8".to_string(),
            default_fileformat: "unix".to_string(),
            clipboard: "none".to_string(),
            ignorecase: true,
            smartcase: true,
        }
    }
}

/// 不可視文字表示設定。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InvisibleConfig {
    /// 不可視文字表示を有効にするか（デフォルト: false）
    #[serde(default)]
    pub enabled: bool,
    /// 全角スペース表示記号（デフォルト: "□"）
    #[serde(default = "default_zenkaku")]
    pub zenkaku_space: String,
    /// 半角スペース表示記号（デフォルト: '·'）
    #[serde(default = "default_space_char")]
    pub space: char,
    /// 行末改行記号（デフォルト: '$'）
    #[serde(default = "default_eol_char")]
    pub eol: char,
}

fn default_zenkaku() -> String {
    "□".to_string()
}
fn default_space_char() -> char {
    '·'
}
fn default_eol_char() -> char {
    '$'
}

impl Default for InvisibleConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            zenkaku_space: "□".to_string(),
            space: '·',
            eol: '$',
        }
    }
}

/// ガター・変更行追跡設定。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GutterConfig {
    /// 変更行強調表示を有効にするか（デフォルト: true）
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for GutterConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// キーマップリマップ設定。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeymapConfig {
    /// ノーマルモードでのキー置換マップ
    #[serde(default)]
    pub normal: HashMap<String, String>,
    /// インサートモードでのキー置換マップ
    #[serde(default)]
    pub insert: HashMap<String, String>,
}

/// アプリケーション全体の統合設定。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    /// エディタ基本設定
    #[serde(default)]
    pub editor: EditorConfig,
    /// 不可視文字表示設定
    #[serde(default)]
    pub invisible: InvisibleConfig,
    /// ガター追跡設定
    #[serde(default)]
    pub gutter: GutterConfig,
    /// キーマップ設定
    #[serde(default)]
    pub keymap: KeymapConfig,
}

impl AppConfig {
    /// TOML 文字列から設定を読み込みます。
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    /// 設定を TOML 文字列へシリアライズします。
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// デフォルトの設定ファイル配置パスを取得します。
    ///
    /// 1. `%USERPROFILE%\.vivivi\config.toml` (Windows) または `~/.vivivi/config.toml` (Unix)
    /// 2. `~/.config/vivivi/config.toml` (XDG)
    pub fn default_config_path() -> PathBuf {
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            let p = PathBuf::from(userprofile)
                .join(".vivivi")
                .join("config.toml");
            if p.exists() {
                return p;
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            let p1 = PathBuf::from(&home).join(".vivivi").join("config.toml");
            if p1.exists() {
                return p1;
            }
            let p2 = PathBuf::from(&home)
                .join(".config")
                .join("vivivi")
                .join("config.toml");
            if p2.exists() {
                return p2;
            }
        }

        // デフォルト作成先のベースパス
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            PathBuf::from(userprofile)
                .join(".vivivi")
                .join("config.toml")
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".vivivi").join("config.toml")
        } else {
            PathBuf::from("vivivi_config.toml")
        }
    }

    /// 指定パスまたはデフォルトパスから設定を自動ロードします。
    /// ファイルが存在しない場合は [`AppConfig::default()`] を返します。
    pub fn load_or_default(explicit_path: Option<&Path>) -> Self {
        let path = explicit_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(Self::default_config_path);

        if path.exists()
            && let Ok(content) = std::fs::read_to_string(&path)
            && let Ok(config) = Self::from_toml(&content)
        {
            return config;
        }

        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.editor.tab_width, 4);
        assert!(config.editor.line_number);
        assert_eq!(config.editor.default_encoding, "utf-8");
        assert_eq!(config.editor.clipboard, "none");
        assert!(!config.invisible.enabled);
        assert!(config.gutter.enabled);
    }

    #[test]
    fn test_parse_custom_toml() {
        let toml_str = r#"
[editor]
tab_width = 8
line_number = false
default_encoding = "sjis"
clipboard = "unnamed"

[invisible]
enabled = true
zenkaku_space = "■"

[gutter]
enabled = false
"#;
        let config = AppConfig::from_toml(toml_str).expect("パース成功");
        assert_eq!(config.editor.tab_width, 8);
        assert!(!config.editor.line_number);
        assert_eq!(config.editor.default_encoding, "sjis");
        assert_eq!(config.editor.clipboard, "unnamed");
        assert!(config.invisible.enabled);
        assert_eq!(config.invisible.zenkaku_space, "■");
        assert!(!config.gutter.enabled);
    }
}
