//! カラーテーマ管理モジュール (`themes/<name>.toml`)。
//!
//! TUI (Ratatui) および GUI (egui) で共通の 24bit RGB カラーパレットを管理し、
//! `%USERPROFILE%\.vivivi\themes\` からの動的読み込み、内蔵5大プリセットテーマ、
//! および初期ブートストラップ展開を提供します。

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::PathBuf;

/// 24bit RGB カラー表現。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ThemeColor {
    /// RGB 成分から作成します。
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// 16進数文字列（例: `"#1a1b26"` または `"1a1b26"`）からパースします。
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.trim().trim_start_matches('#');
        if hex.len() != 6 {
            return Err(format!("不正なカラーコード長: {}", hex));
        }
        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|e| format!("赤成分のパースエラー: {}", e))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|e| format!("緑成分のパースエラー: {}", e))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|e| format!("青成分のパースエラー: {}", e))?;
        Ok(Self { r, g, b })
    }

    /// 16進数文字列（`"#RRGGBB"`）へ変換します。
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Ratatui 互換のカラーを取得します。
    pub fn to_ratatui(&self) -> ratatui::style::Color {
        ratatui::style::Color::Rgb(self.r, self.g, self.b)
    }

    /// RGB のタプルを取得します。
    pub const fn rgb(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
}

impl Serialize for ThemeColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for ThemeColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

/// テーマメタ情報。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeMeta {
    pub name: String,
    pub author: String,
    pub variant: String, // "dark" または "light"
}

/// UI 背景・前景カラー定義。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeUi {
    pub bg_editor: ThemeColor,
    pub bg_panel: ThemeColor,
    pub bg_statusline: ThemeColor,
    pub bg_selection: ThemeColor,
    pub bg_hover: ThemeColor,
    pub bg_active: ThemeColor,

    pub fg_text: ThemeColor,
    pub fg_dimmed: ThemeColor,
    pub fg_linenumber: ThemeColor,
    pub fg_cursor: ThemeColor,
    pub bg_cursor: ThemeColor,
}

/// Vi モードバッジカラー定義。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeModes {
    pub normal: ThemeColor,
    pub insert: ThemeColor,
    pub visual: ThemeColor,
    pub command: ThemeColor,
    pub replace: ThemeColor,
    pub keisen: ThemeColor,
}

/// 変更行ガターカラー定義。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeGutter {
    pub change_modified: ThemeColor,
    pub change_saved: ThemeColor,
}

/// 構文ハイライトカラー定義。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeSyntax {
    pub keyword: ThemeColor,
    #[serde(rename = "type")]
    pub type_name: ThemeColor,
    pub function: ThemeColor,
    pub string: ThemeColor,
    pub number: ThemeColor,
    pub comment: ThemeColor,
    #[serde(rename = "macro")]
    pub macro_name: ThemeColor,
    pub operator: ThemeColor,
}

/// 統合カラーテーマ。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Theme {
    pub meta: ThemeMeta,
    pub ui: ThemeUi,
    pub modes: ThemeModes,
    pub gutter: ThemeGutter,
    pub syntax: ThemeSyntax,
}

