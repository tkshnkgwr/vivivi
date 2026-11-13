//! `vivivi_app::panic_log` 公開 API の統合テスト。

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use vivivi_app::panic_log::{
    MAX_RETAINED_LOGS, default_log_dir, format_utc_datetime_from_secs, record_fatal_error_to_dir,
    write_panic_report_to_dir,
};

struct TempDirGuard(PathBuf);

impl TempDirGuard {
    fn new(prefix: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let pid = std::process::id();
        let dir = std::env::temp_dir().join(format!("vivivi_{prefix}_{pid}_{nanos}"));
        fs::create_dir_all(&dir).expect("一時ディレクトリの作成に失敗");
        Self(dir)
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn test_format_utc_datetime_from_secs() {
    assert_eq!(format_utc_datetime_from_secs(0), "1970-01-01 00:00:00");
    // 2026-11-13 00:00:00 UTC = 1794528000 秒
    assert_eq!(
        format_utc_datetime_from_secs(1_794_528_000),
        "2026-11-13 00:00:00"
    );
}

#[test]
fn test_write_panic_report_creates_structured_log_file() {
    let guard = TempDirGuard::new("panic_report");
    let log_dir = guard.0.join("nested").join("logs");

    let saved = write_panic_report_to_dir(
        &log_dir,
        "GUI",
        "unexpected index out of bounds",
        Some("crates/vivivi-gui/src/app/canvas.rs:42:9"),
        Some("main"),
        "0: vivivi_gui::draw\n1: std::sys::backtrace",
    )
    .expect("パニックログの書き出しに失敗");

    assert!(saved.exists());
    let file_name = saved
        .file_name()
        .and_then(|n| n.to_str())
        .expect("ファイル名が不正");
    assert!(file_name.starts_with("panic-"));
    assert!(file_name.ends_with(".log"));

    let body = fs::read_to_string(&saved).expect("生成ログの読み込みに失敗");
    assert!(body.contains("vivivi Panic Report"));
    assert!(body.contains("Frontend       : GUI"));
    assert!(body.contains("Location       : crates/vivivi-gui/src/app/canvas.rs:42:9"));
    assert!(body.contains("Thread         : main"));
    assert!(body.contains("Message        : unexpected index out of bounds"));
    assert!(body.contains("0: vivivi_gui::draw"));
}

#[test]
fn test_record_fatal_error_creates_fatal_log_file() {
    let guard = TempDirGuard::new("fatal_report");
    let saved = record_fatal_error_to_dir(&guard.0, "TUI", "failed to initialize terminal")
        .expect("致命的エラーログの書き出しに失敗");

    let file_name = saved
        .file_name()
        .and_then(|n| n.to_str())
        .expect("ファイル名が不正");
    assert!(file_name.starts_with("fatal-"));
    assert!(file_name.ends_with(".log"));

    let body = fs::read_to_string(&saved).expect("生成ログの読み込みに失敗");
    assert!(body.contains("vivivi Fatal Error Report"));
    assert!(body.contains("Frontend       : TUI"));
    assert!(body.contains("Message        : failed to initialize terminal"));
}

#[test]
fn test_log_rotation_retains_latest_and_preserves_unrelated_files() {
    let guard = TempDirGuard::new("log_rotation");
    let unrelated = guard.0.join("notes.txt");
    fs::write(&unrelated, "do not delete").expect("無関係ファイル作成に失敗");

    let total_to_write = MAX_RETAINED_LOGS + 5;
    for i in 0..total_to_write {
        if i % 2 == 0 {
            write_panic_report_to_dir(&guard.0, "GUI", &format!("panic #{i}"), None, None, "bt")
                .expect("パニックログ生成に失敗");
        } else {
            record_fatal_error_to_dir(&guard.0, "TUI", &format!("fatal #{i}"))
                .expect("致命エラーログ生成に失敗");
        }
    }

    assert!(unrelated.exists(), "無関係ファイルが誤削除されていないこと");

    let crash_log_count = fs::read_dir(&guard.0)
        .expect("ログディレクトリ走査に失敗")
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            (name.starts_with("panic-") || name.starts_with("fatal-")) && name.ends_with(".log")
        })
        .count();

    assert_eq!(crash_log_count, MAX_RETAINED_LOGS);
}

#[test]
fn test_default_log_dir_ends_with_vivivi_logs() {
    let dir = default_log_dir();
    let components: Vec<String> = dir
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    let len = components.len();
    assert!(len >= 2);
    assert_eq!(components[len - 2], ".vivivi");
    assert_eq!(components[len - 1], "logs");
}
