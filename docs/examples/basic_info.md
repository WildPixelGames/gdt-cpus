# Output from examples/basic_info.rs on various platforms

## Windows 11

```bash
     Running `target\release\examples\basic_info.exe`
CPU Information:
---------------
Vendor: AMD
Model: AMD Ryzen 9 5950X 16-Core Processor
Sockets: 1
Physical cores: 16
Logical cores: 32
Performance cores: 16
Efficiency cores: 0
LP-Efficiency cores: 0
NUMA nodes: 1
Hybrid architecture: No

L3 domains: 2
  domain 0: 32 MiB, 8 cores, 16 threads
  domain 1: 32 MiB, 8 cores, 16 threads

Per-kind caches:
  Performance: L1d 32 KB / L1i 32 KB / L2 512 KB (L2 shared by 2 threads)

Logical processors:
  lp   0: core   0 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp   1: core   0 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp   2: core   1 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp   3: core   1 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp   4: core   2 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp   5: core   2 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp   6: core   3 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp   7: core   3 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp   8: core   4 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp   9: core   4 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  10: core   5 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  11: core   5 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  12: core   6 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  13: core   6 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  14: core   7 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  15: core   7 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  16: core   8 smt 0 socket 0 l3-domain 1 numa 0 perf    0 kind Performance
  lp  17: core   8 smt 1 socket 0 l3-domain 1 numa 0 perf    0 kind Performance
  lp  18: core   9 smt 0 socket 0 l3-domain 1 numa 0 perf    0 kind Performance
  lp  19: core   9 smt 1 socket 0 l3-domain 1 numa 0 perf    0 kind Performance
  lp  20: core  10 smt 0 socket 0 l3-domain 1 numa 0 perf    0 kind Performance
  lp  21: core  10 smt 1 socket 0 l3-domain 1 numa 0 perf    0 kind Performance
  lp  22: core  11 smt 0 socket 0 l3-domain 1 numa 0 perf    0 kind Performance
  lp  23: core  11 smt 1 socket 0 l3-domain 1 numa 0 perf    0 kind Performance
  lp  24: core  12 smt 0 socket 0 l3-domain 1 numa 0 perf    0 kind Performance
  lp  25: core  12 smt 1 socket 0 l3-domain 1 numa 0 perf    0 kind Performance
  lp  26: core  13 smt 0 socket 0 l3-domain 1 numa 0 perf    0 kind Performance
  lp  27: core  13 smt 1 socket 0 l3-domain 1 numa 0 perf    0 kind Performance
  lp  28: core  14 smt 0 socket 0 l3-domain 1 numa 0 perf    0 kind Performance
  lp  29: core  14 smt 1 socket 0 l3-domain 1 numa 0 perf    0 kind Performance
  lp  30: core  15 smt 0 socket 0 l3-domain 1 numa 0 perf    0 kind Performance
  lp  31: core  15 smt 1 socket 0 l3-domain 1 numa 0 perf    0 kind Performance

CPU Features:
  MMX, SSE, SSE2, SSE3, SSSE3, SSE4_1, SSE4_2, FMA3, AVX, AVX2, AES, SHA, CRC32, POPCNT, BMI1, BMI2, F16C
```

## Linux (WSL2), Windows 11

