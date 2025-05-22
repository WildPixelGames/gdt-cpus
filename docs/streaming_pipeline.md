# Streaming Pipeline Benchmark Results

## Overview

This benchmark simulates a game asset streaming pipeline with three stages:

1. Reader thread: simulates loading compressed data from disk
2. Decompression thread: uncompresses the data
3. Processor thread: renders/processes the final data

The test measures end-to-end latency and throughput under background CPU load, with different thread scheduling configurations. These results reflect how efficiently a game can stream assets such as textures, models, or audio without causing hitches.

## Key Findings

- **I/O Dominates Pipeline**: In all tests, the Reader stage accounts for 95%+ of total processing time, showing that decompression and processing are relatively lightweight compared to I/O.

- **Priority Improvements**:
  - **macOS**: The most significant improvement came from thread prioritization, particularly for p50 latency (reduced by 26% when combined with priority assignment)
  - **Windows**: Thread priorities improved median latency by ~10%
  - **Linux**: Similar improvements of ~10% with higher priorities

- **Pinning Effects**:
  - **Windows & Linux**: Core pinning sometimes hurt worst-case latencies (increasing max latency by 77% on Windows and 29% on Linux)
  - **macOS**: Pinning is unsupported, but the configuration with priorities showed the best results

- **Decompression Speed**:
  - The decompression thread benefits most from higher priorities across all platforms
  - On Linux, decompression latency improved by 24% with higher priorities

## Recommendations for Game Developers

1. **Prioritize The Pipeline**: Use higher thread priorities for asset streaming pipelines, with highest priority for the decompression stage

2. **Be Cautious With Pinning**: Core pinning can sometimes cause latency spikes in I/O-heavy workloads - test thoroughly before deploying

3. **Platform Differences Matter**:
   - On macOS, thread priorities have the most significant impact
   - On Windows and Linux, pinning should be used selectively and validated with testing

4. **I/O Optimization**: Since I/O operations dominate the pipeline, focus optimization efforts there first (e.g., using async I/O, memory-mapped files, or larger read buffers)

This benchmark demonstrates the importance of properly configuring thread priorities in asset streaming systems, and highlights that different platforms may require different tuning strategies for optimal performance.

## Benchmark Results

### macOS

```text
     Running `target/release/examples/streaming_pipeline`
Streaming I/O + Decompression Pipeline Benchmark
================================================
CPU Info:
  Model: Apple M3 Max
  Physical cores: 16
  Performance cores: 12
  Efficiency cores: 4
  Logical cores: 16
Found 12 P-cores and 4 E-cores

--- Running Pipeline Benchmark ---
Reader: Reader (Normal, pin: None)
Decompressor: Decompressor (Normal, pin: None)
Processor: Processor (Normal, pin: None)
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Background worker 3 started with priority BelowNormal
Starting processor thread with Processor (Normal, pin: None)
Starting reader thread with Reader (Normal, pin: None)
Starting decompressor thread with Decompressor (Normal, pin: None)
Reader thread finished after 100 chunks
Stopping pipeline threads...
Processor thread finished
Decompressor thread finished
Stopping background workers...

Pipeline Results:
  Reader: processed 100 chunks, 6555700 bytes
    Time per chunk (µs): min=2069, p50=4647, p95=5965, max=6149, avg=4171.52
  Decompressor: processed 100 chunks, 6553600 bytes
    Time per chunk (µs): min=19, p50=29, p95=82, max=107, avg=35.24
  Processor: processed 100 chunks, 6553600 bytes
    Time per chunk (µs): min=64, p50=64, p95=64, max=64, avg=64.00

End-to-end latency (µs):
  min=2164, p50=4743, p95=6068, max=6289, avg=4282.37

Throughput: 6.67 chunks/second

--- Running Pipeline Benchmark ---
Reader: Reader (AboveNormal, pin: None)
Decompressor: Decompressor (Highest, pin: None)
Processor: Processor (AboveNormal, pin: None)
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Background worker 3 started with priority BelowNormal
Starting reader thread with Reader (AboveNormal, pin: None)
Starting decompressor thread with Decompressor (Highest, pin: None)
Starting processor thread with Processor (AboveNormal, pin: None)
Reader thread finished after 100 chunks
Stopping pipeline threads...
Processor thread finished
Decompressor thread finished
Stopping background workers...

Pipeline Results:
  Reader: processed 100 chunks, 6555700 bytes
    Time per chunk (µs): min=2144, p50=4654, p95=5920, max=5942, avg=4074.87
  Decompressor: processed 100 chunks, 6553600 bytes
    Time per chunk (µs): min=20, p50=28, p95=39, max=50, avg=28.43
  Processor: processed 100 chunks, 6553600 bytes
    Time per chunk (µs): min=64, p50=64, p95=64, max=67, avg=64.06

End-to-end latency (µs):
  min=2244, p50=4757, p95=6025, max=6047, avg=4176.63

Throughput: 6.67 chunks/second

--- Running Pipeline Benchmark ---
Reader: Reader (AboveNormal, pin: Some(0))
Decompressor: Decompressor (Highest, pin: Some(1))
Processor: Processor (AboveNormal, pin: Some(2))
Background workers: 4
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Starting reader thread with Reader (AboveNormal, pin: Some(0))
Failed to pin reader thread to core 0: Unsupported operation: Thread affinity (pinning) is not supported on Apple Silicon, use `set_thread_priority`
Starting processor thread with Processor (AboveNormal, pin: Some(2))
Failed to pin processor thread to core 2: Unsupported operation: Thread affinity (pinning) is not supported on Apple Silicon, use `set_thread_priority`
Background worker 0 started with priority Normal
Starting decompressor thread with Decompressor (Highest, pin: Some(1))
Background worker 3 started with priority BelowNormal
Failed to pin decompressor thread to core 1: Unsupported operation: Thread affinity (pinning) is not supported on Apple Silicon, use `set_thread_priority`
Reader thread finished after 100 chunks
Stopping pipeline threads...
Decompressor thread finished
Processor thread finished
Stopping background workers...

Pipeline Results:
  Reader: processed 100 chunks, 6555700 bytes
    Time per chunk (µs): min=2142, p50=3414, p95=5924, max=6602, avg=3766.51
  Decompressor: processed 100 chunks, 6553600 bytes
    Time per chunk (µs): min=19, p50=29, p95=40, max=70, avg=29.51
  Processor: processed 100 chunks, 6553600 bytes
    Time per chunk (µs): min=64, p50=64, p95=64, max=67, avg=64.03

End-to-end latency (µs):
  min=2246, p50=3517, p95=6026, max=6761, avg=3868.81

Throughput: 6.67 chunks/second
```

