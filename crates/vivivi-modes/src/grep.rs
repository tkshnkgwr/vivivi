//! プロジェクト内高速 Grep 検索モジュール。
//!
//! ViVi / OpenViVi の `:grep` コマンドに対応し、
//! 指定ディレクトリ配下のソースファイルを走査して正規表現マッチングを行い、
//! 該当行・列・マッチテキストの抽出および QuickFix 型ジャンプリストを提供します。

use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

/// 単一のマッチ結果。
///
/// ファイルパス、1-indexed の行番号および列番号、該当行テキスト、マッチ長を保持します。
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use vivivi_modes::grep::GrepMatch;
///
/// let m = GrepMatch {
///     file_path: PathBuf::from("src/main.rs"),
///     lineno: 10,
///     col: 5,
///     line_text: "    println!(\"hello\");".to_string(),
///     match_len: 7,
/// };
/// assert_eq!(m.lineno, 10);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrepMatch {
    /// 一致したファイルのパス
    pub file_path: PathBuf,
    /// 1-indexed の行番号
    pub lineno: usize,
    /// 1-indexed の列番号（文字単位）
    pub col: usize,
    /// 該当行の文字列
    pub line_text: String,
    /// マッチした文字列の長さ（文字単位）
    pub match_len: usize,
}

/// Grep 検索結果セッション（QuickFix リスト）。
///
/// マッチ一覧を保持し、`:cnext` / `:cprev` のような順方向・逆方向の循環ナビゲーションを提供します。
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use vivivi_modes::grep::{GrepMatch, GrepSession};
///
/// let mut session = GrepSession {
///     pattern: "fn".to_string(),
///     matches: vec![
///         GrepMatch {
///             file_path: PathBuf::from("a.rs"),
///             lineno: 1,
///             col: 1,
///             line_text: "fn main() {}".to_string(),
///             match_len: 2,
///         },
///     ],
///     current_idx: 0,
/// };
/// assert!(session.current_match().is_some());
/// ```
#[derive(Debug, Clone, Default)]
pub struct GrepSession {
    /// 検索に用いた正規表現パターン
    pub pattern: String,
    /// マッチ結果リスト
    pub matches: Vec<GrepMatch>,
    /// 現在選択されているマッチのインデックス（0-indexed）
    pub current_idx: usize,
}

impl GrepSession {
    /// 空の `GrepSession` を生成します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::grep::GrepSession;
    ///
    /// let session = GrepSession::new();
    /// assert!(session.matches.is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// 現在選択されているマッチの参照を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::grep::GrepSession;
    ///
    /// let session = GrepSession::new();
    /// assert!(session.current_match().is_none());
    /// ```
    pub fn current_match(&self) -> Option<&GrepMatch> {
        self.matches.get(self.current_idx)
    }

    /// 次のマッチへ進めます（`:cnext` / `:cn`）。末尾に達した場合は先頭へループします。
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use vivivi_modes::grep::{GrepMatch, GrepSession};
    ///
    /// let mut session = GrepSession {
    ///     pattern: "a".into(),
    ///     matches: vec![
    ///         GrepMatch {
    ///             file_path: PathBuf::from("a.rs"),
    ///             lineno: 1,
    ///             col: 1,
    ///             line_text: "a".into(),
    ///             match_len: 1,
    ///         },
    ///         GrepMatch {
    ///             file_path: PathBuf::from("b.rs"),
    ///             lineno: 2,
    ///             col: 1,
    ///             line_text: "a".into(),
    ///             match_len: 1,
    ///         },
    ///     ],
    ///     current_idx: 0,
    /// };
    /// assert_eq!(session.next().unwrap().lineno, 2);
    /// assert_eq!(session.next().unwrap().lineno, 1); // ループ
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<&GrepMatch> {
        if self.matches.is_empty() {
            return None;
        }
        if self.current_idx + 1 < self.matches.len() {
            self.current_idx += 1;
        } else {
            self.current_idx = 0; // ループ
        }
        self.current_match()
    }

    /// 前のマッチへ戻します（`:cprev` / `:cp`）。先頭に達した場合は末尾へループします。
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use vivivi_modes::grep::{GrepMatch, GrepSession};
    ///
    /// let mut session = GrepSession {
    ///     pattern: "a".into(),
    ///     matches: vec![
    ///         GrepMatch {
    ///             file_path: PathBuf::from("a.rs"),
    ///             lineno: 1,
    ///             col: 1,
    ///             line_text: "a".into(),
    ///             match_len: 1,
    ///         },
    ///         GrepMatch {
    ///             file_path: PathBuf::from("b.rs"),
    ///             lineno: 2,
    ///             col: 1,
    ///             line_text: "a".into(),
    ///             match_len: 1,
    ///         },
    ///     ],
    ///     current_idx: 0,
    /// };
    /// assert_eq!(session.prev().unwrap().lineno, 2); // 先頭から末尾へループ
    /// ```
    pub fn prev(&mut self) -> Option<&GrepMatch> {
        if self.matches.is_empty() {
            return None;
        }
        if self.current_idx > 0 {
            self.current_idx -= 1;
        } else {
            self.current_idx = self.matches.len() - 1; // ループ
        }
        self.current_match()
    }

