# vivivi: Resource Metrics & Footprints (FOOTPRINTS)

This document defines resource consumption targets and benchmarks for `vivivi`.

---

## 1. Target Performance Metrics

| Metric                          | Target Value | Objective                                            |
| :------------------------------ | :----------: | :--------------------------------------------------- |
| **Startup Time (Cold Start)**   | **< 100 ms** | Instantaneous launch response                        |
| **Local Edits (Insert/Delete)** |  **$O(1)$**  | Ultra-fast typing latency via GapBuffer              |
| **Idle CPU Utilization**        |   **0.0%**   | Zero background overhead during idle                 |
| **RAM Footprint (Normal Docs)** | **< 20 MB**  | Minimal memory overhead                              |
| **Undo Block Rollback Time**    |  **< 5 ms**  | Immediate transactional undo execution               |

---

## 2. Benchmark Status (Phase 4.7 Progress)

| Measurement Item               | Measured Value | Status  | Notes                                                              |
| :----------------------------- | :------------: | :-----: | :----------------------------------------------------------------- |
| **Full Unit Test Duration**    |   **~0.8 s**   | ✅ PASS | Instantaneous test pass across 302 unit tests + 254 doctests       |
| **Incremental Build Time**     |   **~1.8 s**   | ✅ PASS | Rapid developer iteration cycles maintained                        |
| **Static Analysis Time**       |   **~1.0 s**   | ✅ PASS | Zero Clippy warnings maintained (`-D warnings`)                    |
| **Startup & Frame Latency**    |  **< 15 ms**   | ✅ PASS | Immediate alternate screen switch and event intake                 |
| **Window Split & Tree Search** |  **< 0.1 ms**  | ✅ PASS | Binary layout tree traversal and geometric neighbor spatial lookup |
| **Migemo Query Expansion**     |  **< 1.0 ms**  | ✅ PASS | Built-in SKK dictionary regex generation and incremental jumping   |
| **PDF Generation Duration**    |   **< 5 ms**   | ✅ PASS | Zero-dependency pure Rust direct binary output                     |
