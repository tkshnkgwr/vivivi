//! moult-update の設定構造体

use std::path::PathBuf;

/// 自動アップデートの設定パラメータ
#[derive(Debug, Clone)]
pub struct UpdateConfig {
    /// GitHub リポジトリ所有者 (例: "tkshnkgwr")
    pub repo_owner: String,
    /// GitHub リポジトリ名 (例: "vivivi")
    pub repo_name: String,
    /// 現在稼働中のバージョン (例: "0.9.14")
    pub current_version: String,
    /// 更新対象の実行ファイル名 (例: "vivivi-gui.exe" または "vivivi.exe")
    pub target_bin_name: String,
    /// 同時に一括同期するペア実行ファイル名 (例: Some("vivivi.exe"))
    pub pair_bin_name: Option<String>,
    /// アセット選定用フィルタ文字列 (例: "windows-x86_64")
    pub asset_filter: Option<String>,
    /// キャッシュ保存ディレクトリ (未指定時はシステム一時ディレクトリ)
    pub cache_dir: Option<PathBuf>,
    /// キャッシュ有効秒数 (デフォルト: 86400秒 = 24時間)
    pub cache_ttl_secs: u64,
}

impl UpdateConfig {
    /// 新規設定を生成
    pub fn new(
        repo_owner: impl Into<String>,
        repo_name: impl Into<String>,
        current_version: impl Into<String>,
        target_bin_name: impl Into<String>,
    ) -> Self {
        Self {
            repo_owner: repo_owner.into(),
            repo_name: repo_name.into(),
            current_version: current_version.into(),
            target_bin_name: target_bin_name.into(),
            pair_bin_name: None,
            asset_filter: None,
            cache_dir: None,
            cache_ttl_secs: 86400,
        }
    }

    /// ペアバイナリ名（GUI/TUIの相方）を設定
    pub fn with_pair_bin(mut self, pair_bin_name: impl Into<String>) -> Self {
        self.pair_bin_name = Some(pair_bin_name.into());
        self
    }

    /// ダウンロード対象アーカイブの判定文字列を設定
    pub fn with_asset_filter(mut self, filter: impl Into<String>) -> Self {
        self.asset_filter = Some(filter.into());
        self
    }

    /// 照会キャッシュの保存先ディレクトリを設定
    pub fn with_cache_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.cache_dir = Some(path.into());
        self
    }

    /// キャッシュ有効秒数を設定
    pub fn with_cache_ttl_secs(mut self, ttl: u64) -> Self {
        self.cache_ttl_secs = ttl;
        self
    }
}