```bash
     Running `target/release/examples/basic_info`
CPU Information:
---------------
Vendor: AMD
Model: AMD Ryzen 9 5950X 16-Core Processor
Sockets: 1
Physical cores: 16
Logical cores: 32
Performance cores: 16
Efficiency cores: 0
LP-Efficiency cores: 0
NUMA nodes: 1
Hybrid architecture: No

L3 domains: 1
  domain 0: 32 MiB, 16 cores, 32 threads

Per-kind caches:
  Performance: L1d 32 KB / L1i 32 KB / L2 512 KB (L2 shared by 2 threads)

Logical processors:
  lp   0: core   0 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp   1: core   0 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp   2: core   1 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp   3: core   1 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp   4: core   2 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp   5: core   2 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp   6: core   3 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp   7: core   3 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp   8: core   4 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp   9: core   4 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  10: core   5 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  11: core   5 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  12: core   6 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  13: core   6 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  14: core   7 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  15: core   7 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  16: core   8 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  17: core   8 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  18: core   9 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  19: core   9 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  20: core  10 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  21: core  10 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  22: core  11 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  23: core  11 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  24: core  12 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  25: core  12 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  26: core  13 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  27: core  13 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  28: core  14 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  29: core  14 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  30: core  15 smt 0 socket 0 l3-domain 0 numa 0 perf    0 kind Performance
  lp  31: core  15 smt 1 socket 0 l3-domain 0 numa 0 perf    0 kind Performance

CPU Features:
  MMX, SSE, SSE2, SSE3, SSSE3, SSE4_1, SSE4_2, FMA3, AVX, AVX2, AES, SHA, CRC32, POPCNT, BMI1, BMI2, F16C
```

## Windows 11 (on Apple M3 Max via Parallels)

```bash
     Running `target\release\examples\basic_info.exe`
CPU Information:
---------------
Vendor: Apple
Model: Apple Silicon
Sockets: 8
Physical cores: 8
Logical cores: 8
Performance cores: 8
Efficiency cores: 0
LP-Efficiency cores: 0
NUMA nodes: 1
Hybrid architecture: No

L3 domains: 0

Per-kind caches:
  Performance: L1d 64 KB / L1i 128 KB / L2 4096 KB (L2 shared by 1 threads)

Logical processors:
  lp   0: core   0 smt 0 socket 0 l3-domain - numa 0 perf    0 kind Performance
  lp   1: core   1 smt 0 socket 1 l3-domain - numa 0 perf    0 kind Performance
  lp   2: core   2 smt 0 socket 2 l3-domain - numa 0 perf    0 kind Performance
  lp   3: core   3 smt 0 socket 3 l3-domain - numa 0 perf    0 kind Performance
  lp   4: core   4 smt 0 socket 4 l3-domain - numa 0 perf    0 kind Performance
  lp   5: core   5 smt 0 socket 5 l3-domain - numa 0 perf    0 kind Performance
  lp   6: core   6 smt 0 socket 6 l3-domain - numa 0 perf    0 kind Performance
  lp   7: core   7 smt 0 socket 7 l3-domain - numa 0 perf    0 kind Performance

CPU Features:
  NEON, AES, SHA, CRC32, FP16, DOTPROD, BF16, LSE, JSCVT, LRCPC, PMULL
```

## macOS 26.5.1

