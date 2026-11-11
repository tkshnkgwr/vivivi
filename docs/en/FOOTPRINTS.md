# vivivi: Resource Metrics & Footprints (FOOTPRINTS)

This document defines resource consumption targets and measured benchmarks for `vivivi`.

---

## 1. Target Performance Metrics

| Metric                          | Target Value | Objective                               |
| :------------------------------ | :----------: | :-------------------------------------- |
| **Startup Time (Cold Start)**   | **< 100 ms** | Instantaneous launch response           |
| **Local Edits (Insert/Delete)** |   **O(1)**   | Ultra-fast typing latency via GapBuffer |
| **Idle CPU Utilization**        |   **0.0%**   | Zero background overhead during idle    |
| **RAM Footprint (Normal Docs)** | **< 20 MB**  | Minimal memory overhead                 |
| **Undo Block Rollback Time**    |  **< 5 ms**  | Immediate transactional undo execution  |

---

## 2. Measured Benchmark Status (v0.9.17-beta)

| Measurement Item                | Measured Value | Status | Notes                                                                      |
| :------------------------------ | :------------: | :----: | :------------------------------------------------------------------------- |
| **GapBuffer 1,000 Char Insert** |  **~362 µs**   |  PASS  | ~8.3x faster than standard `String` (~3.00 ms) at the center of 100k chars |
| **Early Charset Detection**     |  **< 50 µs**   |  PASS  | Instant early return on first signature byte and 3 line endings            |
| **Startup & Frame Latency**     |  **< 15 ms**   |  PASS  | Immediate alternate screen switch and event intake                         |
| **Window Split & Tree Search**  |  **< 0.1 ms**  |  PASS  | Binary layout tree traversal and geometric neighbor spatial lookup         |
| **Migemo Query Expansion**      |  **< 1.0 ms**  |  PASS  | Built-in MIT dictionary regex generation and incremental jumping           |
| **Tree-sitter Incremental AST** |  **< 1.0 ms**  |  PASS  | Zero-latency incremental AST parsing upon buffer edits                     |
| **PDF Generation Duration**     |   **< 5 ms**   |  PASS  | Zero-dependency pure Rust direct binary output                             |
