use super::*;

#[test]
fn test_window_split_and_cycle() {
    let mut app = App::new(
        "line1\nline2\nline3",
        None,
        CharEncoding::Utf8,
        NewlineType::Lf,
    );
    assert_eq!(app.windows.len(), 1);
    assert_eq!(app.active_window, 0);

    // 水平分割
    app.split_horizontal();
    assert_eq!(app.windows.len(), 2);
    assert_eq!(app.active_window, 1);
    assert_eq!(app.split_axis, SplitAxis::Horizontal);

    // 垂直分割
    app.split_vertical();
    assert_eq!(app.windows.len(), 3);
    assert_eq!(app.active_window, 2);
    assert_eq!(app.split_axis, SplitAxis::Vertical);

    // ウィンドウ巡回
    app.cycle_window();
    assert_eq!(app.active_window, 0);

    // ウィンドウ閉鎖
    app.close_window();
    assert_eq!(app.windows.len(), 2);

    // 独立スクロール・独立カーソル検証（同期スクロール防止）
    let multi_line_text = (0..100)
        .map(|i| format!("line {i}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut app_scroll = App::new(&multi_line_text, None, CharEncoding::Utf8, NewlineType::Lf);
    assert_eq!(app_scroll.scroll, (0, 0));
    assert_eq!(app_scroll.cursor.pos.line, 0);

    // 分割して新ウィンドウを50行目にスクロール移動
    app_scroll.split_horizontal();
    assert_eq!(app_scroll.active_window, 1);
    app_scroll.cursor.pos.line = 50;
    app_scroll.adjust_scroll(20, 80);
    assert!(
        app_scroll.scroll.0 > 0,
        "アクティブウィンドウはスクロールしている"
    );

    // ウィンドウ 0 (非アクティブ) のスクロールとカーソルは同期せず (0, 0) を維持している
    assert_eq!(app_scroll.windows[0].scroll, (0, 0));
    assert_eq!(app_scroll.windows[0].cursor.pos.line, 0);

    // ウィンドウ 0 へ切り替え -> スクロール位置 (0, 0) が復元される
    app_scroll.cycle_window();
    assert_eq!(app_scroll.active_window, 0);
    assert_eq!(app_scroll.scroll, (0, 0));
    assert_eq!(app_scroll.cursor.pos.line, 0);

    // ウィンドウ 0 で10行目に移動
    app_scroll.cursor.pos.line = 10;
    app_scroll.adjust_scroll(20, 80);

    // ウィンドウ 1 へ戻る -> ウィンドウ 1 のスクロール位置が独立して保持されている
    app_scroll.cycle_window();
    assert_eq!(app_scroll.active_window, 1);
    assert!(app_scroll.scroll.0 > 0);
    assert_eq!(app_scroll.cursor.pos.line, 50);

    // cycle_window_prev でウィンドウ 0 へ戻る -> ウィンドウ 0 のカーソル位置 (10行目) が保持されている
    app_scroll.cycle_window_prev();
    assert_eq!(app_scroll.active_window, 0);
    assert_eq!(app_scroll.cursor.pos.line, 10);

    // 単一ウィンドウ統合
    app.only_window();
    assert_eq!(app.windows.len(), 1);
    assert_eq!(app.active_window, 0);
}

#[test]
fn test_nested_split_vim_behavior() {
    let mut app = App::new("test vim split", None, CharEncoding::Utf8, NewlineType::Lf);
    assert_eq!(app.windows.len(), 1);
    assert_eq!(app.active_window, 0);

    // 1. :split (水平分割) -> 上: win 0, 下: win 1
    app.split_horizontal();
    assert_eq!(app.windows.len(), 2);
    assert_eq!(app.active_window, 1); // 下の win 1 がアクティブ

    // 2. 下ウィンドウ (win 1) で :vsplit (垂直分割)
    // Vim では下ウィンドウのみが左右に2分割され、全体が左・中・右の3分割にならない！
    app.split_vertical();
    assert_eq!(app.windows.len(), 3);
    assert_eq!(app.active_window, 2); // 右下の win 2 がアクティブ

    // 画面サイズ 100 x 40 で領域計算を検証
    let rects = app.window_tree.compute_rects(Rect::new(0, 0, 100, 40));
    assert_eq!(rects.len(), 3);
    // 上半分 (win 0): y=0, height=20, width=100
    assert_eq!(rects[0], (0, Rect::new(0, 0, 100, 20)));
    // 左下 (win 1): y=20, height=20, width=50
    assert_eq!(rects[1], (1, Rect::new(0, 20, 50, 20)));
    // 右下 (win 2): y=20, height=20, width=50, x=50
    assert_eq!(rects[2], (2, Rect::new(50, 20, 50, 20)));

    // 3. 幾何学的ナビゲーション (Ctrl-W h/j/k/l) の検証
    app.window_rects = rects;
    // 現在は win 2 (右下) にフォーカス
    // Left (Ctrl-W h) -> 左下 (win 1) へ移動
    app.navigate_window(WindowDirection::Left);
    assert_eq!(app.active_window, 1);

    // Up (Ctrl-W k) -> 上半分 (win 0) へ移動
    app.navigate_window(WindowDirection::Up);
    assert_eq!(app.active_window, 0);

    // Down (Ctrl-W j) -> 下 (win 1) へ移動
    app.navigate_window(WindowDirection::Down);
    assert_eq!(app.active_window, 1);

    // Right (Ctrl-W l) -> 右下 (win 2) へ移動
    app.navigate_window(WindowDirection::Right);
    assert_eq!(app.active_window, 2);

    // 4. close_window で右下 (win 2) を閉じる -> 下半分が win 1 のみになり、上下2分割に戻る
    app.close_window();
    assert_eq!(app.windows.len(), 2);
    let rects2 = app.window_tree.compute_rects(Rect::new(0, 0, 100, 40));
    assert_eq!(rects2.len(), 2);
    assert_eq!(rects2[0], (0, Rect::new(0, 0, 100, 20)));
    assert_eq!(rects2[1], (1, Rect::new(0, 20, 100, 20)));
}

#[test]
fn test_outline_toggle_and_jump() {
    let text = "# Title\nSome description\n## Section 1\nContent";
    let mut app = App::new(
        text,
        Some(PathBuf::from("test.md")),
        CharEncoding::Utf8,
        NewlineType::Lf,
    );

    // アウトライン表示ON
    app.toggle_outline();
    assert!(app.show_outline);
    assert_eq!(app.outline_items.len(), 2);
    assert_eq!(app.outline_items[0].title, "Title");
    assert_eq!(app.outline_items[1].title, "Section 1");

    // 2番目の見出しを選択してジャンプ
    app.selected_outline_idx = 1;
    app.jump_to_selected_outline();
    assert_eq!(app.cursor.pos.line, 2); // "## Section 1" は行2 (0-indexed)
}

#[test]
fn test_convert_buffer_text() {
    let mut app = App::new("Hello 123", None, CharEncoding::Utf8, NewlineType::Lf);
    app.convert_buffer_text(ConversionType::ToZenkakuAlphanumeric);
    assert_eq!(app.buffer.to_string(), "Ｈｅｌｌｏ　１２３");

    app.convert_buffer_text(ConversionType::ToHankakuAlphanumeric);
    assert_eq!(app.buffer.to_string(), "Hello 123");
}

#[test]
fn test_open_help() {
    let mut app = App::new("test", None, CharEncoding::Utf8, NewlineType::Lf);
    assert!(!app.show_help);

    // インデックスヘルプ
    app.open_help(None);
    assert!(app.show_help);
    assert!(!app.help_lines.is_empty());
    assert_eq!(app.help_scroll, 0);

    // トピック指定ヘルプ (keisen)
    app.open_help(Some("keisen"));
    assert!(app.show_help);
    assert!(!app.help_lines.is_empty());

    // トピック指定ヘルプ (syntax)
    app.open_help(Some("syntax"));
    assert!(app.show_help);
    assert!(!app.help_lines.is_empty());
    assert!(
        app.help_lines
            .iter()
            .any(|l| l.contains("シンタックスハイライト"))
    );

    // トピック指定ヘルプ (visual)
    app.open_help(Some("visual"));
    assert!(app.show_help);
    assert!(!app.help_lines.is_empty());
    assert!(
        app.help_lines
            .iter()
            .any(|l| l.contains("視覚・編集支援機能"))
    );

    // 存在しないトピック（フォールバック動作）
    app.open_help(Some("non_existent_topic_12345"));
    assert!(app.show_help);
    assert!(!app.help_lines.is_empty());
}

#[test]
fn test_align_text() {
    let text = "let a = 1;\nlet count = 2;\nlet total_value = 300;";
    let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);
    app.align_selection_or_buffer("=");
    assert_eq!(
        app.buffer.to_string(),
        "let a           = 1;\nlet count       = 2;\nlet total_value = 300;"
    );
}

#[test]
fn test_renumber() {
    let text = "1. りんご\n1. みかん\n9. バナナ";
    let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);
    app.renumber_selection_or_buffer(1, 1);
    assert_eq!(app.buffer.to_string(), "1. りんご\n2. みかん\n3. バナナ");

    // ステップ10
    app.renumber_selection_or_buffer(10, 10);
    assert_eq!(app.buffer.to_string(), "10. りんご\n20. みかん\n30. バナナ");
}

