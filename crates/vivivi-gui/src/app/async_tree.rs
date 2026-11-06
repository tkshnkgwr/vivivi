//! 非同期クラウド耐障害サイドバー・ファイルエクスプローラーモジュール。
//!
//! クラウドストレージ（OneDrive, Google Drive, iCloud, SMB共有等）の同期遅延や
//! I/OブロックによるGUIスレッドのフリーズを根絶するため、バックグラウンドスレッドで
//! ディレクトリ走査を行い、2.0秒タイムアウト監視と耐障害UIを提供します。

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

/// 走査された1件のファイルまたはディレクトリのエントリ情報
#[derive(Debug, Clone)]
pub struct FileEntryNode {
    /// フルパス
    pub path: PathBuf,
    /// ファイル名またはディレクトリ名
    pub name: String,
    /// ディレクトリであるか
    pub is_dir: bool,
    /// シンボリックリンクであるか
    pub is_symlink: bool,
    /// ファイルサイズ（バイト）
    pub size_bytes: u64,
    /// クラウドオンデマンド・プレースホルダーであるか
    pub is_cloud_placeholder: bool,
    /// ツリー表示時の展開状態
    pub is_expanded: bool,
}

impl FileEntryNode {
    /// 拡張子を取得（小文字）
    pub fn extension(&self) -> Option<String> {
        self.path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_ascii_lowercase())
    }
}

/// ディレクトリ走査の進捗状態
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanStatus {
    /// 待機中（走査要求なし）
    Idle,
    /// バックグラウンド走査中
    Scanning { started_at: Instant },
    /// 走査完了
    Ready,
    /// 2.0秒タイムアウト発生
    Timeout,
    /// アクセス拒否・切断等のエラー
    Error(String),
}

/// 非同期ファイルツリー管理構造体
pub struct AsyncFileTree {
    /// 現在参照しているディレクトリパス
    pub current_dir: PathBuf,
    /// 走査済みのエントリリスト
    pub entries: Vec<FileEntryNode>,
    /// 現在の走査ステータス
    pub status: ScanStatus,
    /// タイムアウト判定閾値（デフォルト: 2.0秒）
    pub timeout_duration: Duration,
    /// バックグラウンド走査結果受信チャネル
    rx: Option<Receiver<Result<Vec<FileEntryNode>, String>>>,
    /// リクエストの世代ID（古い走査結果を破棄するため）
    req_id: usize,
}

impl AsyncFileTree {
    /// 指定された起点ディレクトリで非同期ファイルツリーを初期化し、即座に走査を開始します。
    pub fn new(initial_dir: PathBuf) -> Self {
        let mut tree = Self {
            current_dir: initial_dir.clone(),
            entries: Vec::new(),
            status: ScanStatus::Idle,
            timeout_duration: Duration::from_millis(2000),
            rx: None,
            req_id: 0,
        };
        tree.request_scan(initial_dir);
        tree
    }

    /// ディレクトリ走査をバックグラウンドワーカースレッドに要求します。
    pub fn request_scan(&mut self, dir: PathBuf) {
        self.req_id = self.req_id.wrapping_add(1);
        let current_req = self.req_id;
        self.current_dir = dir.clone();
        self.status = ScanStatus::Scanning {
            started_at: Instant::now(),
        };

        let (tx, rx) = channel();
        self.rx = Some(rx);

        if let Err(e) = std::thread::Builder::new()
            .name(format!("async-tree-scanner-{}", current_req))
            .spawn(move || {
                let result = scan_directory_safe(&dir);
                let _ = tx.send(result);
            })
        {
            // スレッド起動失敗時は即座にエラー通知
            let (err_tx, err_rx) = channel();
            let _ = err_tx.send(Err(format!("スレッド作成失敗: {e}")));
            self.rx = Some(err_rx);
        }
    }

    /// 毎フレーム呼び出され、バックグラウンド走査の完了またはタイムアウトを監視します。
    ///
    /// 状態が更新された場合や、スピナー描画などで再描画が必要な場合は `true` を返します。
    pub fn poll(&mut self) -> bool {
        let mut needs_repaint = false;

        if let ScanStatus::Scanning { started_at } = self.status {
            needs_repaint = true; // 走査中はスピナーアニメーションのため継続描画

            // タイムアウト監視（2.0秒超過）
            if started_at.elapsed() > self.timeout_duration {
                self.status = ScanStatus::Timeout;
                self.rx = None;
                return true;
            }

            if let Some(ref rx) = self.rx {
                match rx.try_recv() {
                    Ok(Ok(entries)) => {
                        self.entries = entries;
                        self.status = ScanStatus::Ready;
                        self.rx = None;
                    }
                    Ok(Err(err_msg)) => {
                        self.status = ScanStatus::Error(err_msg);
                        self.rx = None;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        self.status =
                            ScanStatus::Error("走査ワーカースレッドが切断されました".to_string());
                        self.rx = None;
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        // まだ走査中
                    }
                }
            }
        }

        needs_repaint
    }

