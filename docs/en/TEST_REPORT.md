# vivivi: Test Execution Report (TEST_REPORT)

This document records the unit test and static analysis results for the `vivivi` workspace.

---

## 1. Overall Summary

- **Execution Date**: 2026-10-29
- **Rust Version**: rustc 1.96.1 / Cargo 1.96.1 (2024 edition)
- **Overall Result**: **All Tests Passed (613 passed, 0 failed) / Clippy 0 Warnings / rustfmt Passed**

---

## 2. Test Results by Crate

| Crate Name     |  Tests  | Passed  | Failed | Status  | Verified Features                                                                                                                |
| :------------- | :-----: | :-----: | :----: | :-----: | :------------------------------------------------------------------------------------------------------------------------------- |
| `vivivi-core`  |   114   |   114   |   0    | ✅ PASS | GapBuffer operations/slides, Undo blocks & auto-merge, undofile hex-escape, charset/newline detection, bracket search, doctests  |
| `vivivi-vi`    |   123   |   123   |   0    | ✅ PASS | Motions, operators, visual block, register history, macros, ex substitution, expr subst, quote text-objects, doctests            |
| `vivivi-modes` |   303   |   303   |   0    | ✅ PASS | RFC 4180 CSV, keisen joining, char conv, box math, align, SKK Migemo, DIFF prefix/suffix, DiffLog blame, folding, grep, doctests |
| `vivivi-app`   |   57    |   57    |   0    | ✅ PASS | Binary split tree (WindowTreeNode), multi-tab, unsaved changes safeguards, Ctrl+[ keymap, doctests                               |
| `vivivi-gui`   |   16    |   16    |   0    | ✅ PASS | Tokyo Night visuals, custom ui.painter grid, IME preedit integration, minimap, particle drop physics, doctests                   |
| **Total**      | **613** | **613** | **0**  | ✅ PASS | **All Items Passed (613 passed, 0 failed)**                                                                                      |

---

## 3. Static Analysis (Clippy) & Formatting Results

- **Clippy**: `cargo clippy --all-targets -- -D warnings` $\to$ **0 errors, 0 warnings (Clean)**
- **rustfmt**: `cargo fmt --check` $\to$ **0 formatting violations (Clean)**
