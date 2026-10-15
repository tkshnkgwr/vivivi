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
