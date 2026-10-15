//! プロジェクト管理（Project View）モジュール。
//!
//! ViVi 3 / ViVi 4 のプロジェクト管理機能に基づき、
//! 複数ファイル・ディレクトリの統合管理、一括 tags 生成、
//! およびプロジェクト内変更の一括バックアップを提供します。

use std::fs;
use std::path::{Path, PathBuf};

/// プロジェクト内のファイル一覧およびタグ生成を管理するマネージャ。
///
/// プロジェクトのルートディレクトリを起点として、開かれているファイルの追跡や、
/// ソースコードから ctags 互換の `tags` ファイルを一括生成します。
///
/// # Examples
/// ```
/// use vivivi_modes::project::ProjectManager;
///
/// let mut pm = ProjectManager::new(".");
/// pm.add_file("src/lib.rs");
/// assert_eq!(pm.open_files.len(), 1);
/// ```
#[derive(Debug, Clone, Default)]
pub struct ProjectManager {
    /// プロジェクトの基底（ルート）ディレクトリパス
    pub root_dir: PathBuf,
    /// プロジェクトに登録・開かれているファイル一覧
    pub open_files: Vec<PathBuf>,
}

impl ProjectManager {
    /// ルートディレクトリを指定して新しいプロジェクトマネージャを生成します。
    ///
    /// # 引数
    /// - `root`: プロジェクトのルートディレクトリパス
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::project::ProjectManager;
    ///
    /// let pm = ProjectManager::new(".");
    /// assert_eq!(pm.root_dir, std::path::PathBuf::from("."));
    /// assert!(pm.open_files.is_empty());
    /// ```
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root_dir: root.as_ref().to_path_buf(),
            open_files: Vec::new(),
        }
    }

    /// プロジェクトの管理対象ファイル一覧に指定パスを追加します。
    ///
    /// 既に登録済みのパスである場合は重複追加されません。
    ///
    /// # 引数
    /// - `path`: 追加するファイルのパス
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::project::ProjectManager;
    ///
    /// let mut pm = ProjectManager::new(".");
    /// pm.add_file("crates/vivivi-modes/src/lib.rs");
    /// pm.add_file("crates/vivivi-modes/src/lib.rs"); // 重複は無視
    /// assert_eq!(pm.open_files.len(), 1);
    /// ```
    pub fn add_file<P: AsRef<Path>>(&mut self, path: P) {
        let p = path.as_ref().to_path_buf();
        if !self.open_files.contains(&p) {
            self.open_files.push(p);
        }
    }

    /// プロジェクトの管理対象ファイル一覧から指定パスを除外します。
    ///
    /// # 引数
    /// - `path`: 除外するファイルのパス
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::project::ProjectManager;
    ///
    /// let mut pm = ProjectManager::new(".");
    /// pm.add_file("temp.rs");
    /// pm.remove_file("temp.rs");
    /// assert!(pm.open_files.is_empty());
    /// ```
    pub fn remove_file<P: AsRef<Path>>(&mut self, path: P) {
        let p = path.as_ref();
        self.open_files.retain(|f| f != p);
    }

    /// プロジェクト配下のソースファイルを再帰走査し、ctags 互換の `tags` ファイルを生成・保存します。
    ///
    /// Rust, C/C++, Python, JavaScript/TypeScript の関数・構造体・クラス定義を検出し、
    /// プロジェクトのルートディレクトリ直下に `tags` ファイルとして書き出します。
    /// 隠しディレクトリ（`.` 始まり）、`target`、`node_modules` は自動的に除外されます。
    ///
    /// # 戻り値
    /// 検出・出力されたシンボルタグの総件数
    ///
    /// # Errors
    /// `tags` ファイルの書き込みに失敗した場合にエラー文字列を返します。
    ///
    /// # Examples
    /// ```rust,no_run
    /// use vivivi_modes::project::ProjectManager;
    ///
    /// let pm = ProjectManager::new(".");
    /// let count = pm.generate_tags().unwrap();
    /// println!("生成されたタグ件数: {}", count);
    /// ```
    pub fn generate_tags(&self) -> Result<usize, String> {
        let mut tag_lines = Vec::new();
        tag_lines.push("!_TAG_FILE_FORMAT\t2\t/extended format/".to_string());
        tag_lines.push("!_TAG_FILE_SORTED\t1\t/0=unsorted, 1=sorted/".to_string());

        let mut stack = vec![self.root_dir.clone()];
        let mut count = 0;

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

                if file_name.starts_with('.')
                    || file_name == "target"
                    || file_name == "node_modules"
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

                    if matches!(
                        ext.as_str(),
                        "rs" | "c" | "cpp" | "h" | "hpp" | "py" | "js" | "ts"
                    ) && let Ok(content) = fs::read_to_string(&path)
                    {
                        let rel_path = path
                            .strip_prefix(&self.root_dir)
                            .unwrap_or(&path)
                            .to_string_lossy()
                            .replace('\\', "/");

                        for (lineno, line) in content.lines().enumerate() {
                            let trimmed = line.trim();
                            if trimmed.starts_with("fn ")
                                || trimmed.starts_with("pub fn ")
                                || trimmed.starts_with("struct ")
                                || trimmed.starts_with("pub struct ")
                                || trimmed.starts_with("def ")
                                || trimmed.starts_with("class ")
                            {
                                // 識別子の抽出
                                let words: Vec<&str> = trimmed.split_whitespace().collect();
                                let mut tag_name = "";
                                for (w_idx, &w) in words.iter().enumerate() {
                                    if (w == "fn" || w == "struct" || w == "def" || w == "class")
                                        && w_idx + 1 < words.len()
                                    {
                                        tag_name = words[w_idx + 1]
                                            .split(&['(', '<', ':', '{'][..])
                                            .next()
                                            .unwrap_or("");
                                        break;
                                    }
                                }

                                if !tag_name.is_empty() {
                                    let tag_entry =
                                        format!("{}\t{}\t{}", tag_name, rel_path, lineno + 1);
                                    tag_lines.push(tag_entry);
                                    count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        // タグ名昇順ソート
        if tag_lines.len() > 2 {
            tag_lines[2..].sort();
        }

        let tags_content = tag_lines.join("\n") + "\n";
        let tags_path = self.root_dir.join("tags");
        fs::write(tags_path, tags_content).map_err(|e| e.to_string())?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_manager_add_remove() {
        let mut pm = ProjectManager::new(".");
        pm.add_file("src/main.rs");
        assert_eq!(pm.open_files.len(), 1);
        pm.remove_file("src/main.rs");
        assert_eq!(pm.open_files.len(), 0);
    }
}
