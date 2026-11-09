//! アセットのストリーミングダウンロード、SHA-256 チェックサム検証、ZIP展開

use crate::check::AssetInfo;
use crate::error::{MoultError, Result};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};

/// ダウンロード進捗の通知用コールバックトレイト
pub trait ProgressCallback: Fn(u64, Option<u64>) + Send + Sync {}
impl<F: Fn(u64, Option<u64>) + Send + Sync> ProgressCallback for F {}

/// チェックサム文字列を取得（SHA256SUMS や .sha256 ファイルの内容から対象ファイル名のハッシュを抽出）
pub fn parse_checksum(content: &str, target_filename: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        // 行フォーマット1: "<hash>  <filename>" または "<hash> *<filename>"
        if parts.len() >= 2 {
            let hash = parts[0];
            let fname = parts[1].trim_start_matches('*');
            if fname.eq_ignore_ascii_case(target_filename) {
                return Some(hash.to_lowercase());
            }
        }

        // 行フォーマット2: ファイル全体がハッシュ値のみ（単体 .sha256 ファイル）
        if parts.len() == 1 && parts[0].len() == 64 {
            return Some(parts[0].to_lowercase());
        }
    }
    None
}

/// チェックサムアセットの内容をダウンロードして取得
pub fn fetch_checksum_content(sha256_asset: &AssetInfo) -> Result<String> {
    let response = ureq::get(&sha256_asset.download_url)
        .set(
            "User-Agent",
            &format!("moult-update/{}", env!("CARGO_PKG_VERSION")),
        )
        .call()
        .map_err(|e| MoultError::Network(format!("Failed to fetch checksum: {}", e)))?;

    response
        .into_string()
        .map_err(|e| MoultError::Network(format!("Failed to read checksum content: {}", e)))
}

/// アセットを一時ディレクトリへダウンロードし、SHA-256 検証を実施後、ZIP解凍する
pub fn download_and_extract(
    asset: &AssetInfo,
    expected_sha256: Option<&str>,
    dest_dir: &Path,
    on_progress: Option<&dyn ProgressCallback>,
) -> Result<PathBuf> {
    fs::create_dir_all(dest_dir)?;

    let archive_path = dest_dir.join(&asset.name);
    let mut file = File::create(&archive_path)?;

    let response = ureq::get(&asset.download_url)
        .set(
            "User-Agent",
            &format!("moult-update/{}", env!("CARGO_PKG_VERSION")),
        )
        .call()
        .map_err(|e| MoultError::Network(format!("Failed to download {}: {}", asset.name, e)))?;

    let total_size = response
        .header("Content-Length")
        .and_then(|v| v.parse::<u64>().ok())
        .or(if asset.size > 0 {
            Some(asset.size)
        } else {
            None
        });

    let mut reader = response.into_reader();
    let mut buffer = [0u8; 16384]; // 16KB チャンク
    let mut downloaded: u64 = 0;
    let mut hasher = Sha256::new();

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .map_err(|e| MoultError::Network(format!("Download stream read error: {}", e)))?;

        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])?;
        hasher.update(&buffer[..bytes_read]);
        downloaded += bytes_read as u64;

        if let Some(cb) = on_progress {
            cb(downloaded, total_size);
        }
    }

    file.flush()?;
    drop(file);

    // SHA-256 チェックサムの照合
    let calculated_hash = format!("{:x}", hasher.finalize());
    if let Some(expected) = expected_sha256 {
        let clean_expected = expected.trim().to_lowercase();
        if clean_expected != calculated_hash {
            let _ = fs::remove_file(&archive_path);
            return Err(MoultError::ChecksumMismatch {
                expected: clean_expected,
                actual: calculated_hash,
            });
        }
    }

    // ZIP 展開
    let extract_dir = dest_dir.join("extracted");
    extract_zip(&archive_path, &extract_dir)?;

    // 元アーカイブの削除（ディスク容量節約）
    let _ = fs::remove_file(&archive_path);

    Ok(extract_dir)
}

/// ZIP アーカイブを安全に解凍（Zip Slip 脆弱性保護）
pub fn extract_zip(archive_path: &Path, extract_to: &Path) -> Result<()> {
    fs::create_dir_all(extract_to)?;

    let file = File::open(archive_path)?;
    let reader = BufReader::new(file);
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|e| MoultError::Zip(format!("Failed to read zip archive: {}", e)))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| MoultError::Zip(format!("Failed to inspect zip entry {}: {}", i, e)))?;

        let enclosed_name = entry
            .enclosed_name()
            .ok_or_else(|| {
                MoultError::Zip(format!("Malicious zip path traversal: {:?}", entry.name()))
            })?
            .to_path_buf();

        let out_path = extract_to.join(enclosed_name);

        if entry.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out_file = File::create(&out_path)?;
            std::io::copy(&mut entry, &mut out_file)
                .map_err(|e| MoultError::Zip(format!("Failed to extract {:?}: {}", out_path, e)))?;
        }
    }

    Ok(())
}
