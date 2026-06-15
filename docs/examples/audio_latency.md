# Output from examples/audio_latency.rs on various platforms

## Windows 11

```bash
     Running `target\release\examples\audio_latency.exe`
CPU: AMD Ryzen 9 5950X 16-Core Processor - 16 cores / 32 threads (16 P + 0 E)
Distinct priority levels here: 7 of 7 (each rung maps to a different scheduler weight)
Synthetic load: 16 spinners at Normal priority pinned to 16 P-cores (the feeder shares core 0).

Running audio feeder ~10s (cap 15s) at each priority:

  Background THREAD_PRIORITY -15
    p50     0us  p95    2.6s  p99    3.9s     39/1875 buffers  STARVED
  Lowest THREAD_PRIORITY -2
    p50     0us  p95    2.4s  p99    2.4s     47/1875 buffers  STARVED
  BelowNormal THREAD_PRIORITY -1
    p50     0us  p95    2.4s  p99    2.7s     49/1875 buffers  STARVED
  Normal THREAD_PRIORITY 0
    p50     0us  p95  31.3ms  p99  33.2ms   1537/1875 buffers  STARVED
  AboveNormal THREAD_PRIORITY 1
    p50     0us  p95     0us  p99  30.6ms   1875/1875 buffers
  Highest THREAD_PRIORITY 2
    p50     0us  p95     0us  p99  30.8ms   1875/1875 buffers
  TimeCritical THREAD_PRIORITY 15
    p50     0us  p95     0us  p99   304us   1875/1875 buffers

AboveNormal and up keep up (every buffer delivered); weaker levels starved.
```

## Linux (WSL2), Windows 11

```bash
     Running `target/release/examples/audio_latency`
CPU: AMD Ryzen 9 5950X 16-Core Processor - 16 cores / 32 threads (16 P + 0 E)
Distinct priority levels here: 7 of 7 (each rung maps to a different scheduler weight)
Synthetic load: 16 spinners at Normal priority pinned to 16 P-cores (the feeder shares core 0).

Running audio feeder ~10s (cap 15s) at each priority:

  Background nice 19
    p50 270.7ms  p95 274.7ms  p99 274.7ms     55/1875 buffers  STARVED
  Lowest nice 10
    p50  34.7ms  p95  38.7ms  p99  38.7ms    365/1875 buffers  STARVED
  BelowNormal nice 5
    p50  10.7ms  p95  14.7ms  p99  14.7ms    925/1875 buffers  STARVED
  Normal nice 0
    p50   2.7ms  p95   2.7ms  p99   2.7ms   1865/1875 buffers  STARVED
  AboveNormal nice -5
    p50     0us  p95     4us  p99   4.0ms   1875/1875 buffers
  Highest nice -10
    p50     0us  p95   2.7ms  p99   2.7ms   1875/1875 buffers
  TimeCritical nice -20
    p50     0us  p95     2us  p99    18us   1875/1875 buffers

AboveNormal and up keep up (every buffer delivered); weaker levels starved.
```

## Windows 11 (on Apple M3 Max via Parallels)

```bash
     Running `target\release\examples\audio_latency.exe`
CPU: Apple Silicon - 8 cores / 8 threads (8 P + 0 E)
Distinct priority levels here: 7 of 7 (each rung maps to a different scheduler weight)
Synthetic load: 8 spinners at Normal priority pinned to 8 P-cores (the feeder shares core 0).

Running audio feeder ~10s (cap 15s) at each priority:

  Background THREAD_PRIORITY -15
    p50     0us  p95    2.6s  p99    2.6s     42/1875 buffers  STARVED
  Lowest THREAD_PRIORITY -2
    p50     0us  p95    2.6s  p99    2.6s     40/1875 buffers  STARVED
  BelowNormal THREAD_PRIORITY -1
    p50     0us  p95    2.6s  p99    2.6s     40/1875 buffers  STARVED
  Normal THREAD_PRIORITY 0
    p50     0us  p95  29.6ms  p99  32.7ms   1557/1875 buffers  STARVED
  AboveNormal THREAD_PRIORITY 1
    p50     0us  p95     6us  p99  30.6ms   1875/1875 buffers
  Highest THREAD_PRIORITY 2
    p50     0us  p95     2us  p99  30.8ms   1875/1875 buffers
  TimeCritical THREAD_PRIORITY 15
    p50     0us  p95     0us  p99    11us   1875/1875 buffers

AboveNormal and up keep up (every buffer delivered); weaker levels starved.
```

