# vivivi: Test Execution Report (TEST_REPORT)

This document records the unit test and static analysis results for the `vivivi` workspace.

---

## 1. Overall Summary

- **Execution Date**: 2026-10-13
- **Rust Version**: rustc 1.96.1 / Cargo 1.96.1
- **Overall Result**: **All Tests Passed (28 passed, 0 failed) / Clippy 0 Warnings**

---

## 2. Test Results by Crate

| Crate Name     | Tests | Passed | Failed | Status  | Verified Features                                            |
| :------------- | :---: | :----: | :----: | :-----: | :----------------------------------------------------------- |
| `vivivi-core`  | 9     | 9      | 0      | ✅ PASS | GapBuffer operations, Undo blocks, merge logic, line offsets |
| `vivivi-modes` | 8     | 8      | 0      | ✅ PASS | CSV quote hiding, column align, keisen step & box align      |
| `vivivi-vi`    | 11    | 11     | 0      | ✅ PASS | Motions, operator+motion, registers, search, ex substitution |
| **Total**      | **28**| **28** | **0**  | ✅ PASS | **All Items Passed**                                         |

---

## 3. Linter (Clippy) Results
- `cargo clippy --all-targets -- -D warnings`: **0 errors, 0 warnings (Clean)**
