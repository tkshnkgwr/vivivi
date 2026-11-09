//! Windows インプレース・リネーム＆スワップ機構、およびペアバイナリ同期

use crate::config::UpdateConfig;
use crate::error::{MoultError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 展開ディレクトリから該当するバイナリファイル（サブディレクトリ内も走査）を探索
pub fn find_binary_in_extracted(extracted_dir: &Path, bin_name: &str) -> Option<PathBuf> {
    // まず直下をチェック
    let direct = extracted_dir.join(bin_name);
    if direct.exists() && direct.is_file() {
        return Some(direct);
    }

    // 再帰的に探索
    if let Ok(entries) = fs::read_dir(extracted_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && let Some(name) = path.file_name().and_then(|n| n.to_str())
                && name.eq_ignore_ascii_case(bin_name)
            {
                return Some(path);
            } else if path.is_dir()
                && let Some(found) = find_binary_in_extracted(&path, bin_name)
            {
                return Some(found);
            }
        }
    }
    None
}

/// 単一のバイナリを安全にスワップ（既存ファイルを `.old` にリネームして新バイナリを配置）
pub fn swap_single_binary(installed_bin: &Path, new_bin_src: &Path) -> Result<()> {
    if !new_bin_src.exists() {
        return Err(MoultError::SwapFailed(format!(
            "Source binary does not exist: {:?}",
            new_bin_src
        )));
    }

    let bin_name = installed_bin
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| MoultError::SwapFailed("Invalid binary path".to_string()))?;

    let bin_dir = installed_bin
        .parent()
        .ok_or_else(|| MoultError::SwapFailed("Binary has no parent directory".to_string()))?;

    let old_path = bin_dir.join(format!("{}.old", bin_name));

    // 既存の .old が存在する場合は削除を試みる
    if old_path.exists() {
        let _ = fs::remove_file(&old_path);
    }

    // インプレース・リネーム（実行中であっても Windows はリネームを許可）
    let was_installed = installed_bin.exists();
    if was_installed {
        fs::rename(installed_bin, &old_path).map_err(|e| {
            MoultError::SwapFailed(format!(
                "Failed to rename running binary {:?} to {:?}: {}",
                installed_bin, old_path, e
            ))
        })?;
    }

    // 新バイナリの配置（コピー）
    if let Err(e) = fs::copy(new_bin_src, installed_bin) {
        // 失敗した場合はロールバック試行
        if was_installed && old_path.exists() {
            let _ = fs::rename(&old_path, installed_bin);
        }
        return Err(MoultError::SwapFailed(format!(
            "Failed to copy new binary to {:?}: {}",
            installed_bin, e
        )));
    }

    Ok(())
}

/// カレント実行ファイルおよびペアバイナリを一括スワップ
pub fn swap_in_place(extracted_dir: &Path, config: &UpdateConfig) -> Result<()> {
    let current_exe = std::env::current_exe()?;
    let bin_dir = current_exe
        .parent()
        .ok_or_else(|| MoultError::SwapFailed("Current exe has no parent directory".to_string()))?;

    // 1. カレント実行ファイルの特定とスワップ
    let current_name = current_exe
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&config.target_bin_name);

    if let Some(new_target) = find_binary_in_extracted(extracted_dir, current_name) {
        swap_single_binary(&current_exe, &new_target)?;
    } else if let Some(new_target) =
        find_binary_in_extracted(extracted_dir, &config.target_bin_name)
    {
        swap_single_binary(&current_exe, &new_target)?;
    } else {
        return Err(MoultError::SwapFailed(format!(
            "New binary for target {} not found in extracted archive",
            config.target_bin_name
        )));
    }

    // 2. ペアバイナリ（GUIに対するTUI、またはその逆）が存在する場合のスワップ
    if let Some(ref pair_name) = config.pair_bin_name {
        let pair_installed = bin_dir.join(pair_name);
        if let Some(new_pair) = find_binary_in_extracted(extracted_dir, pair_name) {
            // ペアファイルが既にインストール先に存在する場合、あるいは新規配置する場合
            if let Err(e) = swap_single_binary(&pair_installed, &new_pair) {
                // ペアバイナリの更新失敗は警告扱いとして本体の更新は維持
                eprintln!(
                    "[moult-update] Warning: Failed to swap pair binary {:?}: {}",
                    pair_installed, e
                );
            }
        }
    }

    Ok(())
}

/// 指定した任意のパスを対象としたスワップ処理（テスト用）
pub fn swap_explicit(
    extracted_dir: &Path,
    target_installed_path: &Path,
    pair_installed_path: Option<&Path>,
    target_bin_name: &str,
    pair_bin_name: Option<&str>,
) -> Result<()> {
    let target_bin = find_binary_in_extracted(extracted_dir, target_bin_name).ok_or_else(|| {
        MoultError::SwapFailed(format!(
            "Target {} not found in extracted files",
            target_bin_name
        ))
    })?;

    swap_single_binary(target_installed_path, &target_bin)?;

    if let (Some(pair_installed), Some(pair_name)) = (pair_installed_path, pair_bin_name)
        && let Some(pair_bin) = find_binary_in_extracted(extracted_dir, pair_name)
    {
        swap_single_binary(pair_installed, &pair_bin)?;
    }

    Ok(())
}

/// 更新された新バイナリを子プロセスとして起動し、引数を引き継ぐ
pub fn spawn_new_process() -> Result<std::process::Child> {
    let current_exe = std::env::current_exe()?;
    let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();

    Command::new(&current_exe).args(args).spawn().map_err(|e| {
        MoultError::SwapFailed(format!(
            "Failed to spawn updated process {:?}: {}",
            current_exe, e
        ))
    })
}

/// 新プロセスを起動して現在の旧プロセスを即時終了する
pub fn restart_process() -> ! {
    match spawn_new_process() {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("[moult-update] Fatal: Failed to restart process: {}", e);
            std::process::exit(1);
        }
    }
}
