# vivivi: Contribution Guide (CONTRIBUTING)

We welcome contributions to the `vivivi` project! Please follow these guidelines when participating.

---

## 1. Development Flow
1. Create a feature branch for your work.
2. Implement changes and apply formatting (`cargo fmt`).
3. Ensure all unit tests pass (`cargo test`).
4. Ensure Clippy produces zero warnings (`cargo clippy --all-targets -- -D warnings`).
5. Create commits following the conventional prefix format.

---

## 2. Coding Standards
- **Edition 2024**: Use idiomatic and safe modern Rust.
- **Doc Comments**: Every public API (`pub fn`, `pub struct`) must include doc comments (`///`).
- **Markdown Tables**: Keep table pipes and hyphens strictly vertically aligned for plain-text readability.
