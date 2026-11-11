//! `CommandParser` の公開API統合テスト
//!
//! クレート外部の利用者視点（ブラックボックス）で、カウント付きモーション、
//! オペレータとモーションの組み合わせ、レジスタ指定、コマンドラインモードのパースを検証します。

use vivivi_vi::CommandParser;
use vivivi_vi::command::{Motion, Operator, ViCommand};
use vivivi_vi::mode::ViMode;

#[test]
fn test_count_and_motion() {
    let mut parser = CommandParser::new();

    // 3j -> Move(Down(3))
    assert_eq!(parser.feed_char('3', ViMode::Normal), None);
    assert_eq!(
        parser.feed_char('j', ViMode::Normal),
        Some(ViCommand::Move(Motion::Down(3)))
    );

    // 12w -> Move(NextWord(12))
    assert_eq!(parser.feed_char('1', ViMode::Normal), None);
    assert_eq!(parser.feed_char('2', ViMode::Normal), None);
    assert_eq!(
        parser.feed_char('w', ViMode::Normal),
        Some(ViCommand::Move(Motion::NextWord(12)))
    );
}

#[test]
fn test_operator_and_motion() {
    let mut parser = CommandParser::new();

    // dw -> Operate(Delete, NextWord(1))
    assert_eq!(parser.feed_char('d', ViMode::Normal), None);
    assert_eq!(
        parser.feed_char('w', ViMode::Normal),
        Some(ViCommand::Operate {
            op: Operator::Delete,
            motion: Motion::NextWord(1),
            register: None,
        })
    );

    // dd -> OperateLine(Delete, 1)
    assert_eq!(parser.feed_char('d', ViMode::Normal), None);
    assert_eq!(
        parser.feed_char('d', ViMode::Normal),
        Some(ViCommand::OperateLine {
            op: Operator::Delete,
            count: 1,
            register: None,
        })
    );
}

#[test]
fn test_register_and_command() {
    let mut parser = CommandParser::new();

    // "a yy -> OperateLine(Yank, 1, Some('a'))
    assert_eq!(parser.feed_char('"', ViMode::Normal), None);
    assert_eq!(parser.feed_char('a', ViMode::Normal), None);
    assert_eq!(parser.feed_char('y', ViMode::Normal), None);
    assert_eq!(
        parser.feed_char('y', ViMode::Normal),
        Some(ViCommand::OperateLine {
            op: Operator::Yank,
            count: 1,
            register: Some('a'),
        })
    );
}

#[test]
fn test_command_line_parsing() {
    let mut parser = CommandParser::new();

    // :w<Enter> -> ExCommand("w")
    assert_eq!(
        parser.feed_char(':', ViMode::Normal),
        Some(ViCommand::EnterMode(ViMode::CommandLine))
    );
    assert_eq!(parser.feed_char('w', ViMode::CommandLine), None);
    assert_eq!(
        parser.feed_char('\n', ViMode::CommandLine),
        Some(ViCommand::ExCommand("w".to_string()))
    );

    // /hello<Enter> -> SearchForward("hello")
    assert_eq!(
        parser.feed_char('/', ViMode::Normal),
        Some(ViCommand::EnterMode(ViMode::CommandLine))
    );
    assert_eq!(parser.feed_char('h', ViMode::CommandLine), None);
    assert_eq!(parser.feed_char('i', ViMode::CommandLine), None);
    assert_eq!(
        parser.feed_char('\n', ViMode::CommandLine),
        Some(ViCommand::SearchForward("hi".to_string()))
    );
}
#[test]
fn test_goto_line_commands() {
    let mut parser = CommandParser::new();

    // G -> DocumentEnd(None)
    assert_eq!(
        parser.feed_char('G', ViMode::Normal),
        Some(ViCommand::Move(Motion::DocumentEnd(None)))
    );

    // 1G -> DocumentEnd(Some(1))
    assert_eq!(parser.feed_char('1', ViMode::Normal), None);
    assert_eq!(
        parser.feed_char('G', ViMode::Normal),
        Some(ViCommand::Move(Motion::DocumentEnd(Some(1))))
    );

    // 100G -> DocumentEnd(Some(100))
    assert_eq!(parser.feed_char('1', ViMode::Normal), None);
    assert_eq!(parser.feed_char('0', ViMode::Normal), None);
    assert_eq!(parser.feed_char('0', ViMode::Normal), None);
    assert_eq!(
        parser.feed_char('G', ViMode::Normal),
        Some(ViCommand::Move(Motion::DocumentEnd(Some(100))))
    );

    // gg -> DocumentStart
    assert_eq!(parser.feed_char('g', ViMode::Normal), None);
    assert_eq!(
        parser.feed_char('g', ViMode::Normal),
        Some(ViCommand::Move(Motion::DocumentStart))
    );

    // 100gg -> DocumentEnd(Some(100))
    assert_eq!(parser.feed_char('1', ViMode::Normal), None);
    assert_eq!(parser.feed_char('0', ViMode::Normal), None);
    assert_eq!(parser.feed_char('0', ViMode::Normal), None);
    assert_eq!(parser.feed_char('g', ViMode::Normal), None);
    assert_eq!(
        parser.feed_char('g', ViMode::Normal),
        Some(ViCommand::Move(Motion::DocumentEnd(Some(100))))
    );
}