#[test]
fn test_insert_beg_end() {
    let text = "println!(\"hello\");\nprintln!(\"world\");";
    let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);
    app.insert_beg_end_in_range("// ", " // comment", true, true);
    assert_eq!(
        app.buffer.to_string(),
        "// println!(\"hello\"); // comment\n// println!(\"world\"); // comment"
    );
}

#[test]
fn test_abbrev_expansion() {
    let text = "hw";
    let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);
    app.abbrev.register("hw", "Hello World!");
    app.cursor.pos.col = 2; // "hw"の直後
    app.trigger_abbrev_expansion();
    assert_eq!(app.buffer.to_string(), "Hello World!");
}

#[test]
fn test_zen_coding_expansion() {
    let text = "ul>li*2";
    let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);
    app.cursor.pos.col = 7;
    app.trigger_zen_coding();
    assert_eq!(
        app.buffer.to_string(),
        "<ul>\n  <li></li>\n  <li></li>\n</ul>"
    );
}

#[test]
fn test_dynamic_macro() {
    let text = "";
    let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);
    // "abcabc" を記録
    for ch in "abcabc".chars() {
        app.dynamic_macro.record_char(ch);
    }
    app.trigger_dynamic_macro();
    assert_eq!(app.buffer.to_string(), "abc");

    // もう一度実行すると同じパターンが再実行される
    app.trigger_dynamic_macro();
    assert_eq!(app.buffer.to_string(), "abcabc");
}

