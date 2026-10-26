//! フォント初期化モジュール（PlemolJP 優先設定・ステップ 1）。
//!
//! 1. 実行ファイル同階層 / カレントディレクトリの `fonts/PlemolJP*.ttf`
//! 2. システムインストール済み PlemolJP（`%LOCALAPPDATA%\Microsoft\Windows\Fonts`）
//! 3. BIZ UDゴシック（`C:\Windows\Fonts\BIZ-UDGothicR.ttc` 等）
//! 4. システム等幅フォント（`Consolas`, `MS Gothic` 等）
//!
//! 上記フォントを優先度順に探索・ロードし、等幅フォントおよびプロポーショナルフォントへ登録します。

use egui::{FontData, FontDefinitions, FontFamily};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// 指定ディレクトリ内の `PlemolJP` フォントファイルを優先度順に探索して追加します。
fn scan_plemol_in_dir(dir: &Path, prefix_tag: &str, out: &mut Vec<(String, PathBuf)>) {
    if !dir.is_dir() {
        return;
    }

    let preferred_names = [
        "PlemolJPConsole-Regular.ttf",
        "PlemolJP35-Regular.ttf",
        "PlemolJP-Regular.ttf",
        "PlemolJPConsoleNF-Regular.ttf",
    ];

    for name in &preferred_names {
        let p = dir.join(name);
        if p.is_file() {
            out.push((format!("{}_{}", prefix_tag, name), p));
        }
    }

    // ワイルドカード相当: ディレクトリ内のその他の PlemolJP-*.ttf も探索
    if let Ok(read_dir) = std::fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_file()
                && let Some(fname) = path.file_name().and_then(|n| n.to_str())
                && fname.starts_with("PlemolJP")
                && fname.ends_with(".ttf")
                && !preferred_names.contains(&fname)
            {
                out.push((format!("{}_{}", prefix_tag, fname), path));
            }
        }
    }
}

/// フォント探索候補のファイルパスを優先度順で収集します。
fn collect_candidate_paths() -> Vec<(String, PathBuf)> {
    let mut candidates = Vec::new();

    // 1. 実行ファイル同階層の fonts/ およびカレントディレクトリの fonts/
    let mut local_font_dirs = vec![PathBuf::from("fonts"), PathBuf::from("../fonts")];
    if let Ok(exe_path) = std::env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        let exe_fonts = exe_dir.join("fonts");
        if !local_font_dirs.contains(&exe_fonts) {
            local_font_dirs.insert(0, exe_fonts);
        }
    }

    for (idx, dir) in local_font_dirs.iter().enumerate() {
        scan_plemol_in_dir(dir, &format!("local{}", idx), &mut candidates);
    }

    // 2. Windows ユーザーフォントディレクトリ (%LOCALAPPDATA%\Microsoft\Windows\Fonts)
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let user_fonts_dir = Path::new(&local_app_data)
            .join("Microsoft")
            .join("Windows")
            .join("Fonts");
        scan_plemol_in_dir(&user_fonts_dir, "user", &mut candidates);
    }

    // 3. Windows システムフォントディレクトリ (C:\Windows\Fonts)
    let win_font_dir = std::env::var("WINDIR")
        .map(|w| PathBuf::from(w).join("Fonts"))
        .unwrap_or_else(|_| PathBuf::from(r"C:\Windows\Fonts"));

    // BIZ UDゴシック (優先日本語フォント)
    let biz_ud = win_font_dir.join("BIZ-UDGothicR.ttc");
    if biz_ud.is_file() {
        candidates.push(("BIZ-UDGothicR".to_string(), biz_ud));
    }

    // Consolas (等幅英数フォント)
    let consolas = win_font_dir.join("consola.ttf");
    if consolas.is_file() {
        candidates.push(("Consolas".to_string(), consolas));
    }

    // MS Gothic (フォールバック日本語等幅)
    let msgothic = win_font_dir.join("msgothic.ttc");
    if msgothic.is_file() {
        candidates.push(("MS-Gothic".to_string(), msgothic));
    }

    // Meiryo
    let meiryo = win_font_dir.join("meiryo.ttc");
    if meiryo.is_file() {
        candidates.push(("Meiryo".to_string(), meiryo));
    }

    candidates
}

/// [`egui::Context`] に PlemolJP 優先・BIZ UDゴシックフォールバックの等幅フォント定義を設定します。
///
/// 実行ファイル直下の `fonts/` や OS のフォントディレクトリから利用可能なフォントを探索し、
/// `FontFamily::Monospace` および `FontFamily::Proportional` の先頭へ優先度順に登録します。
/// 対象フォントが1つも存在しない環境でもパニックせず、`egui` 標準の内蔵フォントへ安全にフォールバックします。
///
/// # Examples
/// ```
/// use vivivi_gui::setup_fonts;
///
/// let ctx = egui::Context::default();
/// setup_fonts(&ctx);
/// ```
pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let candidates = collect_candidate_paths();

    let mut loaded_font_names = Vec::new();

    for (name, path) in candidates {
        if let Ok(bytes) = std::fs::read(&path) {
            fonts
                .font_data
                .insert(name.clone(), Arc::new(FontData::from_owned(bytes)));
            loaded_font_names.push(name);
        }
    }

    // Monospace ファミリーの先頭にロード順（優先度順）で挿入
    if let Some(mono_family) = fonts.families.get_mut(&FontFamily::Monospace) {
        for (i, name) in loaded_font_names.iter().enumerate() {
            mono_family.insert(i, name.clone());
        }
    }

    // Proportional ファミリーにも日本語文字欠損防止のためフォールバック登録
    if let Some(prop_family) = fonts.families.get_mut(&FontFamily::Proportional) {
        for (i, name) in loaded_font_names.iter().enumerate() {
            prop_family.insert(i, name.clone());
        }
    }

    ctx.set_fonts(fonts);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_candidate_paths_returns_valid_files() {
        let candidates = collect_candidate_paths();
        for (name, path) in &candidates {
            assert!(!name.is_empty());
            assert!(path.is_file());
        }
    }
}
