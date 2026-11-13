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
    /// 展開済みの子ノード一覧
    pub children: Vec<FileEntryNode>,
    /// 子ディレクトリが一度以上ロード済みであるか
    pub children_loaded: bool,
    /// 子ディレクトリを非同期読み込み中であるか
    pub is_loading_children: bool,
}

/// 階層展開状態を反映してフラット化された 1 行分のツリー表示項目
#[derive(Debug, Clone)]
pub struct VisibleTreeRow {
    /// 対象パス
    pub path: PathBuf,
    /// 表示名
    pub name: String,
    /// ルート直下を `0` とする階層の深さ
    pub depth: usize,
    /// ディレクトリであるか
    pub is_dir: bool,
    /// 展開中であるか
    pub is_expanded: bool,
    /// シンボリックリンクであるか
    pub is_symlink: bool,
    /// クラウドオンデマンド・プレースホルダーであるか
    pub is_cloud_placeholder: bool,
    /// 子要素をバックグラウンド読込中であるか
    pub is_loading: bool,
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

/// バックグラウンドのディレクトリ走査結果を受け取るチャネル種別
type ScanResultReceiver = Receiver<Result<Vec<FileEntryNode>, String>>;

/// サブディレクトリの非同期走査要求 1 件分の状態
struct PendingChildScan {
    req_id: usize,
    dir_path: PathBuf,
    started_at: Instant,
    rx: ScanResultReceiver,
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
    rx: Option<ScanResultReceiver>,
    /// 展開されたサブディレクトリのバックグラウンド走査チャネル一覧
    pending_child_scans: Vec<PendingChildScan>,
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
            pending_child_scans: Vec::new(),
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
        self.pending_child_scans.clear();
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

        // 展開されたサブディレクトリの非同期走査結果を受信
        if !self.pending_child_scans.is_empty() {
            needs_repaint = true;
            let active_req = self.req_id;
            let timeout = self.timeout_duration;
            let mut remaining = Vec::new();
            let pending = std::mem::take(&mut self.pending_child_scans);
            for scan in pending {
                if scan.req_id != active_req {
                    continue;
                }
                if scan.started_at.elapsed() > timeout {
                    if let Some(node) = find_node_mut(&mut self.entries, &scan.dir_path) {
                        node.is_loading_children = false;
                        node.is_expanded = false;
                    }
                    continue;
                }
                match scan.rx.try_recv() {
                    Ok(Ok(children)) => {
                        if let Some(node) = find_node_mut(&mut self.entries, &scan.dir_path) {
                            node.children = children;
                            node.children_loaded = true;
                            node.is_loading_children = false;
                        }
                    }
                    Ok(Err(_)) | Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        if let Some(node) = find_node_mut(&mut self.entries, &scan.dir_path) {
                            node.is_loading_children = false;
                            node.is_expanded = false;
                        }
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        remaining.push(scan);
                    }
                }
            }
            self.pending_child_scans = remaining;
        }

        needs_repaint
    }

    /// 指定ディレクトリの展開／折りたたみ状態を切り替えます。
    ///
    /// 未展開かつ未読込のディレクトリを開く場合は、バックグラウンドスレッドで子要素の走査を開始します。
    pub fn toggle_expand(&mut self, dir_path: &Path) -> bool {
        let Some(node) = find_node_mut(&mut self.entries, dir_path) else {
            return false;
        };
        if !node.is_dir {
            return false;
        }
        node.is_expanded = !node.is_expanded;
        if node.is_expanded && !node.children_loaded && !node.is_loading_children {
            node.is_loading_children = true;
            let scan_path = node.path.clone();
            let current_req = self.req_id;
            let (tx, rx) = channel();
            if std::thread::Builder::new()
                .name(format!("async-subtree-scanner-{}", current_req))
                .spawn({
                    let scan_path_bg = scan_path.clone();
                    move || {
                        let res = scan_directory_safe(&scan_path_bg);
                        let _ = tx.send(res);
                    }
                })
                .is_ok()
            {
                self.pending_child_scans.push(PendingChildScan {
                    req_id: current_req,
                    dir_path: scan_path,
                    started_at: Instant::now(),
                    rx,
                });
            } else {
                node.is_loading_children = false;
                node.is_expanded = false;
            }
        }
        true
    }

    /// 展開状態を反映した階層ツリー表示用のフラット行一覧を返します。
    pub fn visible_rows(&self) -> Vec<VisibleTreeRow> {
        let mut out = Vec::new();
        collect_visible_rows(&self.entries, 0, &mut out);
        out
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
        matches!(self.status, ScanStatus::Scanning { .. }) || !self.pending_child_scans.is_empty()
    }
}

fn find_node_mut<'a>(
    nodes: &'a mut [FileEntryNode],
    target: &Path,
) -> Option<&'a mut FileEntryNode> {
    for node in nodes {
        if node.path == target {
            return Some(node);
        }
        if node.is_dir
            && target.starts_with(&node.path)
            && let Some(found) = find_node_mut(&mut node.children, target)
        {
            return Some(found);
        }
    }
    None
}

fn collect_visible_rows(nodes: &[FileEntryNode], depth: usize, out: &mut Vec<VisibleTreeRow>) {
    for node in nodes {
        out.push(VisibleTreeRow {
            path: node.path.clone(),
            name: node.name.clone(),
            depth,
            is_dir: node.is_dir,
            is_expanded: node.is_expanded,
            is_symlink: node.is_symlink,
            is_cloud_placeholder: node.is_cloud_placeholder,
            is_loading: node.is_loading_children,
        });
        if node.is_dir && node.is_expanded && !node.children.is_empty() {
            collect_visible_rows(&node.children, depth + 1, out);
        }
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
            children: Vec::new(),
            children_loaded: false,
            is_loading_children: false,
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

    #[test]
    fn test_async_file_tree_expand_and_collapse_hierarchy() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let src_dir = manifest_dir.join("src");
        let mut tree = AsyncFileTree::new(manifest_dir);

        let start = Instant::now();
        while tree.is_scanning() && start.elapsed() < Duration::from_secs(3) {
            tree.poll();
            std::thread::sleep(Duration::from_millis(20));
        }

        let initial_rows = tree.visible_rows();
        assert!(initial_rows.iter().all(|r| r.depth == 0));

        // src ディレクトリを展開すると非同期走査後に depth=1 の子行が表示される
        assert!(tree.toggle_expand(&src_dir));
        let start_sub = Instant::now();
        while tree.is_scanning() && start_sub.elapsed() < Duration::from_secs(3) {
            tree.poll();
            std::thread::sleep(Duration::from_millis(20));
        }
        let expanded_rows = tree.visible_rows();
        assert!(
            expanded_rows
                .iter()
                .any(|r| r.depth == 1 && r.name == "lib.rs"),
            "展開後のツリーに src/lib.rs が depth=1 で含まれること"
        );

        // 再びクリックすると即座に折りたたまれる
        assert!(tree.toggle_expand(&src_dir));
        let collapsed_rows = tree.visible_rows();
        assert!(!collapsed_rows.iter().any(|r| r.depth >= 1));
    }
}