#[test]
fn test_lex_edit() {
    let text = "user_name = 10;";
    let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);
    app.lex_edit.record_edit("user", "member");
    app.cursor.pos.col = 2; // "user_name" の途中
    app.trigger_lex_edit();
    assert_eq!(app.buffer.to_string(), "member_name = 10;");
}

#[test]
fn test_macro_recording_toggle() {
    let mut app = App::new("test", None, CharEncoding::Utf8, NewlineType::Lf);
    assert!(!app.is_recording_macro);

    app.toggle_macro_recording();
    assert!(app.is_recording_macro);

    app.toggle_macro_recording();
    assert!(!app.is_recording_macro);
    assert!(app.last_macro_events.is_some());
}

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
fn test_phase4_4_features_in_app() {
    let text = "count = 41;\nflag = false;\nfunc();";
    let mut app = App::new(
        text,
        Some(PathBuf::from("test.rs")),
        CharEncoding::Utf8,
        NewlineType::Lf,
    );
    app.cursor.pos = CursorPosition::new(0, 8); // '4'

    // 1. #+ (41 -> 42)
    let cmd = app.vi.feed_char('#');
    assert_eq!(cmd, None);
    let cmd = app.vi.feed_char('+');
    assert!(cmd.is_some());
    app.vi
        .execute(&cmd.unwrap(), &mut app.buffer, &mut app.cursor);
    assert!(app.buffer.to_string().contains("count = 42;"));

    // 2. #! (false -> true)
    app.cursor.pos = CursorPosition::new(1, 7); // "false"
    let cmd = app.vi.feed_char('#');
    assert_eq!(cmd, None);
    let cmd = vi_cmd_bang(&mut app.vi);
    assert!(cmd.is_some());
    app.vi
        .execute(&cmd.unwrap(), &mut app.buffer, &mut app.cursor);
    assert!(app.buffer.to_string().contains("flag = true;"));

    // 3. マーク設定と一覧
    app.cursor.pos = CursorPosition::new(1, 0);
    let cmd = app.vi.feed_char('m');
    assert_eq!(cmd, None);
    let cmd = app.vi.feed_char('Z');
    assert!(cmd.is_some());
    app.vi
        .execute(&cmd.unwrap(), &mut app.buffer, &mut app.cursor);
    let res = app.vi.execute(
        &vivivi_vi::ViCommand::ListMarks,
        &mut app.buffer,
        &mut app.cursor,
    );
    assert!(res.message.unwrap().contains("'Z': 2行"));
}

#[test]
fn test_phase4_5_features_in_app() {
    let text = "fn main() {\n    // TODO: refactor logic\n    let x = 10;\n    let y = 20;\n}\n";
    let mut app = App::new(
        text,
        Some(PathBuf::from("dummy.rs")),
        CharEncoding::Utf8,
        NewlineType::Lf,
    );

    // 1. コード折り畳み (:fold, zc, zo, za)
    let res = app.vi.execute(
        &vivivi_vi::ViCommand::ExCommand("fold".to_string()),
        &mut app.buffer,
        &mut app.cursor,
    );
    assert!(res.message.unwrap().contains("コード折り畳み: 1 箇所"));
    assert_eq!(app.vi.fold_manager.folds.len(), 1);

    let cmd = app.vi.feed_char('z');
    assert_eq!(cmd, None);
    let cmd = app.vi.feed_char('c');
    assert!(cmd.is_some());
    let res = app
        .vi
        .execute(&cmd.unwrap(), &mut app.buffer, &mut app.cursor);
    assert_eq!(res.message, Some("折り畳みを閉じました (zc)".to_string()));

    // 2. コード内 ToDo 抽出 (:todo)
    let res = app.vi.execute(
        &vivivi_vi::ViCommand::ExCommand("todo".to_string()),
        &mut app.buffer,
        &mut app.cursor,
    );
    assert!(res.message.unwrap().contains("ToDo: 1 件見つかりました"));
    assert_eq!(app.cursor.pos.line, 1); // 2行目 (0-indexed 1)

    // 3. 印刷プレビュー (:preview)
    let res = app.vi.execute(
        &vivivi_vi::ViCommand::ExCommand("preview".to_string()),
        &mut app.buffer,
        &mut app.cursor,
    );
    assert!(res.preview_requested.is_some());
    assert!(
        res.message
            .unwrap()
            .contains("印刷プレビューを生成しました")
    );

    // 4. Grep検索 (:grep)
    let res = app.vi.execute(
        &vivivi_vi::ViCommand::ExCommand("grep main src".to_string()),
        &mut app.buffer,
        &mut app.cursor,
    );
    assert!(res.message.unwrap().contains("Grep完了"));
}

