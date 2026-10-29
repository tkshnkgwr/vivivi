# vivivi: Developer Guide (DEVELOPMENT)

This document provides guidelines for local development, workspace build commands, build profile optimization architecture, and code quality verification workflows for `vivivi`.

---

## 1. Environment Setup

### Prerequisites
- **Rust Toolchain**: Rust 1.80+ (Latest stable recommended, Edition 2024 support)
- **Git**: Version control

```powershell
# Verify toolchain
rustc --version
cargo --version
```

### Clone Repository
```powershell
git clone https://github.com/tkshnkgwr/vivivi.git
cd vivivi
```

---

## 2. Command Quick Reference (Cheat Sheet)

Summary of build and execution commands for development and production. No manual configuration edits are needed; Cargo automatically applies the appropriate profile flags.

### 2.1 GUI Edition (vivivi-gui)

| Target / Purpose                 | Command                                    | Characteristics & Performance                                      |
| :------------------------------- | :----------------------------------------- | :----------------------------------------------------------------- |
| **Daily Dev & Debug**            | `cargo run -p vivivi-gui`                  | Fast incremental compilation (few seconds); dependencies optimized.|
| **Launch with a File**           | `cargo run -p vivivi-gui -- <file_path>`   | Opens file with auto-detected encoding and newline format.         |
| **Release Verification**         | `cargo run -p vivivi-gui --release`        | Runs fully optimized executable (LTO, SIMD, stripped symbols).     |
| **Build Standalone Binary**      | `cargo build -p vivivi-gui --release`      | Outputs single executable at `target/release/vivivi-gui.exe`.      |

### 2.2 TUI Edition (vivivi)

| Target / Purpose                 | Command                                    | Characteristics & Performance                                      |
| :------------------------------- | :----------------------------------------- | :----------------------------------------------------------------- |
| **Daily Dev & Debug**            | `cargo run` or `cargo run -p vivivi`       | Runs terminal UI editor via Crossterm and Ratatui.                 |
| **Release Standalone Binary**    | `cargo build --release`                    | Outputs single executable at `target/release/vivivi.exe`.          |

---

## 3. Build Profile Optimization Architecture

To achieve both **fast incremental compilation times during development** and **buttery smooth GUI frame rates** in egui/eframe, the root `Cargo.toml` implements a hybrid optimization strategy.

### 3.1 Profile Configurations

```toml
# --- Build Profile Optimization Settings ---

# 1. Full optimization for external dependencies (egui, wgpu, glow, etc.) even in dev builds
[profile.dev.package."*"]
opt-level = 3

# 2. Lightweight optimization for local crates to keep dev compilation fast
[profile.dev]
opt-level = 1

# 3. Maximum optimization settings for release builds and packaging
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = true
```

### 3.2 Optimization Flags Overview

| Configuration Key                | Scope                | Technical Purpose & Benefits                                                      |
| :------------------------------- | :------------------- | :-------------------------------------------------------------------------------- |
| `[profile.dev.package."*"] = 3`  | External crates      | Fully optimizes egui/epaint/wgpu vertex generation & glyph caching. Eliminates lag.|
| `[profile.dev] = 1`              | Local crates         | Applies minimal inlining while maintaining fast 1-3 second incremental builds.    |
| `[profile.release] = 3`          | Entire workspace     | Aggressive inlining, loop unrolling, and SIMD vectorization across all functions. |
| `lto = "thin"`                   | Release linking      | Cross-crate ThinLTO for dead code elimination and inter-procedural optimizations. |
| `codegen-units = 1`              | Release codegen      | Single compilation unit for maximum LLVM optimization coverage and binary size reduction. |
| `strip = true`                   | Release executable   | Removes debug symbols and symbol tables to minimize binary footprint.            |

### 3.3 Why First Build Takes Longer, But Subsequent Builds Are Instant
- **Initial Build**: Takes several minutes as LLVM solves heavy optimization math for dozens of external GUI crates.
- **Incremental Builds**: Because external dependencies remain unchanged during local feature coding, **compiled optimized artifacts are 100% cached**. Recompiling local code changes takes only a few seconds.

---

## 4. Testing & Quality Checks

Run the following checks before committing code changes. Ensure all checks pass with zero warnings.

```powershell
# 1. Workspace type & target check
cargo check --all-targets

# 2. Run unit and integration tests
cargo test --all-targets

# 3. Run documentation tests
cargo test --doc

# 4. Clippy linter check (zero warnings enforced)
cargo clippy --all-targets -- -D warnings

# 5. Formatting check (format with `cargo fmt` if needed)
cargo fmt --check
```