## macOS 26.5.1

```bash
     Running `target/release/examples/audio_latency`
CPU: Apple M3 Max - 16 cores / 16 threads (12 P + 4 E)
Distinct priority levels here: 7 of 7 (each rung maps to a different scheduler weight)
Synthetic load: 12 spinners at Normal priority pinned to 12 P-cores (the feeder shares core 0).

Running audio feeder ~10s (cap 15s) at each priority:

  Background QoS Background
    p50     0us  p95     2us  p99    63us   1875/1875 buffers
  Lowest QoS Utility
    p50     0us  p95     3us  p99    40us   1875/1875 buffers
  BelowNormal QoS Default
    p50     0us  p95     4us  p99   231us   1875/1875 buffers
  Normal QoS UserInitiated
    p50     0us  p95     2us  p99    28us   1875/1875 buffers
  AboveNormal QoS UserInteractive
    p50     0us  p95     2us  p99    44us   1875/1875 buffers
  Highest QoS UserInteractive
    p50     0us  p95     2us  p99    67us   1875/1875 buffers
  TimeCritical [Realtime] SCHED_RR 47
    p50     0us  p95     0us  p99     0us   1875/1875 buffers

Even Background delivered every buffer here - this box isn't contended enough to starve the feeder.
```

## Linux (baremetal), CachyOS, Desktop, 7.0.11-1-cachyos

```bash
     Running `target/release/examples/audio_latency`
CPU: AMD Ryzen 9 5950X 16-Core Processor - 16 cores / 32 threads (16 P + 0 E)
Distinct priority levels here: 7 of 7 (each rung maps to a different scheduler weight)
Synthetic load: 16 spinners at Normal priority pinned to 16 P-cores (the feeder shares core 0).

Running audio feeder ~10s (cap 15s) at each priority:

  Background nice 19
    p50 133.7ms  p95 133.7ms  p99 134.7ms    110/1875 buffers  STARVED
  Lowest nice 10
    p50  15.7ms  p95  15.7ms  p99  15.7ms    728/1875 buffers  STARVED
  BelowNormal nice 5
    p50   2.7ms  p95   3.7ms  p99   3.7ms   1832/1875 buffers  STARVED
  Normal nice 0
    p50   429us  p95   1.3ms  p99   1.3ms   1875/1875 buffers
  AboveNormal [Brokered] nice -5
    p50     0us  p95   999us  p99   1.0ms   1875/1875 buffers
  Highest [Brokered] nice -10
    p50     0us  p95    13us  p99   1.7ms   1875/1875 buffers
  TimeCritical [Brokered, Clamped] nice -15
    p50     0us  p95   334us  p99   336us   1875/1875 buffers

Normal and up keep up (every buffer delivered); weaker levels starved.
```

## Linux (baremetal), Debian 13.5, Proxmox, 7.0.2-7-pve

```bash
     Running `target/release/examples/audio_latency`
CPU: Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz - 4 cores / 8 threads (4 P + 0 E)
Distinct priority levels here: 7 of 7 (each rung maps to a different scheduler weight)
Synthetic load: 4 spinners at Normal priority pinned to 4 P-cores (the feeder shares core 0).

Running audio feeder ~10s (cap 15s) at each priority:

  Background nice 19
    p50 202.7ms  p95 203.7ms  p99 208.7ms     80/1875 buffers  STARVED
  Lowest nice 10
    p50  25.7ms  p95  25.7ms  p99  26.7ms    520/1875 buffers  STARVED
  BelowNormal nice 5
    p50   6.7ms  p95   7.7ms  p99   8.7ms   1299/1875 buffers  STARVED
  Normal nice 0
    p50   666us  p95   987us  p99   2.7ms   1875/1875 buffers
  AboveNormal nice -5
    p50     0us  p95   2.3ms  p99   2.7ms   1875/1875 buffers
  Highest nice -10
    p50     0us  p95   1.8ms  p99   3.1ms   1875/1875 buffers
  TimeCritical nice -20
    p50     0us  p95   537us  p99   2.6ms   1875/1875 buffers

Normal and up keep up (every buffer delivered); weaker levels starved.
```

## Linux (LXC, limited to 2 cores, inside Proxmox), Debian 13.5, 7.0.2-7-pve

