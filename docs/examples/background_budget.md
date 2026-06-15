# Output from examples/background_budget.rs on various platforms

## Windows 11

```bash
     Running `target\release\examples\background_budget.exe`
AMD Ryzen 9 5950X 16-Core Processor - 16 physical cores / 32 threads
Render: lp 0, AboveNormal, 60 FPS, 1.5ms ALU + 16 MiB frame touch. Background work: 32 x 64 MiB cold assets.
Background workers: 15 primary LPs after reserving render, L3 round-robin order, median of 3 trial(s).

building asset bank: 32/32

width    throughput  frame p99    drops
    1     1.02 GB/s     2.50ms    0/120
    2     1.97 GB/s     3.62ms    0/120
    4     3.46 GB/s     5.41ms    0/120
    8     4.07 GB/s    12.65ms    0/120
   15     4.12 GB/s    19.62ms   20/120

Priority check at comparison width (8 workers):
  pool @ Normal           4.07 GB/s  p99   12.65ms  drops   0/120
  pool @ BelowNormal      4.17 GB/s  p99   15.26ms  drops   0/120
  pool @ Lowest           4.18 GB/s  p99   12.57ms  drops   0/120
  pool @ Background       4.07 GB/s  p99   12.68ms  drops   0/120
  render priority: AboveNormal THREAD_PRIORITY 1; clean 24/24, clamped 0/24, fallback 0/24, failed 0/24

Frame-safe budget: 8 workers (4.07 GB/s, p99 12.65ms, drops 0/120); throughput knee 4; background priority at 8 workers drops 0/120 -> 0/120.
```

## Linux (WSL2), Windows 11

```bash
     Running `target/release/examples/background_budget`
AMD Ryzen 9 5950X 16-Core Processor - 16 physical cores / 32 threads
Render: lp 0, AboveNormal, 60 FPS, 1.5ms ALU + 16 MiB frame touch. Background work: 32 x 64 MiB cold assets.
Background workers: 15 primary LPs after reserving render, L3 round-robin order, median of 3 trial(s).

building asset bank: 32/32

width    throughput  frame p99    drops
    1     0.99 GB/s     2.84ms    0/120
    2     1.85 GB/s     3.59ms    0/120
    4     3.23 GB/s     5.16ms    0/120
    8     4.05 GB/s    10.67ms    0/120
   15     4.20 GB/s    20.14ms   32/120

Priority check at comparison width (15 workers):
  pool @ Normal           4.20 GB/s  p99   20.14ms  drops  32/120
  pool @ BelowNormal      4.18 GB/s  p99   20.64ms  drops  13/120
  pool @ Lowest           4.13 GB/s  p99   20.32ms  drops  17/120
  pool @ Background       4.17 GB/s  p99   19.85ms  drops  12/120
  render priority: AboveNormal nice -5; clean 24/24, clamped 0/24, fallback 0/24, failed 0/24

Frame-safe budget: 8 workers (4.05 GB/s, p99 10.67ms, drops 0/120); throughput knee 8; background priority at 15 workers drops 32/120 -> 12/120.
```

## Windows 11 (on Apple M3 Max via Parallels)

```bash
     Running `target\release\examples\background_budget.exe`
Apple Silicon - 8 physical cores / 8 threads
Render: lp 0, AboveNormal, 60 FPS, 1.5ms ALU + 16 MiB frame touch. Background work: 16 x 64 MiB cold assets.
Background workers: 7 primary LPs after reserving render, L3 round-robin order, median of 3 trial(s).

building asset bank: 16/16

width    throughput  frame p99    drops
    1     1.11 GB/s     3.50ms    0/120
    2     2.28 GB/s     2.32ms    0/120
    4     2.94 GB/s     8.21ms    0/120
    7     3.54 GB/s     2.69ms    0/120

Priority check at comparison width (7 workers):
  pool @ Normal           3.54 GB/s  p99    2.69ms  drops   0/120
  pool @ BelowNormal      3.60 GB/s  p99    3.48ms  drops   0/120
  pool @ Lowest           4.68 GB/s  p99    6.68ms  drops   0/120
  pool @ Background       3.38 GB/s  p99    6.43ms  drops   0/120
  render priority: AboveNormal THREAD_PRIORITY 1; clean 21/21, clamped 0/21, fallback 0/21, failed 0/21

Frame-safe budget: 7 workers (3.54 GB/s, p99 2.69ms, drops 0/120); throughput knee 7; background priority at 7 workers drops 0/120 -> 0/120.
```

