//! GUI / TUI 共通のパニックログ・致命的エラーログ自動保存モジュール。
//!
//! Windows のデスクトップ GUI ビルド（`#![windows_subsystem = "windows"]`）では
//! コンソールが存在しないため、パニックや初期化失敗が発生しても原因を確認できずに
//! 無言終了してしまいます。本モジュールは起動時に共通フックをインストールし、
//! パニック内容・発生箇所・バックトレースを `%USERPROFILE%\.vivivi\logs\`
//! （Unix 系では `~/.vivivi/logs/`）へ自動保存します。

use std::any::Any;
use std::backtrace::Backtrace;
use std::fs;
use std::io::Write;
use std::panic::PanicHookInfo;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, Once, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// ログディレクトリ内に保持する最新クラッシュログファイル数の上限。
pub const MAX_RETAINED_LOGS: usize = 20;

type CleanupFn = Arc<dyn Fn() + Send + Sync + 'static>;
type OriginalPanicHook = Box<dyn Fn(&PanicHookInfo<'_>) + Send + Sync + 'static>;

#[derive(Clone)]
struct HookRuntimeConfig {
    frontend: &'static str,
    cleanup: Option<CleanupFn>,
}

static HOOK_INIT: Once = Once::new();
static ORIGINAL_HOOK: OnceLock<OriginalPanicHook> = OnceLock::new();
static HOOK_CONFIG: OnceLock<Mutex<HookRuntimeConfig>> = OnceLock::new();

/// 指定 UNIX 秒から UTC の年月日時分秒（`(year, month, day, hour, min, sec)`）を算出します。
fn utc_components_from_secs(secs: u64) -> (i64, u32, u32, u64, u64, u64) {
    let days = (secs / 86400) as i64;
    let day_secs = secs % 86400;
    let hour = day_secs / 3600;
    let min = (day_secs % 3600) / 60;
    let sec = day_secs % 60;

    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1024 + doe / 1461 - doe / 142401) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    (y, m, d, hour, min, sec)
}

/// 指定した UNIX エポック秒を UTC の `YYYY-MM-DD HH:MM:SS` 形式文字列に変換します。
///
/// # 引数
/// - `secs`: 1970-01-01 00:00:00 UTC からの経過秒数
///
/// # 副作用
/// なし（純粋関数）。
pub fn format_utc_datetime_from_secs(secs: u64) -> String {
    let (y, m, d, hour, min, sec) = utc_components_from_secs(secs);
    format!("{y:04}-{m:02}-{d:02} {hour:02}:{min:02}:{sec:02}")
}

/// 現在の UNIX エポック時間（秒・ミリ秒）を取得します。
fn now_timestamp() -> (u64, u128) {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    (dur.as_secs(), dur.as_millis() % 1000)
}

/// ファイル名用のコンパクトな UTC タイムスタンプ（`YYYYMMDD-HHMMSS-mmm`）を生成します。
fn compact_timestamp_from_now() -> (String, String) {
    let (secs, millis) = now_timestamp();
    let (y, m, d, hour, min, sec) = utc_components_from_secs(secs);
    let human = format!("{y:04}-{m:02}-{d:02} {hour:02}:{min:02}:{sec:02}");
    let compact = format!("{y:04}{m:02}{d:02}-{hour:02}{min:02}{sec:02}-{millis:03}");
    (human, compact)
}

/// パニックログのデフォルト保存先ディレクトリを返します。
///
/// 以下の優先順で `.vivivi/logs` ディレクトリを解決します。
/// 1. `%USERPROFILE%\.vivivi\logs`（Windows 環境変数 `USERPROFILE`）
/// 2. `$HOME/.vivivi/logs`（環境変数 `HOME`）
/// 3. `./.vivivi/logs`（上記環境変数が未設定の場合のフォールバック）
///
/// # 副作用
/// ディレクトリの作成自体は行わず、パス構築のみを行います。
pub fn default_log_dir() -> PathBuf {
    if let Ok(userprofile) = std::env::var("USERPROFILE")
        && !userprofile.is_empty()
    {
        return PathBuf::from(userprofile).join(".vivivi").join("logs");
    }
    if let Ok(home) = std::env::var("HOME")
        && !home.is_empty()
    {
        return PathBuf::from(home).join(".vivivi").join("logs");
    }
    PathBuf::from(".vivivi").join("logs")
}

