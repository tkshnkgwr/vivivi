//! `moult-update` クレートの公開API統合テスト（`.agents/rules/rust_testing.md` 準拠）。

use moult_update::mock::MockReleaseBuilder;
use moult_update::{cleanup, extract_zip, parse_checksum, swap};
use tempfile::tempdir;

#[test]
fn test_version_semver_comparison() {
    let cur = semver::Version::parse("0.9.14").unwrap();
    let new_ver = semver::Version::parse("0.9.15").unwrap();
    assert!(new_ver > cur);

    let v_tag = "v0.9.15".trim_start_matches('v');
    let parsed = semver::Version::parse(v_tag).unwrap();
    assert_eq!(parsed, new_ver);
}

#[test]
fn test_parse_checksum() {
    let sha256_content = "\
# Checksums
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  vivivi-windows-x86_64.zip
ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad *vivivi-linux-x86_64.tar.gz
";
    let parsed = parse_checksum(sha256_content, "vivivi-windows-x86_64.zip");
    assert_eq!(
        parsed,
        Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string())
    );

    let single_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n";
    assert_eq!(
        parse_checksum(single_hash, "any.zip"),
        Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string())
    );
}

#[test]
fn test_swap_and_cleanup_lifecycle() {
    let test_dir = tempdir().unwrap();
    let bin_dir = test_dir.path().join("bin");
    let extract_dir = test_dir.path().join("extracted");
    std::fs::create_dir_all(&bin_dir).unwrap();
    std::fs::create_dir_all(&extract_dir).unwrap();

    // 既存のインストール済みバイナリ (GUI & TUI)
    let installed_gui = bin_dir.join("vivivi-gui.exe");
    let installed_tui = bin_dir.join("vivivi.exe");
    std::fs::write(&installed_gui, b"OLD_GUI_V0.9.14").unwrap();
    std::fs::write(&installed_tui, b"OLD_TUI_V0.9.14").unwrap();

    // 展開された新バイナリ
    let new_gui = extract_dir.join("vivivi-gui.exe");
    let new_tui = extract_dir.join("vivivi.exe");
    std::fs::write(&new_gui, b"NEW_GUI_V0.9.15").unwrap();
    std::fs::write(&new_tui, b"NEW_TUI_V0.9.15").unwrap();

    // スワップ実行
    swap::swap_explicit(
        &extract_dir,
        &installed_gui,
        Some(&installed_tui),
        "vivivi-gui.exe",
        Some("vivivi.exe"),
    )
    .unwrap();

    // 検証: 新バイナリに置き換わっていること
    assert_eq!(std::fs::read(&installed_gui).unwrap(), b"NEW_GUI_V0.9.15");
    assert_eq!(std::fs::read(&installed_tui).unwrap(), b"NEW_TUI_V0.9.15");

    // 検証: .old ファイルが作成されていること
    let old_gui = bin_dir.join("vivivi-gui.exe.old");
    let old_tui = bin_dir.join("vivivi.exe.old");
    assert!(old_gui.exists());
    assert!(old_tui.exists());
    assert_eq!(std::fs::read(&old_gui).unwrap(), b"OLD_GUI_V0.9.14");
    assert_eq!(std::fs::read(&old_tui).unwrap(), b"OLD_TUI_V0.9.14");

    // クリーンアップ実行
    let cleaned = cleanup::cleanup_old_files_in_dir(&bin_dir);
    assert_eq!(cleaned, 2);

    // 検証: .old ファイルが完全消去されていること
    assert!(!old_gui.exists());
    assert!(!old_tui.exists());
}

#[test]
fn test_mock_release_builder_and_extract() {
    let test_dir = tempdir().unwrap();
    let builder = MockReleaseBuilder::new("0.9.15", "vivivi-gui.exe")
        .with_pair_bin("vivivi.exe", b"PAIR_TUI_BINARY_DATA")
        .with_target_content(b"TARGET_GUI_BINARY_DATA");

    let (release, zip_path, sha256_hash) = builder.build(test_dir.path());

    assert_eq!(release.version, "0.9.15");
    assert!(zip_path.exists());
    assert_eq!(sha256_hash.len(), 64);

    // ZIP解凍のテスト
    let extract_dir = test_dir.path().join("extracted");
    extract_zip(&zip_path, &extract_dir).unwrap();

    let extracted_gui = extract_dir.join("vivivi-gui.exe");
    let extracted_tui = extract_dir.join("vivivi.exe");
    assert!(extracted_gui.exists());
    assert!(extracted_tui.exists());
    assert_eq!(
        std::fs::read(&extracted_gui).unwrap(),
        b"TARGET_GUI_BINARY_DATA"
    );
    assert_eq!(
        std::fs::read(&extracted_tui).unwrap(),
        b"PAIR_TUI_BINARY_DATA"
    );
}
