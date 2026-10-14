//! 世代バックアップ（DiffLog）管理モジュール。
//!
//! ViVi 3 / 4 のキラー機能である DiffLog を実装します。
//! ファイル保存ごとに差分（Unified Diff）を `.difflog` ファイルに追記蓄積し、
//! ディスク消費を抑えながら任意世代への復元、世代一覧、
//! および各行の最終更新世代（簡易blame）を提供します。

use crate::diff::{DiffType, apply_unified_patch, create_unified_diff, diff_lines};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// 1世代分の DiffLog エントリ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLogEntry {
    /// 世代番号（1から始まる連番）
    pub generation: usize,
    /// 記録日時（タイムスタンプ文字列）
    pub timestamp: String,
    /// 変更要約または説明
    pub comment: String,
    /// Unified Diff パッチ文字列
    pub patch: String,
}

/// 世代バックアップログの管理構造体。
#[derive(Debug, Clone, Default)]
pub struct DiffLog {
    pub entries: Vec<DiffLogEntry>,
}

/// 行ごとの最終更新世代情報（簡易blame）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlameInfo {
    pub lineno: usize,
    pub generation: usize,
    pub timestamp: String,
    pub text: String,
}

impl DiffLog {
    /// 新しい空の DiffLog を生成します。
    pub fn new() -> Self {
        Self::default()
    }

    /// 対象ファイルパスから対応する `.difflog` のパスを取得します。
    pub fn difflog_path<P: AsRef<Path>>(file_path: P) -> PathBuf {
        let p = file_path.as_ref();
        let file_name = p.file_name().and_then(|n| n.to_str()).unwrap_or("untitled");
        let log_name = format!(".{}.difflog", file_name);
        match p.parent() {
            Some(parent) => parent.join(log_name),
            None => PathBuf::from(log_name),
        }
    }

    /// ファイル保存時に新世代の差分を `.difflog` に追記します。
    pub fn record_generation<P: AsRef<Path>>(
        file_path: P,
        old_text: &str,
        new_text: &str,
        timestamp: &str,
        comment: &str,
    ) -> Result<Option<usize>, String> {
        let patch = create_unified_diff(old_text, new_text, "prev", "current");
        if patch.is_empty() {
            // 差分がない場合は追記しない
            return Ok(None);
        }

        let log_path = Self::difflog_path(&file_path);
        let mut difflog = Self::load_from_file(&log_path).unwrap_or_default();
        let next_gen = difflog.entries.len() + 1;

        let entry = DiffLogEntry {
            generation: next_gen,
            timestamp: timestamp.to_string(),
            comment: comment.to_string(),
            patch,
        };

        difflog.entries.push(entry);
        difflog.save_to_file(&log_path)?;

        Ok(Some(next_gen))
    }