/// パニックペイロードから人間が読めるメッセージ文字列を抽出します。
fn extract_panic_message(payload: &(dyn Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "<non-string panic payload>".to_string()
    }
}

/// パニック発生時の構造化レポート本文を生成します。
///
/// # 引数
/// - `frontend`: 起動元フロントエンド名（例: `"GUI"`, `"TUI"`）
/// - `message`: パニックメッセージ
/// - `location`: ソースコード位置情報（例: `"crates/vivivi-gui/src/main.rs:10:5"`）
/// - `thread_name`: パニックが発生したスレッド名
/// - `backtrace`: キャプチャ済みのバックトレース文字列
///
/// # 副作用
/// なし。
pub fn format_panic_report(
    frontend: &str,
    message: &str,
    location: Option<&str>,
    thread_name: Option<&str>,
    backtrace: &str,
) -> String {
    let (human_ts, _) = compact_timestamp_from_now();
    format_report_body(
        "vivivi Panic Report",
        &human_ts,
        frontend,
        message,
        location,
        thread_name,
        backtrace,
    )
}

/// 致命的エラー（`main` の `Err` 終了等）の構造化レポート本文を生成します。
///
/// # 引数
/// - `frontend`: 起動元フロントエンド名（例: `"GUI"`, `"TUI"`）
/// - `error_message`: 発生したエラー内容
///
/// # 副作用
/// なし。
pub fn format_fatal_error_report(frontend: &str, error_message: &str) -> String {
    let (human_ts, _) = compact_timestamp_from_now();
    let backtrace = Backtrace::force_capture().to_string();
    format_report_body(
        "vivivi Fatal Error Report",
        &human_ts,
        frontend,
        error_message,
        None,
        std::thread::current().name(),
        &backtrace,
    )
}

fn format_report_body(
    title: &str,
    timestamp_utc: &str,
    frontend: &str,
    message: &str,
    location: Option<&str>,
    thread_name: Option<&str>,
    backtrace: &str,
) -> String {
    let version = env!("CARGO_PKG_VERSION");
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let pid = std::process::id();
    let thread_disp = thread_name.unwrap_or("<unnamed>");
    let location_disp = location.unwrap_or("<unknown>");

    format!(
        "=== {title} ===\n\
         Timestamp (UTC): {timestamp_utc}\n\
         Version        : {version}\n\
         Frontend       : {frontend}\n\
         OS / Arch      : {os} / {arch}\n\
         PID            : {pid}\n\
         Thread         : {thread_disp}\n\
         Location       : {location_disp}\n\
         Message        : {message}\n\
         \n\
         --- Backtrace ---\n\
         {backtrace}\n"
    )
}

fn is_crash_log_file_name(name: &str) -> bool {
    (name.starts_with("panic-") || name.starts_with("fatal-")) && name.ends_with(".log")
}

/// 指定ディレクトリ内のクラッシュログを最新 `max_files` 件に整理します。
fn cleanup_old_logs(dir: &Path, max_files: usize) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    let mut logs: Vec<(SystemTime, String, PathBuf)> = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_file() {
                return None;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if !is_crash_log_file_name(&name) {
                return None;
            }
            let modified = entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(UNIX_EPOCH);
            Some((modified, name, path))
        })
        .collect();

    if logs.len() <= max_files {
        return;
    }

    // 新しい順（更新時刻降順、同一時はファイル名降順）に並べて古い末尾を削除
    logs.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
    for (_, _, old_path) in logs.into_iter().skip(max_files) {
        let _ = fs::remove_file(old_path);
    }
}

