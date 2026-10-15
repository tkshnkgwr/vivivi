# vivivi: Test Execution Report (TEST_REPORT)

This document records the unit test and static analysis results for the `vivivi` workspace.

---

## 1. Overall Summary

- **Execution Date**: 2026-10-15
- **Rust Version**: rustc 1.96.1 / Cargo 1.96.1 (2024 edition)
- **Overall Result**: **All Tests Passed (556 passed, 0 failed) / Clippy 0 Warnings / rustfmt Passed**

---

## 2. Test Results by Crate

| Crate Name     |  Tests  | Passed  | Failed | Status  | Verified Features                                                                                                                                                                                                              |
| :------------- | :-----: | :-----: | :----: | :-----: | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `vivivi-core`  |   27    |   27    |   0    | ✅ PASS | GapBuffer operations/slides, Undo blocks & auto-merge, persistent undo (undofile serialization), charset/newline auto-detection, matching bracket search                                                                       |
| `vivivi-vi`    |   123   |   123   |   0    | ✅ PASS | Cursor motions, operators, visual block, register history, macros, ex substitution, expr subst, fFtT, word search, #+/#-, marks, tags, doctests                                                                                |
| `vivivi-modes` |   303   |   303   |   0    | ✅ PASS | CSV, keisen corner-joining, char conv, box math, align, expr subst, builtin SKK Migemo, DIFF, DiffLog, template, ToDo, folding, grep, filer, project, PDF, syntax highlighting (all 11 languages), doctests                    |
| `vivivi-app`   |   103   |   103   |   0    | ✅ PASS | Binary nested split tree (WindowTreeNode), geometric focus navigation, independent scrolling, Visual Column horizontal scroll tracking, outline navigation, invisible chars, gutter highlights, tool integration              |
| **Total**      | **556** | **556** | **0**  | ✅ PASS | **All Items Passed (556 passed, 0 failed)**                                                                                                                                                                                    |

---

## 3. Static Analysis (Clippy) & Formatting Results

- **Clippy**: `cargo clippy --all-targets -- -D warnings` $\to$ **0 errors, 0 warnings (Clean)**
- **rustfmt**: `cargo fmt --check` $\to$ **0 formatting violations (Clean)**
