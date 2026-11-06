# vivivi: Test Execution Report (TEST_REPORT)

This document records the unit test and static analysis results for the `vivivi` workspace.

---

## 1. Overall Summary

- **Execution Date**: 2026-11-06
- **Rust Version**: rustc 1.96.1 / Cargo 1.96.1 (2024 edition)
- **Overall Result**: **All Tests Passed (684 passed, 0 failed) / Clippy 0 Warnings / rustfmt Passed**

---

## 2. Test Results by Crate

| Crate Name     |  Tests  | Passed  | Failed | Status  | Verified Features                                                                                                                                              |
| :------------- | :-----: | :-----: | :----: | :-----: | :------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `vivivi-core`  |   130   |   130   |   0    | ✅ PASS | GapBuffer local edits, Undo blocks & auto-merge, undofile persistence, early charset/newline detection, bracket search, 99 doctests                             |
| `vivivi-vi`    |   90    |   90    |   0    | ✅ PASS | Motions, operators, visual block editing, register history, macros, ex substitution, expr subst, quote text-objects, 46 doctests                                |
| `vivivi-modes` |   359   |   359   |   0    | ✅ PASS | RFC 4180 CSV, keisen full-width joining, char conv, box math, align, Migemo, DIFF optimization, DiffLog, folding, grep, 209 doctests                          |
| `vivivi-app`   |   69    |   69    |   0    | ✅ PASS | Binary split tree, multi-tab, independent scrolling, Windows-style editing, external process piping, TUI settings modal, Ex configuration commands, doctests    |
| `vivivi-gui`   |   36    |   36    |   0    | ✅ PASS | 5 themes (Tokyo Night, etc.), custom grid painting, IME preedit, minimap linear sync, particle drop physics, 2-pane settings dialog, window state persistence |
| **Total**      | **684** | **684** | **0**  | ✅ PASS | **All Items Passed (684 passed, 0 failed)**                                                                                                                    |

---

## 3. Static Analysis (Clippy) & Formatting Results

- **Clippy**: `cargo clippy --workspace -- -D warnings` $\to$ **0 errors, 0 warnings (Clean)**
- **rustfmt**: `cargo fmt --check` $\to$ **0 formatting violations (Clean)**