fn write_report_file(dir: &Path, prefix: &str, body: &str) -> std::io::Result<PathBuf> {
    fs::create_dir_all(dir)?;
    let (_, compact_ts) = compact_timestamp_from_now();
    let pid = std::process::id();

    // 同一ミリ秒・同一PIDでの衝突回避のため最大16回連番を試行
    let mut last_err = None;
    for seq in 0..16u32 {
        let file_name = if seq == 0 {
            format!("{prefix}-{compact_ts}-{pid}.log")
        } else {
            format!("{prefix}-{compact_ts}-{pid}-{seq}.log")
        };
        let path = dir.join(&file_name);
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                file.write_all(body.as_bytes())?;
                let _ = file.flush();
                cleanup_old_logs(dir, MAX_RETAINED_LOGS);
                return Ok(path);
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                last_err = Some(err);
                continue;
            }
            Err(err) => return Err(err),
        }
    }

    Err(last_err.unwrap_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "failed to allocate unique crash log file name",
        )
    }))
}

/// パニックレポートを指定ディレクトリへ書き出します。
///
/// 書き出し先ディレクトリが存在しない場合は自動作成し、保持上限（[`MAX_RETAINED_LOGS`] 件）を
/// 超えた古いクラッシュログを整理します。
///
/// # 引数
/// - `dir`: 保存先ディレクトリパス
/// - `frontend`: 起動元フロントエンド名（例: `"GUI"`, `"TUI"`）
/// - `message`: パニックメッセージ
/// - `location`: ソースコード位置情報
/// - `thread_name`: 発生スレッド名
/// - `backtrace`: バックトレース文字列
///
/// # Errors
/// ディレクトリ作成またはファイル書き込みに失敗した場合に [`std::io::Error`] を返します。
pub fn write_panic_report_to_dir(
    dir: &Path,
    frontend: &str,
    message: &str,
    location: Option<&str>,
    thread_name: Option<&str>,
    backtrace: &str,
) -> std::io::Result<PathBuf> {
    let body = format_panic_report(frontend, message, location, thread_name, backtrace);
    write_report_file(dir, "panic", &body)
}

/// 致命的エラーレポートを指定ディレクトリへ書き出します。
///
/// # 引数
/// - `dir`: 保存先ディレクトリパス
/// - `frontend`: 起動元フロントエンド名（例: `"GUI"`, `"TUI"`）
/// - `error_message`: 記録するエラー文字列
///
/// # Errors
/// ディレクトリ作成またはファイル書き込みに失敗した場合に [`std::io::Error`] を返します。
pub fn record_fatal_error_to_dir(
    dir: &Path,
    frontend: &str,
    error_message: &str,
) -> std::io::Result<PathBuf> {
    let body = format_fatal_error_report(frontend, error_message);
    write_report_file(dir, "fatal", &body)
}

/// 致命的エラー（`main` が `Err` を返して終了する直前など）をデフォルトログディレクトリへ保存します。
///
/// デフォルト保存先への書き込みに失敗した場合は OS の一時ディレクトリ（`std::env::temp_dir().join("vivivi_logs")`）
/// への保存を試みます。保存できた場合はファイルパスを返します。
///
/// # 副作用
/// ディスクへのログファイル書き込みと、必要に応じた古いログの削除を行います。
pub fn record_fatal_error(frontend: &str, error_message: &str) -> Option<PathBuf> {
    let primary_dir = default_log_dir();
    if let Ok(path) = record_fatal_error_to_dir(&primary_dir, frontend, error_message) {
        return Some(path);
    }
    let fallback_dir = std::env::temp_dir().join("vivivi_logs");
    record_fatal_error_to_dir(&fallback_dir, frontend, error_message).ok()
}

fn save_panic_from_hook(frontend: &str, panic_info: &PanicHookInfo<'_>) -> Option<PathBuf> {
    let message = extract_panic_message(panic_info.payload());
    let location = panic_info
        .location()
        .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()));
    let current_thread = std::thread::current();
    let thread_name = current_thread.name();
    let backtrace = Backtrace::force_capture().to_string();

    let primary_dir = default_log_dir();
    if let Ok(path) = write_panic_report_to_dir(
        &primary_dir,
        frontend,
        &message,
        location.as_deref(),
        thread_name,
        &backtrace,
    ) {
        return Some(path);
    }

    let fallback_dir = std::env::temp_dir().join("vivivi_logs");
    write_panic_report_to_dir(
        &fallback_dir,
        frontend,
        &message,
        location.as_deref(),
        thread_name,
        &backtrace,
    )
    .ok()
}

