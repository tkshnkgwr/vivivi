//! 自動アップデート (`moult-update`) 連携メソッドモジュール。

use super::App;

impl App {
    /// 更新があるかバックグラウンドで照会
    pub fn check_for_updates(&self, force: bool) {
        let mgr = std::sync::Arc::clone(&self.update_manager);
        let target_lock = std::sync::Arc::clone(&self.update_target);
        let _ = std::thread::Builder::new()
            .name("vivivi-update-check".to_string())
            .spawn(move || {
                if let Ok(moult_update::UpdateCheckResult::UpdateAvailable {
                    latest_release,
                    target_asset,
                    sha256_asset,
                    ..
                }) = mgr.check(force)
                {
                    *target_lock.lock().unwrap() =
                        Some((*latest_release, target_asset, sha256_asset));
                }
            });
    }

    /// 新バージョンのダウンロード＆展開をバックグラウンドで開始
    pub fn download_update(&self) -> bool {
        let target = self.update_target.lock().unwrap().clone();
        if let Some((release, target_asset, sha256_asset)) = target {
            let mgr = std::sync::Arc::clone(&self.update_manager);
            let staging_lock = std::sync::Arc::clone(&self.update_staging_dir);
            let _ = std::thread::Builder::new()
                .name("vivivi-update-download".to_string())
                .spawn(move || {
                    if let Ok(extracted) =
                        mgr.download(&release, &target_asset, sha256_asset.as_ref(), None)
                    {
                        *staging_lock.lock().unwrap() = Some(extracted);
                    }
                });
            true
        } else {
            false
        }
    }

    /// ダウンロード完了した新バイナリをスワップ適用して即時再起動
    pub fn apply_update_and_restart(&self) -> Result<(), String> {
        let staging = self.update_staging_dir.lock().unwrap().clone();
        if let Some(staging_dir) = staging {
            self.update_manager
                .apply_and_restart(&staging_dir)
                .map_err(|e| e.to_string())
        } else {
            Err("更新バイナリがまだ準備できていません".to_string())
        }
    }

    /// 現在のアップデートステータスを取得
    pub fn update_status(&self) -> moult_update::UpdateStatus {
        self.update_manager.status()
    }
}
