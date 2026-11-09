//! moult-update のエラー定義

use thiserror::Error;

/// moult-update で発生するエラー型
#[derive(Debug, Error)]
pub enum MoultError {
    /// ネットワーク通信エラー
    #[error("Network error: {0}")]
    Network(String),

    /// HTTP ステータスエラー
    #[error("HTTP error ({0}): {1}")]
    Http(u16, String),

    /// JSON 解析エラー
    #[error("JSON parse error: {0}")]
    Json(String),

    /// 入出力エラー
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// ZIP 展開エラー
    #[error("ZIP extraction error: {0}")]
    Zip(String),

    /// SHA-256 チェックサム不一致
    #[error("Checksum mismatch: expected {expected}, calculated {actual}")]
    ChecksumMismatch {
        /// 期待されるハッシュ値
        expected: String,
        /// 実際に計算されたハッシュ値
        actual: String,
    },

    /// セマンティックバージョン解析エラー
    #[error("Version parse error: {0}")]
    VersionParse(String),

    /// 更新対象アセットが見つからない
    #[error("Target asset not found for pattern: {pattern}")]
    AssetNotFound {
        /// 探索パターン
        pattern: String,
    },

    /// バイナリスワップ処理失敗
    #[error("Swap operation failed: {0}")]
    SwapFailed(String),

    /// 既に最新バージョンである
    #[error("Application is already at latest version ({0})")]
    AlreadyLatest(String),

    /// 不正な設定値
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// moult-update の Result 型エイリアス
pub type Result<T> = std::result::Result<T, MoultError>;
