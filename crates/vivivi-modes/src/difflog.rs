//! 世代バックアップ（DiffLog）管理モジュール。
//!
//! ViVi 3 / 4 のキラー機能である DiffLog を実装します。
//! ファイル保存ごとに差分（Unified Diff）を `.difflog` ファイルに追記蓄積し、
//! ディスク消費を抑えながら任意世代への復元、世代一覧、
//! および各行の最終更新世代（簡易blame）を提供します。

use crate::diff::{apply_unified_patch, create_unified_diff};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// 1世代分の DiffLog エントリ。
///
/// 世代番号、記録日時、コメント、および Unified Diff パッチ文字列を保持します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::difflog::DiffLogEntry;
///
/// let entry = DiffLogEntry {
///     generation: 1,
///     timestamp: "2026-01-01 12:00:00".to_string(),
///     comment: "Initial revision".to_string(),
///     patch: String::new(),
/// };
/// assert_eq!(entry.generation, 1);
/// ```
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
///
/// 各世代の差分パッチを累積管理し、任意世代の復元や行ごとの更新世代（blame）判定を提供します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::difflog::DiffLog;
///
/// let log = DiffLog::new();
/// assert!(log.entries.is_empty());
/// ```
#[derive(Debug, Clone, Default)]
pub struct DiffLog {
    /// 世代エントリリスト
    pub entries: Vec<DiffLogEntry>,
}

/// 行ごとの最終更新世代情報（簡易blame）。
///
/// # Examples
///
/// ```
/// use vivivi_modes::difflog::BlameInfo;
///
/// let blame = BlameInfo {
///     lineno: 1,
///     generation: 2,
///     timestamp: "2026-01-01 12:30:00".to_string(),
///     text: "let a = 1;".to_string(),
/// };
/// assert_eq!(blame.generation, 2);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlameInfo {
    /// 1-indexed の行番号
    pub lineno: usize,
    /// 最終更新世代番号
    pub generation: usize,
    /// 最終更新日時
    pub timestamp: String,
    /// 行文字列
    pub text: String,
}

impl DiffLog {
    /// 新しい空の `DiffLog` を生成します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::difflog::DiffLog;
    ///
    /// let log = DiffLog::new();
    /// assert_eq!(log.entries.len(), 0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// 対象ファイルパスから対応する隠し世代ログファイル（`.<filename>.difflog`）のパスを算出します。
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use vivivi_modes::difflog::DiffLog;
    ///
    /// let path = DiffLog::difflog_path("src/main.rs");
    /// assert_eq!(path.file_name().unwrap(), ".main.rs.difflog");
    /// ```
    pub fn difflog_path<P: AsRef<Path>>(file_path: P) -> PathBuf {
        let p = file_path.as_ref();
        let file_name = p.file_name().and_then(|n| n.to_str()).unwrap_or("untitled");
        let log_name = format!(".{}.difflog", file_name);
        match p.parent() {
            Some(parent) => parent.join(log_name),
            None => PathBuf::from(log_name),
        }
    }

    /// ファイル保存時に新旧テキストの差分を算出し、`.difflog` ファイルに新世代として追記します。
    ///
    /// 差分が全く存在しない場合は追記せず `Ok(None)` を返します。
    ///
    /// # Errors
    /// ファイルの読み書きに失敗した場合にエラーメッセージを返します。
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use vivivi_modes::difflog::DiffLog;
    ///
    /// let generation = DiffLog::record_generation("test.txt", "old", "new", "2026-01-01", "save").unwrap();
    /// ```
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
        let next_gen = difflog
            .entries
            .last()
            .map(|e| e.generation + 1)
            .unwrap_or(1);

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

    /// `.difflog` ファイルを読み込み、`DiffLog` 構造体として復元します。ファイルが存在しない場合は空のログを返します。
    ///
    /// # Errors
    /// ファイルの読み込みに失敗した場合、またはフォーマットが不正な場合にエラーメッセージを返します。
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use vivivi_modes::difflog::DiffLog;
    ///
    /// let log = DiffLog::load_from_file(".main.rs.difflog").unwrap();
    /// ```
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

