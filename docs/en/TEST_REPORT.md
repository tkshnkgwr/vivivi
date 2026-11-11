# vivivi: Test Execution Report (TEST_REPORT)

This document records the unit test, public API integration test, doctest, and static analysis results across the 6-crate `vivivi` workspace.

---

## 1. Overall Summary

- **Execution Date**: 2026-11-11
- **Target Version**: `v0.9.17-beta`
- **Rust Version**: rustc 1.96.1 / Cargo 1.96.1 (2024 edition)
- **Overall Result**: **All Tests Passed (724 passed, 0 failed) / Clippy 0 Warnings / Rustdoc 0 Warnings / rustfmt Passed**

---

## 2. Test Results by Crate

| Crate Name     |  Tests  | Passed  | Failed | Status | Verified Features                                                                                                                               |
| :------------- | :-----: | :-----: | :----: | :----: | :---------------------------------------------------------------------------------------------------------------------------------------------- |
| `vivivi-core`  |   130   |   130   |   0    |  PASS  | GapBuffer local edits, Undo blocks & auto-merge, undofile persistence, early charset/newline detection, bracket search, 99 doctests             |
| `vivivi-vi`    |   90    |   90    |   0    |  PASS  | Motions, operators, visual block editing, register history, macros, ex substitution, expr subst, quote text-objects, 46 doctests                |
| `vivivi-modes` |   386   |   386   |   0    |  PASS  | RFC 4180 CSV, keisen joining, char conv, box math, Migemo, DIFF, folding, grep, trimming, Hex Editor, LSP, Tree-sitter AST, 216 doctests        |
| `moult-update` |    5    |    5    |   0    |  PASS  | Semver version comparison, SHA-256 checksum parsing, ZIP extraction, Windows in-place binary swap & cleanup lifecycle, doctest                  |
| `vivivi-app`   |   76    |   76    |   0    |  PASS  | Binary split tree, multi-tab, Windows keys, external piping (:r!, :[range]!), Hex/LSP/Tree-sitter Ex commands, TUI settings modal, doctests     |
| `vivivi-gui`   |   37    |   37    |   0    |  PASS  | 5 themes, custom grid painting, IME preedit protection, minimap, particle physics, 2-pane settings, 2D graph, Hex/LSP/AST integration, doctests |
| **Total**      | **724** | **724** | **0**  |  PASS  | **All Items Passed (724 passed, 0 failed)**                                                                                                     |

---

## 3. Static Analysis (Clippy), Documentation & Formatting Results

- **Clippy**: `cargo clippy --workspace --all-targets` $\to$ **0 errors, 0 warnings (Clean)**
- **Rustdoc**: `cargo doc --workspace --no-deps` $\to$ **0 broken links, 0 warnings (Clean)**
- **rustfmt**: `cargo fmt --check` $\to$ **0 formatting violations (Clean)**
