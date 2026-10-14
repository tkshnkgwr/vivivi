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
    /// 行内文字検索・前方ジャンプ (`f` / `t`)
    FindCharForward {
        target: char,
        till: bool,
        count: usize,
    },
    /// 行内文字検索・後方ジャンプ (`F` / `T`)
    FindCharBackward {
        target: char,
        till: bool,
        count: usize,
    },
    /// 直前の行内文字検索を繰り返す (`;` 同方向, `,` 逆方向)
    RepeatFindChar { reverse: bool, count: usize },
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

/// 単語検索の一致条件種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordSearchKind {
    /// 完全単語一致 (`*`, `#` / `\bword\b`)
    ExactWord,
    /// 部分一致 (`g*`, `g#`)
    Partial,
    /// 前方一致 (`g<` / `\bword`)
    Prefix,
    /// 後方一致 (`g=` / `word\b`)
    Suffix,
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
    /// カーソル下の単語または選択テキストを検索 (`*`, `#`, `g*`, `g#`, `g<`, `g=`)
    SearchWordUnderCursor { forward: bool, kind: WordSearchKind },
    /// 数値・英字のインクリメント/デクリメント (`#+`, `##`, `#-`)
    Increment { delta: i64 },
    /// カーソル下の真偽値トグル (`#!`)
    ToggleBoolean,
    /// 直前の編集位置・行へジャンプ (`@`, `'@`, `` `@ ``)
    JumpToLastEdit { line_only: bool },
    /// 直前行（上行）の文字または指定文字までを複製挿入 (`=<char>`)
    CopyFromAboveLine { target: Option<char> },
    /// マークの設定 (`m<a-zA-Z>`)
    SetMark(char),
    /// マーク位置へのジャンプ (`'<mark>`, `` `<mark> ``)
    JumpMark { mark: char, line_only: bool },
    /// マーク一覧の表示 (`:marks`)
    ListMarks,
    /// タグジャンプ (`Ctrl-]`, `:tag <name>`)
    TagJump(Option<String>),
    /// タグスタックからの復元ポップ (`Ctrl-t`, `Ctrl-[`)
    TagPop,
    /// タグ履歴一覧の表示 (`:tags`)
    ListTags,
    /// ビジュアルモードでの操作 (削除、ヤンク、変更)
    VisualOperate {
        op: Operator,
        register: Option<char>,
    },
    /// ビジュアルモードでの大文字・小文字反転 (`~`)
    VisualSwapCase,
    /// ビジュアルブロックでの一括挿入 (`I` または `A`)
    VisualBlockInsert { at_end: bool },
    /// exコマンドの実行 (`:w`, `:q`, `:%s/...` 等)
    ExCommand(String),
    /// キーボードマクロの記録開始 (`q<reg>`)
    RecordMacro(char),
    /// キーボードマクロの記録終了 (`q`)
    StopRecordMacro,
    /// キーボードマクロの再生 (`@<reg>` または `@@`)
    ExecuteMacro(char),
}