    /// `.difflog` ファイルに現在の世代履歴を保存（シリアライズ）します。
    ///
    /// # Errors
    /// ファイル書き込みに失敗した場合にエラーメッセージを返します。
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use vivivi_modes::difflog::DiffLog;
    ///
    /// let log = DiffLog::new();
    /// log.save_to_file(".main.rs.difflog").unwrap();
    /// ```
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

    /// DiffLog をテキスト形式（`VIVIVI_DIFFLOG_V1`）にシリアライズします。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::difflog::DiffLog;
    ///
    /// let log = DiffLog::new();
    /// let text = log.serialize();
    /// assert!(text.starts_with("VIVIVI_DIFFLOG_V1\n"));
    /// ```
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

    /// シリアライズされたテキストから `DiffLog` を解析・復元します。
    ///
    /// # Errors
    /// ヘッダーが `VIVIVI_DIFFLOG_V1` で始まらないなど、フォーマットが不正な場合にエラーメッセージを返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::difflog::DiffLog;
    ///
    /// let parsed = DiffLog::parse("VIVIVI_DIFFLOG_V1\n").unwrap();
    /// assert!(parsed.entries.is_empty());
    /// ```
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
    ///
    /// 第1世代から `target_gen` までの差分パッチを順次適用してテキストを再構築します。
    ///
    /// # Errors
    /// パッチの適用に失敗した場合にエラーメッセージを返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::difflog::{DiffLog, DiffLogEntry};
    /// use vivivi_modes::diff::create_unified_diff;
    ///
    /// let v1 = "a\nb\n";
    /// let v2 = "a\nB\n";
    /// let patch = create_unified_diff(v1, v2, "v1", "v2");
    ///
    /// let mut log = DiffLog::new();
    /// log.entries.push(DiffLogEntry {
    ///     generation: 1,
    ///     timestamp: "2026-01-01".into(),
    ///     comment: "update".into(),
    ///     patch,
    /// });
    ///
    /// let restored = log.restore_from_initial(v1, 1).unwrap();
    /// assert_eq!(restored, v2);
    /// ```
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

    /// 現在のテキストと DiffLog 履歴から、行ごとの最終更新世代（簡易blame）を算出します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::difflog::DiffLog;
    ///
    /// let log = DiffLog::new();
    /// let blames = log.compute_blame("line 1\nline 2\n");
    /// assert_eq!(blames.len(), 2);
    /// assert_eq!(blames[0].lineno, 1);
    /// ```
    pub fn compute_blame(&self, current_text: &str) -> Vec<BlameInfo> {
        let lines: Vec<&str> = current_text.lines().collect();
        let first_gen = self.entries.first().map(|e| e.generation).unwrap_or(1);
        let first_ts = self
            .entries
            .first()
            .map(|e| e.timestamp.clone())
            .unwrap_or_else(|| "initial".to_string());

        let mut blames = Vec::with_capacity(lines.len());
        for (i, line) in lines.iter().enumerate() {
            blames.push(BlameInfo {
                lineno: i + 1,
                generation: first_gen,
                timestamp: first_ts.clone(),
                text: line.to_string(),
            });
        }

        // 各世代のパッチで導入された行 (+ で始まる行) をその世代としてマーク
        for entry in &self.entries {
            for pline in entry.patch.lines() {
                if pline.starts_with('+') && !pline.starts_with("+++") {
                    let added_text = &pline[1..];
                    for blame in &mut blames {
                        if blame.text == added_text {
                            blame.generation = entry.generation;
                            blame.timestamp = entry.timestamp.clone();
                        }
                    }
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

        // blame 計算の検証
        let blames = log.compute_blame(v3);
        assert_eq!(blames.len(), 4);
        assert_eq!(blames[1].text, "Line 2 (edited)");
        assert_eq!(blames[1].generation, 1);
        assert_eq!(blames[3].text, "Line 4 (new)");
        assert_eq!(blames[3].generation, 2);
    }
}
