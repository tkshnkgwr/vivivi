//! 拡張子別テンプレート自動読み込み・マクロ展開モジュール。
//!
//! 新規ファイル作成時に拡張子に応じたテンプレート（`.template.<ext>`）を自動探索し、
//! `%FileName%`, `%Date%`, `%UserName%` 等のマクロを展開して初期バッファを生成します。

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// テンプレートマクロ展開結果。
///
/// プレースホルダー置換後のテキストと、`%Cursor%` マーカーで指定された初期キャレット位置を保持します。
///
/// # Examples
/// ```
/// use vivivi_modes::template::TemplateResult;
///
/// let res = TemplateResult {
///     text: "fn main() {}\n".to_string(),
///     cursor_pos: 10,
/// };
/// assert_eq!(res.cursor_pos, 10);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateResult {
    /// マクロ展開後の完成テキスト
    pub text: String,
    /// `%Cursor%` が指定されていた場合の初期カーソルオフセット（文字位置）
    pub cursor_pos: usize,
}

/// 拡張子別テンプレートを管理・展開するマネージャ。
///
/// メモリ内に登録されたカスタムテンプレート文字列や、ファイルシステム上の
/// `.template.<ext>` ファイルからテンプレートを読み込み、マクロ置換を行います。
///
/// # Examples
/// ```
/// use vivivi_modes::template::TemplateManager;
///
/// let mut mgr = TemplateManager::new();
/// mgr.register_template("rs", "// %FileName%\n");
/// let res = mgr.load_and_expand("test.rs", "2026-10-15", "10:00").unwrap();
/// assert_eq!(res.text, "// test.rs\n");
/// ```
#[derive(Debug, Clone, Default)]
pub struct TemplateManager {
    /// メモリ上に保持するビルトインまたは登録済みテンプレートマップ（拡張子 -> テンプレート内容）
    custom_templates: HashMap<String, String>,
}

impl TemplateManager {
    /// 空のテンプレートマネージャを生成します。
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::template::TemplateManager;
    ///
    /// let mgr = TemplateManager::new();
    /// assert!(mgr.load_and_expand("sample.unknown", "2026-10-15", "10:00").is_none());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// 指定した拡張子に対するメモリ内カスタムテンプレート文字列を登録します。
    ///
    /// 拡張子文字列の先頭のドット `.` は自動的に無視され、大文字小文字は区別されません。
    ///
    /// # 引数
    /// - `ext`: 拡張子（例: `"rs"`, `".cpp"`）
    /// - `content`: テンプレート本文文字列
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::template::TemplateManager;
    ///
    /// let mut mgr = TemplateManager::new();
    /// mgr.register_template(".py", "#!/usr/bin/env python3\n# %FileName%\n");
    /// ```
    pub fn register_template(&mut self, ext: &str, content: &str) {
        self.custom_templates.insert(
            ext.trim_start_matches('.').to_lowercase(),
            content.to_string(),
        );
    }

    /// 対象ファイルパスに対応するテンプレートファイルをファイルシステムから探索します。
    ///
    /// 以下の順序で候補ファイルを検索します：
    /// 1. 同一ディレクトリ内の `.template.<ext>`
    /// 2. ユーザーホームディレクトリ配下の `~/.vivivi/templates/.template.<ext>`
    /// 3. ユーザーホームディレクトリ配下の `~/.vivivi/templates/template.<ext>`
    ///
    /// # 引数
    /// - `file_path`: 作成対象の新規ファイルパス
    ///
    /// # 戻り値
    /// 発見されたテンプレートファイルのパス。見つからない場合は `None`
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::template::TemplateManager;
    ///
    /// let template_path = TemplateManager::find_template_file("nonexistent/dummy_file.unknown");
    /// assert!(template_path.is_none());
    /// ```
    pub fn find_template_file<P: AsRef<Path>>(file_path: P) -> Option<PathBuf> {
        let p = file_path.as_ref();
        let ext = p.extension()?.to_str()?.to_lowercase();

        // 1. 同一ディレクトリ内の .template.<ext>
        if let Some(parent) = p.parent() {
            let cand = parent.join(format!(".template.{}", ext));
            if cand.is_file() {
                return Some(cand);
            }
        }

        // 2. ユーザーホーム/.vivivi/templates/.template.<ext>
        if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
            let template_dir = PathBuf::from(home).join(".vivivi").join("templates");
            let candidates = [
                template_dir.join(format!(".template.{}", ext)),
                template_dir.join(format!("template.{}", ext)),
            ];
            for cand in candidates {
                if cand.is_file() {
                    return Some(cand);
                }
            }
        }

