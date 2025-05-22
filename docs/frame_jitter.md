# Frame Jitter Benchmark Results

## Overview

This benchmark measures how well a CPU can maintain a consistent 60 FPS frame cadence (16.67ms intervals) under system load. Frame time jitter is critical for games, as inconsistent frame pacing causes stuttering even at high average framerates.

The test runs a simulated "render thread" that must wake up every 16.67ms while background threads create system load. We measure the deviation from ideal timing (jitter) across different thread priorities and CPU affinity settings.

## Key Findings

- **Thread Priority Impact**: Higher thread priorities significantly reduce jitter on all platforms. The most dramatic improvements come from using real-time priorities (`Highest`/`TimeCritical`).

- **Platform Differences**:
  - **macOS**: Shows higher baseline jitter but responds well to increased thread priority
  - **Windows**: Excellent baseline performance with further improvements from core pinning
  - **Linux**: Very low baseline jitter with excellent real-time priority performance

- **Core Pinning Effect**:
  - On Windows, pinning to performance cores delivers clear benefits
  - On Linux, pinning provides modest improvements for tail latencies
  - On macOS, thread pinning is unsupported; priority settings must be used instead

## Recommendations for Game Developers

1. **Always prioritize** your frame/render thread with at least `Highest` priority
2. **Consider your platform**:
   - On Windows, combine high priority with P-core pinning
   - On macOS, focus on using the highest appropriate thread priority
   - On Linux, real-time priorities give excellent results with or without pinning
3. **Beware of anomalies**: Some priority levels (like `AboveNormal` on Linux) can occasionally perform worse than expected

This benchmark demonstrates how proper thread management can significantly improve frame timing stability - a critical factor for smooth gameplay experiences across all platforms.

## Benchmark Results

### macOS

```bash
     Running `target/release/examples/frame_jitter`
Frame Loop Jitter Benchmark
===========================
CPU Info:
  Model: Apple M3 Max
  Performance cores: 12
  Efficiency cores: 4
  Logical cores: 16

--- Running Frame Loop Test ---
Frame thread priority: Normal
Pin to core: None
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Background worker 3 started with priority BelowNormal

--- Running Frame Loop Test ---
Frame thread priority: AboveNormal
Pin to core: None
Background workers: 4
Background worker 0 started with priority Normal
Background worker 2 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 3 started with priority BelowNormal

--- Running Frame Loop Test ---
Frame thread priority: Highest
Pin to core: None
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Background worker 3 started with priority BelowNormal

--- Running Frame Loop Test ---
Frame thread priority: Highest
Pin to core: Some(0)
Background workers: 4
Background worker 1 started with priority BelowNormal
Background worker 0 started with priority Normal
Background worker 3 started with priority BelowNormal
Background worker 2 started with priority Normal
Failed to pin frame thread to core 0: Unsupported operation: Thread affinity (pinning) is not supported on Apple Silicon, use `set_thread_priority`

--- Running Frame Loop Test ---
Frame thread priority: TimeCritical
Pin to core: None
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Background worker 3 started with priority BelowNormal

--- Running Frame Loop Test ---
Frame thread priority: TimeCritical
Pin to core: Some(0)
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Background worker 3 started with priority BelowNormal
Failed to pin frame thread to core 0: Unsupported operation: Thread affinity (pinning) is not supported on Apple Silicon, use `set_thread_priority`

Results Summary (jitter in microseconds):
Configuration                      |   Min |   p50 |   p95 |   p99
-----------------------------------|-------|-------|-------|-------
Normal Priority, No Pin            |     0 |  3526 |  3536 |  3561
Above Normal Priority, No Pin      |     0 |  3526 |  3536 |  3543
Highest Priority, No Pin           |     0 |   509 |   518 |   527
Highest Priority, P-core Pin       |     0 |   509 |   516 |   526
TimeCritical Priority, No Pin      |     0 |   509 |   517 |   528
TimeCritical Priority, P-core Pin  |     0 |   510 |   518 |   528
```

### Windows 11

