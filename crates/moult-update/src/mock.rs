//! テスト容易性のためのモック生成・シミュレーション支援

use crate::check::{AssetInfo, ReleaseInfo};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

/// テスト用のダミーリリースアセット一式をローカルディレクトリに生成
pub struct MockReleaseBuilder {
    pub version: String,
    pub target_bin_name: String,
    pub target_bin_content: Vec<u8>,
    pub pair_bin_name: Option<String>,
    pub pair_bin_content: Option<Vec<u8>>,
}

impl MockReleaseBuilder {
    /// 新規ビルダーを作成
    pub fn new(version: impl Into<String>, target_bin_name: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            target_bin_name: target_bin_name.into(),
            target_bin_content: b"MOCK_NEW_BINARY_DATA".to_vec(),
            pair_bin_name: None,
            pair_bin_content: None,
        }
    }

    /// ペアバイナリを追加
    pub fn with_pair_bin(mut self, name: impl Into<String>, content: impl Into<Vec<u8>>) -> Self {
        self.pair_bin_name = Some(name.into());
        self.pair_bin_content = Some(content.into());
        self
    }

    /// ターゲットバイナリの内容をカスタマイズ
    pub fn with_target_content(mut self, content: impl Into<Vec<u8>>) -> Self {
        self.target_bin_content = content.into();
        self
    }

    /// 指定ディレクトリに ZIP アーカイブと SHA256SUMS を生成し、ReleaseInfo を返す
    pub fn build(self, output_dir: &Path) -> (ReleaseInfo, PathBuf, String) {
        fs::create_dir_all(output_dir).unwrap();

        let zip_name = format!("mock-release-{}-windows-x86_64.zip", self.version);
        let zip_path = output_dir.join(&zip_name);

        // ZIP 作成
        let file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // ターゲットバイナリ追加
        zip.start_file(&self.target_bin_name, options).unwrap();
        zip.write_all(&self.target_bin_content).unwrap();

        // ペアバイナリ追加
        if let (Some(pair_name), Some(pair_content)) = (self.pair_bin_name, self.pair_bin_content) {
            zip.start_file(&pair_name, options).unwrap();
            zip.write_all(&pair_content).unwrap();
        }

        zip.finish().unwrap();

        // SHA-256 チェックサム計算
        let zip_bytes = fs::read(&zip_path).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(&zip_bytes);
        let sha256_hash = format!("{:x}", hasher.finalize());

        let sha256_filename = format!("{}.sha256", zip_name);
        let sha256_content = format!("{}  {}\n", sha256_hash, zip_name);
        let sha256_path = output_dir.join(&sha256_filename);
        fs::write(&sha256_path, &sha256_content).unwrap();

        let release = ReleaseInfo {
            version: self.version.clone(),
            tag_name: format!("v{}", self.version),
            name: format!("Version {}", self.version),
            body: "Mock Release Notes".to_string(),
            html_url: format!(
                "https://github.com/mock/mock/releases/tag/v{}",
                self.version
            ),
            assets: vec![
                AssetInfo {
                    name: zip_name,
                    download_url: format!(
                        "file://{}",
                        zip_path.to_string_lossy().replace('\\', "/")
                    ),
                    size: zip_bytes.len() as u64,
                },
                AssetInfo {
                    name: sha256_filename,
                    download_url: format!(
                        "file://{}",
                        sha256_path.to_string_lossy().replace('\\', "/")
                    ),
                    size: sha256_content.len() as u64,
                },
            ],
            published_at: Some("2026-11-09T00:00:00Z".to_string()),
        };

        (release, zip_path, sha256_hash)
    }
}