impl Theme {
    /// TOML 文字列からテーマをパースします。
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    /// テーマを TOML 文字列へシリアライズします。
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// 標準プリセット: Tokyo Night (Dark)
    pub fn builtin_tokyo_night() -> Self {
        Self {
            meta: ThemeMeta {
                name: "Tokyo Night".to_string(),
                author: "folke / vivivi port".to_string(),
                variant: "dark".to_string(),
            },
            ui: ThemeUi {
                bg_editor: ThemeColor::new(0x1a, 0x1b, 0x26),
                bg_panel: ThemeColor::new(0x16, 0x16, 0x1e),
                bg_statusline: ThemeColor::new(0x1f, 0x23, 0x35),
                bg_selection: ThemeColor::new(0x28, 0x34, 0x57),
                bg_hover: ThemeColor::new(0x29, 0x2e, 0x42),
                bg_active: ThemeColor::new(0x41, 0x48, 0x68),
                fg_text: ThemeColor::new(0xc0, 0xca, 0xf5),
                fg_dimmed: ThemeColor::new(0x56, 0x5f, 0x89),
                fg_linenumber: ThemeColor::new(0x3b, 0x42, 0x61),
                fg_cursor: ThemeColor::new(0xc0, 0xca, 0xf5),
                bg_cursor: ThemeColor::new(0x7a, 0xa2, 0xf7),
            },
            modes: ThemeModes {
                normal: ThemeColor::new(0x7a, 0xa2, 0xf7),
                insert: ThemeColor::new(0x9e, 0xce, 0x6a),
                visual: ThemeColor::new(0xbb, 0x9a, 0xf7),
                command: ThemeColor::new(0xe0, 0xaf, 0x68),
                replace: ThemeColor::new(0xf7, 0x76, 0x8e),
                keisen: ThemeColor::new(0x7d, 0xcf, 0xff),
            },
            gutter: ThemeGutter {
                change_modified: ThemeColor::new(0xe0, 0xaf, 0x68),
                change_saved: ThemeColor::new(0x9e, 0xce, 0x6a),
            },
            syntax: ThemeSyntax {
                keyword: ThemeColor::new(0xbb, 0x9a, 0xf7),
                type_name: ThemeColor::new(0x7d, 0xcf, 0xff),
                function: ThemeColor::new(0x7a, 0xa2, 0xf7),
                string: ThemeColor::new(0x9e, 0xce, 0x6a),
                number: ThemeColor::new(0xff, 0x9e, 0x64),
                comment: ThemeColor::new(0x56, 0x5f, 0x89),
                macro_name: ThemeColor::new(0x7d, 0xcf, 0xff),
                operator: ThemeColor::new(0x89, 0xdd, 0xff),
            },
        }
    }

    /// 標準プリセット: OpenViVi (Dark)
    pub fn builtin_openvivi() -> Self {
        Self {
            meta: ThemeMeta {
                name: "OpenViVi".to_string(),
                author: "vivisuke / vivivi port".to_string(),
                variant: "dark".to_string(),
            },
            ui: ThemeUi {
                bg_editor: ThemeColor::new(0x1e, 0x1e, 0x1e),
                bg_panel: ThemeColor::new(0x25, 0x25, 0x26),
                bg_statusline: ThemeColor::new(0x2d, 0x2d, 0x30),
                bg_selection: ThemeColor::new(0x26, 0x4f, 0x78),
                bg_hover: ThemeColor::new(0x3e, 0x3e, 0x42),
                bg_active: ThemeColor::new(0x00, 0x7a, 0xcc),
                fg_text: ThemeColor::new(0xd4, 0xd4, 0xd4),
                fg_dimmed: ThemeColor::new(0x80, 0x80, 0x80),
                fg_linenumber: ThemeColor::new(0x5a, 0x5a, 0x5a),
                fg_cursor: ThemeColor::new(0xff, 0xff, 0xff),
                bg_cursor: ThemeColor::new(0x00, 0x7a, 0xcc),
            },
            modes: ThemeModes {
                normal: ThemeColor::new(0x00, 0x7a, 0xcc),
                insert: ThemeColor::new(0x4e, 0xc9, 0xb0),
                visual: ThemeColor::new(0xc5, 0x86, 0xc0),
                command: ThemeColor::new(0xdc, 0xdc, 0xaa),
                replace: ThemeColor::new(0xf4, 0x47, 0x47),
                keisen: ThemeColor::new(0x4f, 0xc1, 0xff),
            },
            gutter: ThemeGutter {
                change_modified: ThemeColor::new(0xdc, 0xdc, 0xaa),
                change_saved: ThemeColor::new(0x4e, 0xc9, 0xb0),
            },
            syntax: ThemeSyntax {
                keyword: ThemeColor::new(0xc5, 0x86, 0xc0),
                type_name: ThemeColor::new(0x4e, 0xc9, 0xb0),
                function: ThemeColor::new(0xdc, 0xdc, 0xaa),
                string: ThemeColor::new(0xce, 0x91, 0x78),
                number: ThemeColor::new(0xb5, 0xce, 0xa8),
                comment: ThemeColor::new(0x6a, 0x99, 0x55),
                macro_name: ThemeColor::new(0x4f, 0xc1, 0xff),
                operator: ThemeColor::new(0xd4, 0xd4, 0xd4),
            },
        }
    }