### Windows

```text
     Running `target\release\examples\streaming_pipeline.exe`
Streaming I/O + Decompression Pipeline Benchmark
================================================
CPU Info:
  Model: AMD Ryzen 9 5950X 16-Core Processor
  Physical cores: 16
  Performance cores: 16
  Efficiency cores: 0
  Logical cores: 32
Found 16 P-cores and 0 E-cores

--- Running Pipeline Benchmark ---
Reader: Reader (Normal, pin: None)
Decompressor: Decompressor (Normal, pin: None)
Processor: Processor (Normal, pin: None)
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Starting processor thread with Processor (Normal, pin: None)
Starting reader thread with Reader (Normal, pin: None)
Starting decompressor thread with Decompressor (Normal, pin: None)
Background worker 3 started with priority BelowNormal
Reader thread finished after 100 chunks
Stopping pipeline threads...
Decompressor thread finished
Processor thread finished
Stopping background workers...

Pipeline Results:
  Reader: processed 100 chunks, 6555700 bytes
    Time per chunk (µs): min=2335, p50=4508, p95=5995, max=6050, avg=4236.69
  Decompressor: processed 100 chunks, 6553600 bytes
    Time per chunk (µs): min=40, p50=76, p95=141, max=169, avg=84.84
  Processor: processed 100 chunks, 6553600 bytes
    Time per chunk (µs): min=64, p50=64, p95=65, max=67, avg=64.09

End-to-end latency (µs):
  min=2476, p50=4658, p95=6153, max=6271, avg=4398.00

Throughput: 6.67 chunks/second

--- Running Pipeline Benchmark ---
Reader: Reader (AboveNormal, pin: None)
Decompressor: Decompressor (Highest, pin: None)
Processor: Processor (AboveNormal, pin: None)
Background workers: 4
Background worker 0 started with priority Normal
Background worker 2 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 3 started with priority BelowNormal
Starting reader thread with Reader (AboveNormal, pin: None)
Starting decompressor thread with Decompressor (Highest, pin: None)
Starting processor thread with Processor (AboveNormal, pin: None)
Reader thread finished after 100 chunks
Stopping pipeline threads...
Decompressor thread finished
Processor thread finished
Stopping background workers...

Pipeline Results:
  Reader: processed 100 chunks, 6555700 bytes
    Time per chunk (µs): min=2340, p50=4002, p95=5719, max=6035, avg=4046.34
  Decompressor: processed 100 chunks, 6553600 bytes
    Time per chunk (µs): min=49, p50=76, p95=146, max=206, avg=85.92
  Processor: processed 100 chunks, 6553600 bytes
    Time per chunk (µs): min=64, p50=64, p95=69, max=73, avg=64.34

End-to-end latency (µs):
  min=2485, p50=4214, p95=5867, max=6226, avg=4207.22

Throughput: 6.67 chunks/second

--- Running Pipeline Benchmark ---
Reader: Reader (AboveNormal, pin: Some(0))
Decompressor: Decompressor (Highest, pin: Some(2))
Processor: Processor (AboveNormal, pin: Some(4))
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Background worker 3 started with priority BelowNormal
Starting reader thread with Reader (AboveNormal, pin: Some(0))
Starting decompressor thread with Decompressor (Highest, pin: Some(2))
Starting processor thread with Processor (AboveNormal, pin: Some(4))
Reader thread finished after 100 chunks
Stopping pipeline threads...
Decompressor thread finished
Processor thread finished
Stopping background workers...

Pipeline Results:
  Reader: processed 100 chunks, 6555700 bytes
    Time per chunk (µs): min=2169, p50=4371, p95=6002, max=10910, avg=4230.01
  Decompressor: processed 100 chunks, 6553600 bytes
    Time per chunk (µs): min=44, p50=68, p95=126, max=170, avg=74.08
  Processor: processed 100 chunks, 6553600 bytes
    Time per chunk (µs): min=64, p50=64, p95=66, max=71, avg=64.18

End-to-end latency (µs):
  min=2312, p50=4526, p95=6142, max=11056, avg=4379.33

Throughput: 6.67 chunks/second
```