#[test]
fn test_phase4_6_gutter_and_recent() {
    let initial_text = "line1\nline2\nline3";
    let mut app = App::new(initial_text, None, CharEncoding::Utf8, NewlineType::Lf);

    // ガターマーカー初期状態
    let lines: Vec<String> = (0..app.buffer.line_count())
        .filter_map(|i| app.buffer.get_line(i))
        .collect();
    let markers = app.change_tracker.compute_markers(&lines);
    assert_eq!(
        markers,
        vec![
            vivivi_modes::LineChangeState::Unchanged,
            vivivi_modes::LineChangeState::Unchanged,
            vivivi_modes::LineChangeState::Unchanged,
        ]
    );

    // 1行目を編集
    app.buffer.insert_char(0, 'x');
    let lines: Vec<String> = (0..app.buffer.line_count())
        .filter_map(|i| app.buffer.get_line(i))
        .collect();
    let markers = app.change_tracker.compute_markers(&lines);
    assert_eq!(markers[0], vivivi_modes::LineChangeState::Modified);

    // お気に入りの登録トグル
    app.recent_manager = RecentManager::new();
    let unique_fav = PathBuf::from("unique_test_fav_file_12345.rs");
    app.file_path = Some(unique_fav.clone());
    app.toggle_favorite_current_file();
    assert!(app.recent_manager.is_favorite(&unique_fav));

    // 不可視文字設定の初期値
    assert!(!app.list_chars_config.enabled);
    assert_eq!(app.list_chars_config.zenkaku_space, "　");
    app.list_chars_config.enabled = true;
    assert!(app.list_chars_config.enabled);
}

#[test]
fn test_key_release_ignored() {
    let mut app = App::new("hello", None, CharEncoding::Utf8, NewlineType::Lf);

    // Releaseイベントを作成
    let release_event = crossterm::event::KeyEvent {
        code: crossterm::event::KeyCode::Char('x'),
        modifiers: crossterm::event::KeyModifiers::empty(),
        kind: crossterm::event::KeyEventKind::Release,
        state: crossterm::event::KeyEventState::empty(),
    };

    // handle_key_event に渡しても Release イベントは完全に無視されること
    crate::tui::event::handle_key_event(&mut app, release_event);
    assert_eq!(app.buffer.to_string(), "hello");
    assert_eq!(app.cursor.pos.col, 0);
}

#[test]
fn test_ctrl_shortcuts_and_basic_keys() {
    let mut app = App::new(
        "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\nline 12\nline 13",
        None,
        CharEncoding::Utf8,
        NewlineType::Lf,
    );

    // 1. Home / End キーテスト
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::End,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.cursor.pos.col, 5); // "line 1" is 6 chars, end is 5 in normal mode

    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Home,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.cursor.pos.col, 0);

    // 2. Ctrl-D / Ctrl-U (半画面スクロール) テスト
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('d'),
            crossterm::event::KeyModifiers::CONTROL,
        ),
    );
    assert_eq!(app.cursor.pos.line, 10); // 10行進む

    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('u'),
            crossterm::event::KeyModifiers::CONTROL,
        ),
    );
    assert_eq!(app.cursor.pos.line, 0); // 10行戻る

    // 3. Ctrl-V (VisualBlock) テスト
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('v'),
            crossterm::event::KeyModifiers::CONTROL,
        ),
    );
    assert_eq!(app.mode(), ViMode::VisualBlock);

    // もう一度 Ctrl-V で Normal モード復帰
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('v'),
            crossterm::event::KeyModifiers::CONTROL,
        ),
    );
    assert_eq!(app.mode(), ViMode::Normal);

    // 4. Ctrl-R (Redo) テスト
    // 'x' で1文字削除
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('x'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.buffer.line_text(0).unwrap(), "ine 1");

    // 'u' で Undo
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('u'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.buffer.line_text(0).unwrap(), "line 1");

    // Ctrl-R で Redo
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('r'),
            crossterm::event::KeyModifiers::CONTROL,
        ),
    );
    assert_eq!(app.buffer.line_text(0).unwrap(), "ine 1");
}

