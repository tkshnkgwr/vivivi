//! 設定ファイル (`config.toml`) の読み込み・永続化 ＆ 初期配備（Auto Bootstrap）モジュール。
//!
//! `%USERPROFILE%\.vivivi\config.toml` から 6大カテゴリ（`general`, `appearance`, `invisibles`,
//! `editing`, `tools`, `keymap`）のエディタ設定をロードし、存在しない場合は解説付きテンプレート
//! および内蔵5大テーマを自動配備します。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::theme::Theme;

fn default_version() -> String {
    "1.0".to_string()
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
    "unnamedplus".to_string()
}
fn default_theme() -> String {
    "tokyo-night".to_string()
}
fn default_font_family() -> String {
    "PlemolJP Console NF".to_string()
}
fn default_font_size() -> f32 {
    15.0
}
fn default_line_height_multiplier() -> f32 {
    1.35
}
fn default_minimap_width() -> f32 {
    90.0
}
fn default_zenkaku() -> String {
    "　".to_string()
}
fn default_space_char() -> char {
    '·'
}
fn default_tab_string() -> String {
    "▸---".to_string()
}
fn default_eol_char() -> char {
    '$'
}
fn default_tab_width() -> usize {
    4
}
fn default_difflog_generations() -> usize {
    50
}
fn default_outline_width() -> usize {
    30
}
fn default_preset() -> String {
    "vi".to_string()
}

/// 1. 一般設定 (General)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneralConfig {
    /// デフォルト文字エンコーディング ("utf-8", "sjis", "euc-jp")
    #[serde(default = "default_encoding")]
    pub default_encoding: String,
    /// 新規作成時のデフォルト改行形式 ("unix" または "dos")
    #[serde(default = "default_fileformat")]
    pub default_fileformat: String,
    /// クリップボード同期設定 ("none", "unnamed", "unnamedplus")
    #[serde(default = "default_clipboard")]
    pub clipboard: String,
    /// 異常終了時の自動スワップ保存
    #[serde(default = "default_true")]
    pub crash_recovery: bool,
    /// 起動時の自動更新確認
    #[serde(default = "default_true")]
    pub check_update: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_encoding: default_encoding(),
            default_fileformat: default_fileformat(),
            clipboard: default_clipboard(),
            crash_recovery: true,
            check_update: true,
        }
    }
}

/// GUI 固有の外観設定
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppearanceGuiConfig {
    /// 優先フォントファミリ名
    #[serde(default = "default_font_family")]
    pub font_family: String,
    /// フォントサイズ (pt)
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    /// 行の高さ倍率
    #[serde(default = "default_line_height_multiplier")]
    pub line_height_multiplier: f32,
    /// ミニマップを表示するか
    #[serde(default = "default_true")]
    pub show_minimap: bool,
    /// ミニマップ幅 (px)
    #[serde(default = "default_minimap_width")]
    pub minimap_width: f32,
    /// 文字削除時の物理重力落下演出 (OpenViVi)
    #[serde(default = "default_true")]
    pub enable_gravity_drop: bool,
    /// アイドル時CPU 0%省電力制御
    #[serde(default = "default_true")]
    pub reactive_idle_sleep: bool,
}

impl Default for AppearanceGuiConfig {
    fn default() -> Self {
        Self {
            font_family: default_font_family(),
            font_size: default_font_size(),
            line_height_multiplier: default_line_height_multiplier(),
            show_minimap: true,
            minimap_width: default_minimap_width(),
            enable_gravity_drop: true,
            reactive_idle_sleep: true,
        }
    }
}

/// 2. 表示・外観設定 (Appearance)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppearanceConfig {
    /// アクティブテーマ名 (`themes/<name>.toml`)
    #[serde(default = "default_theme")]
    pub theme: String,
    /// 行番号表示
    #[serde(default = "default_true")]
    pub line_number: bool,
    /// 相対行番号表示
    #[serde(default)]
    pub relative_number: bool,
    /// 右端自動折り返し
    #[serde(default)]
    pub wrap: bool,
    /// 変更行ガター強調 (未保存:黄 / 保存済み:緑)
    #[serde(default = "default_true")]
    pub gutter_markers: bool,
    /// GUI 固有設定
    #[serde(default)]
    pub gui: AppearanceGuiConfig,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            line_number: true,
            relative_number: false,
            wrap: false,
            gutter_markers: true,
            gui: AppearanceGuiConfig::default(),
        }
    }
}

