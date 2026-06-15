# Output from examples/reserved_core.rs on various platforms

## Windows 11

```bash
     Running `target\release\examples\reserved_core.exe`
AMD Ryzen 9 5950X 16-Core Processor - 16 P-core primaries
Feeder: 48 kHz / 256-frame buffer (5333us deadline). Load: 15 Highest spinners on the other P-cores, spawned once.

Load spinner priority: Highest THREAD_PRIORITY 2; clean 15/15, clamped 0/15, fallback 0/15, failed 0/15
Lower jitter is better. Which lever buys it - own core, or top priority?

  reserved core  @ Normal         p50      0  p95      0  p99     44 us   [THREAD_PRIORITY 0]
  contended core @ TimeCritical   p99 over 5 runs: 0 0 0 0 6
                                  best 0 us ... worst 6 us

Placement vs priority: shared core @ TimeCritical wins (contended 0..6 us vs reserved p99 44 us).
```

## Linux (WSL2), Windows 11

```bash
     Running `target/release/examples/reserved_core`
AMD Ryzen 9 5950X 16-Core Processor - 16 P-core primaries
Feeder: 48 kHz / 256-frame buffer (5333us deadline). Load: 15 Highest spinners on the other P-cores, spawned once.

Load spinner priority: Highest nice -10; clean 15/15, clamped 0/15, fallback 0/15, failed 0/15
Lower jitter is better. Which lever buys it - own core, or top priority?

  reserved core  @ Normal         p50      0  p95      1  p99      7 us   [nice 0]
  contended core @ TimeCritical   p99 over 5 runs: 2670 2670 2671 2671 2670
                                  best 2670 us ... worst 2671 us

Placement vs priority: own core @ Normal wins the tail (reserved p99 7 us vs contended 2670..2671 us).
```

## Windows 11 (on Apple M3 Max via Parallels)

```bash
     Running `target\release\examples\reserved_core.exe`
Apple Silicon - 8 P-core primaries
Feeder: 48 kHz / 256-frame buffer (5333us deadline). Load: 7 Highest spinners on the other P-cores, spawned once.

Load spinner priority: Highest THREAD_PRIORITY 2; clean 7/7, clamped 0/7, fallback 0/7, failed 0/7
Lower jitter is better. Which lever buys it - own core, or top priority?

  reserved core  @ Normal         p50      0  p95      1  p99    319 us   [THREAD_PRIORITY 0]
  contended core @ TimeCritical   p99 over 5 runs: 8 6 6 4 10
                                  best 4 us ... worst 10 us

Placement vs priority: shared core @ TimeCritical wins (contended 4..10 us vs reserved p99 319 us).
```

## macOS 26.5.1

```bash
     Running `target/release/examples/reserved_core`
Apple M3 Max - 12 P-core primaries
Feeder: 48 kHz / 256-frame buffer (5333us deadline). Load: 11 Highest spinners on the other P-cores, spawned once.

Load spinner priority: Highest QoS UserInteractive; clean 11/11, clamped 0/11, fallback 0/11, failed 0/11
Lower jitter is better. Which lever buys it - own core, or top priority?

  reserved core  @ Normal         p50      0  p95      2  p99     10 us   [QoS UserInitiated]
  contended core @ TimeCritical   p99 over 5 runs: 0 0 0 0 0
                                  best 0 us ... worst 0 us

Placement vs priority: shared core @ TimeCritical wins (contended 0..0 us vs reserved p99 10 us).
```

## Linux (baremetal), CachyOS, Desktop, 7.0.11-1-cachyos

```bash
     Running `target/release/examples/reserved_core`
AMD Ryzen 9 5950X 16-Core Processor - 16 P-core primaries
Feeder: 48 kHz / 256-frame buffer (5333us deadline). Load: 15 Highest spinners on the other P-cores, spawned once.

Load spinner priority: Highest [Brokered] nice -10; clean 15/15, clamped 0/15, fallback 0/15, failed 0/15
Lower jitter is better. Which lever buys it - own core, or top priority?

  reserved core  @ Normal         p50      0  p95      1  p99      2 us   [nice 0]
  contended core @ TimeCritical   p99 over 5 runs: 1001 1001 1001 1001 1002
                                  best 1001 us ... worst 1002 us

Placement vs priority: own core @ Normal wins the tail (reserved p99 2 us vs contended 1001..1002 us).
```

