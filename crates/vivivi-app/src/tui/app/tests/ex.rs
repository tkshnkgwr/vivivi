use super::*;

#[test]
fn test_expression_substitute_in_app() {
    let text = "price: 100\nprice: 250";
    let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);
    let res = app.vi.execute(
        &vivivi_vi::ViCommand::ExCommand("%s/([0-9]+)/$1 * 2/ge".to_string()),
        &mut app.buffer,
        &mut app.cursor,
    );
    assert_eq!(res.message, Some("2 箇所の式置換を行いました".to_string()));
    assert_eq!(app.buffer.to_string(), "price: 200\nprice: 500");
}

#[test]
fn test_encoding_and_newline_ex_commands() {
    let mut app = App::new("hello", None, CharEncoding::Utf8, NewlineType::Lf);

    // :set fenc=sjis
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(':'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    for c in "set fenc=sjis".chars() {
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(c),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
    }
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.encoding, CharEncoding::ShiftJis);

    // :set ff=dos
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(':'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    for c in "set ff=dos".chars() {
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(c),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
    }
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.newline, NewlineType::Crlf);

    // :e ++enc=euc-jp
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(':'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    for c in "e ++enc=euc-jp".chars() {
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(c),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
    }
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.encoding, CharEncoding::EucJp);
    assert_eq!(app.newline, NewlineType::Crlf);

    // :set clipboard=unnamed
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(':'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    for c in "set clipboard=unnamed".chars() {
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(c),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
    }
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(
        app.vi.clipboard_option(),
        vivivi_vi::ClipboardOption::Unnamed
    );
}

#[test]
fn test_se_nu_and_wrap_commands() {
    let mut app = App::new("line1\nline2", None, CharEncoding::Utf8, NewlineType::Lf);

    // デフォルト状態の検証
    assert!(app.show_line_numbers);
    assert!(!app.wrap); // 通常時は nowrap (false)

    // 1. :se nonu で行番号非表示
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(':'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    for c in "se nonu".chars() {
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(c),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
    }
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert!(!app.show_line_numbers);
    assert!(app.status_message.as_ref().unwrap().contains("OFF"));

    // 2. :se nu で行番号表示
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(':'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    for c in "se nu".chars() {
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(c),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
    }
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert!(app.show_line_numbers);
    assert!(app.status_message.as_ref().unwrap().contains("ON"));

    // 3. :se wrap で折り返し有効
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(':'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    for c in "se wrap".chars() {
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(c),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
    }
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert!(app.wrap);
    assert_eq!(app.scroll.1, 0);
    assert!(app.status_message.as_ref().unwrap().contains("ON"));

    // 4. :se nowrap で折り返し無効
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(':'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    for c in "se nowrap".chars() {
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(c),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
    }
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert!(!app.wrap);
    assert!(app.status_message.as_ref().unwrap().contains("OFF"));

    // 5. :se nu? / :se wrap? で状態確認
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(':'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    for c in "se nu?".chars() {
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(c),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
    }
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert!(app.status_message.as_ref().unwrap().contains("number"));

    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(':'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    for c in "se wrap?".chars() {
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(c),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
    }
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert!(app.status_message.as_ref().unwrap().contains("nowrap"));
}

#[test]
fn test_ex_substitute_normal_and_expr() {
    let mut app = App::new(
        "apple banana apple\napple cherry apple\ncount: 5\n",
        None,
        Default::default(),
        Default::default(),
    );

    // 1. カレント行 (0行目) の単一置換 (:s/apple/orange/)
    app.cursor.pos = vivivi_core::CursorPosition::new(0, 0);
    crate::tui::event::ex::execute_ex_command(&mut app, "s/apple/orange/");
    assert_eq!(app.buffer.line_text(0).unwrap(), "orange banana apple");
    assert!(
        app.status_message
            .as_ref()
            .unwrap()
            .contains("1 箇所の置換")
    );

    // 2. バッファ全体の全置換 (:%s/apple/grape/g)
    crate::tui::event::ex::execute_ex_command(&mut app, "%s/apple/grape/g");
    assert_eq!(app.buffer.line_text(0).unwrap(), "orange banana grape");
    assert_eq!(app.buffer.line_text(1).unwrap(), "grape cherry grape");
    assert!(
        app.status_message
            .as_ref()
            .unwrap()
            .contains("3 箇所の置換")
    );

    // 3. 大文字小文字無視置換 (:%s/grape/melon/i)
    crate::tui::event::ex::execute_ex_command(&mut app, "%s/GRAPE/melon/i");
    assert_eq!(app.buffer.line_text(0).unwrap(), "orange banana melon");

    // 4. 式置換 (:%s/count: (\d+)/"total: " + ($1 * 10)/e)
    crate::tui::event::ex::execute_ex_command(
        &mut app,
        "%s/count: (\\d+)/\"total: \" + ($1 * 10)/e",
    );
    assert_eq!(app.buffer.line_text(2).unwrap(), "total: 50");
    assert!(
        app.status_message
            .as_ref()
            .unwrap()
            .contains("1 箇所の式置換")
    );

    // 5. Undo による置換巻き戻し
    app.buffer.undo();
    assert_eq!(app.buffer.line_text(2).unwrap(), "count: 5");
}

#[test]
fn test_ex_line_number_jump() {
    let mut app = App::new(
        "line1\nline2\nline3\nline4\nline5",
        None,
        Default::default(),
        Default::default(),
    );

    // :3 で 3行目 (インデックス 2) へジャンプ
    crate::tui::event::ex::execute_ex_command(&mut app, "3");
    assert_eq!(app.cursor.pos.line, 2);
    assert_eq!(app.status_message.as_ref().unwrap(), "3 行目へ移動しました");

    // :1 で 1行目 (インデックス 0) へジャンプ
    crate::tui::event::ex::execute_ex_command(&mut app, "1");
    assert_eq!(app.cursor.pos.line, 0);

    // 行数を超える値 (:100) は最終行へクランプ
    crate::tui::event::ex::execute_ex_command(&mut app, "100");
    assert_eq!(app.cursor.pos.line, 4);
    assert_eq!(app.status_message.as_ref().unwrap(), "5 行目へ移動しました");
}

#[test]
fn test_ex_grep_and_quickfix_navigation() {
    let temp_dir = std::env::temp_dir().join("vivivi_test_grep_ex");
    let _ = std::fs::create_dir_all(&temp_dir);
    let file_a = temp_dir.join("a.rs");
    let file_b = temp_dir.join("b.rs");
    std::fs::write(&file_a, "fn target_fn() {}\n").unwrap();
    std::fs::write(&file_b, "fn other() {\n    target_fn();\n}\n").unwrap();

    let mut app = App::new(
        "",
        Some(file_a.clone()),
        Default::default(),
        Default::default(),
    );

    // :grep target_fn <temp_dir>
    let grep_cmd = format!("grep target_fn {}", temp_dir.display());
    crate::tui::event::ex::execute_ex_command(&mut app, &grep_cmd);
    assert!(app.show_preview);
    assert!(app.preview_lines.iter().any(|l| l.contains("target_fn")));
    assert!(app.vi.grep_session.is_some());

    // :cnext
    crate::tui::event::ex::execute_ex_command(&mut app, "cnext");
    assert!(app.status_message.as_ref().unwrap().contains("[2/2]"));

    // :cprev
    crate::tui::event::ex::execute_ex_command(&mut app, "cprev");
    assert!(app.status_message.as_ref().unwrap().contains("[1/2]"));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_ex_marks_and_tags_and_fold_and_undofile() {
    let mut app = App::new(
        "fn outer() {\n    fn inner() {\n        let x = 1;\n    }\n}\n",
        Some(PathBuf::from("test_file.rs")),
        Default::default(),
        Default::default(),
    );

    // 1. :marks (空時)
    crate::tui::event::ex::execute_ex_command(&mut app, "marks");
    assert_eq!(
        app.status_message.as_ref().unwrap(),
        "登録されているマークはありません"
    );

    // マーク 'a' を設定後、:marks を実行
    app.vi.execute(
        &vivivi_vi::ViCommand::SetMark('a'),
        &mut app.buffer,
        &mut app.cursor,
    );
    crate::tui::event::ex::execute_ex_command(&mut app, "marks");
    assert!(app.status_message.as_ref().unwrap().contains("'a':"));

    // 2. :fold (インデント折り畳み検出)
    crate::tui::event::ex::execute_ex_command(&mut app, "fold");
    assert!(
        app.status_message
            .as_ref()
            .unwrap()
            .contains("コード折り畳み:")
    );

    // 3. :undofile
    crate::tui::event::ex::execute_ex_command(&mut app, "undofile");
    assert!(
        app.status_message
            .as_ref()
            .unwrap()
            .contains("永続 undofile を保存しました")
    );
    let undofile_path = PathBuf::from(".test_file.rs.undofile");
    if undofile_path.exists() {
        let _ = std::fs::remove_file(undofile_path);
    }
}

#[test]
fn test_ex_diff_with_external_file() {
    let temp_dir = std::env::temp_dir().join("vivivi_test_diff_ex");
    let _ = std::fs::create_dir_all(&temp_dir);
    let other_file = temp_dir.join("other.txt");
    std::fs::write(&other_file, "Line A\nLine B\n").unwrap();

    let mut app = App::new(
        "Line A\nLine MODIFIED\n",
        None,
        Default::default(),
        Default::default(),
    );

    let diff_cmd = format!("diff {}", other_file.display());
    crate::tui::event::ex::execute_ex_command(&mut app, &diff_cmd);
    assert!(app.show_preview);
    assert!(app.preview_lines.iter().any(|l| l.contains("-Line B")));
    assert!(
        app.preview_lines
            .iter()
            .any(|l| l.contains("+Line MODIFIED"))
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_vi_folding_commands_and_render() {
    let code = "fn main() {\n    let a = 1;\n    let b = 2;\n    println!(\"{}\", a + b);\n}\n";
    let mut app = App::new(code, None, Default::default(), Default::default());

    // 1. :fold でブロック検出
    crate::tui::event::ex::execute_ex_command(&mut app, "fold");
    assert_eq!(app.vi.fold_manager.folds.len(), 1);
    assert!(app.vi.fold_manager.folds[0].is_open);

    // 2. カーソルを行 1 (1-based: 2行目) に移動して 'zc' で閉じる
    app.cursor.pos.line = 1;
    app.feed_vi_char('z');
    app.feed_vi_char('c');
    assert!(!app.vi.fold_manager.folds[0].is_open);
    assert!(app.vi.fold_manager.is_line_hidden(2));
    assert!(app.vi.fold_manager.is_line_hidden(3));

    // 3. 'zo' で再度開く
    app.feed_vi_char('z');
    app.feed_vi_char('o');
    assert!(app.vi.fold_manager.folds[0].is_open);
    assert!(!app.vi.fold_manager.is_line_hidden(2));

    // 4. 'za' でトグル
    app.feed_vi_char('z');
    app.feed_vi_char('a');
    assert!(!app.vi.fold_manager.folds[0].is_open);

    // 5. 'zR' ですべて展開
    app.feed_vi_char('z');
    app.feed_vi_char('R');
    assert!(app.vi.fold_manager.folds[0].is_open);
}