/// 3. 不可視文字・空白設定 (Invisibles)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InvisiblesConfig {
    /// 不可視文字表示有効フラグ
    #[serde(default)]
    pub enabled: bool,
    /// 全角スペース表示記号（デフォルト: "　"、幅2マス背景強調）
    #[serde(default = "default_zenkaku")]
    pub zenkaku_space: String,
    /// 半角スペース表示記号
    #[serde(default = "default_space_char")]
    pub space: char,
    /// タブ文字表示記号
    #[serde(default = "default_tab_string")]
    pub tab: String,
    /// 改行記号
    #[serde(default = "default_eol_char")]
    pub eol: char,
}

impl Default for InvisiblesConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            zenkaku_space: default_zenkaku(),
            space: default_space_char(),
            tab: default_tab_string(),
            eol: default_eol_char(),
        }
    }
}

/// 旧名互換エイリアス
pub type InvisibleConfig = InvisiblesConfig;

/// 4. 入力・編集支援設定 (Editing & Assists)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EditingConfig {
    /// タブ幅 (半角スペース換算)
    #[serde(default = "default_tab_width")]
    pub tab_width: usize,
    /// Tabキー押下時にスペースへ展開するか
    #[serde(default)]
    pub expand_tab: bool,
    /// 検索時大文字小文字無視 (ic)
    #[serde(default = "default_true")]
    pub ignorecase: bool,
    /// 大文字混在時は区別 (sc)
    #[serde(default = "default_true")]
    pub smartcase: bool,
    /// 罫線保護編集 (枠線内自動空白伸縮)
    #[serde(default = "default_true")]
    pub keisen_protect: bool,
    /// 動的補完 (dabbrev) での大文字小文字無視
    #[serde(default = "default_true")]
    pub dabbrev_ignorecase: bool,
    /// 対応括弧の強調表示 (%)
    #[serde(default = "default_true")]
    pub match_paren: bool,
}

impl Default for EditingConfig {
    fn default() -> Self {
        Self {
            tab_width: 4,
            expand_tab: false,
            ignorecase: true,
            smartcase: true,
            keisen_protect: true,
            dabbrev_ignorecase: true,
            match_paren: true,
        }
    }
}

/// 5. 独自ツール・ファイル管理設定 (Tools & Views)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolsConfig {
    /// 世代バックアップ (.difflog) 自動蓄積
    #[serde(default = "default_true")]
    pub difflog_enabled: bool,
    /// 世代保持上限数
    #[serde(default = "default_difflog_generations")]
    pub difflog_max_generations: usize,
    /// 起動時にアウトラインペインを開くか
    #[serde(default)]
    pub outline_auto_open: bool,
    /// アウトラインペイン幅 (文字数)
    #[serde(default = "default_outline_width")]
    pub outline_width: usize,
    /// 初期折り畳みレベル (0: すべて展開)
    #[serde(default)]
    pub code_folding_level: usize,
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            difflog_enabled: true,
            difflog_max_generations: 50,
            outline_auto_open: false,
            outline_width: 30,
            code_folding_level: 0,
        }
    }
}

/// 6. キーマップ・操作体系設定 (Keymaps)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeymapConfig {
    /// プリセット体系 ("vi" または "windows")
    #[serde(default = "default_preset")]
    pub preset: String,
    /// ノーマルモードでのキー置換マップ
    #[serde(default)]
    pub normal: HashMap<String, String>,
    /// インサートモードでのキー置換マップ
    #[serde(default)]
    pub insert: HashMap<String, String>,
}

impl Default for KeymapConfig {
    fn default() -> Self {
        Self {
            preset: default_preset(),
            normal: HashMap::new(),
            insert: HashMap::new(),
        }
    }
}

/// 下位互換用エディタ基本設定型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EditorConfig {
    #[serde(default = "default_tab_width")]
    pub tab_width: usize,
    #[serde(default = "default_true")]
    pub line_number: bool,
    #[serde(default)]
    pub relative_number: bool,
    #[serde(default = "default_encoding")]
    pub default_encoding: String,
    #[serde(default = "default_fileformat")]
    pub default_fileformat: String,
    #[serde(default = "default_clipboard")]
    pub clipboard: String,
    #[serde(default = "default_true")]
    pub ignorecase: bool,
    #[serde(default = "default_true")]
    pub smartcase: bool,
    #[serde(default)]
    pub wrap: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_width: 4,
            line_number: true,
            relative_number: false,
            default_encoding: default_encoding(),
            default_fileformat: default_fileformat(),
            clipboard: default_clipboard(),
            ignorecase: true,
            smartcase: true,
            wrap: false,
        }
    }
}