    /// `.difflog` ファイルを読み込みます。
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }

        let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| e.to_string())?;

        Self::parse(&content)
    }

    /// `.difflog` ファイルに保存します。
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let serialized = self.serialize();
        if let Some(parent) = path.as_ref().parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        file.write_all(serialized.as_bytes())
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// DiffLog をテキスト形式にシリアライズします。
    pub fn serialize(&self) -> String {
        let mut out = String::new();
        out.push_str("VIVIVI_DIFFLOG_V1\n");
        for entry in &self.entries {
            out.push_str(&format!(
                "=== GENERATION {} | {} | {} ===\n",
                entry.generation, entry.timestamp, entry.comment
            ));
            out.push_str(&entry.patch);
            if !entry.patch.ends_with('\n') {
                out.push('\n');
            }
        }
        out
    }

    /// テキストから DiffLog を解析・復元します。
    pub fn parse(content: &str) -> Result<Self, String> {
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() || lines[0].trim() != "VIVIVI_DIFFLOG_V1" {
            return Err("不正な DiffLog フォーマットです".into());
        }

        let mut entries = Vec::new();
        let mut current_entry: Option<DiffLogEntry> = None;
        let mut patch_lines = Vec::new();

        for &line in &lines[1..] {
            if line.starts_with("=== GENERATION ") && line.ends_with(" ===") {
                if let Some(mut prev) = current_entry.take() {
                    prev.patch = patch_lines.join("\n");
                    entries.push(prev);
                    patch_lines.clear();
                }

                let trimmed = &line[15..line.len() - 4];
                let parts: Vec<&str> = trimmed.split('|').map(|s| s.trim()).collect();
                let generation_num = parts[0].parse::<usize>().unwrap_or(entries.len() + 1);
                let ts = parts.get(1).copied().unwrap_or("").to_string();
                let cm = parts.get(2).copied().unwrap_or("").to_string();

                current_entry = Some(DiffLogEntry {
                    generation: generation_num,
                    timestamp: ts,
                    comment: cm,
                    patch: String::new(),
                });
            } else if current_entry.is_some() {
                patch_lines.push(line);
            }
        }

        if let Some(mut prev) = current_entry {
            prev.patch = patch_lines.join("\n");
            entries.push(prev);
        }

        Ok(Self { entries })
    }

    /// 初版のテキスト `initial_text` から指定世代 `target_gen` の内容を復元します。
    pub fn restore_from_initial(
        &self,
        initial_text: &str,
        target_gen: usize,
    ) -> Result<String, String> {
        let mut current = initial_text.to_string();
        for entry in &self.entries {
            if entry.generation > target_gen {
                break;
            }
            current = apply_unified_patch(&current, &entry.patch)?;
        }
        Ok(current)
    }

    /// 現在のテキストと DiffLog 履歴から、行ごとの最終更新世代（簡易blame）を計算します。
    pub fn compute_blame(&self, current_text: &str) -> Vec<BlameInfo> {
        let lines: Vec<&str> = current_text.lines().collect();
        let latest_gen = self.entries.last().map(|e| e.generation).unwrap_or(1);
        let latest_ts = self
            .entries
            .last()
            .map(|e| e.timestamp.clone())
            .unwrap_or_else(|| "initial".to_string());

        let mut blames = Vec::with_capacity(lines.len());
        for (i, line) in lines.iter().enumerate() {
            blames.push(BlameInfo {
                lineno: i + 1,
                generation: latest_gen,
                timestamp: latest_ts.clone(),
                text: line.to_string(),
            });
        }

        // 過去の各世代エントリの差分を逆向きに追跡して、行の生成世代を特定
        for entry in self.entries.iter().rev() {
            let hunk_lines = diff_lines(&entry.patch, current_text);
            for dl in hunk_lines {
                if dl.diff_type == DiffType::Equal
                    && let Some(nline) = dl.new_lineno
                    && nline <= blames.len()
                {
                    blames[nline - 1].generation = entry.generation;
                    blames[nline - 1].timestamp = entry.timestamp.clone();
                }
            }
        }

        blames
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difflog_record_and_restore() {
        let v1 = "Line 1\nLine 2\nLine 3\n";
        let v2 = "Line 1\nLine 2 (edited)\nLine 3\n";
        let v3 = "Line 1\nLine 2 (edited)\nLine 3\nLine 4 (new)\n";

        let mut log = DiffLog::new();

        let patch1 = create_unified_diff(v1, v2, "v1", "v2");
        log.entries.push(DiffLogEntry {
            generation: 1,
            timestamp: "2026-10-14 10:00:00".into(),
            comment: "edit line 2".into(),
            patch: patch1,
        });

        let patch2 = create_unified_diff(v2, v3, "v2", "v3");
        log.entries.push(DiffLogEntry {
            generation: 2,
            timestamp: "2026-10-14 11:00:00".into(),
            comment: "add line 4".into(),
            patch: patch2,
        });

        // シリアライズ & パースの検証
        let serialized = log.serialize();
        let parsed = DiffLog::parse(&serialized).expect("parse difflog");
        assert_eq!(parsed.entries.len(), 2);

        // 世代1の復元検証
        let restored_v2 = log.restore_from_initial(v1, 1).expect("restore v2");
        assert_eq!(restored_v2.trim(), v2.trim());

        // 世代2の復元検証
        let restored_v3 = log.restore_from_initial(v1, 2).expect("restore v3");
        assert_eq!(restored_v3.trim(), v3.trim());
    }
}