```bash
     Running `target/release/examples/basic_info`
CPU Information:
---------------
Vendor: Apple
Model: Apple M3 Max
Sockets: 1
Physical cores: 16
Logical cores: 16
Performance cores: 12
Efficiency cores: 4
LP-Efficiency cores: 0
NUMA nodes: 1
Hybrid architecture: Yes

L3 domains: 0

L2 domains: 3
  domain 0: 16384 KB, 6 cores, 6 threads, l3-domain -, lps [0-5]
  domain 1: 16384 KB, 6 cores, 6 threads, l3-domain -, lps [6-11]
  domain 2: 4096 KB, 4 cores, 4 threads, l3-domain -, lps [12-15]

Per-kind caches:
  Performance: L1d 128 KB / L1i 192 KB / L2 16384 KB (L2 shared by 6 threads)
  Efficiency: L1d 64 KB / L1i 128 KB / L2 4096 KB (L2 shared by 4 threads)

Logical processors:
  lp   0: core   0 smt 0 socket 0 l3-domain - l2-domain 0 numa 0 perf    2 kind Performance
  lp   1: core   1 smt 0 socket 0 l3-domain - l2-domain 0 numa 0 perf    2 kind Performance
  lp   2: core   2 smt 0 socket 0 l3-domain - l2-domain 0 numa 0 perf    2 kind Performance
  lp   3: core   3 smt 0 socket 0 l3-domain - l2-domain 0 numa 0 perf    2 kind Performance
  lp   4: core   4 smt 0 socket 0 l3-domain - l2-domain 0 numa 0 perf    2 kind Performance
  lp   5: core   5 smt 0 socket 0 l3-domain - l2-domain 0 numa 0 perf    2 kind Performance
  lp   6: core   6 smt 0 socket 0 l3-domain - l2-domain 1 numa 0 perf    2 kind Performance
  lp   7: core   7 smt 0 socket 0 l3-domain - l2-domain 1 numa 0 perf    2 kind Performance
  lp   8: core   8 smt 0 socket 0 l3-domain - l2-domain 1 numa 0 perf    2 kind Performance
  lp   9: core   9 smt 0 socket 0 l3-domain - l2-domain 1 numa 0 perf    2 kind Performance
  lp  10: core  10 smt 0 socket 0 l3-domain - l2-domain 1 numa 0 perf    2 kind Performance
  lp  11: core  11 smt 0 socket 0 l3-domain - l2-domain 1 numa 0 perf    2 kind Performance
  lp  12: core  12 smt 0 socket 0 l3-domain - l2-domain 2 numa 0 perf    1 kind Efficiency
  lp  13: core  13 smt 0 socket 0 l3-domain - l2-domain 2 numa 0 perf    1 kind Efficiency
  lp  14: core  14 smt 0 socket 0 l3-domain - l2-domain 2 numa 0 perf    1 kind Efficiency
  lp  15: core  15 smt 0 socket 0 l3-domain - l2-domain 2 numa 0 perf    1 kind Efficiency

CPU Features:
  NEON, AES, SHA, CRC32, FP16, DOTPROD, I8MM, BF16, LSE, JSCVT, LRCPC, PMULL, RDM, FHM, FCMA, LSE2, LRCPC2
```

## Linux (baremetal), CachyOS, Desktop, 7.0.11-1-cachyos

