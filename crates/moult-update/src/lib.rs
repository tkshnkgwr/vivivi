//! moult-update: 脱皮型自己更新＆Windowsインプレース・スワップエンジン
//!
//! アプリケーション非依存で、GitHub Releases からのバージョン照会、
//! バックグラウンドダウンロード、SHA-256 改ざん検証、Windows ファイルロックを回避した
//! インプレース・リネーム＆スワップ、およびペアバイナリの一括同期を提供します。

pub mod check;
pub mod cleanup;
pub mod config;
pub mod download;
pub mod error;
pub mod mock;
pub mod swap;

pub use check::{AssetInfo, ReleaseInfo, UpdateCheckResult, check_for_update};
pub use cleanup::{cleanup_old_binaries, cleanup_old_files_in_dir};
pub use config::UpdateConfig;
pub use download::{ProgressCallback, download_and_extract, extract_zip, parse_checksum};
pub use error::{MoultError, Result};
pub use swap::{
    find_binary_in_extracted, restart_process, spawn_new_process, swap_in_place, swap_single_binary,
};

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// アップデートマネージャーの動作ステータス
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    /// 待機中（未確認）
    Idle,
    /// バージョン照会中
    Checking,
    /// 現在のバージョンが最新
    UpToDate,
    /// 新バージョンが利用可能
    UpdateAvailable(ReleaseInfo),
    /// ダウンロード進行中
    Downloading {
        /// ダウンロード済みバイト数
        downloaded: u64,
        /// 合計予想バイト数
        total: Option<u64>,
    },
    /// ダウンロードおよび展開完了（適用準備完了）
    ReadyToApply {
        /// リリース情報
        release: ReleaseInfo,
        /// 展開済みバイナリの一時ステージングディレクトリ
        staging_dir: PathBuf,
    },
    /// スワップ完了（再起動待ち）
    Applied,
    /// エラー発生
    Failed(String),
}

/// 高レベルなアップデート管理エンジン
///
/// # Examples
/// ```
/// use moult_update::{UpdateConfig, UpdateManager, UpdateStatus};
///
/// let cfg = UpdateConfig::new("owner", "repo", "0.1.0", "vivivi");
/// let mgr = UpdateManager::new(cfg);
/// assert_eq!(mgr.status(), UpdateStatus::Idle);
/// ```
pub struct UpdateManager {
    config: UpdateConfig,
    status: Arc<Mutex<UpdateStatus>>,
}

impl UpdateManager {
    /// 新規マネージャーを作成
    pub fn new(config: UpdateConfig) -> Self {
        Self {
            config,
            status: Arc::new(Mutex::new(UpdateStatus::Idle)),
        }
    }

    /// 現在の設定を参照
    pub fn config(&self) -> &UpdateConfig {
        &self.config
    }

    /// 現在のステータスを取得
    pub fn status(&self) -> UpdateStatus {
        self.status.lock().unwrap().clone()
    }

    /// ステータス共有ハンドルを取得（UIスレッド等からの監視用）
    pub fn status_handle(&self) -> Arc<Mutex<UpdateStatus>> {
        Arc::clone(&self.status)
    }

    /// 更新があるか確認
    pub fn check(&self, force: bool) -> Result<UpdateCheckResult> {
        *self.status.lock().unwrap() = UpdateStatus::Checking;
        match check_for_update(&self.config, force) {
            Ok(UpdateCheckResult::UpdateAvailable {
                latest_release,
                target_asset,
                sha256_asset,
                current_version,
            }) => {
                let res = UpdateCheckResult::UpdateAvailable {
                    current_version,
                    latest_release: latest_release.clone(),
                    target_asset,
                    sha256_asset,
                };
                *self.status.lock().unwrap() = UpdateStatus::UpdateAvailable(*latest_release);
                Ok(res)
            }
            Ok(UpdateCheckResult::UpToDate { current_version }) => {
                *self.status.lock().unwrap() = UpdateStatus::UpToDate;
                Ok(UpdateCheckResult::UpToDate { current_version })
            }
            Err(e) => {
                *self.status.lock().unwrap() = UpdateStatus::Failed(e.to_string());
                Err(e)
            }
        }
    }

    /// 新バージョンのダウンロード＆展開を実行
    pub fn download(
        &self,
        release: &ReleaseInfo,
        target_asset: &AssetInfo,
        sha256_asset: Option<&AssetInfo>,
        staging_parent: Option<&Path>,
    ) -> Result<PathBuf> {
        let parent = staging_parent
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("moult_staging"));

        let stage_dir = parent.join(format!("{}_{}", self.config.repo_name, release.version));

        // 期待される SHA-256 チェックサムの取得
        let expected_sha256 = if let Some(sha_asset) = sha256_asset {
            download::fetch_checksum_content(sha_asset)
                .ok()
                .and_then(|c| download::parse_checksum(&c, &target_asset.name))
        } else {
            None
        };

        let status_arc = Arc::clone(&self.status);
        let progress_cb = move |downloaded: u64, total: Option<u64>| {
            *status_arc.lock().unwrap() = UpdateStatus::Downloading { downloaded, total };
        };

        let extract_path = download_and_extract(
            target_asset,
            expected_sha256.as_deref(),
            &stage_dir,
            Some(&progress_cb),
        )?;

        *self.status.lock().unwrap() = UpdateStatus::ReadyToApply {
            release: release.clone(),
            staging_dir: extract_path.clone(),
        };

        Ok(extract_path)
    }

    /// スワップを適用（新バイナリへ置換）
    pub fn apply(&self, staging_dir: &Path) -> Result<()> {
        swap_in_place(staging_dir, &self.config)?;
        *self.status.lock().unwrap() = UpdateStatus::Applied;
        Ok(())
    }

    /// スワップを適用して新プロセスへ即時再起動
    pub fn apply_and_restart(&self, staging_dir: &Path) -> Result<()> {
        self.apply(staging_dir)?;
        restart_process();
    }
}
