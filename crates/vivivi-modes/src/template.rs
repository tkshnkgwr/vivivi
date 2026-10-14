//! 拡張子別テンプレート自動読み込み・マクロ展開モジュール。
//!
//! 新規ファイル作成時に拡張子に応じたテンプレート（`.template.<ext>`）を自動探索し、
//! `%FileName%`, `%Date%`, `%UserName%` 等のマクロを展開して初期バッファを生成します。

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// テンプレート展開結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateResult {
    /// 展開後のテキスト
    pub text: String,
    /// `%Cursor%` が指定されていた場合の初期カーソルオフセット（文字位置）
    pub cursor_pos: usize,
}

/// テンプレートマネージャ。
#[derive(Debug, Clone, Default)]
pub struct TemplateManager {
    /// メモリ上に保持するビルトインまたは登録済みテンプレートマップ（拡張子 -> テンプレート内容）
    custom_templates: HashMap<String, String>,
}

impl TemplateManager {
    /// 新しい `TemplateManager` を生成します。
    pub fn new() -> Self {
        Self::default()
    }

    /// メモリ内カスタムテンプレートを登録します。
    pub fn register_template(&mut self, ext: &str, content: &str) {
        self.custom_templates.insert(
            ext.trim_start_matches('.').to_lowercase(),
            content.to_string(),
        );
    }

    /// 対象ファイルパスに対応するテンプレートファイルを探索します。
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

    /// ファイルパスと現在時刻からマクロ変数を構築し、テンプレートテキストを展開します。
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

        let guard = format!("{}_{}_H_", file_base.to_uppercase(), ext.to_uppercase());

        let mut expanded = template_raw.to_string();
        expanded = expanded.replace("%FileName%", file_name);
        expanded = expanded.replace("%FileBase%", file_base);
        expanded = expanded.replace("%Ext%", ext);
        expanded = expanded.replace("%Date%", date_str);
        expanded = expanded.replace("%Time%", time_str);
        expanded = expanded.replace("%UserName%", &user_name);
        expanded = expanded.replace("%Author%", &user_name);
        expanded = expanded.replace("%Guard%", &guard);

        // %Cursor% の位置を特定して削除
        let cursor_marker = "%Cursor%";
        let cursor_pos = if let Some(idx) = expanded.find(cursor_marker) {
            let pos = expanded[..idx].chars().count();
            expanded.replace_range(idx..idx + cursor_marker.len(), "");
            pos
        } else {
            expanded.chars().count()
        };

        TemplateResult {
            text: expanded,
            cursor_pos,
        }
    }

    /// 新規ファイル向けにテンプレートを取得・展開します。
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
}
