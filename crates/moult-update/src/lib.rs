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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockReleaseBuilder;
    use tempfile::tempdir;

    #[test]
    fn test_version_semver_comparison() {
        let cur = semver::Version::parse("0.9.14").unwrap();
        let new_ver = semver::Version::parse("0.9.15").unwrap();
        assert!(new_ver > cur);

        let v_tag = "v0.9.15".trim_start_matches('v');
        let parsed = semver::Version::parse(v_tag).unwrap();
        assert_eq!(parsed, new_ver);
    }

    #[test]
    fn test_parse_checksum() {
        let sha256_content = "\
# Checksums
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  vivivi-windows-x86_64.zip
ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad *vivivi-linux-x86_64.tar.gz
";
        let parsed = parse_checksum(sha256_content, "vivivi-windows-x86_64.zip");
        assert_eq!(
            parsed,
            Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string())
        );

        let single_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n";
        assert_eq!(
            parse_checksum(single_hash, "any.zip"),
            Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string())
        );
    }

    #[test]
    fn test_swap_and_cleanup_lifecycle() {
        let test_dir = tempdir().unwrap();
        let bin_dir = test_dir.path().join("bin");
        let extract_dir = test_dir.path().join("extracted");
        std::fs::create_dir_all(&bin_dir).unwrap();
        std::fs::create_dir_all(&extract_dir).unwrap();

        // 既存のインストール済みバイナリ (GUI & TUI)
        let installed_gui = bin_dir.join("vivivi-gui.exe");
        let installed_tui = bin_dir.join("vivivi.exe");
        std::fs::write(&installed_gui, b"OLD_GUI_V0.9.14").unwrap();
        std::fs::write(&installed_tui, b"OLD_TUI_V0.9.14").unwrap();

        // 展開された新バイナリ
        let new_gui = extract_dir.join("vivivi-gui.exe");
        let new_tui = extract_dir.join("vivivi.exe");
        std::fs::write(&new_gui, b"NEW_GUI_V0.9.15").unwrap();
        std::fs::write(&new_tui, b"NEW_TUI_V0.9.15").unwrap();

        // スワップ実行
        swap::swap_explicit(
            &extract_dir,
            &installed_gui,
            Some(&installed_tui),
            "vivivi-gui.exe",
            Some("vivivi.exe"),
        )
        .unwrap();

        // 検証: 新バイナリに置き換わっていること
        assert_eq!(std::fs::read(&installed_gui).unwrap(), b"NEW_GUI_V0.9.15");
        assert_eq!(std::fs::read(&installed_tui).unwrap(), b"NEW_TUI_V0.9.15");

        // 検証: .old ファイルが作成されていること
        let old_gui = bin_dir.join("vivivi-gui.exe.old");
        let old_tui = bin_dir.join("vivivi.exe.old");
        assert!(old_gui.exists());
        assert!(old_tui.exists());
        assert_eq!(std::fs::read(&old_gui).unwrap(), b"OLD_GUI_V0.9.14");
        assert_eq!(std::fs::read(&old_tui).unwrap(), b"OLD_TUI_V0.9.14");

        // クリーンアップ実行
        let cleaned = cleanup::cleanup_old_files_in_dir(&bin_dir);
        assert_eq!(cleaned, 2);

        // 検証: .old ファイルが完全消去されていること
        assert!(!old_gui.exists());
        assert!(!old_tui.exists());
    }

    #[test]
    fn test_mock_release_builder_and_extract() {
        let test_dir = tempdir().unwrap();
        let builder = MockReleaseBuilder::new("0.9.15", "vivivi-gui.exe")
            .with_pair_bin("vivivi.exe", b"PAIR_TUI_BINARY_DATA")
            .with_target_content(b"TARGET_GUI_BINARY_DATA");

        let (release, zip_path, sha256_hash) = builder.build(test_dir.path());

        assert_eq!(release.version, "0.9.15");
        assert!(zip_path.exists());
        assert_eq!(sha256_hash.len(), 64);

        // ZIP解凍のテスト
        let extract_dir = test_dir.path().join("extracted");
        extract_zip(&zip_path, &extract_dir).unwrap();

        let extracted_gui = extract_dir.join("vivivi-gui.exe");
        let extracted_tui = extract_dir.join("vivivi.exe");
        assert!(extracted_gui.exists());
        assert!(extracted_tui.exists());
        assert_eq!(
            std::fs::read(&extracted_gui).unwrap(),
            b"TARGET_GUI_BINARY_DATA"
        );
        assert_eq!(
            std::fs::read(&extracted_tui).unwrap(),
            b"PAIR_TUI_BINARY_DATA"
        );
    }
}