```bash
     Running `target/release/examples/basic_info`
CPU Information:
---------------
Vendor: AMD
Model: AMD Ryzen 9 5950X 16-Core Processor
Sockets: 1
Physical cores: 16
Logical cores: 32
Performance cores: 16
Efficiency cores: 0
LP-Efficiency cores: 0
NUMA nodes: 1
Hybrid architecture: No

L3 domains: 2
  domain 0: 32 MiB, 8 cores, 16 threads, lps [0-7, 16-23]
  domain 1: 32 MiB, 8 cores, 16 threads, lps [8-15, 24-31]

L2 domains: 16
  domain  0: 512 KB, 1 cores, 2 threads, l3-domain 0, lps [0, 16]
  domain  1: 512 KB, 1 cores, 2 threads, l3-domain 0, lps [1, 17]
  domain  2: 512 KB, 1 cores, 2 threads, l3-domain 0, lps [2, 18]
  domain  3: 512 KB, 1 cores, 2 threads, l3-domain 0, lps [3, 19]
  domain  4: 512 KB, 1 cores, 2 threads, l3-domain 0, lps [4, 20]
  domain  5: 512 KB, 1 cores, 2 threads, l3-domain 0, lps [5, 21]
  domain  6: 512 KB, 1 cores, 2 threads, l3-domain 0, lps [6, 22]
  domain  7: 512 KB, 1 cores, 2 threads, l3-domain 0, lps [7, 23]
  domain  8: 512 KB, 1 cores, 2 threads, l3-domain 1, lps [8, 24]
  domain  9: 512 KB, 1 cores, 2 threads, l3-domain 1, lps [9, 25]
  domain 10: 512 KB, 1 cores, 2 threads, l3-domain 1, lps [10, 26]
  domain 11: 512 KB, 1 cores, 2 threads, l3-domain 1, lps [11, 27]
  domain 12: 512 KB, 1 cores, 2 threads, l3-domain 1, lps [12, 28]
  domain 13: 512 KB, 1 cores, 2 threads, l3-domain 1, lps [13, 29]
  domain 14: 512 KB, 1 cores, 2 threads, l3-domain 1, lps [14, 30]
  domain 15: 512 KB, 1 cores, 2 threads, l3-domain 1, lps [15, 31]

Per-kind caches:
  Performance: L1d 32 KB / L1i 32 KB / L2 512 KB (L2 shared by 2 threads)

Logical processors:
  lp   0: core   0 smt 0 socket 0 l3-domain 0 l2-domain  0 numa 0 perf 1024 kind Performance
  lp   1: core   1 smt 0 socket 0 l3-domain 0 l2-domain  1 numa 0 perf 1024 kind Performance
  lp   2: core   2 smt 0 socket 0 l3-domain 0 l2-domain  2 numa 0 perf 1024 kind Performance
  lp   3: core   3 smt 0 socket 0 l3-domain 0 l2-domain  3 numa 0 perf 1024 kind Performance
  lp   4: core   4 smt 0 socket 0 l3-domain 0 l2-domain  4 numa 0 perf 1024 kind Performance
  lp   5: core   5 smt 0 socket 0 l3-domain 0 l2-domain  5 numa 0 perf 1024 kind Performance
  lp   6: core   6 smt 0 socket 0 l3-domain 0 l2-domain  6 numa 0 perf 1024 kind Performance
  lp   7: core   7 smt 0 socket 0 l3-domain 0 l2-domain  7 numa 0 perf 1024 kind Performance
  lp   8: core   8 smt 0 socket 0 l3-domain 1 l2-domain  8 numa 0 perf 1024 kind Performance
  lp   9: core   9 smt 0 socket 0 l3-domain 1 l2-domain  9 numa 0 perf 1024 kind Performance
  lp  10: core  10 smt 0 socket 0 l3-domain 1 l2-domain 10 numa 0 perf 1024 kind Performance
  lp  11: core  11 smt 0 socket 0 l3-domain 1 l2-domain 11 numa 0 perf 1024 kind Performance
  lp  12: core  12 smt 0 socket 0 l3-domain 1 l2-domain 12 numa 0 perf 1024 kind Performance
  lp  13: core  13 smt 0 socket 0 l3-domain 1 l2-domain 13 numa 0 perf 1024 kind Performance
  lp  14: core  14 smt 0 socket 0 l3-domain 1 l2-domain 14 numa 0 perf 1024 kind Performance
  lp  15: core  15 smt 0 socket 0 l3-domain 1 l2-domain 15 numa 0 perf 1024 kind Performance
  lp  16: core   0 smt 1 socket 0 l3-domain 0 l2-domain  0 numa 0 perf 1024 kind Performance
  lp  17: core   1 smt 1 socket 0 l3-domain 0 l2-domain  1 numa 0 perf 1024 kind Performance
  lp  18: core   2 smt 1 socket 0 l3-domain 0 l2-domain  2 numa 0 perf 1024 kind Performance
  lp  19: core   3 smt 1 socket 0 l3-domain 0 l2-domain  3 numa 0 perf 1024 kind Performance
  lp  20: core   4 smt 1 socket 0 l3-domain 0 l2-domain  4 numa 0 perf 1024 kind Performance
  lp  21: core   5 smt 1 socket 0 l3-domain 0 l2-domain  5 numa 0 perf 1024 kind Performance
  lp  22: core   6 smt 1 socket 0 l3-domain 0 l2-domain  6 numa 0 perf 1024 kind Performance
  lp  23: core   7 smt 1 socket 0 l3-domain 0 l2-domain  7 numa 0 perf 1024 kind Performance
  lp  24: core   8 smt 1 socket 0 l3-domain 1 l2-domain  8 numa 0 perf 1024 kind Performance
  lp  25: core   9 smt 1 socket 0 l3-domain 1 l2-domain  9 numa 0 perf 1024 kind Performance
  lp  26: core  10 smt 1 socket 0 l3-domain 1 l2-domain 10 numa 0 perf 1024 kind Performance
  lp  27: core  11 smt 1 socket 0 l3-domain 1 l2-domain 11 numa 0 perf 1024 kind Performance
  lp  28: core  12 smt 1 socket 0 l3-domain 1 l2-domain 12 numa 0 perf 1024 kind Performance
  lp  29: core  13 smt 1 socket 0 l3-domain 1 l2-domain 13 numa 0 perf 1024 kind Performance
  lp  30: core  14 smt 1 socket 0 l3-domain 1 l2-domain 14 numa 0 perf 1024 kind Performance
  lp  31: core  15 smt 1 socket 0 l3-domain 1 l2-domain 15 numa 0 perf 1024 kind Performance

CPU Features:
  MMX, SSE, SSE2, SSE3, SSSE3, SSE4_1, SSE4_2, FMA3, AVX, AVX2, AES, SHA, CRC32, POPCNT, BMI1, BMI2, F16C
```

