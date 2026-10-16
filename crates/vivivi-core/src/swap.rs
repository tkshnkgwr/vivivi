//! クラッシュリカバリ・スワップファイル (`.swp`) 管理モジュール。
//!
//! エディタの編集中バッファを定期的にスワップファイル（`.<filename>.swp`）へ保存し、
//! クラッシュや異常終了後の次回起動時に未保存データの検出・復元を行います。

use crate::encoding::{CharEncoding, NewlineType};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const SWAP_MAGIC: &[u8] = b"VIVIVI_SWP_V1\n";

/// スワップファイルから復元されたレコード情報。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwapRecord {
    /// 元ファイルのパス（無名バッファの場合は `None`）
    pub original_path: Option<PathBuf>,
    /// 保存時の UNIX エポック秒
    pub timestamp: u64,
    /// 文字エンコーディング
    pub encoding: CharEncoding,
    /// 改行コード種別
    pub newline: NewlineType,
    /// 保存されていたテキスト内容
    pub content: String,
}

/// スワップファイルの読み書き・管理を行う構造体。
pub struct SwapFile;

impl SwapFile {
    /// 対象ファイルに対応するスワップファイルのパス（`.<filename>.swp`）を算出します。
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use vivivi_core::swap::SwapFile;
    ///
    /// let p = Path::new("src/main.rs");
    /// let swp = SwapFile::swap_path_for(p);
    /// assert_eq!(swp.file_name().unwrap(), ".main.rs.swp");
    /// ```
    pub fn swap_path_for(file_path: &Path) -> PathBuf {
        let parent = file_path.parent().unwrap_or_else(|| Path::new(""));
        let file_name = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unnamed".to_string());
        parent.join(format!(".{file_name}.swp"))
    }

    /// スワップファイルが存在するかどうかを判定します。
    pub fn exists_for(file_path: &Path) -> bool {
        Self::swap_path_for(file_path).exists()
    }

    /// スワップファイルへ現在のバッファ内容とメタデータを書き込みます。
    pub fn save(
        swap_path: &Path,
        original_path: Option<&Path>,
        text: &str,
        encoding: CharEncoding,
        newline: NewlineType,
    ) -> std::io::Result<()> {
        let mut file = File::create(swap_path)?;
        file.write_all(SWAP_MAGIC)?;

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        writeln!(file, "timestamp: {ts}")?;

        if let Some(p) = original_path {
            writeln!(file, "original_path: {}", p.display())?;
        } else {
            writeln!(file, "original_path: None")?;
        }

        writeln!(file, "encoding: {}", encoding.name())?;
        writeln!(file, "newline: {}", newline.name())?;
        writeln!(file, "content_bytes: {}", text.len())?;
        file.write_all(b"---CONTENT_START---\n")?;
        file.write_all(text.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    /// スワップファイルからメタデータとテキスト内容を復元します。
    pub fn load(swap_path: &Path) -> std::io::Result<SwapRecord> {
        let file = File::open(swap_path)?;
        let mut reader = BufReader::new(file);

        let mut magic = String::new();
        reader.read_line(&mut magic)?;
        if magic.as_bytes() != SWAP_MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "無効なスワップファイル形式です",
            ));
        }

        let mut timestamp = 0u64;
        let mut original_path = None;
        let mut encoding = CharEncoding::Utf8;
        let mut newline = NewlineType::Lf;

        let mut line = String::new();
        while reader.read_line(&mut line)? > 0 {
            let trimmed = line.trim();
            if trimmed == "---CONTENT_START---" {
                break;
            }
            if let Some(ts_str) = trimmed.strip_prefix("timestamp: ") {
                timestamp = ts_str.parse().unwrap_or(0);
            } else if let Some(orig_str) = trimmed.strip_prefix("original_path: ")
                && orig_str != "None"
            {
                original_path = Some(PathBuf::from(orig_str));
            } else if let Some(enc_str) = trimmed.strip_prefix("encoding: ")
                && let Some(enc) = CharEncoding::parse_enc(enc_str)
            {
                encoding = enc;
            } else if let Some(nl_str) = trimmed.strip_prefix("newline: ")
                && let Some(nl) = NewlineType::parse_ff(nl_str)
            {
                newline = nl;
            }
            line.clear();
        }

        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        Ok(SwapRecord {
            original_path,
            timestamp,
            encoding,
            newline,
            content,
        })
    }

    /// スワップファイルを安全に削除します。
    pub fn remove(swap_path: &Path) -> std::io::Result<()> {
        if swap_path.exists() {
            std::fs::remove_file(swap_path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_path_generation() {
        let file = Path::new("my_project/src/code.rs");
        let swp = SwapFile::swap_path_for(file);
        assert_eq!(swp, Path::new("my_project/src/.code.rs.swp"));
    }

    #[test]
    fn test_swap_save_and_load_roundtrip() {
        let temp_dir = std::env::temp_dir();
        let swp_path = temp_dir.join(".vivivi_test_recovery.swp");
        let orig_path = Path::new("test_dir/sample.txt");
        let sample_text = "第1行\n第2行: 日本語テキスト\n第3行\n";

        SwapFile::save(
            &swp_path,
            Some(orig_path),
            sample_text,
            CharEncoding::Utf8,
            NewlineType::Lf,
        )
        .expect("スワップ保存成功");

        assert!(swp_path.exists());

        let record = SwapFile::load(&swp_path).expect("スワップ復元成功");
        assert_eq!(record.original_path, Some(orig_path.to_path_buf()));
        assert_eq!(record.encoding, CharEncoding::Utf8);
        assert_eq!(record.newline, NewlineType::Lf);
        assert_eq!(record.content, sample_text);

        SwapFile::remove(&swp_path).expect("スワップ削除成功");
        assert!(!swp_path.exists());
    }
}
