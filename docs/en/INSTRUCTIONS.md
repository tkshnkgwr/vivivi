# vivivi: Developer & Agent Instructions (INSTRUCTIONS)

This document provides guidelines and directives for human developers and AI assistants working on `vivivi`.

---

## 1. Top-Priority Guardrails
All participants must adhere strictly to the repository root [**`RULES.md`**](../../RULES.md).

1. **No Automatic Commits/Pushes**: Never run `git commit` or `git push` without explicit user instruction.
2. **Markdown Table Alignment**: Keep tables vertically aligned for plain-text readability.
3. **User Addressing**: Address the user respectfully in 100% natural Japanese.
4. **Environment Variable Path Rule**: Always normalize absolute paths using `%USERPROFILE%`.
5. **High-Speed CLI Tools**: Prioritize `rg`, `fd`, `jq`, and `bat --paging=never`.

---

## 2. Reference Assets
- Decompiled proprietary help files in `docs/references/` are local assets for specification lookups.
- They are excluded from git via `.gitignore` to protect intellectual property.
