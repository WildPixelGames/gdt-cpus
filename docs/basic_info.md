# Output from examples/basic_info.rs on various platforms

## Running on Windows

```bash
C:\Develop\Projects\Rust\gdt-cpus>cargo run --example basic_info
   Compiling gdt-cpus v0.1.0 (C:\Develop\Projects\Rust\gdt-cpus)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.64s
     Running `target\debug\examples\basic_info.exe`
CPU Information:
---------------
Vendor: AMD
Model: AMD Ryzen 9 5950X 16-Core Processor
Physical cores: 16
Logical cores: 32
Performance cores: 16
Efficiency cores: 0
Hybrid architecture: No

Processor #0 (Socket ID: 0)
  L3 Cache: 32768 KB
  Cores:
    Core #0: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #1: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #2: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #3: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #4: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #5: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #6: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #7: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #8: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #9: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #10: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #11: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #12: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #13: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #14: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #15: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB

CPU Features:
  MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, FMA3, AVX, AVX2, AES, SHA
```

## Running on Linux (WSL2)

```bash
❯ cargo run --example basic_info
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.26s
     Running `target/debug/examples/basic_info`
CPU Information:
---------------
Vendor: AMD
Model: AMD Ryzen 9 5950X 16-Core Processor
Physical cores: 16
Logical cores: 32
Performance cores: 16
Efficiency cores: 0
Hybrid architecture: No

Processor #0 (Socket ID: 0)
  L3 Cache: 32768 KB
  Cores:
    Core #0: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #1: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #2: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #3: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #4: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #5: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #6: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #7: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #8: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #9: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #10: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #11: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #12: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #13: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #14: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB
    Core #15: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 512 KB

CPU Features:
  MMX, SSE, SSE2, SSE3, SSSE3, SSE4_1, SSE4_2, FMA3, AVX, AVX2, AES, SHA
```

## Running on Windows (on Apple M3 Max via Parallels)

```bash
C:\Develop\rust\gdt-cpus>cargo run --example basic_info
   Compiling gdt-cpus v0.1.0 (C:\Develop\rust\gdt-cpus)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.39s
     Running `target\debug\examples\basic_info.exe`
CPU Information:
---------------
Vendor: Apple
Model: Apple Silicon
Physical cores: 8
Logical cores: 8
Performance cores: 8
Efficiency cores: 0
Hybrid architecture: No

Processor #0 (Socket ID: 0)
  Cores:
    Core #0: Performance core with 1 threads
      L1i Cache: 192 KB
      L1d Cache: 128 KB
      L2 Cache: 16384 KB

Processor #1 (Socket ID: 1)
  Cores:
    Core #1: Performance core with 1 threads
      L1i Cache: 192 KB
      L1d Cache: 128 KB
      L2 Cache: 16384 KB

Processor #2 (Socket ID: 2)
  Cores:
    Core #2: Performance core with 1 threads
      L1i Cache: 192 KB
      L1d Cache: 128 KB
      L2 Cache: 16384 KB

Processor #3 (Socket ID: 3)
  Cores:
    Core #3: Performance core with 1 threads
      L1i Cache: 192 KB
      L1d Cache: 128 KB
      L2 Cache: 16384 KB

Processor #4 (Socket ID: 4)
  Cores:
    Core #4: Performance core with 1 threads
      L1i Cache: 192 KB
      L1d Cache: 128 KB
      L2 Cache: 16384 KB

Processor #5 (Socket ID: 5)
  Cores:
    Core #5: Performance core with 1 threads
      L1i Cache: 192 KB
      L1d Cache: 128 KB
      L2 Cache: 16384 KB

Processor #6 (Socket ID: 6)
  Cores:
    Core #6: Performance core with 1 threads
      L1i Cache: 192 KB
      L1d Cache: 128 KB
      L2 Cache: 16384 KB

Processor #7 (Socket ID: 7)
  Cores:
    Core #7: Performance core with 1 threads
      L1i Cache: 192 KB
      L1d Cache: 128 KB
      L2 Cache: 16384 KB

CPU Features:
  NEON
```

## Running on macOS

