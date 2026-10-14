# vivivi: Test Execution Report (TEST_REPORT)

This document records the unit test and static analysis results for the `vivivi` workspace.

---

## 1. Overall Summary

- **Execution Date**: 2026-10-14
- **Rust Version**: rustc 1.96.1 / Cargo 1.96.1 (2024 edition)
- **Overall Result**: **All Tests Passed (119 passed, 0 failed) / Clippy 0 Warnings / rustfmt Passed**

---

## 2. Test Results by Crate

| Crate Name     |  Tests  | Passed  | Failed | Status  | Verified Features                                                                                                                                                                                                              |
| :------------- | :-----: | :-----: | :----: | :-----: | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `vivivi-core`  |   15    |   15    |   0    | ✅ PASS | GapBuffer operations/slides, Undo blocks & auto-merge, persistent undo (undofile serialization), charset/newline auto-detection, matching bracket search                                                                       |
| `vivivi-vi`    |   23    |   23    |   0    | ✅ PASS | Cursor motions, operators, visual block, register history, macros, ex substitution, expr subst, fFtT, word search, #+/#-, marks, tags                                                                                          |
| `vivivi-modes` |   66    |   66    |   0    | ✅ PASS | CSV, keisen, char conv, box math, align, renumber, expr subst, fFtT, macros, migemo, DIFF, DiffLog, template, ToDo, folding, grep, filer, project, PDF, syntax highlighting (11 languages: Rust/C++/Py/JS/JSON/MD/SQL/PL-SQL/Java/HTML/Bash) |
| `vivivi-app`   |   15    |   15    |   0    | ✅ PASS | Window splits, outline jumping, text conversion, help viewer, align, renumber, line edit, macros, expr subst, custom vi, tool integration (DiffLog/undofile/template/preview)                                                  |
| **Total**      | **119** | **119** | **0**  | ✅ PASS | **All Items Passed (119 passed, 0 failed)**                                                                                                                                                                                    |

---

## 3. Static Analysis (Clippy) & Formatting Results

- **Clippy**: `cargo clippy --all-targets -- -D warnings` $\to$ **0 errors, 0 warnings (Clean)**
- **rustfmt**: `cargo fmt --check` $\to$ **0 formatting violations (Clean)**