    /// 標準プリセット: ViVi Classic (Light)
    pub fn builtin_vivi_classic() -> Self {
        Self {
            meta: ThemeMeta {
                name: "ViVi Classic".to_string(),
                author: "Tsuda Nobuhide / vivivi port".to_string(),
                variant: "light".to_string(),
            },
            ui: ThemeUi {
                bg_editor: ThemeColor::new(0xff, 0xff, 0xff),
                bg_panel: ThemeColor::new(0xf0, 0xf0, 0xf0),
                bg_statusline: ThemeColor::new(0xe0, 0xe0, 0xe0),
                bg_selection: ThemeColor::new(0xb5, 0xd5, 0xff),
                bg_hover: ThemeColor::new(0xe8, 0xe8, 0xe8),
                bg_active: ThemeColor::new(0xd0, 0xd0, 0xd0),
                fg_text: ThemeColor::new(0x00, 0x00, 0x00),
                fg_dimmed: ThemeColor::new(0x70, 0x70, 0x70),
                fg_linenumber: ThemeColor::new(0x88, 0x88, 0x88),
                fg_cursor: ThemeColor::new(0x00, 0x00, 0x00),
                bg_cursor: ThemeColor::new(0x00, 0x00, 0xff),
            },
            modes: ThemeModes {
                normal: ThemeColor::new(0x00, 0x00, 0xcc),
                insert: ThemeColor::new(0x00, 0x88, 0x00),
                visual: ThemeColor::new(0x88, 0x00, 0x88),
                command: ThemeColor::new(0xbb, 0x88, 0x00),
                replace: ThemeColor::new(0xcc, 0x00, 0x00),
                keisen: ThemeColor::new(0x00, 0x88, 0xaa),
            },
            gutter: ThemeGutter {
                change_modified: ThemeColor::new(0xbb, 0x88, 0x00),
                change_saved: ThemeColor::new(0x00, 0x88, 0x00),
            },
            syntax: ThemeSyntax {
                keyword: ThemeColor::new(0x00, 0x00, 0xff),
                type_name: ThemeColor::new(0x00, 0x88, 0xaa),
                function: ThemeColor::new(0x88, 0x00, 0x00),
                string: ThemeColor::new(0xa3, 0x15, 0x15),
                number: ThemeColor::new(0x09, 0x86, 0x58),
                comment: ThemeColor::new(0x00, 0x80, 0x00),
                macro_name: ThemeColor::new(0x79, 0x5e, 0x26),
                operator: ThemeColor::new(0x00, 0x00, 0x00),
            },
        }
    }