/// 下位互換用ガター設定型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GutterConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for GutterConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// アプリケーション全体の統合設定。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    /// 設定スキーマバージョン
    #[serde(default = "default_version")]
    pub version: String,
    /// 1. 一般設定
    #[serde(default)]
    pub general: GeneralConfig,
    /// 2. 表示・外観設定
    #[serde(default)]
    pub appearance: AppearanceConfig,
    /// 3. 不可視文字設定（旧 `invisible` エイリアス対応）
    #[serde(default, alias = "invisible")]
    pub invisibles: InvisiblesConfig,
    /// 4. 入力・編集支援設定
    #[serde(default)]
    pub editing: EditingConfig,
    /// 5. 独自ツール設定
    #[serde(default)]
    pub tools: ToolsConfig,
    /// 6. キーマップ設定
    #[serde(default)]
    pub keymap: KeymapConfig,

    // --- 下位互換用フィールド（旧 [editor] / [gutter] からの自動引き継ぎ） ---
    #[serde(default, skip_serializing)]
    editor: Option<EditorConfig>,
    #[serde(default, skip_serializing)]
    gutter: Option<GutterConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            general: GeneralConfig::default(),
            appearance: AppearanceConfig::default(),
            invisibles: InvisiblesConfig::default(),
            editing: EditingConfig::default(),
            tools: ToolsConfig::default(),
            keymap: KeymapConfig::default(),
            editor: None,
            gutter: None,
        }
    }
}

