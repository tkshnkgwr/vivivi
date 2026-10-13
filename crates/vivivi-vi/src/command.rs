//! Vi コマンド、モーション、オペレータ定義モジュール。
//!
//! OpenViVi の `ViEngine.h` の `ViCmd` 列挙体をベースに、
//! カウントや引数を伴う型安全な Rust の列挙体として設計されています。

use crate::mode::ViMode;

/// カーソル移動（モーション）の種類と移動量。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Motion {
    /// 左移動 (`h`, BackSpace)
    Left(usize),
    /// 右移動 (`l`, Space)
    Right(usize),
    /// 上移動 (`k`)
    Up(usize),
    /// 下移動 (`j`)
    Down(usize),
    /// 行頭へ移動 (`0`)
    StartOfLine,
    /// 行末へ移動 (`$`)
    EndOfLine,
    /// 行の最初の非空白文字へ移動 (`^`)
    FirstNonBlank,
    /// 次の単語先頭へ移動 (`w`)
    NextWord(usize),
    /// 前の単語先頭へ移動 (`b`)
    PrevWord(usize),
    /// 単語末尾へ移動 (`e`)
    EndWord(usize),
    /// 空白区切りの次単語先頭へ移動 (`W`)
    NextBigWord(usize),
    /// 空白区切りの前単語先頭へ移動 (`B`)
    PrevBigWord(usize),
    /// 空白区切りの単語末尾へ移動 (`E`)
    EndBigWord(usize),
    /// 文書先頭へ移動 (`gg`)
    DocumentStart,
    /// 文書末尾または指定行へ移動 (`G` / `[count]G`)
    DocumentEnd(Option<usize>),
    /// 対応する括弧へ移動 (`%`)
    MatchingParen,
}

/// オペレータ（操作種別）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// 削除 (`d`)
    Delete,
    /// ヤンク（コピー） (`y`)
    Yank,
    /// 変更（削除してInsertモードへ移行） (`c`)
    Change,
}

/// パースされた Vi コマンド。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViCommand {
    /// カーソル移動
    Move(Motion),
    /// オペレータ＋モーション (例: `dw`, `c$`, `yG`)
    Operate {
        op: Operator,
        motion: Motion,
        register: Option<char>,
    },
    /// 行単位オペレータ (例: `dd`, `yy`, `cc`)
    OperateLine {
        op: Operator,
        count: usize,
        register: Option<char>,
    },
    /// カーソル位置に文字挿入 (`i`)
    InsertBefore,
    /// カーソル直後に文字挿入 (`a`)
    InsertAfter,
    /// 行頭（非空白）に文字挿入 (`I`)
    InsertLineStart,
    /// 行末に文字挿入 (`A`)
    InsertLineEnd,
    /// 下に行を空けて文字挿入 (`o`)
    OpenLineBelow,
    /// 上に行を空けて文字挿入 (`O`)
    OpenLineAbove,
    /// カーソル位置の文字を削除 (`x`)
    DeleteChar(usize),
    /// カーソル直前の文字を削除 (`X`)
    DeletePrevChar(usize),
    /// 行末まで削除 (`D`)
    DeleteToEol(Option<char>),
    /// 行末まで変更 (`C`)
    ChangeToEol(Option<char>),
    /// 直前・上行に貼り付け (`P`)
    PasteBefore {
        count: usize,
        register: Option<char>,
    },
    /// 直後・下行に貼り付け (`p`)
    PasteAfter {
        count: usize,
        register: Option<char>,
    },
    /// 元に戻す (`u`)
    Undo,
    /// やり直す (`Ctrl-R`)
    Redo,
    /// 指定モードへの遷移
    EnterMode(ViMode),
    /// 前方文字列検索 (`/pattern`)
    SearchForward(String),
    /// 後方文字列検索 (`?pattern`)
    SearchBackward(String),
    /// 次の一致へジャンプ (`n`)
    SearchNext,
    /// 前の一致へジャンプ (`N`)
    SearchPrev,
    /// カーソル下の単語を検索 (`*` / `#`)
    SearchWordUnderCursor { forward: bool },
    /// exコマンドの実行 (`:w`, `:q`, `:%s/...` 等)
    ExCommand(String),
}