## macOS 26.5.1

```bash
     Running `target/release/examples/background_budget`
Apple M3 Max - 16 physical cores / 16 threads
Render: lp 0, AboveNormal, 60 FPS, 1.5ms ALU + 16 MiB frame touch. Background work: 24 x 64 MiB cold assets.
Background workers: 11 primary LPs after reserving render, L3 round-robin order, median of 3 trial(s).

building asset bank: 24/24

width    throughput  frame p99    drops
    1     1.13 GB/s     1.97ms    0/120
    2     2.36 GB/s     1.90ms    0/120
    4     4.69 GB/s     1.92ms    0/120
    8     9.36 GB/s     2.41ms    0/120
   11    11.19 GB/s     2.82ms    0/120

Priority check at comparison width (11 workers):
  pool @ Normal          11.19 GB/s  p99    2.82ms  drops   0/120
  pool @ BelowNormal     11.03 GB/s  p99    2.81ms  drops   0/120
  pool @ Lowest           5.61 GB/s  p99    3.57ms  drops   0/120
  pool @ Background       1.61 GB/s  p99    5.01ms  drops   0/120
  render priority: AboveNormal QoS UserInteractive; clean 24/24, clamped 0/24, fallback 0/24, failed 0/24

Frame-safe budget: 11 workers (11.19 GB/s, p99 2.82ms, drops 0/120); throughput knee 8; background priority at 11 workers drops 0/120 -> 0/120.
```

## Linux (baremetal), CachyOS, Desktop, 7.0.11-1-cachyos

```bash
     Running `target/release/examples/background_budget`
AMD Ryzen 9 5950X 16-Core Processor - 16 physical cores / 32 threads
Render: lp 0, AboveNormal, 60 FPS, 1.5ms ALU + 16 MiB frame touch. Background work: 32 x 64 MiB cold assets.
Background workers: 15 primary LPs after reserving render, L3 round-robin order, median of 3 trial(s).

building asset bank: 32/32

width    throughput  frame p99    drops
    1     1.15 GB/s     5.69ms    0/120
    2     2.17 GB/s     6.33ms    0/120
    4     3.95 GB/s     8.48ms    0/120
    8     5.25 GB/s    13.82ms    0/120
   15     5.05 GB/s    19.34ms   74/120

Priority check at comparison width (15 workers):
  pool @ Normal           5.05 GB/s  p99   19.34ms  drops  74/120
  pool @ BelowNormal      5.06 GB/s  p99   19.66ms  drops  87/120
  pool @ Lowest           5.05 GB/s  p99   19.82ms  drops  86/120
  pool @ Background       5.09 GB/s  p99   20.03ms  drops  66/120
  render priority: AboveNormal [Brokered] nice -5; clean 24/24, clamped 0/24, fallback 0/24, failed 0/24

Frame-safe budget: 8 workers (5.25 GB/s, p99 13.82ms, drops 0/120); throughput knee 8; background priority at 15 workers drops 74/120 -> 66/120.
```

## Linux (baremetal), Debian 13.5, Proxmox, 7.0.2-7-pve

```bash
     Running `target/release/examples/background_budget`
Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz - 4 physical cores / 8 threads
Render: lp 0, AboveNormal, 60 FPS, 1.5ms ALU + 16 MiB frame touch. Background work: 8 x 64 MiB cold assets.
Background workers: 3 primary LPs after reserving render, L3 round-robin order, median of 3 trial(s).

building asset bank: 8/8

width    throughput  frame p99    drops
    1     0.82 GB/s     2.94ms    0/120
    2     1.59 GB/s     3.19ms    0/120
    3     2.03 GB/s     4.45ms    0/120

Priority check at comparison width (3 workers):
  pool @ Normal           2.03 GB/s  p99    4.45ms  drops   0/120
  pool @ BelowNormal      2.03 GB/s  p99    4.15ms  drops   0/120
  pool @ Lowest           2.18 GB/s  p99    3.74ms  drops   0/120
  pool @ Background       2.28 GB/s  p99    3.72ms  drops   0/120
  render priority: AboveNormal nice -5; clean 18/18, clamped 0/18, fallback 0/18, failed 0/18

Frame-safe budget: 3 workers (2.03 GB/s, p99 4.45ms, drops 0/120); throughput knee 3; background priority at 3 workers drops 0/120 -> 0/120.
```

