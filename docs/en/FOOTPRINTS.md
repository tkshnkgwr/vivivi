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

## 2. Benchmark Status (Phase 3 & 4 Progress)

| Measurement Item             | Measured Value | Status  | Notes                                                |
| :--------------------------- | :------------: | :-----: | :--------------------------------------------------- |
| **Full Unit Test Duration**  |  **~0.03 s**   | ✅ PASS | Instantaneous test pass across all 103 tests         |
| **Incremental Build Time**   |  **~1.5 s**    | ✅ PASS | Rapid developer iteration cycles maintained          |
| **Static Analysis Time**     |  **~0.9 s**    | ✅ PASS | Zero Clippy warnings maintained                      |
| **Startup & Frame Latency**  |  **< 15 ms**   | ✅ PASS | Immediate alternate screen switch and event intake   |