    /// 標準プリセット: vivivi Dark (Dark)
    pub fn builtin_vivivi_dark() -> Self {
        Self {
            meta: ThemeMeta {
                name: "vivivi Dark".to_string(),
                author: "vivivi developers".to_string(),
                variant: "dark".to_string(),
            },
            ui: ThemeUi {
                bg_editor: ThemeColor::new(0x12, 0x12, 0x14),
                bg_panel: ThemeColor::new(0x18, 0x18, 0x1c),
                bg_statusline: ThemeColor::new(0x20, 0x20, 0x26),
                bg_selection: ThemeColor::new(0x2c, 0x32, 0x44),
                bg_hover: ThemeColor::new(0x26, 0x26, 0x30),
                bg_active: ThemeColor::new(0x36, 0x36, 0x46),
                fg_text: ThemeColor::new(0xe2, 0xe4, 0xea),
                fg_dimmed: ThemeColor::new(0x6c, 0x70, 0x86),
                fg_linenumber: ThemeColor::new(0x45, 0x47, 0x5a),
                fg_cursor: ThemeColor::new(0xe2, 0xe4, 0xea),
                bg_cursor: ThemeColor::new(0x89, 0xb4, 0xfa),
            },
            modes: ThemeModes {
                normal: ThemeColor::new(0x89, 0xb4, 0xfa),
                insert: ThemeColor::new(0xa6, 0xe3, 0xa1),
                visual: ThemeColor::new(0xcb, 0xa6, 0xf7),
                command: ThemeColor::new(0xf9, 0xe2, 0xaf),
                replace: ThemeColor::new(0xf3, 0x8b, 0xa8),
                keisen: ThemeColor::new(0x94, 0xe2, 0xd5),
            },
            gutter: ThemeGutter {
                change_modified: ThemeColor::new(0xf9, 0xe2, 0xaf),
                change_saved: ThemeColor::new(0xa6, 0xe3, 0xa1),
            },
            syntax: ThemeSyntax {
                keyword: ThemeColor::new(0xcb, 0xa6, 0xf7),
                type_name: ThemeColor::new(0x94, 0xe2, 0xd5),
                function: ThemeColor::new(0x89, 0xb4, 0xfa),
                string: ThemeColor::new(0xa6, 0xe3, 0xa1),
                number: ThemeColor::new(0xfa, 0xb3, 0x87),
                comment: ThemeColor::new(0x6c, 0x70, 0x86),
                macro_name: ThemeColor::new(0x74, 0xc7, 0xec),
                operator: ThemeColor::new(0x89, 0xdc, 0xeb),
            },
        }
    }

    /// 標準プリセット: vivivi Light (Light)
    pub fn builtin_vivivi_light() -> Self {
        Self {
            meta: ThemeMeta {
                name: "vivivi Light".to_string(),
                author: "vivivi developers".to_string(),
                variant: "light".to_string(),
            },
            ui: ThemeUi {
                bg_editor: ThemeColor::new(0xfa, 0xf8, 0xf5),
                bg_panel: ThemeColor::new(0xf2, 0xee, 0xe8),
                bg_statusline: ThemeColor::new(0xe8, 0xe2, 0xd8),
                bg_selection: ThemeColor::new(0xdc, 0xd4, 0xc6),
                bg_hover: ThemeColor::new(0xed, 0xe6, 0xdc),
                bg_active: ThemeColor::new(0xdf, 0xd7, 0xcb),
                fg_text: ThemeColor::new(0x3c, 0x38, 0x36),
                fg_dimmed: ThemeColor::new(0x7c, 0x6f, 0x64),
                fg_linenumber: ThemeColor::new(0xa8, 0x99, 0x84),
                fg_cursor: ThemeColor::new(0x3c, 0x38, 0x36),
                bg_cursor: ThemeColor::new(0x07, 0x66, 0x78),
            },
            modes: ThemeModes {
                normal: ThemeColor::new(0x07, 0x66, 0x78),
                insert: ThemeColor::new(0x79, 0x74, 0x0e),
                visual: ThemeColor::new(0x8f, 0x3f, 0x71),
                command: ThemeColor::new(0xb5, 0x76, 0x14),
                replace: ThemeColor::new(0x9d, 0x00, 0x06),
                keisen: ThemeColor::new(0x42, 0x7b, 0x58),
            },
            gutter: ThemeGutter {
                change_modified: ThemeColor::new(0xb5, 0x76, 0x14),
                change_saved: ThemeColor::new(0x79, 0x74, 0x0e),
            },
            syntax: ThemeSyntax {
                keyword: ThemeColor::new(0x8f, 0x3f, 0x71),
                type_name: ThemeColor::new(0x42, 0x7b, 0x58),
                function: ThemeColor::new(0x07, 0x66, 0x78),
                string: ThemeColor::new(0x79, 0x74, 0x0e),
                number: ThemeColor::new(0xd6, 0x5d, 0x0e),
                comment: ThemeColor::new(0x92, 0x83, 0x74),
                macro_name: ThemeColor::new(0xb1, 0x62, 0x86),
                operator: ThemeColor::new(0x3c, 0x38, 0x36),
            },
        }
    }