        None
    }

    /// ファイルパスおよび日付・時刻情報に基づき、テンプレート文字列内のマクロを展開します。
    ///
    /// 対応しているマクロ一覧：
    /// - `%FileName%`: ファイル名（例: `main.rs`）
    /// - `%FileBase%`: 拡張子を除いたファイル名（例: `main`）
    /// - `%Ext%`: 拡張子（例: `rs`）
    /// - `%Date%`: 指定された日付文字列
    /// - `%Time%`: 指定された時刻文字列
    /// - `%UserName%`, `%Author%`: 環境変数 `USERNAME` または `USER` の値
    /// - `%Guard%`: C/C++インクルードガード用文字列（例: `FOO_H_`）
    /// - `%Cursor%`: 展開後の初期カーソル位置を指定し、テキスト中からは削除
    ///
    /// # 引数
    /// - `file_path`: 新規ファイルのパス
    /// - `template_raw`: 展開前のテンプレート生文字列
    /// - `date_str`: 置換用日付文字列
    /// - `time_str`: 置換用時刻文字列
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::template::TemplateManager;
    ///
    /// let mgr = TemplateManager::new();
    /// let raw = "/* File: %FileName% */\nfn main() { %Cursor% }";
    /// let res = mgr.expand_template("src/main.rs", raw, "2026-10-15", "10:00:00");
    /// assert!(res.text.starts_with("/* File: main.rs */"));
    /// assert!(!res.text.contains("%Cursor%"));
    /// ```
    pub fn expand_template<P: AsRef<Path>>(
        &self,
        file_path: P,
        template_raw: &str,
        date_str: &str,
        time_str: &str,
    ) -> TemplateResult {
        let p = file_path.as_ref();
        let file_name = p.file_name().and_then(|n| n.to_str()).unwrap_or("untitled");
        let file_base = p.file_stem().and_then(|n| n.to_str()).unwrap_or("untitled");
        let ext = p.extension().and_then(|n| n.to_str()).unwrap_or("");

        let user_name = std::env::var("USERNAME")
            .or_else(|_| std::env::var("USER"))
            .unwrap_or_else(|_| "user".to_string());

        let sanitized_base: String = file_base
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        let sanitized_ext: String = ext
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        let guard = format!(
            "{}_{}_H_",
            sanitized_base.to_uppercase(),
            sanitized_ext.to_uppercase()
        );

        let mut expanded = template_raw.to_string();
        expanded = expanded.replace("%FileName%", file_name);
        expanded = expanded.replace("%FileBase%", file_base);
        expanded = expanded.replace("%Ext%", ext);
        expanded = expanded.replace("%Date%", date_str);
        expanded = expanded.replace("%Time%", time_str);
        expanded = expanded.replace("%UserName%", &user_name);
        expanded = expanded.replace("%Author%", &user_name);
        expanded = expanded.replace("%Guard%", &guard);

        // %Cursor% の位置を特定し、全出現箇所を削除
        let cursor_marker = "%Cursor%";
        let cursor_pos = if let Some(idx) = expanded.find(cursor_marker) {
            expanded[..idx].chars().count()
        } else {
            expanded.chars().count()
        };
        expanded = expanded.replace(cursor_marker, "");

        TemplateResult {
            text: expanded,
            cursor_pos,
        }
    }

    /// 新規ファイル向けに最適なテンプレートを取得し、マクロを展開した結果を返します。
    ///
    /// まずファイルシステム上のテンプレートを探索し、存在しなければ
    /// メモリ内に登録されたカスタムテンプレートを使用します。
    /// いずれも存在しない場合は `None` を返します。
    ///
    /// # 引数
    /// - `file_path`: 新規ファイルのパス
    /// - `date_str`: 置換用日付文字列
    /// - `time_str`: 置換用時刻文字列
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::template::TemplateManager;
    ///
    /// let mut mgr = TemplateManager::new();
    /// mgr.register_template("md", "# %FileBase%\n\nDate: %Date%\n");
    /// let res = mgr.load_and_expand("docs/intro.md", "2026-10-15", "10:00").unwrap();
    /// assert_eq!(res.text, "# intro\n\nDate: 2026-10-15\n");
    /// ```
    pub fn load_and_expand<P: AsRef<Path>>(
        &self,
        file_path: P,
        date_str: &str,
        time_str: &str,
    ) -> Option<TemplateResult> {
        let p = file_path.as_ref();
        let ext = p.extension()?.to_str()?.to_lowercase();

        // 1. ファイルシステムからの探索
        if let Some(template_path) = Self::find_template_file(p)
            && let Ok(raw) = fs::read_to_string(template_path)
        {
            return Some(self.expand_template(p, &raw, date_str, time_str));
        }

        // 2. メモリ内登録カスタムテンプレート
        if let Some(raw) = self.custom_templates.get(&ext) {
            return Some(self.expand_template(p, raw, date_str, time_str));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_macro_expansion() {
        let mut mgr = TemplateManager::new();
        let raw = "// File: %FileName%\n// Author: %UserName%\n// Date: %Date%\nfn main() {\n    %Cursor%\n}\n";
        mgr.register_template("rs", raw);

        let result = mgr
            .load_and_expand("src/app.rs", "2026-10-14", "12:00:00")
            .expect("template loaded");

        assert!(result.text.contains("// File: app.rs"));
        assert!(result.text.contains("// Date: 2026-10-14"));
        assert!(!result.text.contains("%Cursor%"));
        assert!(result.cursor_pos > 0);
    }

    #[test]
    fn test_template_guard_sanitization_and_multiple_cursors() {
        let mut mgr = TemplateManager::new();
        let raw = "#ifndef %Guard%\n#define %Guard%\n// %Cursor%\n// %Cursor%\n#endif\n";
        mgr.register_template("h", raw);

        let result = mgr
            .load_and_expand("my-header.h", "2026-10-14", "12:00:00")
            .unwrap();

        assert!(result.text.contains("MY_HEADER_H_H_"));
        assert!(!result.text.contains("%Cursor%"));
    }
}