#[test]
fn test_command_and_search_history_navigation() {
    let mut parser = CommandParser::new();

    // 1. Exコマンド履歴の記録 (:set nu, :w, :set wrap)
    for cmd in ["set nu", "w", "set wrap"] {
        parser.feed_char(':', ViMode::Normal);
        for ch in cmd.chars() {
            parser.feed_char(ch, ViMode::CommandLine);
        }
        parser.feed_char('\n', ViMode::CommandLine);
    }
    assert_eq!(
        parser.ex_history(),
        &[
            "set nu".to_string(),
            "w".to_string(),
            "set wrap".to_string()
        ]
    );

    // 2. 空の状態から上下キーで全履歴を巡回
    parser.feed_char(':', ViMode::Normal);
    assert!(parser.history_prev());
    assert_eq!(parser.command_line_buffer(), Some((&':', "set wrap")));
    assert!(parser.history_prev());
    assert_eq!(parser.command_line_buffer(), Some((&':', "w")));
    assert!(parser.history_prev());
    assert_eq!(parser.command_line_buffer(), Some((&':', "set nu")));
    assert!(!parser.history_prev()); // 最古で停止

    assert!(parser.history_next());
    assert_eq!(parser.command_line_buffer(), Some((&':', "w")));
    assert!(parser.history_next());
    assert_eq!(parser.command_line_buffer(), Some((&':', "set wrap")));
    assert!(parser.history_next());
    assert_eq!(parser.command_line_buffer(), Some((&':', ""))); // 元の空入力へ復帰
    assert!(!parser.history_next());
    parser.feed_char('\x1b', ViMode::CommandLine);

    // 3. プレフィックス入力からの絞り込み履歴巡回 (:set + Up -> set wrap -> set nu)
    parser.feed_char(':', ViMode::Normal);
    for ch in "set".chars() {
        parser.feed_char(ch, ViMode::CommandLine);
    }
    assert!(parser.history_prev());
    assert_eq!(parser.command_line_buffer(), Some((&':', "set wrap")));
    assert!(parser.history_prev());
    assert_eq!(parser.command_line_buffer(), Some((&':', "set nu")));
    assert!(parser.history_next());
    assert_eq!(parser.command_line_buffer(), Some((&':', "set wrap")));
    assert!(parser.history_next());
    assert_eq!(parser.command_line_buffer(), Some((&':', "set"))); // 元のドラフト文字列へ復帰
    parser.feed_char('\x1b', ViMode::CommandLine);

    // 4. 検索履歴 (/ と ? で共通、Exコマンド履歴とは独立)
    for pat in ["alpha", "beta"] {
        parser.feed_char('/', ViMode::Normal);
        for ch in pat.chars() {
            parser.feed_char(ch, ViMode::CommandLine);
        }
        parser.feed_char('\n', ViMode::CommandLine);
    }
    parser.feed_char('/', ViMode::Normal);
    assert!(parser.history_prev());
    assert_eq!(parser.command_line_buffer(), Some((&'/', "beta")));
    assert!(parser.history_prev());
    assert_eq!(parser.command_line_buffer(), Some((&'/', "alpha")));
}
