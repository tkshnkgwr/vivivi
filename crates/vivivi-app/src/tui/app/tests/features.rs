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
    let rects = app
        .window_tree
        .compute_logical_rects(LogicalRect::new(0, 0, 100, 40));
    assert_eq!(rects.len(), 3);
    // 上半分 (win 0): y=0, height=20, width=100
    assert_eq!(rects[0], (0, LogicalRect::new(0, 0, 100, 20)));
    // 左下 (win 1): y=20, height=20, width=50
    assert_eq!(rects[1], (1, LogicalRect::new(0, 20, 50, 20)));
    // 右下 (win 2): y=20, height=20, width=50, x=50
    assert_eq!(rects[2], (2, LogicalRect::new(50, 20, 50, 20)));

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
    let rects2 = app
        .window_tree
        .compute_logical_rects(LogicalRect::new(0, 0, 100, 40));
    assert_eq!(rects2.len(), 2);
    assert_eq!(rects2[0], (0, LogicalRect::new(0, 0, 100, 20)));
    assert_eq!(rects2[1], (1, LogicalRect::new(0, 20, 100, 20)));
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

    // 4. :tabclose でタブ2を閉じる（未保存変更があるためE37警告で拒否されることを検証）
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
    // 未保存のためタブは閉じられない
    assert_eq!(app.tabs.len(), 2);
    assert!(
        app.status_message
            .as_ref()
            .unwrap()
            .contains("前回の変更が保存されていません")
    );

    // :tabclose! で強制的にタブ2を閉じる
    crate::tui::event::handle_key_event(
        &mut app,
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(':'),
            crossterm::event::KeyModifiers::empty(),
        ),
    );
    for c in "tabclose!".chars() {
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
fn test_open_file_template_expansion() {
    let temp_dir = std::env::temp_dir().join("vivivi_test_template_open");
    let _ = std::fs::create_dir_all(&temp_dir);
    let template_file = temp_dir.join(".template.rs");
    std::fs::write(
        &template_file,
        "// File: %FileName%\n// Date: %Date%\nfn main() {}\n",
    )
    .unwrap();
    let new_rs = temp_dir.join("brand_new.rs");

    let mut app = App::new("", None, Default::default(), Default::default());

    // 新規 .rs ファイルを open_file で開く
    app.open_file(new_rs.clone()).unwrap();

    // テンプレートが展開され、"brand_new.rs" と "fn main()" が含まれていること
    let content = app.buffer.to_string();
    assert!(
        !content.is_empty(),
        "新規ファイルオープン時にテンプレートが適用されること"
    );
    assert!(content.contains("// File: brand_new.rs"));
    assert!(content.contains("fn main()"));

    let _ = std::fs::remove_dir_all(&temp_dir);
}
