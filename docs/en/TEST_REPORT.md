# vivivi: Test Execution Report (TEST_REPORT)

This document records the unit test, public API integration test, doctest, and static analysis results across the 6-crate `vivivi` workspace.

---

## 1. Overall Summary

- **Execution Date**: 2026-11-16
- **Target Version**: `v0.9.21-beta`
- **Rust Version**: rustc 1.100.0 / Cargo 1.100.0 (2024 edition)
- **Overall Result**: **All Tests Passed (773 passed, 0 failed / 46 ignored doctests in vivivi-app) / Clippy 0 Warnings / Rustdoc 0 Warnings / rustfmt Passed**

---

## 2. Test Results by Crate

| Crate Name     |  Tests  | Passed  | Failed | Status | Verified Features                                                                                                                                    |
| :------------- | :-----: | :-----: | :----: | :----: | :--------------------------------------------------------------------------------------------------------------------------------------------------- |
| `vivivi-core`  |   130   |   130   |   0    |  PASS  | GapBuffer local edits, Undo blocks & auto-merge, undofile persistence, early charset/newline detection, bracket search, 99 doctests                  |
| `vivivi-vi`    |    96   |    96   |   0    |  PASS  | Motions, operators, visual block editing, register history, macros, ex substitution, expr subst, quote text-objects, `:sort`, 50 doctests            |
| `vivivi-modes` |   392   |   392   |   0    |  PASS  | RFC 4180 CSV, keisen joining, char conv, box math, Migemo, LCS diff, folding, grep, trimming, Hex Editor, LSP, Tree-sitter AST, 218 doctests         |
| `moult-update` |    5    |    5    |   0    |  PASS  | Semver version comparison, SHA-256 checksum parsing, ZIP extraction, Windows in-place binary swap & cleanup lifecycle, 1 doctest                     |
| `vivivi-app`   |    91   |    91   |   0    |  PASS  | Binary split tree, multi-tab, Windows keys, external piping, `:version`, cursor settings, panic log, Hex/LSP/AST, 4 doctests (46 ignored)            |
| `vivivi-gui`   |    59   |    59   |   0    |  PASS  | 5 themes, custom grid, split panes, minimap, Zed-style sidebar, IME preedit, particle physics, 2-pane settings, 2D graph, Hex/LSP/AST, 15 doctests   |
| **Total**      | **773** | **773** | **0**  |  PASS  | **All Items Passed (773 passed, 0 failed / 46 ignored)**                                                                                             |

---

## 3. Static Analysis (Clippy), Documentation & Formatting Results

- **Tests**: `cargo test -j 4 --workspace --all-targets` & `cargo test -j 4 --workspace --doc` $\to$ **773 passed, 0 failed (46 ignored)**
- **Clippy**: `cargo clippy -j 4 --workspace --all-targets -- -D warnings` $\to$ **0 errors, 0 warnings (Clean)**
- **Rustdoc**: `cargo doc -j 4 --no-deps --workspace` $\to$ **0 broken links, 0 warnings (Clean)**
- **rustfmt**: `cargo fmt --check` $\to$ **0 formatting violations (Clean)**