## Linux (LXC, limited to 2 cores, inside Proxmox), Debian 13.5, 7.0.2-7-pve

```bash
     Running `target/release/examples/background_budget`
Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz - 2 physical cores / 2 threads
Render: lp 1, AboveNormal, 60 FPS, 1.5ms ALU + 16 MiB frame touch. Background work: 8 x 64 MiB cold assets.
Background workers: 1 primary LPs after reserving render, L3 round-robin order, median of 3 trial(s).

building asset bank: 8/8

width    throughput  frame p99    drops
    1     0.79 GB/s     3.01ms    0/120

Priority check at comparison width (1 workers):
  pool @ Normal           0.79 GB/s  p99    3.01ms  drops   0/120
  pool @ BelowNormal      0.79 GB/s  p99    2.98ms  drops   0/120
  pool @ Lowest           0.79 GB/s  p99    3.02ms  drops   0/120
  pool @ Background       0.79 GB/s  p99    3.14ms  drops   0/120
  render priority: AboveNormal -> Normal [NoBroker] nice 0; clean 0/12, clamped 0/12, fallback 12/12, failed 0/12

Frame-safe budget: 1 workers (0.79 GB/s, p99 3.01ms, drops 0/120); throughput knee 1; background priority at 1 workers drops 0/120 -> 0/120.
```

## Linux (baremetal), Debian 12.14, MS-R1, 6.6.10-cix-build-generic

```bash
     Running `target/release/examples/background_budget`
CIX P1 CP8180 - 12 physical cores / 12 threads
Render: lp 0, AboveNormal, 60 FPS, 1.5ms ALU + 16 MiB frame touch. Background work: 16 x 64 MiB cold assets.
Background workers: 7 primary LPs after reserving render, L3 round-robin order, median of 3 trial(s).

building asset bank: 16/16

width    throughput  frame p99    drops
    1     0.62 GB/s     1.98ms    0/120
    2     1.18 GB/s     2.17ms    0/120
    4     2.26 GB/s     2.49ms    0/120
    7     3.92 GB/s     3.08ms    0/120

Priority check at comparison width (7 workers):
  pool @ Normal           3.92 GB/s  p99    3.08ms  drops   0/120
  pool @ BelowNormal      3.92 GB/s  p99    3.17ms  drops   0/120
  pool @ Lowest           3.94 GB/s  p99    3.10ms  drops   0/120
  pool @ Background       3.94 GB/s  p99    3.12ms  drops   0/120
  render priority: AboveNormal [Brokered] nice -5; clean 21/21, clamped 0/21, fallback 0/21, failed 0/21

Frame-safe budget: 7 workers (3.92 GB/s, p99 3.08ms, drops 0/120); throughput knee 7; background priority at 7 workers drops 0/120 -> 0/120.
```

## Linux (baremetal), SteamOS, Steam Deck, 6.11.11-valve29-1-neptune-611-g2dcfaf4df7ac

```bash
     Running `target/release/examples/background_budget`
AMD Custom APU 0405 - 4 physical cores / 8 threads
Render: lp 0, AboveNormal, 60 FPS, 1.5ms ALU + 16 MiB frame touch. Background work: 8 x 64 MiB cold assets.
Background workers: 3 primary LPs after reserving render, L3 round-robin order, median of 3 trial(s).

building asset bank: 8/8

width    throughput  frame p99    drops
    1     0.79 GB/s     3.75ms    0/120
    2     1.58 GB/s     4.00ms    0/120
    3     2.13 GB/s     4.80ms    0/120

Priority check at comparison width (3 workers):
  pool @ Normal           2.13 GB/s  p99    4.80ms  drops   0/120
  pool @ BelowNormal      2.14 GB/s  p99    4.45ms  drops   0/120
  pool @ Lowest           2.14 GB/s  p99    4.46ms  drops   0/120
  pool @ Background       2.14 GB/s  p99    4.45ms  drops   0/120
  render priority: AboveNormal [Brokered] nice -5; clean 18/18, clamped 0/18, fallback 0/18, failed 0/18

Frame-safe budget: 3 workers (2.13 GB/s, p99 4.80ms, drops 0/120); throughput knee 3; background priority at 3 workers drops 0/120 -> 0/120.
```