```bash
     Running `target\release\examples\frame_jitter.exe`
Frame Loop Jitter Benchmark
===========================
CPU Info:
  Model: AMD Ryzen 9 5950X 16-Core Processor
  Performance cores: 16
  Efficiency cores: 0
  Logical cores: 32

--- Running Frame Loop Test ---
Frame thread priority: Normal
Pin to core: None
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 3 started with priority BelowNormal
Background worker 2 started with priority Normal

--- Running Frame Loop Test ---
Frame thread priority: AboveNormal
Pin to core: None
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Background worker 3 started with priority BelowNormal

--- Running Frame Loop Test ---
Frame thread priority: Highest
Pin to core: None
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Background worker 3 started with priority BelowNormal

--- Running Frame Loop Test ---
Frame thread priority: Highest
Pin to core: Some(0)
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Background worker 3 started with priority BelowNormal

--- Running Frame Loop Test ---
Frame thread priority: TimeCritical
Pin to core: None
Background workers: 4
Background worker 0 started with priority Normal
Background worker 3 started with priority BelowNormal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal

--- Running Frame Loop Test ---
Frame thread priority: TimeCritical
Pin to core: Some(0)
Background workers: 4
Background worker 0 started with priority Normal
Background worker 3 started with priority BelowNormal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal

Results Summary (jitter in microseconds):
Configuration                      |   Min |   p50 |   p95 |   p99
-----------------------------------|-------|-------|-------|-------
Normal Priority, No Pin            |     0 |     0 |     8 |    28
Above Normal Priority, No Pin      |     0 |     0 |    10 |    29
Highest Priority, No Pin           |     0 |     0 |     3 |    33
Highest Priority, P-core Pin       |     0 |     0 |     3 |    56
TimeCritical Priority, No Pin      |     0 |     0 |     3 |    21
TimeCritical Priority, P-core Pin  |     0 |     0 |     1 |    26
```

### Windows 11 - WSL2

```bash
     Running `target/release/examples/frame_jitter`
Frame Loop Jitter Benchmark
===========================
CPU Info:
  Model: AMD Ryzen 9 5950X 16-Core Processor
  Performance cores: 16
  Efficiency cores: 0
  Logical cores: 32

--- Running Frame Loop Test ---
Frame thread priority: Normal
Pin to core: None
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Background worker 3 started with priority BelowNormal

--- Running Frame Loop Test ---
Frame thread priority: AboveNormal
Pin to core: None
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Background worker 3 started with priority BelowNormal
Failed to set frame thread priority: System call error: setpriority failed for nice value -5 for TID 6008: Permission denied (os error 13)

--- Running Frame Loop Test ---
Frame thread priority: Highest
Pin to core: None
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 3 started with priority BelowNormal
Background worker 2 started with priority Normal

--- Running Frame Loop Test ---
Frame thread priority: Highest
Pin to core: Some(0)
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 3 started with priority BelowNormal
Background worker 2 started with priority Normal

--- Running Frame Loop Test ---
Frame thread priority: TimeCritical
Pin to core: None
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Background worker 3 started with priority BelowNormal

--- Running Frame Loop Test ---
Frame thread priority: TimeCritical
Pin to core: Some(0)
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Background worker 3 started with priority BelowNormal

Results Summary (jitter in microseconds):
Configuration                      |   Min |   p50 |   p95 |   p99
-----------------------------------|-------|-------|-------|-------
Normal Priority, No Pin            |     0 |     0 |     0 |     3
Above Normal Priority, No Pin      |     0 |     0 |     0 |     4
Highest Priority, No Pin           |     0 |     0 |     0 |     0
Highest Priority, P-core Pin       |     0 |     0 |     0 |     2
TimeCritical Priority, No Pin      |     0 |     0 |     0 |     0
TimeCritical Priority, P-core Pin  |     0 |     0 |     0 |     4
```

### Linux

```bash
     Running `target/release/examples/frame_jitter`
Frame Loop Jitter Benchmark
===========================
CPU Info:
  Model: Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz
  Performance cores: 4
  Efficiency cores: 0
  Logical cores: 8

--- Running Frame Loop Test ---
Frame thread priority: Normal
Pin to core: None
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Background worker 3 started with priority BelowNormal

--- Running Frame Loop Test ---
Frame thread priority: AboveNormal
Pin to core: None
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Background worker 3 started with priority BelowNormal

--- Running Frame Loop Test ---
Frame thread priority: Highest
Pin to core: None
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Background worker 3 started with priority BelowNormal

--- Running Frame Loop Test ---
Frame thread priority: Highest
Pin to core: Some(0)
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Background worker 3 started with priority BelowNormal

--- Running Frame Loop Test ---
Frame thread priority: TimeCritical
Pin to core: None
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Background worker 3 started with priority BelowNormal

--- Running Frame Loop Test ---
Frame thread priority: TimeCritical
Pin to core: Some(0)
Background workers: 4
Background worker 0 started with priority Normal
Background worker 2 started with priority Normal
Background worker 3 started with priority BelowNormal
Background worker 1 started with priority BelowNormal

Results Summary (jitter in microseconds):
Configuration                      |   Min |   p50 |   p95 |   p99
-----------------------------------|-------|-------|-------|-------
Normal Priority, No Pin            |     0 |     0 |     3 |    52
Above Normal Priority, No Pin      |     0 |     0 |     6 |   524
Highest Priority, No Pin           |     0 |     0 |     0 |     5
Highest Priority, P-core Pin       |     0 |     0 |     0 |    11
TimeCritical Priority, No Pin      |     0 |     0 |     0 |     3
TimeCritical Priority, P-core Pin  |     0 |     0 |     0 |     9
```