#[test]
fn test_search_jump_to_head_and_highlight() {
    let text = "こんにちは世界\nHello Antigravity\nさようなら世界";
    let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);

    // 1. 通常検索 "/Antigravity"
    app.feed_vi_char('/');
    for c in "Antigravity\n".chars() {
        app.feed_vi_char(c);
    }
    // "Hello Antigravity" の行1、"Hello " の直後（col 6）に着地
    assert_eq!(app.cursor.pos.line, 1);
    assert_eq!(app.cursor.pos.col, 6);

    // active_search_regex でハイライト用Regexが取得できること
    let re = app.active_search_regex();
    assert!(re.is_some());
    let re = re.unwrap();
    assert!(re.is_match("Hello Antigravity"));

    // 2. 日本語文字列検索 "/世界"
    app.cursor.pos = vivivi_core::CursorPosition::new(0, 0);
    app.feed_vi_char('/');
    for c in "世界\n".chars() {
        app.feed_vi_char(c);
    }
    // "こんにちは世界" の先頭文字 "世" は 5文字目（col 5）
    assert_eq!(app.cursor.pos.line, 0);
    assert_eq!(app.cursor.pos.col, 5);

    // 'n' で次の "世界" へジャンプ
    app.feed_vi_char('n');
    // "さようなら世界" の先頭文字 "世" は 5文字目（col 5）
    assert_eq!(app.cursor.pos.line, 2);
    assert_eq!(app.cursor.pos.col, 5);
    assert!(app.active_search_regex().is_some());

    // 3. :noh で検索ハイライトを消去
    app.clear_search_highlight();
    assert!(app.active_search_regex().is_none());

    // 4. 'n' を押すとハイライトが再表示される
    app.feed_vi_char('n');
    assert!(app.active_search_regex().is_some());

    // 5. ノーマルモードで Esc 1回押しではハイライトは消えない（誤操作防止）
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Esc,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert!(app.active_search_regex().is_some());

    // 6. ノーマルモードで素早く Esc をもう一度押す（ダブルEsc連打）とハイライトが消去される
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Esc,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert!(app.active_search_regex().is_none());
}

#[test]
fn test_keisen_key_modifier_requirement() {
    let text = "hello\nworld";
    let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);

    // 罫線モードを有効化
    app.keisen_mode.is_active = true;

    // 1. 修飾キーなしの通常の矢印キー（Right）を押す
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Right,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    // 罫線は引かれず、カーソルだけが右（col 1）に移動
    assert_eq!(app.cursor.pos.col, 1);
    assert_eq!(app.buffer.line_text(0).unwrap(), "hello");

    // 2. 修飾キーなしの通常の矢印キー（Down）を押す
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Down,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    // 罫線は引かれず、カーソルだけが下（line 1）に移動
    assert_eq!(app.cursor.pos.line, 1);
    assert_eq!(app.buffer.line_text(1).unwrap(), "world");

    // 3. Ctrl+Right を押す -> 罫線が描画される！
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Right,
            crossterm::event::KeyModifiers::CONTROL,
        ),
    );
    // line 1 の文字が罫線（─）に更新されていること
    assert_eq!(
        app.buffer.char_at(app.buffer.line_start_position(1) + 1),
        Some('─')
    );
}

#[test]
fn test_replace_mode_workflow() {
    let text = "hello world";
    let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);

    // 1. 'R' を押して置換モードへ入る
    app.feed_vi_char('R');
    assert_eq!(app.mode(), ViMode::Replace);

    // 2. "HELLO" とタイプして上書き
    for c in "HELLO".chars() {
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(c),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
    }
    assert_eq!(app.buffer.line_text(0).unwrap(), "HELLO world");
    assert_eq!(app.cursor.pos.col, 5);

    // 3. Esc でノーマルモードへ復帰
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Esc,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.mode(), ViMode::Normal);
    assert_eq!(app.cursor.pos.col, 4);
}

#[test]
fn test_keisen_normal_x_protected() {
    // 罫線ボックス内のテキスト
    let text = "┌──────────┐\n│ hello    │\n└──────────┘";
    let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);
    app.keisen_mode.is_active = true;

    // 1行目の 'h' (line: 1, col: 2) にカーソルを合わせる
    app.cursor.pos = CursorPosition::new(1, 2);

    // ノーマルモードで 'x' を押す
    app.feed_vi_char('x');

    // 'h' が削除され、右枠線手前に半角スペースが1つ自動補填されて全体の行幅（枠線位置）が完全維持される
    let line1 = app.buffer.line_text(1).unwrap();
    assert_eq!(line1, "│ ello     │");
    assert_eq!(line1.chars().count(), 12);
}

#[test]
fn test_horizontal_scroll_over_80_columns() {
    // 120文字の長い行
    let long_line = "a".repeat(120);
    let mut app = App::new(&long_line, None, CharEncoding::Utf8, NewlineType::Lf);
    assert_eq!(app.scroll, (0, 0));

    // 行末 '$' へ移動
    app.feed_vi_char('$');
    assert_eq!(app.cursor.pos.col, 119);

    // 表示幅 80 カラムでスクロール調整
    app.adjust_scroll(24, 80);
    // 水平スクロール位置が 0 から右にスクロールしていること
    assert!(
        app.scroll.1 > 0,
        "水平スクロールが発生していること (scroll.1 = {})",
        app.scroll.1
    );
    // カーソルがスクロール後の画面可視範囲 (scroll.1..scroll.1 + 80) 内に収まっていること
    assert!(app.cursor.pos.col >= app.scroll.1);
    assert!(app.cursor.pos.col < app.scroll.1 + 80);
}