```bash
 cargo run --example basic_info
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/examples/basic_info`
CPU Information:
---------------
Vendor: Apple
Model: Apple M3 Max
Physical cores: 16
Logical cores: 16
Performance cores: 12
Efficiency cores: 4
Hybrid architecture: Yes

Processor #0 (Socket ID: 0)
  Cores:
    Core #0: Performance core with 1 threads
      L1i Cache: 192 KB
      L1d Cache: 128 KB
      L2 Cache: 16384 KB
    Core #1: Performance core with 1 threads
      L1i Cache: 192 KB
      L1d Cache: 128 KB
      L2 Cache: 16384 KB
    Core #2: Performance core with 1 threads
      L1i Cache: 192 KB
      L1d Cache: 128 KB
      L2 Cache: 16384 KB
    Core #3: Performance core with 1 threads
      L1i Cache: 192 KB
      L1d Cache: 128 KB
      L2 Cache: 16384 KB
    Core #4: Performance core with 1 threads
      L1i Cache: 192 KB
      L1d Cache: 128 KB
      L2 Cache: 16384 KB
    Core #5: Performance core with 1 threads
      L1i Cache: 192 KB
      L1d Cache: 128 KB
      L2 Cache: 16384 KB
    Core #6: Performance core with 1 threads
      L1i Cache: 192 KB
      L1d Cache: 128 KB
      L2 Cache: 16384 KB
    Core #7: Performance core with 1 threads
      L1i Cache: 192 KB
      L1d Cache: 128 KB
      L2 Cache: 16384 KB
    Core #8: Performance core with 1 threads
      L1i Cache: 192 KB
      L1d Cache: 128 KB
      L2 Cache: 16384 KB
    Core #9: Performance core with 1 threads
      L1i Cache: 192 KB
      L1d Cache: 128 KB
      L2 Cache: 16384 KB
    Core #10: Performance core with 1 threads
      L1i Cache: 192 KB
      L1d Cache: 128 KB
      L2 Cache: 16384 KB
    Core #11: Performance core with 1 threads
      L1i Cache: 192 KB
      L1d Cache: 128 KB
      L2 Cache: 16384 KB
    Core #12: Efficiency core with 1 threads
      L1i Cache: 128 KB
      L1d Cache: 64 KB
      L2 Cache: 4096 KB
    Core #13: Efficiency core with 1 threads
      L1i Cache: 128 KB
      L1d Cache: 64 KB
      L2 Cache: 4096 KB
    Core #14: Efficiency core with 1 threads
      L1i Cache: 128 KB
      L1d Cache: 64 KB
      L2 Cache: 4096 KB
    Core #15: Efficiency core with 1 threads
      L1i Cache: 128 KB
      L1d Cache: 64 KB
      L2 Cache: 4096 KB

CPU Features:
  NEON, AES, SHA, CRC32
```

## Running on Linux (baremetal)

```bash
$ cargo run --example basic_info
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running `target/debug/examples/basic_info`
CPU Information:
---------------
Vendor: Intel
Model: Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz
Physical cores: 4
Logical cores: 8
Performance cores: 4
Efficiency cores: 0
Hybrid architecture: No

Processor #0 (Socket ID: 0)
  L3 Cache: 8192 KB
  Cores:
    Core #0: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 256 KB
    Core #1: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 256 KB
    Core #2: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 256 KB
    Core #3: Performance core with 2 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 256 KB

CPU Features:
  MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, FMA3, AVX, AVX2, AES
```

## Running on Linux (LXC, limited to 2 cores, same machine as above)

```bash
$ cargo run --example basic_info
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/examples/basic_info`
CPU Information:
---------------
Vendor: Intel
Model: Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz
Physical cores: 2
Logical cores: 2
Performance cores: 2
Efficiency cores: 0
Hybrid architecture: No

Processor #0 (Socket ID: 0)
  L3 Cache: 8192 KB
  Cores:
    Core #0: Performance core with 1 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 256 KB
    Core #1: Performance core with 1 threads
      L1i Cache: 32 KB
      L1d Cache: 32 KB
      L2 Cache: 256 KB

CPU Features:
  MMX, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, FMA3, AVX, AVX2, AES
```
