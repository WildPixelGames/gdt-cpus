# Audio Latency Benchmark

## macOS

```bash
     Running `target/release/examples/audio`
CPU Info:
  Model: Apple M3 Max
  Physical cores: 16
  Logical cores: 16
  Performance cores: 12
  Efficiency cores: 4
Benchmarking audio latency with 12 cores occupied
 Benchmarking priority: Background
  p50: 0µs, p95: 36µs, p99: 661µs
 Benchmarking priority: Lowest
  p50: 0µs, p95: 162µs, p99: 1101µs
 Benchmarking priority: BelowNormal
  p50: 0µs, p95: 32µs, p99: 706µs
 Benchmarking priority: Normal
  p50: 0µs, p95: 78µs, p99: 726µs
 Benchmarking priority: AboveNormal
  p50: 0µs, p95: 218µs, p99: 4019µs
 Benchmarking priority: Highest
  p50: 0µs, p95: 0µs, p99: 372µs
 Benchmarking priority: TimeCritical
  p50: 0µs, p95: 0µs, p99: 0µs
```

## Windows 11

```bash
     Running `target\release\examples\audio.exe`
CPU Info:
  Model: AMD Ryzen 9 5950X 16-Core Processor
  Physical cores: 16
  Logical cores: 32
  Performance cores: 16
  Efficiency cores: 0
Benchmarking audio latency with 16 cores occupied
 Benchmarking priority: Background
  p50: 0µs, p95: 0µs, p99: 0µs
 Benchmarking priority: Lowest
  p50: 0µs, p95: 0µs, p99: 0µs
 Benchmarking priority: BelowNormal
  p50: 0µs, p95: 0µs, p99: 0µs
 Benchmarking priority: Normal
  p50: 0µs, p95: 0µs, p99: 0µs
 Benchmarking priority: AboveNormal
  p50: 0µs, p95: 0µs, p99: 0µs
 Benchmarking priority: Highest
  p50: 0µs, p95: 0µs, p99: 0µs
 Benchmarking priority: TimeCritical
  p50: 0µs, p95: 0µs, p99: 0µs
```

## Windows 11 - WSL2

```bash
     Running `target/release/examples/audio`
CPU Info:
  Model: AMD Ryzen 9 5950X 16-Core Processor
  Physical cores: 16
  Logical cores: 32
  Performance cores: 16
  Efficiency cores: 0
Benchmarking audio latency with 16 cores occupied
 Benchmarking priority: Background
  p50: 0µs, p95: 0µs, p99: 2µs
 Benchmarking priority: Lowest
  p50: 0µs, p95: 0µs, p99: 3µs
 Benchmarking priority: BelowNormal
  p50: 0µs, p95: 0µs, p99: 2µs
 Benchmarking priority: Normal
  p50: 0µs, p95: 0µs, p99: 2µs
 Benchmarking priority: AboveNormal
  p50: 0µs, p95: 0µs, p99: 2µs
 Benchmarking priority: Highest
  p50: 0µs, p95: 0µs, p99: 2µs
 Benchmarking priority: TimeCritical
  p50: 0µs, p95: 0µs, p99: 3µs
```

## Linux - Bare Metal, Proxmox, typical system load for a homelab server

```bash
     Running `target/release/examples/audio`
CPU Info:
  Model: Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz
  Physical cores: 4
  Logical cores: 8
  Performance cores: 4
  Efficiency cores: 0
Benchmarking audio latency with 4 cores occupied
 Benchmarking priority: Background
  p50: 0µs, p95: 946920µs, p99: 948188µs
 Benchmarking priority: Lowest
  p50: 0µs, p95: 946635µs, p99: 947979µs
 Benchmarking priority: BelowNormal
  p50: 0µs, p95: 0µs, p99: 72µs
 Benchmarking priority: Normal
  p50: 0µs, p95: 946926µs, p99: 948129µs
 Benchmarking priority: AboveNormal
  p50: 0µs, p95: 2µs, p99: 11µs
 Benchmarking priority: Highest
  p50: 0µs, p95: 0µs, p99: 10µs
 Benchmarking priority: TimeCritical
  p50: 0µs, p95: 0µs, p99: 0µs
```

## Linux - LXC, Proxmox, limited to 2 cores

```bash
     Running `target/release/examples/audio`
CPU Info:
  Model: Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz
  Physical cores: 2
  Logical cores: 2
  Performance cores: 2
  Efficiency cores: 0
Benchmarking audio latency with 2 cores occupied
 Benchmarking priority: Background
  p50: 31654µs, p95: 272665µs, p99: 274662µs
 Benchmarking priority: Lowest
  p50: 693µs, p95: 112667µs, p99: 114667µs
 Benchmarking priority: BelowNormal
  p50: 2275µs, p95: 36665µs, p99: 38664µs
 Benchmarking priority: Normal
  p50: 0µs, p95: 2685µs, p99: 4667µs
 Benchmarking priority: AboveNormal
  p50: 3µs, p95: 3001µs, p99: 4666µs
 Benchmarking priority: Highest
  p50: 9µs, p95: 3333µs, p99: 5001µs
 Benchmarking priority: TimeCritical
  p50: 4µs, p95: 3335µs, p99: 5658µs
```