## Linux (baremetal), Debian 13.5, Proxmox, 7.0.2-7-pve

```bash
     Running `target/release/examples/basic_info`
CPU Information:
---------------
Vendor: Intel
Model: Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz
Sockets: 1
Physical cores: 4
Logical cores: 8
Performance cores: 4
Efficiency cores: 0
LP-Efficiency cores: 0
NUMA nodes: 1
Hybrid architecture: No

L3 domains: 1
  domain 0: 8 MiB, 4 cores, 8 threads, lps [0-7]

L2 domains: 4
  domain 0: 256 KB, 1 cores, 2 threads, l3-domain 0, lps [0, 4]
  domain 1: 256 KB, 1 cores, 2 threads, l3-domain 0, lps [1, 5]
  domain 2: 256 KB, 1 cores, 2 threads, l3-domain 0, lps [2, 6]
  domain 3: 256 KB, 1 cores, 2 threads, l3-domain 0, lps [3, 7]

Per-kind caches:
  Performance: L1d 32 KB / L1i 32 KB / L2 256 KB (L2 shared by 2 threads)

Logical processors:
  lp   0: core   0 smt 0 socket 0 l3-domain 0 l2-domain 0 numa 0 perf 1024 kind Performance
  lp   1: core   1 smt 0 socket 0 l3-domain 0 l2-domain 1 numa 0 perf 1024 kind Performance
  lp   2: core   2 smt 0 socket 0 l3-domain 0 l2-domain 2 numa 0 perf 1024 kind Performance
  lp   3: core   3 smt 0 socket 0 l3-domain 0 l2-domain 3 numa 0 perf 1024 kind Performance
  lp   4: core   0 smt 1 socket 0 l3-domain 0 l2-domain 0 numa 0 perf 1024 kind Performance
  lp   5: core   1 smt 1 socket 0 l3-domain 0 l2-domain 1 numa 0 perf 1024 kind Performance
  lp   6: core   2 smt 1 socket 0 l3-domain 0 l2-domain 2 numa 0 perf 1024 kind Performance
  lp   7: core   3 smt 1 socket 0 l3-domain 0 l2-domain 3 numa 0 perf 1024 kind Performance

CPU Features:
  MMX, SSE, SSE2, SSE3, SSSE3, SSE4_1, SSE4_2, FMA3, AVX, AVX2, AES, CRC32, POPCNT, BMI1, BMI2, F16C
```

## Linux (LXC, limited to 2 cores, inside Proxmox), Debian 13.5, 7.0.2-7-pve

