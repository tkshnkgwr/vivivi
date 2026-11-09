//! GitHub Releases API 照会およびバージョン判定ロジック

use crate::config::UpdateConfig;
use crate::error::{MoultError, Result};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// GitHub Release の情報
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseInfo {
    /// バージョン文字列 (例: "0.9.15")
    pub version: String,
    /// Git タグ名 (例: "v0.9.15")
    pub tag_name: String,
    /// リリースタイトル
    pub name: String,
    /// リリースノート本文 (Markdown)
    pub body: String,
    /// GitHub リリース Web ページ URL
    pub html_url: String,
    /// 添付アセット一覧
    pub assets: Vec<AssetInfo>,
    /// 公開日時
    pub published_at: Option<String>,
}

/// 添付アセットの情報
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetInfo {
    /// ファイル名 (例: "vivivi-windows-x86_64.zip")
    pub name: String,
    /// 直接ダウンロード URL
    pub download_url: String,
    /// ファイルサイズ (バイト数)
    pub size: u64,
}

/// アップデート照会結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateCheckResult {
    /// 既に最新版を利用中
    UpToDate {
        /// 現在のバージョン
        current_version: String,
    },
    /// 新バージョンが利用可能
    UpdateAvailable {
        /// 現在のバージョン
        current_version: String,
        /// 最新リリースの情報
        latest_release: Box<ReleaseInfo>,
        /// ダウンロード対象のアセット
        target_asset: AssetInfo,
        /// チェックサムアセット (存在する場合)
        sha256_asset: Option<AssetInfo>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    timestamp: u64,
    release: ReleaseInfo,
}

#[derive(Debug, Deserialize)]
struct GithubReleaseResponse {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    html_url: String,
    published_at: Option<String>,
    assets: Vec<GithubAssetResponse>,
}

#[derive(Debug, Deserialize)]
struct GithubAssetResponse {
    name: String,
    browser_download_url: String,
    size: u64,
}

/// 現在時刻の UNIX エポック秒を取得
fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// キャッシュファイルのパスを取得
fn get_cache_file_path(config: &UpdateConfig) -> PathBuf {
    let dir = config.cache_dir.clone().unwrap_or_else(|| {
        std::env::temp_dir().join(format!(
            "{}_{}_update_cache",
            config.repo_owner, config.repo_name
        ))
    });
    dir.join("latest_release.json")
}

/// キャッシュからリリース情報を読み込む (TTL 内なら有効)
fn read_cache(cache_path: &Path, ttl_secs: u64) -> Option<ReleaseInfo> {
    if !cache_path.exists() {
        return None;
    }

    let content = fs::read_to_string(cache_path).ok()?;
    let entry: CacheEntry = serde_json::from_str(&content).ok()?;
    let now = current_unix_timestamp();

    if now >= entry.timestamp && (now - entry.timestamp) < ttl_secs {
        Some(entry.release)
    } else {
        None
    }
}

/// キャッシュへリリース情報を保存
fn write_cache(cache_path: &Path, release: &ReleaseInfo) {
    if let Some(parent) = cache_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let entry = CacheEntry {
        timestamp: current_unix_timestamp(),
        release: release.clone(),
    };
    if let Ok(content) = serde_json::to_string_pretty(&entry) {
        let _ = fs::write(cache_path, content);
    }
}

/// GitHub API レスポンスから ReleaseInfo を構築
fn parse_github_response(resp: GithubReleaseResponse) -> Result<ReleaseInfo> {
    let raw_ver = resp.tag_name.trim_start_matches('v').trim();
    let version = Version::parse(raw_ver)
        .map_err(|e| MoultError::VersionParse(format!("{}: {}", raw_ver, e)))?
        .to_string();

    let assets = resp
        .assets
        .into_iter()
        .map(|a| AssetInfo {
            name: a.name,
            download_url: a.browser_download_url,
            size: a.size,
        })
        .collect();

    Ok(ReleaseInfo {
        version,
        tag_name: resp.tag_name,
        name: resp.name.unwrap_or_default(),
        body: resp.body.unwrap_or_default(),
        html_url: resp.html_url,
        assets,
        published_at: resp.published_at,
    })
}

/// GitHub API を実際に呼び出して最新リリースを取得
pub fn fetch_latest_release_from_github(config: &UpdateConfig) -> Result<ReleaseInfo> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        config.repo_owner, config.repo_name
    );

    let user_agent = format!("moult-update/{}", env!("CARGO_PKG_VERSION"));

    let response = ureq::get(&url)
        .set("User-Agent", &user_agent)
        .set("Accept", "application/vnd.github.v3+json")
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(code, resp) => {
                let msg = resp.into_string().unwrap_or_default();
                MoultError::Http(code, msg)
            }
            ureq::Error::Transport(t) => MoultError::Network(t.to_string()),
        })?;

    let github_resp: GithubReleaseResponse = response
        .into_json()
        .map_err(|e| MoultError::Json(e.to_string()))?;

    parse_github_response(github_resp)
}

/// 最新リリースが存在するか確認し、アップデート判定を実施
pub fn check_for_update(config: &UpdateConfig, force: bool) -> Result<UpdateCheckResult> {
    let cache_path = get_cache_file_path(config);

    let release = if !force {
        if let Some(cached) = read_cache(&cache_path, config.cache_ttl_secs) {
            cached
        } else {
            let fetched = fetch_latest_release_from_github(config)?;
            write_cache(&cache_path, &fetched);
            fetched
        }
    } else {
        let fetched = fetch_latest_release_from_github(config)?;
        write_cache(&cache_path, &fetched);
        fetched
    };

    let cur_str = config.current_version.trim_start_matches('v').trim();
    let current_ver = Version::parse(cur_str)
        .map_err(|e| MoultError::VersionParse(format!("{}: {}", cur_str, e)))?;

    let latest_ver = Version::parse(&release.version)
        .map_err(|e| MoultError::VersionParse(format!("{}: {}", release.version, e)))?;

    if latest_ver <= current_ver {
        return Ok(UpdateCheckResult::UpToDate {
            current_version: config.current_version.clone(),
        });
    }

    // 対象アセットの特定
    let filter = config.asset_filter.as_deref().unwrap_or("windows");
    let target_asset = release
        .assets
        .iter()
        .find(|a| {
            let n = a.name.to_lowercase();
            n.contains(&filter.to_lowercase()) && n.ends_with(".zip")
        })
        .or_else(|| {
            // フィルタで見つからない場合、任意の .zip を探索
            release
                .assets
                .iter()
                .find(|a| a.name.to_lowercase().ends_with(".zip"))
        })
        .cloned()
        .ok_or_else(|| MoultError::AssetNotFound {
            pattern: filter.to_string(),
        })?;

    // チェックサムアセットの特定 (例: .sha256 または SHA256SUMS)
    let sha256_asset = release
        .assets
        .iter()
        .find(|a| {
            let n = a.name.to_lowercase();
            n.ends_with(".sha256") || n.contains("sha256")
        })
        .cloned();

    Ok(UpdateCheckResult::UpdateAvailable {
        current_version: config.current_version.clone(),
        latest_release: Box::new(release),
        target_asset,
        sha256_asset,
    })
}