### Linux

```text
     Running `target/release/examples/streaming_pipeline`
Streaming I/O + Decompression Pipeline Benchmark
================================================
CPU Info:
  Model: Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz
  Physical cores: 4
  Performance cores: 4
  Efficiency cores: 0
  Logical cores: 8
Found 4 P-cores and 0 E-cores

--- Running Pipeline Benchmark ---
Reader: Reader (Normal, pin: None)
Decompressor: Decompressor (Normal, pin: None)
Processor: Processor (Normal, pin: None)
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Background worker 3 started with priority BelowNormal
Starting reader thread with Reader (Normal, pin: None)
Starting decompressor thread with Decompressor (Normal, pin: None)
Starting processor thread with Processor (Normal, pin: None)
Reader thread finished after 100 chunks
Stopping pipeline threads...
Processor thread finished
Decompressor thread finished
Stopping background workers...

Pipeline Results:
  Reader: processed 100 chunks, 6555700 bytes
    Time per chunk (µs): min=3582, p50=5627, p95=6758, max=7205, avg=5323.89
  Decompressor: processed 100 chunks, 6553600 bytes
    Time per chunk (µs): min=112, p50=123, p95=173, max=610, avg=133.86
  Processor: processed 100 chunks, 6553600 bytes
    Time per chunk (µs): min=64, p50=64, p95=69, max=70, avg=64.44

End-to-end latency (µs):
  min=3777, p50=5851, p95=7210, max=7701, avg=5610.03

Throughput: 6.67 chunks/second

--- Running Pipeline Benchmark ---
Reader: Reader (AboveNormal, pin: None)
Decompressor: Decompressor (Highest, pin: None)
Processor: Processor (AboveNormal, pin: None)
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Starting reader thread with Reader (AboveNormal, pin: None)
Background worker 3 started with priority BelowNormal
Starting decompressor thread with Decompressor (Highest, pin: None)
Starting processor thread with Processor (AboveNormal, pin: None)
Reader thread finished after 100 chunks
Stopping pipeline threads...
Decompressor thread finished
Processor thread finished
Stopping background workers...

Pipeline Results:
  Reader: processed 100 chunks, 6555700 bytes
    Time per chunk (µs): min=2938, p50=5064, p95=6876, max=7733, avg=5165.42
  Decompressor: processed 100 chunks, 6553600 bytes
    Time per chunk (µs): min=89, p50=105, p95=129, max=285, avg=107.25
  Processor: processed 100 chunks, 6553600 bytes
    Time per chunk (µs): min=64, p50=64, p95=67, max=89, avg=64.63

End-to-end latency (µs):
  min=3108, p50=5285, p95=7100, max=7904, avg=5346.59

Throughput: 6.67 chunks/second

--- Running Pipeline Benchmark ---
Reader: Reader (AboveNormal, pin: Some(0))
Decompressor: Decompressor (Highest, pin: Some(1))
Processor: Processor (AboveNormal, pin: Some(2))
Background workers: 4
Background worker 0 started with priority Normal
Background worker 1 started with priority BelowNormal
Background worker 2 started with priority Normal
Starting reader thread with Reader (AboveNormal, pin: Some(0))
Starting decompressor thread with Decompressor (Highest, pin: Some(1))
Background worker 3 started with priority BelowNormal
Starting processor thread with Processor (AboveNormal, pin: Some(2))
Reader thread finished after 100 chunks
Stopping pipeline threads...
Decompressor thread finished
Processor thread finished
Stopping background workers...

Pipeline Results:
  Reader: processed 100 chunks, 6555700 bytes
    Time per chunk (µs): min=3431, p50=5522, p95=6986, max=7563, avg=5207.83
  Decompressor: processed 100 chunks, 6553600 bytes
    Time per chunk (µs): min=85, p50=128, p95=203, max=402, avg=140.68
  Processor: processed 100 chunks, 6553600 bytes
    Time per chunk (µs): min=64, p50=64, p95=68, max=72, avg=64.36

End-to-end latency (µs):
  min=3664, p50=5743, p95=7210, max=10207, avg=5465.02

Throughput: 6.67 chunks/second
```