```bash
     Running `target/release/examples/basic_info`
CPU Information:
---------------
Vendor: Intel
Model: Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz
Sockets: 1
Physical cores: 2
Logical cores: 2
Performance cores: 2
Efficiency cores: 0
LP-Efficiency cores: 0
NUMA nodes: 1
Hybrid architecture: No

L3 domains: 1
  domain 0: 8 MiB, 2 cores, 8 threads, lps [0-7]

L2 domains: 2
  domain 0: 256 KB, 1 cores, 2 threads, l3-domain 0, lps [1, 5]
  domain 1: 256 KB, 1 cores, 2 threads, l3-domain 0, lps [0, 4]

Per-kind caches:
  Performance: L1d 32 KB / L1i 32 KB / L2 256 KB (L2 shared by 2 threads)

Logical processors:
  lp   1: core   0 smt 0 socket 0 l3-domain 0 l2-domain 0 numa 0 perf 1024 kind Performance
  lp   4: core   1 smt 0 socket 0 l3-domain 0 l2-domain 1 numa 0 perf 1024 kind Performance

CPU Features:
  MMX, SSE, SSE2, SSE3, SSSE3, SSE4_1, SSE4_2, FMA3, AVX, AVX2, AES, CRC32, POPCNT, BMI1, BMI2, F16C
```

## Linux (baremetal), Debian 12.14, MS-R1, 6.6.10-cix-build-generic

```bash
     Running `target/release/examples/basic_info`
CPU Information:
---------------
Vendor: ARM
Model: CIX P1 CP8180
Sockets: 1
Physical cores: 12
Logical cores: 12
Performance cores: 8
Efficiency cores: 0
LP-Efficiency cores: 4
NUMA nodes: 1
Hybrid architecture: Yes

L3 domains: 1
  domain 0: 12 MiB, 12 cores, 12 threads, lps [0-11]

L2 domains: 12
  domain  0: 512 KB, 1 cores, 1 threads, l3-domain 0, lps [0]
  domain  1: 512 KB, 1 cores, 1 threads, l3-domain 0, lps [1]
  domain  2: 0 KB, 1 cores, 1 threads, l3-domain 0, lps [2]
  domain  3: 0 KB, 1 cores, 1 threads, l3-domain 0, lps [3]
  domain  4: 0 KB, 1 cores, 1 threads, l3-domain 0, lps [4]
  domain  5: 0 KB, 1 cores, 1 threads, l3-domain 0, lps [5]
  domain  6: 512 KB, 1 cores, 1 threads, l3-domain 0, lps [6]
  domain  7: 512 KB, 1 cores, 1 threads, l3-domain 0, lps [7]
  domain  8: 512 KB, 1 cores, 1 threads, l3-domain 0, lps [8]
  domain  9: 512 KB, 1 cores, 1 threads, l3-domain 0, lps [9]
  domain 10: 512 KB, 1 cores, 1 threads, l3-domain 0, lps [10]
  domain 11: 512 KB, 1 cores, 1 threads, l3-domain 0, lps [11]

Per-kind caches:
  Performance: L1d 64 KB / L1i 64 KB / L2 512 KB (L2 shared by 1 threads)
  LpEfficiency: L1d 32 KB / L1i 32 KB / L2 0 KB (L2 shared by 1 threads)

Logical processors:
  lp   0: core   0 smt 0 socket 0 l3-domain 0 l2-domain  0 numa 0 perf 1024 kind Performance
  lp   1: core   1 smt 0 socket 0 l3-domain 0 l2-domain  1 numa 0 perf 1024 kind Performance
  lp   2: core   2 smt 0 socket 0 l3-domain 0 l2-domain  2 numa 0 perf  279 kind LpEfficiency
  lp   3: core   3 smt 0 socket 0 l3-domain 0 l2-domain  3 numa 0 perf  279 kind LpEfficiency
  lp   4: core   4 smt 0 socket 0 l3-domain 0 l2-domain  4 numa 0 perf  279 kind LpEfficiency
  lp   5: core   5 smt 0 socket 0 l3-domain 0 l2-domain  5 numa 0 perf  279 kind LpEfficiency
  lp   6: core   6 smt 0 socket 0 l3-domain 0 l2-domain  6 numa 0 perf  905 kind Performance
  lp   7: core   7 smt 0 socket 0 l3-domain 0 l2-domain  7 numa 0 perf  905 kind Performance
  lp   8: core   8 smt 0 socket 0 l3-domain 0 l2-domain  8 numa 0 perf  866 kind Performance
  lp   9: core   9 smt 0 socket 0 l3-domain 0 l2-domain  9 numa 0 perf  866 kind Performance
  lp  10: core  10 smt 0 socket 0 l3-domain 0 l2-domain 10 numa 0 perf  984 kind Performance
  lp  11: core  11 smt 0 socket 0 l3-domain 0 l2-domain 11 numa 0 perf  984 kind Performance

CPU Features:
  NEON, SVE, AES, SHA, CRC32, FP16, DOTPROD, I8MM, BF16, SVE2, LSE, JSCVT, LRCPC, PMULL, RDM, FHM, FCMA, LRCPC2, SM3, SM4, SVEAES, SVEPMULL, SVEBITPERM, SVESHA3, SVESM4, SVEI8MM, SVEBF16
```

