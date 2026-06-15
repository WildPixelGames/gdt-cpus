# Output from examples/frame_jitter.rs on various platforms

## [ ] Windows 11

```bash
```

## [ ] Linux (WSL2), Windows 11

```bash
```

## Windows 11 (on Apple M3 Max via Parallels)

```bash
     Running `target\release\examples\frame_jitter.exe`
CPU: Apple Silicon - 8 P + 0 E cores / 8 threads
frame_jitter: skipped; no SMT detected (logical == physical). Run this on an SMT-capable CPU to compare logical vs physical worker-pool sizing.
```

## macOS 26.5.1

```bash
     Running `target/release/examples/frame_jitter`
CPU: Apple M3 Max - 12 P + 4 E cores / 16 threads
frame_jitter: skipped; no SMT detected (logical == physical). Run this on an SMT-capable CPU to compare logical vs physical worker-pool sizing.
```

## Linux (baremetal), CachyOS, Desktop, 7.0.11-1-cachyos

```bash
     Running `target/release/examples/frame_jitter`
CPU: AMD Ryzen 9 5950X 16-Core Processor - 16 P + 0 E cores / 32 threads
Render thread: 60 FPS, 5.5ms work/frame, 600 frames. Dropped = work overran the 16.7ms budget.

Round 1 - worker pool = 32 LOGICAL cores (one per hardware thread, oversubscribed):
  render @ Normal        nice 0                          dropped  600/600   p99   31.3ms   worst   31.4ms
  render @ AboveNormal   [Brokered] nice -5              dropped  427/600   p99   21.4ms   worst   24.8ms
  render @ Highest       [Brokered] nice -10             dropped  185/600   p99   18.5ms   worst   20.5ms
  render @ TimeCritical  [Brokered, Clamped] nice -15    dropped   76/600   p99   17.5ms   worst   18.1ms

Round 2 - worker pool = 16 PHYSICAL cores (SMT siblings left free):
  render @ Normal        nice 0                          dropped    0/600   p99   15.1ms   worst   15.3ms
  render @ AboveNormal   [Brokered] nice -5              dropped    0/600   p99   15.1ms   worst   15.6ms
  render @ Highest       [Brokered] nice -10             dropped    0/600   p99   15.1ms   worst   15.4ms
  render @ TimeCritical  [Brokered, Clamped] nice -15    dropped    0/600   p99   15.1ms   worst   15.6ms

Render frames dropped at Normal: 600/600 (logical pool) vs 0/600 (physical pool).
```

## Linux (baremetal), Debian 13.5, Proxmox, 7.0.2-7-pve

```bash
     Running `target/release/examples/frame_jitter`
CPU: Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz - 4 P + 0 E cores / 8 threads
Render thread: 60 FPS, 5.5ms work/frame, 600 frames. Dropped = work overran the 16.7ms budget.

Round 1 - worker pool = 8 LOGICAL cores (one per hardware thread, oversubscribed):
  render @ Normal        nice 0                          dropped   11/600   p99   23.8ms   worst  137.1ms
  render @ AboveNormal   nice -5                         dropped    6/600   p99   19.8ms   worst   61.7ms
  render @ Highest       nice -10                        dropped    4/600   p99    7.4ms   worst   25.3ms
  render @ TimeCritical  nice -20                        dropped    1/600   p99    4.7ms   worst   23.4ms

Round 2 - worker pool = 4 PHYSICAL cores (SMT siblings left free):
  render @ Normal        nice 0                          dropped    1/600   p99   10.7ms   worst   17.9ms
  render @ AboveNormal   nice -5                         dropped    1/600   p99    9.9ms   worst   20.7ms
  render @ Highest       nice -10                        dropped    0/600   p99   10.5ms   worst   13.0ms
  render @ TimeCritical  nice -20                        dropped    0/600   p99   10.2ms   worst   15.2ms

Render frames dropped at Normal: 11/600 (logical pool) vs 1/600 (physical pool).
```

## Linux (LXC, limited to 2 cores, inside Proxmox), Debian 13.5, 7.0.2-7-pve

```bash
     Running `target/release/examples/frame_jitter`
CPU: Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz - 2 P + 0 E cores / 2 threads
frame_jitter: skipped; no SMT detected (logical == physical). Run this on an SMT-capable CPU to compare logical vs physical worker-pool sizing.
```

## Linux (baremetal), Debian 12.14, MS-R1, 6.6.10-cix-build-generic

```bash
     Running `target/release/examples/frame_jitter`
CPU: CIX P1 CP8180 - 8 P + 0 E cores / 12 threads
frame_jitter: skipped; no SMT detected (logical == physical). Run this on an SMT-capable CPU to compare logical vs physical worker-pool sizing.
```

## Linux (baremetal), SteamOS, Steam Deck, 6.11.11-valve29-1-neptune-611-g2dcfaf4df7ac

```bash
     Running `target/release/examples/frame_jitter`
CPU: AMD Custom APU 0405 - 4 P + 0 E cores / 8 threads
Render thread: 60 FPS, 5.5ms work/frame, 600 frames. Dropped = work overran the 16.7ms budget.

Round 1 - worker pool = 8 LOGICAL cores (one per hardware thread, oversubscribed):
  render @ Normal        nice 0                          dropped    1/600   p99    9.0ms   worst   18.9ms
  render @ AboveNormal   [Brokered] nice -5              dropped    0/600   p99    7.2ms   worst    7.2ms
  render @ Highest       [Brokered] nice -10             dropped    0/600   p99    7.2ms   worst    7.2ms
  render @ TimeCritical  [Brokered, Clamped] nice -15    dropped    0/600   p99    3.9ms   worst    7.3ms

Round 2 - worker pool = 4 PHYSICAL cores (SMT siblings left free):
  render @ Normal        nice 0                          dropped    0/600   p99    4.0ms   worst   13.9ms
  render @ AboveNormal   [Brokered] nice -5              dropped    0/600   p99    3.9ms   worst    4.3ms
  render @ Highest       [Brokered] nice -10             dropped    0/600   p99    3.9ms   worst    7.3ms
  render @ TimeCritical  [Brokered, Clamped] nice -15    dropped    0/600   p99    3.9ms   worst    4.0ms

Render frames dropped at Normal: 1/600 (logical pool) vs 0/600 (physical pool).
```