    /// 指定インデックスのマッチを選択します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::grep::GrepSession;
    ///
    /// let mut session = GrepSession::new();
    /// assert!(session.select(5).is_none());
    /// ```
    pub fn select(&mut self, idx: usize) -> Option<&GrepMatch> {
        if idx < self.matches.len() {
            self.current_idx = idx;
            self.current_match()
        } else {
            None
        }
    }
}

/// バイナリファイル（先頭1KBに NUL 文字が含まれる等）かを判定します。
///
/// # Examples
///
/// ```rust,no_run
/// use vivivi_modes::grep::is_binary_file;
///
/// let is_bin = is_binary_file("path/to/binary.png");
/// ```
pub fn is_binary_file<P: AsRef<Path>>(path: P) -> bool {
    if let Ok(mut f) = fs::File::open(path) {
        use std::io::Read;
        let mut buf = [0u8; 1024];
        if let Ok(n) = f.read(&mut buf) {
            return buf[..n].contains(&0);
        }
    }
    false
}

/// プロジェクトディレクトリ配下を高速走査し、正規表現に一致する行を抽出します。
///
/// 隠しディレクトリ（`.` 始まり）やビルド成果物（`target/`, `node_modules/` など）、
/// およびバイナリファイルは自動的に除外されます。
///
/// # Parameters
/// - `root_dir`: 検索起点となるルートディレクトリパス。
/// - `pattern`: 検索する正規表現パターン文字列。
/// - `ignore_case`: 大文字小文字を区別しないか (`true`)。
/// - `target_exts`: 対象拡張子スライス（空の場合は全テキストファイルを走査）。
///
/// # Errors
/// ルートパスがディレクトリでない場合、または正規表現パターンが無効な場合にエラーメッセージを返します。
///
/// # Examples
///
/// ```rust,no_run
/// use vivivi_modes::grep::execute_grep;
///
/// let session = execute_grep(".", "fn main", false, &["rs"]).unwrap();
/// println!("Found {} matches", session.matches.len());
/// ```
pub fn execute_grep<P: AsRef<Path>>(
    root_dir: P,
    pattern: &str,
    ignore_case: bool,
    target_exts: &[&str],
) -> Result<GrepSession, String> {
    let root = root_dir.as_ref();
    if !root.is_dir() {
        return Err("指定されたパスはディレクトリではありません".into());
    }

    let regex_pattern = if ignore_case {
        format!("(?i){}", pattern)
    } else {
        pattern.to_string()
    };
    let re = Regex::new(&regex_pattern).map_err(|e| format!("無効な正規表現: {}", e))?;

    let mut matches = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(current) = stack.pop() {
        let entries = match fs::read_dir(&current) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();

            // 隠しフォルダ・ビルド成果物ディレクトリの除外
            if file_name.starts_with('.')
                || file_name == "target"
                || file_name == "node_modules"
                || file_name == "build"
                || file_name == "dist"
            {
                continue;
            }

            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or_default()
                    .to_lowercase();

                if !target_exts.is_empty()
                    && !target_exts.iter().any(|&e| e.eq_ignore_ascii_case(&ext))
                {
                    continue;
                }

                if is_binary_file(&path) {
                    continue;
                }

                if let Ok(content) = fs::read_to_string(&path) {
                    for (lineno, line) in content.lines().enumerate() {
                        for mat in re.find_iter(line) {
                            let col = line[..mat.start()].chars().count() + 1;
                            matches.push(GrepMatch {
                                file_path: path.clone(),
                                lineno: lineno + 1,
                                col,
                                line_text: line.to_string(),
                                match_len: mat.as_str().chars().count(),
                            });
                        }
                    }
                }
            }
        }
    }

    // ファイルパス・行番号順に整列
    matches.sort_by(|a, b| {
        a.file_path
            .cmp(&b.file_path)
            .then_with(|| a.lineno.cmp(&b.lineno))
            .then_with(|| a.col.cmp(&b.col))
    });

    Ok(GrepSession {
        pattern: pattern.to_string(),
        matches,
        current_idx: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grep_session_nav() {
        let mut session = GrepSession {
            pattern: "test".into(),
            matches: vec![
                GrepMatch {
                    file_path: PathBuf::from("a.rs"),
                    lineno: 10,
                    col: 5,
                    line_text: "test 1".into(),
                    match_len: 4,
                },
                GrepMatch {
                    file_path: PathBuf::from("b.rs"),
                    lineno: 20,
                    col: 1,
                    line_text: "test 2".into(),
                    match_len: 4,
                },
            ],
            current_idx: 0,
        };

        assert_eq!(session.current_match().unwrap().lineno, 10);
        session.next();
        assert_eq!(session.current_match().unwrap().lineno, 20);
        session.next();
        assert_eq!(session.current_match().unwrap().lineno, 10); // ループ
        session.prev();
        assert_eq!(session.current_match().unwrap().lineno, 20);
    }
}