    /// 指定されたディレクトリへ移動し、非同期スキャンを要求します。
    pub fn navigate_to(&mut self, target_dir: PathBuf) {
        if target_dir.is_dir() {
            self.request_scan(target_dir);
        }
    }

    /// 親ディレクトリへ移動します。
    pub fn navigate_up(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            let parent_path = parent.to_path_buf();
            self.navigate_to(parent_path);
        }
    }

    /// 現在走査中であるか
    pub fn is_scanning(&self) -> bool {
        matches!(self.status, ScanStatus::Scanning { .. })
    }
}

/// 安全なディレクトリ走査関数（シンボリックリンク循環やリパースポイントを考慮）
fn scan_directory_safe(dir: &Path) -> Result<Vec<FileEntryNode>, String> {
    if !dir.exists() {
        return Err(format!("パスが存在しません: {}", dir.display()));
    }

    let read_dir = std::fs::read_dir(dir).map_err(|e| match e.kind() {
        std::io::ErrorKind::PermissionDenied => {
            format!(
                "アクセス権限がありません (Permission Denied): {}",
                dir.display()
            )
        }
        std::io::ErrorKind::NotFound => {
            format!("ディレクトリが見つかりません: {}", dir.display())
        }
        _ => format!("ディレクトリ読込エラー: {e}"),
    })?;

    let mut entries = Vec::new();

    for entry_res in read_dir {
        let entry = match entry_res {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // symlink_metadata でシンボリックリンクやリパースポイントを実体追従せずに安全に取得
        let meta = match std::fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let file_type = meta.file_type();
        let is_symlink = file_type.is_symlink();
        let is_dir = if is_symlink {
            // シンボリックリンク先がディレクトリであるか確認（壊れたリンクの場合は false）
            path.is_dir()
        } else {
            file_type.is_dir()
        };

        let size_bytes = if is_dir { 0 } else { meta.len() };

        // Windows リパースポイント / クラウドオンデマンドプレースホルダーの判定
        let is_cloud_placeholder = is_reparse_point_or_cloud(&meta);

        entries.push(FileEntryNode {
            path,
            name,
            is_dir,
            is_symlink,
            size_bytes,
            is_cloud_placeholder,
            is_expanded: false,
        });
    }

    // ディレクトリを先頭に、同種別は名前順（大文字小文字無視）でソート
    entries.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            b.is_dir.cmp(&a.is_dir)
        } else {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
    });

    Ok(entries)
}

/// Windows リパースポイントやクラウドプレースホルダー（OneDrive Files On-Demand 等）の判定
#[cfg(windows)]
fn is_reparse_point_or_cloud(meta: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    let attrs = meta.file_attributes();
    // FILE_ATTRIBUTE_REPARSE_POINT (0x00000400)
    // FILE_ATTRIBUTE_RECALL_ON_OPEN (0x00040000)
    // FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS (0x00400000)
    const REPARSE_POINT: u32 = 0x00000400;
    const RECALL_ON_OPEN: u32 = 0x00040000;
    const RECALL_ON_DATA_ACCESS: u32 = 0x00400000;

    (attrs & (REPARSE_POINT | RECALL_ON_OPEN | RECALL_ON_DATA_ACCESS)) != 0
}

#[cfg(not(windows))]
fn is_reparse_point_or_cloud(_meta: &std::fs::Metadata) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_file_tree_scan_current_dir() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut tree = AsyncFileTree::new(manifest_dir);

        // 走査完了まで数秒待機してポーリング
        let start = Instant::now();
        while tree.is_scanning() && start.elapsed() < Duration::from_secs(3) {
            tree.poll();
            std::thread::sleep(Duration::from_millis(20));
        }

        assert_eq!(tree.status, ScanStatus::Ready);
        assert!(!tree.entries.is_empty());
        // Cargo.toml や src が含まれていることを確認
        let has_cargo = tree.entries.iter().any(|e| e.name == "Cargo.toml");
        let has_src = tree.entries.iter().any(|e| e.name == "src" && e.is_dir);
        assert!(has_cargo);
        assert!(has_src);
    }

    #[test]
    fn test_async_file_tree_timeout_simulation() {
        let mut tree = AsyncFileTree::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        // タイムアウトを極小値に設定
        tree.timeout_duration = Duration::from_millis(0);
        tree.status = ScanStatus::Scanning {
            started_at: Instant::now() - Duration::from_millis(100),
        };
        tree.poll();
        assert_eq!(tree.status, ScanStatus::Timeout);
    }
}
