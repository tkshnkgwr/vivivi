//! 起動時の残存旧バイナリ (*.old) 自動クリーンアップ

use std::fs;
use std::path::Path;

/// 指定ディレクトリ内の *.old ファイルを探索して削除を試みる
pub fn cleanup_old_files_in_dir(dir: &Path) -> usize {
    let mut removed_count = 0;

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && let Some(file_name) = path.file_name().and_then(|n| n.to_str())
            && file_name.ends_with(".old")
            && fs::remove_file(&path).is_ok()
        {
            removed_count += 1;
        }
    }

    removed_count
}

/// 現在の実行ファイルが存在するディレクトリ内の *.old ファイルを自動削除
///
/// アプリ起動時（`main()` 冒頭やバックグラウンド初期化時）に呼び出すことで、
/// 前回のアップデートで残された旧バイナリを安全・静粛に一掃します。
pub fn cleanup_old_binaries() -> usize {
    if let Ok(exe_path) = std::env::current_exe()
        && let Some(dir) = exe_path.parent()
    {
        return cleanup_old_files_in_dir(dir);
    }
    0
}
