# vivivi: Test Execution Report (TEST_REPORT)

This document records the unit test and static analysis results for the `vivivi` workspace.

---

## 1. Overall Summary

- **Execution Date**: 2026-10-13
- **Rust Version**: rustc 1.96.1 / Cargo 1.96.1
- **Overall Result**: **All Tests Passed (36 passed, 0 failed) / Clippy 0 Warnings**

---

## 2. Test Results by Crate

| Crate Name     |  Tests  |  Passed  |  Failed  | Status  | Verified Features                                                           |
| :------------- | :-----: | :------: | :------: | :-----: | :-------------------------------------------------------------------------- |
| `vivivi-core`  |   14    |    14    |    0     | ✅ PASS | GapBuffer, Undo blocks, charset/newline auto-detection, paren search        |
| `vivivi-modes` |   10    |    10    |    0     | ✅ PASS | CSV quote hiding, column align, keisen step & box align, syntax tokenizer   |
| `vivivi-vi`    |   12    |    12    |    0     | ✅ PASS | Motions, % paren jump, operator+motion, registers, search, ex substitution  |
| **Total**      | **36**  |  **36**  |  **0**   | ✅ PASS | **All Items Passed**                                                        |

---

## 3. Linter (Clippy) Results
- `cargo clippy --all-targets -- -D warnings`: **0 errors, 0 warnings (Clean)**