    /// テーマの保存ディレクトリ（`%USERPROFILE%\.vivivi\themes\`）を取得します。
    pub fn themes_dir() -> PathBuf {
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            PathBuf::from(userprofile).join(".vivivi").join("themes")
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".vivivi").join("themes")
        } else {
            PathBuf::from(".vivivi").join("themes")
        }
    }

    /// テーマ名から該当するテーマを読み込みます。
    ///
    /// 1. `%USERPROFILE%\.vivivi\themes\<name>.toml` から読み込み
    /// 2. 内蔵5大テーマ（tokyo-night, openvivi, vivi-classic, vivivi-dark, vivivi-light）から検索
    /// 3. 見つからない場合は Tokyo Night をフォールバックとして返します。
    pub fn load_by_name(name: &str) -> Self {
        let clean_name = name.trim().to_lowercase().replace(' ', "-");
        let path = Self::themes_dir().join(format!("{}.toml", clean_name));

        if path.exists()
            && let Ok(content) = std::fs::read_to_string(&path)
            && let Ok(theme) = Self::from_toml(&content)
        {
            return theme;
        }

        match clean_name.as_str() {
            "tokyo-night" | "tokyonight" => Self::builtin_tokyo_night(),
            "openvivi" => Self::builtin_openvivi(),
            "vivi-classic" | "viviclassic" => Self::builtin_vivi_classic(),
            "vivivi-dark" | "vivividark" => Self::builtin_vivivi_dark(),
            "vivivi-light" | "vivivilight" => Self::builtin_vivivi_light(),
            _ => Self::builtin_tokyo_night(),
        }
    }

    /// 利用可能なテーマ名一覧（内蔵 + ディレクトリ内）を取得します。
    pub fn list_available_themes() -> Vec<String> {
        let mut themes = vec![
            "tokyo-night".to_string(),
            "openvivi".to_string(),
            "vivi-classic".to_string(),
            "vivivi-dark".to_string(),
            "vivivi-light".to_string(),
        ];

        let dir = Self::themes_dir();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file()
                    && p.extension().is_some_and(|ext| ext == "toml")
                    && let Some(stem) = p.file_stem().and_then(|s| s.to_str())
                {
                    let name = stem.to_string();
                    if !themes.contains(&name) {
                        themes.push(name);
                    }
                }
            }
        }

        themes
    }

    /// 内蔵5大テーマの TOML ファイルを `%USERPROFILE%\.vivivi\themes\` へ自動展開します。
    pub fn bootstrap_builtin_themes() -> Result<(), anyhow::Error> {
        let dir = Self::themes_dir();
        std::fs::create_dir_all(&dir)?;

        let presets = [
            ("tokyo-night.toml", Self::builtin_tokyo_night()),
            ("openvivi.toml", Self::builtin_openvivi()),
            ("vivi-classic.toml", Self::builtin_vivi_classic()),
            ("vivivi-dark.toml", Self::builtin_vivivi_dark()),
            ("vivivi-light.toml", Self::builtin_vivivi_light()),
        ];

        for (filename, theme) in presets {
            let target = dir.join(filename);
            if !target.exists() {
                let toml_str = theme.to_toml()?;
                std::fs::write(&target, toml_str)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_color_hex_parse_and_serialize() {
        let c = ThemeColor::from_hex("#1a1b26").unwrap();
        assert_eq!(c.r, 0x1a);
        assert_eq!(c.g, 0x1b);
        assert_eq!(c.b, 0x26);
        assert_eq!(c.to_hex(), "#1a1b26");

        let c2 = ThemeColor::from_hex("ff9e64").unwrap();
        assert_eq!(c2.to_hex(), "#ff9e64");
    }

    #[test]
    fn test_builtin_themes_serialize_and_deserialize_roundtrip() {
        let themes = [
            Theme::builtin_tokyo_night(),
            Theme::builtin_openvivi(),
            Theme::builtin_vivi_classic(),
            Theme::builtin_vivivi_dark(),
            Theme::builtin_vivivi_light(),
        ];

        for original in themes {
            let toml_str = original.to_toml().expect("シリアライズ成功");
            let parsed = Theme::from_toml(&toml_str).expect("デシリアライズ成功");
            assert_eq!(original, parsed);
        }
    }
}
