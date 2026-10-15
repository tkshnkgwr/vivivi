//! Vi モード定義および状態遷移モデル。

/// Vi の動作モードを表す列挙体。
///
/// ノーマル、挿入、置換、ビジュアル（文字/行/矩形）、コマンドラインモードをサポートします。
///
/// # Examples
///
/// ```
/// use vivivi_vi::ViMode;
///
/// let mode = ViMode::Normal;
/// assert_eq!(mode.name(), "NORMAL");
/// assert!(!mode.is_visual());
///
/// assert!(ViMode::VisualBlock.is_visual());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViMode {
    /// ノーマルモード（移動・オペレータ受付）
    #[default]
    Normal,
    /// 挿入モード（テキスト入力）
    Insert,
    /// 置換モード（文字上書き入力）
    Replace,
    /// 文字単位ビジュアル選択モード
    Visual,
    /// 行単位ビジュアル選択モード
    VisualLine,
    /// 矩形（ブロック）ビジュアル選択モード
    VisualBlock,
    /// ex コマンド入力モード（`:` プロンプト）
    CommandLine,
}

impl ViMode {
    /// ステータスライン等で表示するためのモード表示文字列を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::ViMode;
    ///
    /// assert_eq!(ViMode::Normal.name(), "NORMAL");
    /// assert_eq!(ViMode::VisualLine.name(), "V-LINE");
    /// ```
    pub fn name(&self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
            Self::Replace => "REPLACE",
            Self::Visual => "VISUAL",
            Self::VisualLine => "V-LINE",
            Self::VisualBlock => "V-BLOCK",
            Self::CommandLine => "COMMAND",
        }
    }

    /// ビジュアル選択モード（文字単位、行単位、矩形単位のいずれか）であるかを判定します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::ViMode;
    ///
    /// assert!(ViMode::Visual.is_visual());
    /// assert!(ViMode::VisualLine.is_visual());
    /// assert!(ViMode::VisualBlock.is_visual());
    /// assert!(!ViMode::Normal.is_visual());
    /// ```
    pub fn is_visual(&self) -> bool {
        matches!(self, Self::Visual | Self::VisualLine | Self::VisualBlock)
    }
}