## Linux (baremetal), SteamOS, Steam Deck, 6.11.11-valve29-1-neptune-611-g2dcfaf4df7ac

```bash
     Running `target/release/examples/basic_info`
CPU Information:
---------------
Vendor: AMD
Model: AMD Custom APU 0405
Sockets: 1
Physical cores: 4
Logical cores: 8
Performance cores: 4
Efficiency cores: 0
LP-Efficiency cores: 0
NUMA nodes: 1
Hybrid architecture: No

L3 domains: 1
  domain 0: 4 MiB, 4 cores, 8 threads, lps [0-7]

L2 domains: 4
  domain 0: 512 KB, 1 cores, 2 threads, l3-domain 0, lps [0-1]
  domain 1: 512 KB, 1 cores, 2 threads, l3-domain 0, lps [2-3]
  domain 2: 512 KB, 1 cores, 2 threads, l3-domain 0, lps [4-5]
  domain 3: 512 KB, 1 cores, 2 threads, l3-domain 0, lps [6-7]

Per-kind caches:
  Performance: L1d 32 KB / L1i 32 KB / L2 512 KB (L2 shared by 2 threads)

Logical processors:
  lp   0: core   0 smt 0 socket 0 l3-domain 0 l2-domain 0 numa 0 perf 1024 kind Performance
  lp   1: core   0 smt 1 socket 0 l3-domain 0 l2-domain 0 numa 0 perf 1024 kind Performance
  lp   2: core   1 smt 0 socket 0 l3-domain 0 l2-domain 1 numa 0 perf 1024 kind Performance
  lp   3: core   1 smt 1 socket 0 l3-domain 0 l2-domain 1 numa 0 perf 1024 kind Performance
  lp   4: core   2 smt 0 socket 0 l3-domain 0 l2-domain 2 numa 0 perf 1024 kind Performance
  lp   5: core   2 smt 1 socket 0 l3-domain 0 l2-domain 2 numa 0 perf 1024 kind Performance
  lp   6: core   3 smt 0 socket 0 l3-domain 0 l2-domain 3 numa 0 perf 1024 kind Performance
  lp   7: core   3 smt 1 socket 0 l3-domain 0 l2-domain 3 numa 0 perf 1024 kind Performance

CPU Features:
  MMX, SSE, SSE2, SSE3, SSSE3, SSE4_1, SSE4_2, FMA3, AVX, AVX2, AES, SHA, CRC32, POPCNT, BMI1, BMI2, F16C
```