fn ensure_hook_registered(frontend: &'static str, cleanup: Option<CleanupFn>) {
    let state = HOOK_CONFIG.get_or_init(|| {
        Mutex::new(HookRuntimeConfig {
            frontend,
            cleanup: cleanup.clone(),
        })
    });

    match state.lock() {
        Ok(mut guard) => {
            guard.frontend = frontend;
            guard.cleanup = cleanup;
        }
        Err(poisoned) => {
            let mut guard = poisoned.into_inner();
            guard.frontend = frontend;
            guard.cleanup = cleanup;
        }
    }

    HOOK_INIT.call_once(|| {
        let taken = std::panic::take_hook();
        let _ = ORIGINAL_HOOK.set(taken);
        std::panic::set_hook(Box::new(|panic_info| {
            let (frontend_name, cleanup_fn) = if let Some(cfg_lock) = HOOK_CONFIG.get() {
                let guard = match cfg_lock.lock() {
                    Ok(g) => g,
                    Err(poisoned) => poisoned.into_inner(),
                };
                (guard.frontend, guard.cleanup.clone())
            } else {
                ("unknown", None)
            };

            if let Some(cleanup) = cleanup_fn {
                cleanup();
            }

            let saved_path = save_panic_from_hook(frontend_name, panic_info);
            if let Some(ref path) = saved_path {
                let _ = writeln!(
                    std::io::stderr(),
                    "[vivivi] パニックログを保存しました: {}",
                    path.display()
                );
            }

            if let Some(original) = ORIGINAL_HOOK.get() {
                original(panic_info);
            }
        }));
    });
}

/// GUI・TUI 共通のパニックフックをインストールします。
///
/// パニック発生時に、メッセージ・発生箇所・バックトレース・アプリバージョン等を
/// [`default_log_dir`]（失敗時は一時ディレクトリ）へ自動保存してから、
/// 既存の標準パニックフックへ処理を委譲します。
///
/// 同じプロセス内で複数回呼び出してもフックは二重登録されず、フロントエンド名のみ安全に更新されます。
///
/// # 引数
/// - `frontend`: ログへ記載するフロントエンド識別子（例: `"GUI"`, `"TUI"`）
///
/// # 副作用
/// プロセスグローバルなパニックフック（`std::panic::set_hook`）を登録・更新します。
pub fn install_panic_hook(frontend: &'static str) {
    ensure_hook_registered(frontend, None);
}

/// パニック時クリーンアップ処理付きの共通パニックフックをインストールします。
///
/// ログのファイル出力や標準エラー出力の前に `pre_report_cleanup` を実行するため、
/// TUI の Raw モード解除や代替スクリーン復帰を行ってからパニック詳細を安全に出力できます。
///
/// # 引数
/// - `frontend`: ログへ記載するフロントエンド識別子（例: `"GUI"`, `"TUI"`）
/// - `pre_report_cleanup`: パニック通知・ログ書き出し直前に実行するクリーンアップ処理（パニックしないこと）
///
/// # 副作用
/// プロセスグローバルなパニックフックを登録し、実行時クリーンアップハンドラを更新します。
pub fn install_panic_hook_with_cleanup<F>(frontend: &'static str, pre_report_cleanup: F)
where
    F: Fn() + Send + Sync + 'static,
{
    ensure_hook_registered(frontend, Some(Arc::new(pre_report_cleanup)));
}

/// 登録されているパニック時クリーンアップ処理（TUI のターミナル復元など）を解除します。
///
/// # 副作用
/// 次回以降のパニック発生時に追加のクリーンアップ処理は呼ばれなくなります（ログ保存フック自体は維持されます）。
pub fn clear_panic_cleanup() {
    if let Some(cfg_lock) = HOOK_CONFIG.get() {
        let mut guard = match cfg_lock.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        guard.cleanup = None;
    }
}
