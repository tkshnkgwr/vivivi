//! 行マネージャ（行長・行折り返し・オフセット管理）。

/// 行管理メタデータ（ブックマーク、折りたたみ等）を保持する行マネージャ。
///
/// # Examples
///
/// ```
/// use vivivi_core::line_mgr::LineMgr;
///
/// let mgr = LineMgr::new();
/// ```
#[derive(Debug, Clone, Default)]
pub struct LineMgr {
    // 行管理メタデータ・フラグ（ブックマーク、折りたたみ等）
}

impl LineMgr {
    /// 新しい [`LineMgr`] を生成します。
    pub fn new() -> Self {
        Self {}
    }
}