impl AppConfig {
    /// TOML 文字列から設定を読み込み、旧形式フィールドの自動マッピングを行います。
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        let mut config: AppConfig = toml::from_str(content)?;
        config.apply_legacy_compat();
        Ok(config)
    }

    /// 設定を TOML 文字列へシリアライズします。
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// 設定内容を TOML 形式で指定ファイルへ保存します。
    /// 親ディレクトリが存在しない場合は自動作成します。
    pub fn save_to(&self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let toml_str = self
            .to_toml()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        let output =
            format!("# vivivi configuration file (generated by :mkvivivirc)\n\n{toml_str}");
        std::fs::write(path, output)?;
        Ok(())
    }

    /// 旧 [editor] および [gutter] セクションが存在した場合、新体系へマッピングします。
    fn apply_legacy_compat(&mut self) {
        if let Some(ref ed) = self.editor {
            self.editing.tab_width = ed.tab_width;
            self.editing.ignorecase = ed.ignorecase;
            self.editing.smartcase = ed.smartcase;
            self.appearance.line_number = ed.line_number;
            self.appearance.relative_number = ed.relative_number;
            self.appearance.wrap = ed.wrap;
            self.general.default_encoding = ed.default_encoding.clone();
            self.general.default_fileformat = ed.default_fileformat.clone();
            self.general.clipboard = ed.clipboard.clone();
        }
        if let Some(ref g) = self.gutter {
            self.appearance.gutter_markers = g.enabled;
        }
    }

    /// 下位互換アクセサ: `editor`
    pub fn legacy_editor(&self) -> EditorConfig {
        EditorConfig {
            tab_width: self.editing.tab_width,
            line_number: self.appearance.line_number,
            relative_number: self.appearance.relative_number,
            default_encoding: self.general.default_encoding.clone(),
            default_fileformat: self.general.default_fileformat.clone(),
            clipboard: self.general.clipboard.clone(),
            ignorecase: self.editing.ignorecase,
            smartcase: self.editing.smartcase,
            wrap: self.appearance.wrap,
        }
    }

    /// 下位互換アクセサ: `gutter`
    pub fn legacy_gutter(&self) -> GutterConfig {
        GutterConfig {
            enabled: self.appearance.gutter_markers,
        }
    }

    /// 下位互換アクセサ: `invisible`
    pub fn legacy_invisible(&self) -> InvisiblesConfig {
        self.invisibles.clone()
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

    /// 初期起動時の自動配備 (Auto Bootstrap) を実行します。
    ///
    /// - `%USERPROFILE%\.vivivi\` および `cache/` ディレクトリを作成
    /// - `themes/` 配下へ内蔵5大テーマ（Tokyo Night, OpenViVi, ViVi Classic, vivivi Dark, vivivi Light）を展開
    /// - `config.toml` が存在しない場合、日本語解説コメント付きテンプレートを自動生成
    pub fn bootstrap() -> Result<PathBuf, anyhow::Error> {
        let config_path = Self::default_config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
            let cache_dir = parent.join("cache");
            std::fs::create_dir_all(&cache_dir)?;
        }

        // 内蔵5大テーマの自動配備
        let _ = Theme::bootstrap_builtin_themes();

        // config.toml が未作成の場合は解説コメント付きテンプレートを保存
        if !config_path.exists() {
            std::fs::write(&config_path, DEFAULT_CONFIG_TEMPLATE)?;
        }

        Ok(config_path)
    }

    /// 指定パスまたはデフォルトパスから設定を自動ロードします。
    /// ファイルが存在しない場合はブートストラップを実行した上で [`AppConfig::default()`] を返します。
    pub fn load_or_default(explicit_path: Option<&Path>) -> Self {
        if explicit_path.is_none() {
            let _ = Self::bootstrap();
        }

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

/// 初回起動時に自動配備される日本語コメント解説付き `config.toml` テンプレート
pub const DEFAULT_CONFIG_TEMPLATE: &str = r#"# vivivi Configuration File
version = "1.0"

# ==========================================
# 1. 一般 (General)
# ==========================================
[general]
default_encoding = "utf-8"     # "utf-8", "sjis", "euc-jp"
default_fileformat = "unix"    # "unix" (LF) または "dos" (CRLF)
clipboard = "unnamedplus"      # "none", "unnamed" (*), "unnamedplus" (+)
crash_recovery = true          # 異常終了時の自動スワップ保存
check_update = true            # 起動時の自動更新確認

# ==========================================
# 2. 表示・外観 (Appearance)
# ==========================================
[appearance]
theme = "tokyo-night"          # アクティブテーマ (themes/<name>.toml)
line_number = true             # 行番号表示 (nu)
relative_number = false        # 相対行番号表示 (rnu)
wrap = false                   # 右端折り返し (wrap)
gutter_markers = true          # 変更行ガター強調 (未保存:黄 / 保存済み:緑)

[appearance.gui]
font_family = "PlemolJP Console NF"
font_size = 15.0
line_height_multiplier = 1.35
show_minimap = true
minimap_width = 90.0
enable_gravity_drop = true     # 文字削除時の物理重力落下演出 (OpenViVi)
reactive_idle_sleep = true     # アイドル時CPU 0%省電力制御

# ==========================================
# 3. 不可視文字・空白 (Invisibles)
# ==========================================
[invisibles]
enabled = false                # 不可視文字表示 (:set list)
zenkaku_space = "　"           # 全角スペース (ワインレッド背景強調)
space = "·"                    # 半角スペース
tab = "▸---"                   # タブ文字
eol = "$"                      # 改行記号

# ==========================================
# 4. 入力・編集支援 (Editing & Assists)
# ==========================================
[editing]
tab_width = 4                  # タブ幅 (半角スペース換算)
expand_tab = false             # Tabキー押下時にスペースへ展開するか
ignorecase = true              # 検索時大文字小文字無視 (ic)
smartcase = true               # 大文字混在時は区別 (sc)
keisen_protect = true          # 罫線保護編集 (枠線内自動空白伸縮)
dabbrev_ignorecase = true      # 動的補完 (dabbrev) での大文字小文字無視
match_paren = true             # 対応括弧の強調表示 (%)

# ==========================================
# 5. 独自ツール・ファイル管理 (Tools & Views)
# ==========================================
[tools]
difflog_enabled = true         # 世代バックアップ (.difflog) 自動蓄積
difflog_max_generations = 50   # 世代保持上限数
outline_auto_open = false      # 起動時にアウトラインペインを開くか
outline_width = 30             # アウトラインペイン幅 (文字数)
code_folding_level = 0         # 初期折り畳みレベル (0: すべて展開)

# ==========================================
# 6. キーマップ・操作体系 (Keymaps)
# ==========================================
[keymap]
preset = "vi"                  # "vi" または "windows" (Shift+矢印, Ctrl+C/V)

[keymap.normal]
"<C-s>" = ":w<CR>"

[keymap.insert]
"jj" = "<Esc>"
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_template_parse() {
        let config = AppConfig::from_toml(DEFAULT_CONFIG_TEMPLATE).expect("テンプレートパース成功");
        assert_eq!(config.version, "1.0");
        assert_eq!(config.general.default_encoding, "utf-8");
        assert_eq!(config.general.clipboard, "unnamedplus");
        assert_eq!(config.appearance.theme, "tokyo-night");
        assert!(config.appearance.line_number);
        assert!(!config.appearance.wrap);
        assert_eq!(config.editing.tab_width, 4);
        assert!(config.editing.ignorecase);
        assert!(config.tools.difflog_enabled);
        assert_eq!(config.keymap.preset, "vi");
    }

    #[test]
    fn test_legacy_editor_and_invisible_compat() {
        let legacy_toml = r#"
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
        let config = AppConfig::from_toml(legacy_toml).expect("旧形式パース成功");
        assert_eq!(config.editing.tab_width, 8);
        assert!(!config.appearance.line_number);
        assert_eq!(config.general.default_encoding, "sjis");
        assert_eq!(config.general.clipboard, "unnamed");
        assert!(config.invisibles.enabled);
        assert_eq!(config.invisibles.zenkaku_space, "■");
        assert!(!config.appearance.gutter_markers);
    }
}