#[test]
fn test_migemo_builtin_kanji_search() {
    let text = "これは日本語の検索テストです。世界中どこでも動きます。";
    let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);

    // 1. "kensaku" で "検索"（漢字）にマッチしてカーソルがジャンプすること
    app.execute_migemo_search("kensaku");
    assert_eq!(app.cursor.pos.col, 7); // "検索" の開始位置

    // 2. "sekai" で "世界" にマッチすること
    app.execute_migemo_search("sekai");
    assert_eq!(app.cursor.pos.col, 15); // "世界" の開始位置

    // 3. "kakko" で "（" や "括弧" にマッチすること
    let mut app2 = App::new(
        "テキスト（重要）および矢印→はこちら",
        None,
        CharEncoding::Utf8,
        NewlineType::Lf,
    );
    app2.execute_migemo_search("kakko");
    assert_eq!(app2.cursor.pos.col, 4); // "（" の開始位置

    // 4. "yajirushi" で "矢印" や "→" にマッチすること
    app2.execute_migemo_search("yajirushi");
    assert_eq!(app2.cursor.pos.col, 11); // "矢印" の開始位置
}

#[test]
fn test_insert_key_mode_toggle() {
    let mut app = App::new("hello", None, CharEncoding::Utf8, NewlineType::Lf);
    assert_eq!(app.mode(), ViMode::Normal);

    // 1. Normalモードで Insertキーを押す -> Insertモードへ突入
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Insert,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.mode(), ViMode::Insert);

    // 2. Insertモードで Insertキーを押す -> Replaceモードへ切り替え
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Insert,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.mode(), ViMode::Replace);

    // 3. Replaceモードで Insertキーを押す -> Insertモードへ復帰
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Insert,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.mode(), ViMode::Insert);

    // 4. Escキーで Normalモードへ復帰
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Esc,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.mode(), ViMode::Normal);
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
fn test_swap_file_and_recovery() {
    let temp_dir = std::env::temp_dir();
    let target_file = temp_dir.join("vivivi_test_doc.txt");
    let swp_file = vivivi_core::SwapFile::swap_path_for(&target_file);

    // 事前クリーンアップ
    let _ = vivivi_core::SwapFile::remove(&swp_file);

    // 1. スワップファイルが存在しない状態で起動
    let mut app = App::new(
        "original text",
        Some(target_file.clone()),
        CharEncoding::Utf8,
        NewlineType::Lf,
    );
    assert!(!app.has_swap_warning);

    // 2. バッファ変更後にスワップ保存
    app.buffer.insert_text(13, " modified");
    assert!(app.buffer.is_modified());
    app.save_swap_file();
    assert!(swp_file.exists());

    // 3. 次回起動時にスワップファイル検出と警告メッセージ
    let mut new_app = App::new(
        "original text",
        Some(target_file.clone()),
        CharEncoding::Utf8,
        NewlineType::Lf,
    );
    assert!(new_app.has_swap_warning);
    assert!(
        new_app
            .status_message
            .as_ref()
            .unwrap()
            .contains("スワップ")
    );

    // 4. :recover コマンドでスワップから復元
    crate::tui::event::handle_key_event(
        &mut new_app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(':'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    for c in "recover".chars() {
        crate::tui::event::handle_key_event(
            &mut new_app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(c),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
    }
    crate::tui::event::handle_key_event(
        &mut new_app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(new_app.buffer.to_string(), "original text modified");
    assert!(!new_app.has_swap_warning);

    // 5. 保存するとスワップファイルが削除される
    let _ = new_app.save_file();
    assert!(!swp_file.exists());
    let _ = std::fs::remove_file(&target_file);
}

#[test]
fn test_multi_tab_workflow() {
    let mut app = App::new("tab1 content", None, CharEncoding::Utf8, NewlineType::Lf);
    assert_eq!(app.tabs.len(), 1);
    assert_eq!(app.active_tab, 0);

    // 1. :tabnew で新規タブ作成
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(':'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    for c in "tabnew".chars() {
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
    assert_eq!(app.tabs.len(), 2);
    assert_eq!(app.active_tab, 1);
    assert_eq!(app.buffer.to_string(), "");

    // タブ2を編集
    app.buffer.insert_text(0, "tab2 content");
    assert_eq!(app.buffer.to_string(), "tab2 content");

    // 2. gt で次のタブ（循環してタブ1へ）
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('g'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('t'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.active_tab, 0);
    assert_eq!(app.buffer.to_string(), "tab1 content");

    // 3. gT で前のタブ（タブ2へ）
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('g'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('T'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.active_tab, 1);
    assert_eq!(app.buffer.to_string(), "tab2 content");

    // 4. :tabclose でタブ2を閉じる
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(':'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    for c in "tabclose".chars() {
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
    assert_eq!(app.tabs.len(), 1);
    assert_eq!(app.active_tab, 0);
    assert_eq!(app.buffer.to_string(), "tab1 content");

    // 5. 最後の1タブで :tabclose を実行しても閉じずにエラーメッセージ
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(':'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    for c in "tabc".chars() {
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
    assert_eq!(app.tabs.len(), 1);
    assert!(app.status_message.as_ref().unwrap().contains("最後のタブ"));

    // 6. switch_tab_1based と :tabonly の検証
    app.new_tab(None);
    app.new_tab(None);
    assert_eq!(app.tabs.len(), 3);
    assert_eq!(app.tabs[0].id, 1);
    app.switch_tab_1based(2);
    assert_eq!(app.active_tab, 1);

    // :tabonly で他タブを一括クローズ
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(':'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    for c in "tabonly".chars() {
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
    assert_eq!(app.tabs.len(), 1);
}

fn vi_cmd_bang(vi: &mut ViEngine) -> Option<vivivi_vi::ViCommand> {
    vi.feed_char('!')
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
fn test_insert_mode_tab_key() {
    let mut app = App::new("", None, CharEncoding::Utf8, NewlineType::Lf);

    // 1. 'i' で挿入モードに入る
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('i'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.mode(), ViMode::Insert);

    // 2. 'h', 'i', Tab, '!' を入力
    for c in "hi".chars() {
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(c),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
    }
    // Tabキー押下
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Tab,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    // '!' を入力
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('!'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );

    // バッファにタブ文字 '\t' が正しく挿入されていること
    assert_eq!(app.buffer.to_string(), "hi\t!");
    // visual_col: "hi" (2) + tab (2 -> 4) + "!" (1) = 5
    assert_eq!(app.visual_col_at(0, 4), 5);
}

#[test]
fn test_two_pane_filer_workflow() {
    let mut app = App::new("", None, CharEncoding::Utf8, NewlineType::Lf);

    // 1. :filer コマンドを実行して2ペインファイラーを開く
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(':'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    for c in "filer".chars() {
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

    assert!(app.show_filer);
    assert!(app.filer.is_some());

    let filer = app.filer.as_ref().unwrap();
    assert_eq!(filer.active, vivivi_modes::filer::ActiveFilerPane::Left);

    // 2. Tab キーで左右ペイン切り替え
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Tab,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(
        app.filer.as_ref().unwrap().active,
        vivivi_modes::filer::ActiveFilerPane::Right
    );

    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Tab,
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(
        app.filer.as_ref().unwrap().active,
        vivivi_modes::filer::ActiveFilerPane::Left
    );

    // 3. j でカーソル移動
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('j'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    let total = app.filer.as_ref().unwrap().left.entries.len();
    if total > 1 {
        assert_eq!(app.filer.as_ref().unwrap().left.cursor, 1);
    }

    // 4. Space で選択反転（カーソルは自動で次行に進む）
    if total > 0 {
        let prev_cursor = app.filer.as_ref().unwrap().left.cursor;
        let is_parent_dir = app.filer.as_ref().unwrap().left.entries[prev_cursor].name == "..";
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(' '),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        if !is_parent_dir {
            assert!(app.filer.as_ref().unwrap().left.entries[prev_cursor].is_selected);
        }
        if total > 1 && prev_cursor + 1 < total {
            assert_eq!(app.filer.as_ref().unwrap().left.cursor, prev_cursor + 1);
        }
    }

    // 5. 'q' でファイラーを閉じる
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('q'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert!(!app.show_filer);
    assert!(app.filer.is_none());
}

#[test]
fn test_two_pane_filer_delete_with_d_key() {
    let temp_dir = std::env::temp_dir().join(format!(
        "vivivi_test_filer_del_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let file_path = temp_dir.join("test_delete.txt");
    std::fs::write(&file_path, "hello filer delete").unwrap();
    assert!(file_path.exists());

    let mut app = App::new("", None, CharEncoding::Utf8, NewlineType::Lf);
    app.open_filer(Some(&temp_dir));
    assert!(app.show_filer);

    // カーソルを test_delete.txt に合わせる
    let left_entries = &app.filer.as_ref().unwrap().left.entries;
    let target_idx = left_entries
        .iter()
        .position(|e| e.name == "test_delete.txt")
        .expect("test_delete.txt should exist in filer");
    app.filer.as_mut().unwrap().left.cursor = target_idx;

    // 'd' キーを押して削除実行
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('d'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );

    // ファイルがファイルシステムから削除されたことを検証
    assert!(!file_path.exists(), "file should be deleted by 'd' key");
    assert_eq!(
        app.status_message.as_deref(),
        Some("1 件のファイルを削除しました")
    );

    // リフレッシュされたエントリ一覧にも存在しないことを検証
    let new_entries = &app.filer.as_ref().unwrap().left.entries;
    assert!(new_entries.iter().all(|e| e.name != "test_delete.txt"));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_current_date_str_format() {
    let date_str = current_date_str();
    assert_eq!(date_str.len(), 10);
    let parts: Vec<&str> = date_str.split('-').collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0].len(), 4);
    assert_eq!(parts[1].len(), 2);
    assert_eq!(parts[2].len(), 2);
    let year: u32 = parts[0].parse().unwrap();
    let month: u32 = parts[1].parse().unwrap();
    let day: u32 = parts[2].parse().unwrap();
    assert!(year >= 2026);
    assert!((1..=12).contains(&month));
    assert!((1..=31).contains(&day));
}

#[test]
fn test_export_pdf_ex_command_and_preview_p_key() {
    let temp_dir = std::env::temp_dir().join(format!(
        "vivivi_test_pdf_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let src_path = temp_dir.join("sample.txt");
    let out_pdf = temp_dir.join("sample.pdf");
    std::fs::write(&src_path, "Hello Vivivi PDF Export\nLine 2 content").unwrap();

    let mut app = App::new(
        "Hello Vivivi PDF Export\nLine 2 content",
        Some(src_path.clone()),
        CharEncoding::Utf8,
        NewlineType::Lf,
    );

    // 1. :pdf コマンドでの出力
    crate::tui::event::ex::execute_ex_command(&mut app, "pdf");
    assert!(out_pdf.exists(), "sample.pdf should be generated by :pdf");
    let pdf_bytes = std::fs::read(&out_pdf).unwrap();
    assert!(pdf_bytes.starts_with(b"%PDF-1.4"));

    // 2. 引数付き :pdf <path> コマンドでの出力
    let custom_pdf = temp_dir.join("custom_out.pdf");
    crate::tui::event::ex::execute_ex_command(
        &mut app,
        &format!("pdf {}", custom_pdf.to_str().unwrap()),
    );
    assert!(
        custom_pdf.exists(),
        "custom_out.pdf should be generated by :pdf <path>"
    );
    let custom_bytes = std::fs::read(&custom_pdf).unwrap();
    assert!(custom_bytes.starts_with(b"%PDF-1.4"));

    // 3. :preview を開いて 'p' キーでのPDF出力
    std::fs::remove_file(&out_pdf).unwrap();
    assert!(!out_pdf.exists());
    crate::tui::event::ex::execute_ex_command(&mut app, "preview");
    assert!(app.show_preview);

    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('p'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert!(
        out_pdf.exists(),
        "sample.pdf should be generated by 'p' key in preview"
    );

    // 4. コマンドライン経由（:pdf + Enter）で二重生成されないことを検証
    std::fs::remove_file(&out_pdf).unwrap();
    if custom_pdf.exists() {
        std::fs::remove_file(&custom_pdf).unwrap();
    }
    assert!(!out_pdf.exists());

    // ':' を入力してコマンドラインモードに入る
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(':'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    // 'p', 'd', 'f' を入力
    for c in "pdf".chars() {
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(c),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
    }
    // Enter で確定
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        ),
    );

    assert!(out_pdf.exists(), "sample.pdf should be generated");
    let duplicate_pdf = temp_dir.join("sample.txt.pdf");
    assert!(
        !duplicate_pdf.exists(),
        "duplicate sample.txt.pdf should NOT exist"
    );

    let pdf_count = std::fs::read_dir(&temp_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "pdf"))
        .count();
    assert_eq!(pdf_count, 1, "exactly one PDF file must be generated");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_visual_line_yank_and_paste() {
    let text = "line1\nline2\nline3\n";
    let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);

    // 1. 'V' を入力して VisualLine モードへ
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('V'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.mode(), ViMode::VisualLine);

    // 2. 'j' を入力して 2行目まで選択
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('j'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );

    // 3. 'y' を入力してヤンク
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('y'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.mode(), ViMode::Normal);
    assert_eq!(
        app.vi.registers().get(None).unwrap().as_str(),
        "line1\nline2\n"
    );

    // 4. 'p' を入力して貼り付け
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('p'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );

    let result = app.buffer.to_string();
    assert_eq!(result, "line1\nline1\nline2\nline2\nline3\n");
}

#[test]
fn test_visual_character_multi_line_yank_and_paste() {
    let text = "abcdef\nghijkl\n";
    let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);

    // カーソルを 'c' (col 2) に移動
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('l'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('l'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.cursor_pos().col, 2);

    // 1. 'v' を入力して Visual モードへ
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('v'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.mode(), ViMode::Visual);

    // 2. 'j' を入力して次行へ移動
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('j'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );

    // 3. 'y' でヤンク
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('y'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.mode(), ViMode::Normal);
    assert_eq!(app.vi.registers().get(None).unwrap().as_str(), "cdef\nghi");

    // 4. 'p' で貼り付け
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('p'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.buffer.to_string(), "abccdef\nghidef\nghijkl\n");
}

#[test]
fn test_phase1_tui_integration() {
    let mut app = App::new(
        "line1\nline2\n\nline3\nline4",
        None,
        CharEncoding::Utf8,
        NewlineType::Lf,
    );

    // 1. '>>' でインデント
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('>'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('>'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.buffer.line_text(0).unwrap(), "    line1");

    // 2. '.' でドットリピート (再度インデント追加)
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('.'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.buffer.line_text(0).unwrap(), "        line1");

    // 3. '<<' でアンインデント
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('<'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('<'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.buffer.line_text(0).unwrap(), "    line1");

    // 4. '}' で段落移動 (空行の2行目へ)
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('}'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.cursor.pos.line, 2);

    // 5. 'zz' で画面中央スクロール調整
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('z'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('z'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(app.scroll.0, 0);
}