## Linux (baremetal), Debian 13.5, Proxmox, 7.0.2-7-pve

```bash
     Running `target/release/examples/reserved_core`
Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz - 4 P-core primaries
Feeder: 48 kHz / 256-frame buffer (5333us deadline). Load: 3 Highest spinners on the other P-cores, spawned once.

Load spinner priority: Highest nice -10; clean 3/3, clamped 0/3, fallback 0/3, failed 0/3
Lower jitter is better. Which lever buys it - own core, or top priority?

  reserved core  @ Normal         p50      0  p95      0  p99      0 us   [nice 0]
  contended core @ TimeCritical   p99 over 5 runs: 2336 2339 2336 2337 2335
                                  best 2335 us ... worst 2339 us

Placement vs priority: own core @ Normal wins the tail (reserved p99 0 us vs contended 2335..2339 us).
```

## Linux (LXC, limited to 2 cores, inside Proxmox), Debian 13.5, 7.0.2-7-pve

```bash
     Running `target/release/examples/reserved_core`
Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz - 2 P-core primaries
Feeder: 48 kHz / 256-frame buffer (5333us deadline). Load: 1 Highest spinners on the other P-cores, spawned once.

Load spinner priority: Highest -> Normal [NoBroker] nice 0
Lower jitter is better. Which lever buys it - own core, or top priority?

  reserved core  @ Normal         p50      0  p95      1  p99     10 us   [nice 0]
  contended core @ TimeCritical   p99 over 5 runs: 1333 2667 1666 756 1673
                                  best 756 us ... worst 2667 us
      a run lost TimeCritical entirely: [-> Normal [NoBroker] nice 0] - rtkit budget? see module NOTE

Placement vs priority: own core @ Normal wins the tail (reserved p99 10 us vs contended 756..2667 us).
```

## Linux (baremetal), Debian 12.14, MS-R1, 6.6.10-cix-build-generic

```bash
     Running `target/release/examples/reserved_core`
CIX P1 CP8180 - 8 P-core primaries
Feeder: 48 kHz / 256-frame buffer (5333us deadline). Load: 7 Highest spinners on the other P-cores, spawned once.

Load spinner priority: Highest [Brokered] nice -10; clean 7/7, clamped 0/7, fallback 0/7, failed 0/7
Lower jitter is better. Which lever buys it - own core, or top priority?

  reserved core  @ Normal         p50      0  p95      0  p99      0 us   [nice 0]
  contended core @ TimeCritical   p99 over 5 runs: 4000 4000 4000 4000 4000
                                  best 4000 us ... worst 4000 us

Placement vs priority: own core @ Normal wins the tail (reserved p99 0 us vs contended 4000..4000 us).
```

## Linux (baremetal), SteamOS, Steam Deck, 6.11.11-valve29-1-neptune-611-g2dcfaf4df7ac

```bash
     Running `target/release/examples/reserved_core`
AMD Custom APU 0405 - 4 P-core primaries
Feeder: 48 kHz / 256-frame buffer (5333us deadline). Load: 3 Highest spinners on the other P-cores, spawned once.

Load spinner priority: Highest [Brokered] nice -10; clean 3/3, clamped 0/3, fallback 0/3, failed 0/3
Lower jitter is better. Which lever buys it - own core, or top priority?

  reserved core  @ Normal         p50      0  p95      0  p99      0 us   [nice 0]
  contended core @ TimeCritical   p99 over 5 runs: 2679 2676 2682 2679 2684
                                  best 2676 us ... worst 2684 us

Placement vs priority: own core @ Normal wins the tail (reserved p99 0 us vs contended 2676..2684 us).
```