```bash
     Running `target/release/examples/audio_latency`
CPU: Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz - 2 cores / 2 threads (2 P + 0 E)
Distinct priority levels here: 7 of 7 (each rung maps to a different scheduler weight)
Synthetic load: 2 spinners at Normal priority pinned to 2 P-cores (the feeder shares core 1).

Running audio feeder ~10s (cap 15s) at each priority:

  Background nice 19
    p50 202.7ms  p95 203.7ms  p99 203.7ms     76/1875 buffers  STARVED
  Lowest nice 10
    p50  25.7ms  p95  31.7ms  p99  40.7ms    607/1875 buffers  STARVED
  BelowNormal nice 5
    p50   6.7ms  p95  10.3ms  p99  16.7ms   1331/1875 buffers  STARVED
  Normal nice 0
    p50   665us  p95   3.3ms  p99   8.9ms   1875/1875 buffers
  AboveNormal -> Normal [NoBroker] nice 0
    p50   665us  p95   2.7ms  p99   5.7ms   1875/1875 buffers
  Highest -> Normal [NoBroker] nice 0
    p50   665us  p95   2.7ms  p99   8.2ms   1875/1875 buffers
  TimeCritical -> Normal [NoBroker] nice 0
    p50   666us  p95   1.7ms  p99   5.0ms   1875/1875 buffers

Normal and up keep up (every buffer delivered); weaker levels starved.
```

## Linux (baremetal), Debian 12.14, MS-R1, 6.6.10-cix-build-generic

```bash
     Running `target/release/examples/audio_latency`
CPU: CIX P1 CP8180 - 12 cores / 12 threads (8 P + 0 E)
Distinct priority levels here: 7 of 7 (each rung maps to a different scheduler weight)
Synthetic load: 8 spinners at Normal priority pinned to 8 P-cores (the feeder shares core 0).

Running audio feeder ~10s (cap 15s) at each priority:

  Background nice 19
    p50 270.7ms  p95 274.7ms  p99 274.7ms     55/1875 buffers  STARVED
  Lowest nice 10
    p50  34.7ms  p95  38.7ms  p99  38.7ms    365/1875 buffers  STARVED
  BelowNormal nice 5
    p50  10.7ms  p95  14.7ms  p99  14.7ms    925/1875 buffers  STARVED
  Normal nice 0
    p50   2.7ms  p95   2.7ms  p99   2.7ms   1875/1875 buffers
  AboveNormal [Brokered] nice -5
    p50     0us  p95     1us  p99   4.0ms   1875/1875 buffers
  Highest [Brokered] nice -10
    p50     0us  p95   2.7ms  p99   2.7ms   1875/1875 buffers
  TimeCritical [Brokered, Clamped] nice -15
    p50     0us  p95     6us  p99   4.0ms   1875/1875 buffers

Normal and up keep up (every buffer delivered); weaker levels starved.
```

## Linux (baremetal), SteamOS, Steam Deck, 6.11.11-valve29-1-neptune-611-g2dcfaf4df7ac

```bash
     Running `target/release/examples/audio_latency`
CPU: AMD Custom APU 0405 - 4 cores / 8 threads (4 P + 0 E)
Distinct priority levels here: 7 of 7 (each rung maps to a different scheduler weight)
Synthetic load: 4 spinners at Normal priority pinned to 4 P-cores (the feeder shares core 0).

Running audio feeder ~10s (cap 15s) at each priority:

  Background nice 19
    p50 224.7ms  p95 228.0ms  p99 401.9ms     70/1875 buffers  STARVED
  Lowest nice 10
    p50  28.0ms  p95  31.3ms  p99  35.6ms    459/1875 buffers  STARVED
  BelowNormal nice 5
    p50   8.0ms  p95  11.3ms  p99  12.5ms   1154/1875 buffers  STARVED
  Normal nice 0
    p50   1.3ms  p95   2.2ms  p99   5.5ms   1875/1875 buffers
  AboveNormal [Brokered] nice -5
    p50     0us  p95   2.7ms  p99   3.1ms   1875/1875 buffers
  Highest [Brokered] nice -10
    p50     0us  p95   1.3ms  p99   2.0ms   1875/1875 buffers
  TimeCritical [Brokered, Clamped] nice -15
    p50     0us  p95     5us  p99   2.7ms   1875/1875 buffers

Normal and up keep up (every buffer delivered); weaker levels starved.
```
