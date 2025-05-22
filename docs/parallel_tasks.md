# Parallel Tasks Benchmark Results

## Overview

This benchmark simulates a typical game engine workload where many small independent tasks (physics calculations, AI decisions, etc.) need to be processed in parallel by a thread pool. We measure both throughput (tasks/second) and per-task latency across different thread scheduling configurations.

## Key Findings

- **Throughput Consistency**: All configurations on all platforms achieved the same throughput (2000 tasks/second), indicating that modern OS schedulers are effective at distributing work.

- **Median Latency Patterns**: Median task latencies (p50) were remarkably consistent across all tests and platforms (~500-520μs), showing that basic scheduling works well for average cases.

- **Tail Latency Improvements**:
  - **macOS**: Significant tail latency improvement (p99 reduced from 1732μs to 997μs) when using separate thread priorities for P-cores and E-cores
  - **Windows**: Already excellent baseline tail latencies with minimal changes between configurations
  - **Linux**: Modest improvements in tail latencies with mixed thread priorities

- **Platform Differences**:
  - **macOS**: Showed the largest variance in task latencies and benefited most from priority tuning
  - **Windows**: Demonstrated the most consistent performance across configurations
  - **Linux**: Showed intermediate results with some improvement from priority adjustments

## Recommendations for Game Developers

1. **On hybrid architectures** (like Apple Silicon), assign higher priorities to threads meant for performance-critical tasks

2. **On Windows**, the default scheduler already performs well, but can be further tuned with thread priorities

3. **Consider tail latencies** (p99) rather than median latencies when evaluating thread scheduling strategies for game workloads

4. **Remember platform differences** - optimization strategies that provide significant benefits on one platform (particularly macOS) might show minimal gains on another (like Windows)

This benchmark demonstrates that proper thread configuration can substantially reduce task latency spikes while maintaining overall throughput, which is crucial for frame-to-frame consistency in games.

## Benchmark Results

### macOS

```bash
     Running `target/release/examples/parallel_tasks`
Multi-Threaded Physics/AI Sweep Benchmark
=========================================
CPU Info:
  Model: Apple M3 Max
  Physical cores: 16
  Performance cores: 12
  Efficiency cores: 4
  Logical cores: 16
Found 12 P-cores and 4 E-cores

--- Running benchmark with 16 workers ---
Worker 2 starting: Normal priority, pin to core: None
Worker 0 starting: Normal priority, pin to core: None
Worker 1 starting: Normal priority, pin to core: None
Worker 3 starting: Normal priority, pin to core: None
Worker 8 starting: Normal priority, pin to core: None
Worker 4 starting: Normal priority, pin to core: None
Worker 5 starting: Normal priority, pin to core: None
Worker 10 starting: Normal priority, pin to core: None
Worker 6 starting: Normal priority, pin to core: None
Worker 7 starting: Normal priority, pin to core: None
Worker 9 starting: Normal priority, pin to core: None
Worker 13 starting: Normal priority, pin to core: None
Worker 14 starting: Normal priority, pin to core: None
Worker 11 starting: Normal priority, pin to core: None
Worker 15 starting: Normal priority, pin to core: None
Worker 12 starting: Normal priority, pin to core: None

--- Running benchmark with 16 workers ---
Worker 0 starting: AboveNormal priority, pin to core: None
Worker 1 starting: Normal priority, pin to core: None
Worker 2 starting: AboveNormal priority, pin to core: None
Worker 3 starting: Normal priority, pin to core: None
Worker 5 starting: Normal priority, pin to core: None
Worker 4 starting: AboveNormal priority, pin to core: None
Worker 7 starting: Normal priority, pin to core: None
Worker 6 starting: AboveNormal priority, pin to core: None
Worker 10 starting: AboveNormal priority, pin to core: None
Worker 8 starting: AboveNormal priority, pin to core: None
Worker 11 starting: Normal priority, pin to core: None
Worker 12 starting: AboveNormal priority, pin to core: None
Worker 13 starting: Normal priority, pin to core: None
Worker 14 starting: AboveNormal priority, pin to core: None
Worker 9 starting: Normal priority, pin to core: None
Worker 15 starting: Normal priority, pin to core: None

--- Running benchmark with 16 workers ---
Worker 0 starting: AboveNormal priority, pin to core: Some(0)
Failed to pin thread to core Worker 1 starting: AboveNormal priority, pin to core: Some(1)
0: Unsupported operation: Thread affinity (pinning) is not supported on Apple Silicon, use `set_thread_priority`
Worker 2 starting: AboveNormal priority, pin to core: Some(2)
Failed to pin thread to core 2: Unsupported operation: Thread affinity (pinning) is not supported on Apple Silicon, use `set_thread_priority`
Worker 4 starting: AboveNormal priority, pin to core: Some(4)
Failed to pin thread to core 4: Unsupported operation: Thread affinity (pinning) is not supported on Apple Silicon, use `set_thread_priority`
Worker 3 starting: AboveNormal priority, pin to core: Some(3)
Failed to pin thread to core 3: Unsupported operation: Worker 5 starting: AboveNormal priority, pin to core: Some(5)
Thread affinity (pinning) is not supported on Apple Silicon, use `set_thread_priority`
Worker 6 starting: AboveNormal priority, pin to core: Some(6)
Failed to pin thread to core 6: Unsupported operation: Thread affinity (pinning) is not supported on Apple Silicon, use `set_thread_priority`
Worker 8 starting: AboveNormal priority, pin to core: Some(8)
Failed to pin thread to core 5: Worker 9 starting: AboveNormal priority, pin to core: Some(9)
Unsupported operation: Thread affinity (pinning) is not supported on Apple Silicon, use `set_thread_priority`
Worker 7 starting: AboveNormal priority, pin to core: Some(7)
Failed to pin thread to core 7: Unsupported operation: Thread affinity (pinning) is not supported on Apple Silicon, use `set_thread_priority`
Worker 11 starting: AboveNormal priority, pin to core: Some(11)
Failed to pin thread to core 11: Unsupported operation: Thread affinity (pinning) is not supported on Apple Silicon, use `set_thread_priority`
Worker 12 starting: BelowNormal priority, pin to core: Some(12)
Failed to pin thread to core 12: Unsupported operation: Thread affinity (pinning) is not supported on Apple Silicon, use `set_thread_priority`
Worker 10 starting: AboveNormal priority, pin to core: Some(10)
Failed to pin thread to core 9: Unsupported operation: Thread affinity (pinning) is not supported on Apple Silicon, use `set_thread_priority`
Worker 13 starting: BelowNormal priority, pin to core: Some(13)
Failed to pin thread to core 13: Unsupported operation: Thread affinity (pinning) is not supported on Apple Silicon, use `set_thread_priority`Worker 14 starting: BelowNormal priority, pin to core: Some(14)

Failed to pin thread to core 14: Unsupported operation: Thread affinity (pinning) is not supported on Apple Silicon, use `set_thread_priority`
Failed to pin thread to core 8: Unsupported operation: Worker 15 starting: BelowNormal priority, pin to core: Some(15)
Thread affinity (pinning) is not supported on Apple Silicon, use `set_thread_priority`
Failed to pin thread to core 15: Unsupported operation: Thread affinity (pinning) is not supported on Apple Silicon, use `set_thread_priority`
Failed to pin thread to core 1: Unsupported operation: Thread affinity (pinning) is not supported on Apple Silicon, use `set_thread_priority`
Failed to pin thread to core 10: Unsupported operation: Thread affinity (pinning) is not supported on Apple Silicon, use `set_thread_priority`

--- Results Summary ---

Test 1: All workers with Normal priority, no pinning
Total tasks processed: 10000
Tasks per second: 2000.00
Task latency (microseconds):
  Min: 10
  p50: 509
  p99: 1732
  Max: 9710

Per-worker statistics:
Worker 0: 661 tasks, latency min/p50/p99/max: 16/487/1413/7242 µs
Worker 1: 628 tasks, latency min/p50/p99/max: 10/500/1872/7471 µs
Worker 2: 647 tasks, latency min/p50/p99/max: 10/497/1420/4978 µs
Worker 3: 620 tasks, latency min/p50/p99/max: 10/505/1620/8227 µs
Worker 4: 604 tasks, latency min/p50/p99/max: 11/516/2244/6786 µs
Worker 5: 623 tasks, latency min/p50/p99/max: 12/500/1473/9710 µs
Worker 6: 643 tasks, latency min/p50/p99/max: 13/481/1309/6138 µs
Worker 7: 629 tasks, latency min/p50/p99/max: 11/518/1441/7198 µs
Worker 8: 649 tasks, latency min/p50/p99/max: 12/526/1332/4320 µs
Worker 9: 616 tasks, latency min/p50/p99/max: 10/484/2183/8926 µs
Worker 10: 634 tasks, latency min/p50/p99/max: 10/505/1159/6673 µs
Worker 11: 629 tasks, latency min/p50/p99/max: 10/517/1389/5366 µs
Worker 12: 613 tasks, latency min/p50/p99/max: 10/503/1358/4339 µs
Worker 13: 596 tasks, latency min/p50/p99/max: 11/544/2740/6620 µs
Worker 14: 629 tasks, latency min/p50/p99/max: 10/521/1571/8731 µs
Worker 15: 579 tasks, latency min/p50/p99/max: 10/521/2385/8922 µs

Test 2: Mixed worker priorities, no pinning
Total tasks processed: 10000
Tasks per second: 2000.00
Task latency (microseconds):
  Min: 10
  p50: 511
  p99: 1968
  Max: 33827

Per-worker statistics:
Worker 0: 634 tasks, latency min/p50/p99/max: 10/536/1179/9468 µs
Worker 1: 670 tasks, latency min/p50/p99/max: 13/487/1277/5281 µs
Worker 2: 633 tasks, latency min/p50/p99/max: 10/512/1305/10444 µs
Worker 3: 660 tasks, latency min/p50/p99/max: 10/509/1272/4718 µs
Worker 4: 645 tasks, latency min/p50/p99/max: 10/517/1452/4833 µs
Worker 5: 658 tasks, latency min/p50/p99/max: 12/503/1340/3628 µs
Worker 6: 645 tasks, latency min/p50/p99/max: 11/504/1800/11971 µs
Worker 7: 627 tasks, latency min/p50/p99/max: 12/537/2412/7590 µs
Worker 8: 629 tasks, latency min/p50/p99/max: 10/496/1630/6970 µs
Worker 9: 611 tasks, latency min/p50/p99/max: 10/515/2327/7300 µs
Worker 10: 454 tasks, latency min/p50/p99/max: 10/567/4898/33827 µs
Worker 11: 632 tasks, latency min/p50/p99/max: 10/506/2122/12346 µs
Worker 12: 632 tasks, latency min/p50/p99/max: 10/511/1420/9535 µs
Worker 13: 639 tasks, latency min/p50/p99/max: 13/500/2276/10313 µs
Worker 14: 635 tasks, latency min/p50/p99/max: 10/489/1568/13306 µs
Worker 15: 596 tasks, latency min/p50/p99/max: 11/531/1929/8517 µs

Test 3: P-cores and E-cores, pinned with different priorities
Total tasks processed: 10000
Tasks per second: 2000.00
Task latency (microseconds):
  Min: 10
  p50: 512
  p99: 997
  Max: 4756

Per-worker statistics:
Worker 0: 616 tasks, latency min/p50/p99/max: 17/530/995/3673 µs
Worker 1: 629 tasks, latency min/p50/p99/max: 11/522/992/2353 µs
Worker 2: 608 tasks, latency min/p50/p99/max: 10/523/994/2423 µs
Worker 3: 628 tasks, latency min/p50/p99/max: 10/515/996/2006 µs
Worker 4: 635 tasks, latency min/p50/p99/max: 10/509/1000/1547 µs
Worker 5: 599 tasks, latency min/p50/p99/max: 10/527/998/1511 µs
Worker 6: 631 tasks, latency min/p50/p99/max: 10/510/1000/2369 µs
Worker 7: 626 tasks, latency min/p50/p99/max: 10/521/998/1593 µs
Worker 8: 630 tasks, latency min/p50/p99/max: 11/514/999/1978 µs
Worker 9: 605 tasks, latency min/p50/p99/max: 10/526/995/4075 µs
Worker 10: 651 tasks, latency min/p50/p99/max: 10/468/999/2255 µs
Worker 11: 635 tasks, latency min/p50/p99/max: 10/504/995/1450 µs
Worker 12: 619 tasks, latency min/p50/p99/max: 11/513/992/1648 µs
Worker 13: 617 tasks, latency min/p50/p99/max: 11/514/1120/2451 µs
Worker 14: 638 tasks, latency min/p50/p99/max: 13/504/994/1918 µs
Worker 15: 633 tasks, latency min/p50/p99/max: 11/487/996/4756 µs
```

### Windows

```bash
     Running `target\release\examples\parallel_tasks.exe`
Multi-Threaded Physics/AI Sweep Benchmark
=========================================
CPU Info:
  Model: AMD Ryzen 9 5950X 16-Core Processor
  Physical cores: 16
  Performance cores: 16
  Efficiency cores: 0
  Logical cores: 32
Found 16 P-cores and 0 E-cores

--- Running benchmark with 32 workers ---
Worker 0 starting: Normal priority, pin to core: None
Worker 2 starting: Normal priority, pin to core: None
Worker 1 starting: Normal priority, pin to core: None
Worker 10 starting: Normal priority, pin to core: None
Worker 13 starting: Normal priority, pin to core: None
Worker 12 starting: Normal priority, pin to core: None
Worker 4 starting: Normal priority, pin to core: None
Worker 15 starting: Normal priority, pin to core: None
Worker 8 starting: Normal priority, pin to core: None
Worker 9 starting: Normal priority, pin to core: None
Worker 3 starting: Normal priority, pin to core: None
Worker 11 starting: Normal priority, pin to core: None
Worker 19 starting: Normal priority, pin to core: None
Worker 14 starting: Normal priority, pin to core: None
Worker 21 starting: Normal priority, pin to core: None
Worker 16 starting: Normal priority, pin to core: None
Worker 17 starting: Normal priority, pin to core: None
Worker 7 starting: Normal priority, pin to core: None
Worker 18 starting: Normal priority, pin to core: None
Worker 5 starting: Normal priority, pin to core: None
Worker 20 starting: Normal priority, pin to core: None
Worker 22 starting: Normal priority, pin to core: None
Worker 6 starting: Normal priority, pin to core: None
Worker 23 starting: Normal priority, pin to core: None
Worker 24 starting: Normal priority, pin to core: None
Worker 25 starting: Normal priority, pin to core: None
Worker 26 starting: Normal priority, pin to core: None
Worker 27 starting: Normal priority, pin to core: None
Worker 28 starting: Normal priority, pin to core: None
Worker 30 starting: Normal priority, pin to core: None
Worker 29 starting: Normal priority, pin to core: None
Worker 31 starting: Normal priority, pin to core: None

--- Running benchmark with 32 workers ---
Worker 0 starting: AboveNormal priority, pin to core: None
Worker 6 starting: AboveNormal priority, pin to core: None
Worker 1 starting: Normal priority, pin to core: None
Worker 10 starting: AboveNormal priority, pin to core: None
Worker 29 starting: Normal priority, pin to core: None
Worker 9 starting: Normal priority, pin to core: None
Worker 3 starting: Normal priority, pin to core: None
Worker 26 starting: AboveNormal priority, pin to core: None
Worker 4 starting: AboveNormal priority, pin to core: None
Worker 18 starting: AboveNormal priority, pin to core: None
Worker 20 starting: AboveNormal priority, pin to core: None
Worker 11 starting: Normal priority, pin to core: None
Worker 22 starting: AboveNormal priority, pin to core: None
Worker 13 starting: Normal priority, pin to core: None
Worker 14 starting: AboveNormal priority, pin to core: None
Worker 15 starting: Normal priority, pin to core: None
Worker 16 starting: AboveNormal priority, pin to core: None
Worker 17 starting: Normal priority, pin to core: None
Worker 5 starting: Normal priority, pin to core: None
Worker 19 starting: Normal priority, pin to core: None
Worker 2 starting: AboveNormal priority, pin to core: None
Worker 21 starting: Normal priority, pin to core: None
Worker 12 starting: AboveNormal priority, pin to core: None
Worker 23 starting: Normal priority, pin to core: None
Worker 24 starting: AboveNormal priority, pin to core: None
Worker 25 starting: Normal priority, pin to core: None
Worker 7 starting: Normal priority, pin to core: None
Worker 27 starting: Normal priority, pin to core: None
Worker 28 starting: AboveNormal priority, pin to core: None
Worker 8 starting: AboveNormal priority, pin to core: None
Worker 30 starting: AboveNormal priority, pin to core: None
Worker 31 starting: Normal priority, pin to core: None

--- Results Summary ---

Test 1: All workers with Normal priority, no pinning
Total tasks processed: 10000
Tasks per second: 2000.00
Task latency (microseconds):
  Min: 10
  p50: 498
  p99: 994
  Max: 13082

Per-worker statistics:
Worker 0: 301 tasks, latency min/p50/p99/max: 17/531/997/4697 µs
Worker 1: 298 tasks, latency min/p50/p99/max: 11/547/999/2226 µs
Worker 2: 325 tasks, latency min/p50/p99/max: 13/494/992/1180 µs
Worker 3: 322 tasks, latency min/p50/p99/max: 10/476/994/997 µs
Worker 4: 330 tasks, latency min/p50/p99/max: 10/501/984/994 µs
Worker 5: 273 tasks, latency min/p50/p99/max: 11/550/998/9550 µs
Worker 6: 305 tasks, latency min/p50/p99/max: 11/512/995/1066 µs
Worker 7: 308 tasks, latency min/p50/p99/max: 11/528/988/1000 µs
Worker 8: 316 tasks, latency min/p50/p99/max: 11/484/995/4826 µs
Worker 9: 332 tasks, latency min/p50/p99/max: 10/440/992/999 µs
Worker 10: 334 tasks, latency min/p50/p99/max: 12/463/989/999 µs
Worker 11: 325 tasks, latency min/p50/p99/max: 10/454/983/999 µs
Worker 12: 315 tasks, latency min/p50/p99/max: 13/488/1159/2728 µs
Worker 13: 302 tasks, latency min/p50/p99/max: 16/547/991/1499 µs
Worker 14: 305 tasks, latency min/p50/p99/max: 17/520/996/999 µs
Worker 15: 304 tasks, latency min/p50/p99/max: 14/509/993/1000 µs
Worker 16: 279 tasks, latency min/p50/p99/max: 13/546/2741/9212 µs
Worker 17: 322 tasks, latency min/p50/p99/max: 10/496/998/2047 µs
Worker 18: 314 tasks, latency min/p50/p99/max: 11/499/993/1058 µs
Worker 19: 319 tasks, latency min/p50/p99/max: 11/498/992/997 µs
Worker 20: 322 tasks, latency min/p50/p99/max: 10/479/984/2539 µs
Worker 21: 299 tasks, latency min/p50/p99/max: 10/498/1000/13082 µs
Worker 22: 316 tasks, latency min/p50/p99/max: 12/512/992/999 µs
Worker 23: 318 tasks, latency min/p50/p99/max: 10/469/1042/2836 µs
Worker 24: 311 tasks, latency min/p50/p99/max: 14/487/1000/2482 µs
Worker 25: 316 tasks, latency min/p50/p99/max: 14/488/981/996 µs
Worker 26: 303 tasks, latency min/p50/p99/max: 18/524/990/999 µs
Worker 27: 316 tasks, latency min/p50/p99/max: 10/515/991/999 µs
Worker 28: 316 tasks, latency min/p50/p99/max: 13/500/994/3408 µs
Worker 29: 332 tasks, latency min/p50/p99/max: 16/476/985/1098 µs
Worker 30: 307 tasks, latency min/p50/p99/max: 24/510/977/1000 µs
Worker 31: 315 tasks, latency min/p50/p99/max: 18/511/986/994 µs

Test 2: Mixed worker priorities, no pinning
Total tasks processed: 10000
Tasks per second: 2000.00
Task latency (microseconds):
  Min: 10
  p50: 501
  p99: 995
  Max: 9748

Per-worker statistics:
Worker 0: 328 tasks, latency min/p50/p99/max: 12/488/995/998 µs
Worker 1: 322 tasks, latency min/p50/p99/max: 15/492/998/2614 µs
Worker 2: 324 tasks, latency min/p50/p99/max: 11/462/993/999 µs
Worker 3: 293 tasks, latency min/p50/p99/max: 25/556/1386/2374 µs
Worker 4: 312 tasks, latency min/p50/p99/max: 12/511/987/999 µs
Worker 5: 271 tasks, latency min/p50/p99/max: 17/547/1267/9748 µs
Worker 6: 313 tasks, latency min/p50/p99/max: 14/528/975/997 µs
Worker 7: 296 tasks, latency min/p50/p99/max: 10/556/1465/3580 µs
Worker 8: 307 tasks, latency min/p50/p99/max: 11/514/994/1000 µs
Worker 9: 324 tasks, latency min/p50/p99/max: 10/487/985/998 µs
Worker 10: 326 tasks, latency min/p50/p99/max: 13/507/970/999 µs
Worker 11: 279 tasks, latency min/p50/p99/max: 26/569/1238/9259 µs
Worker 12: 318 tasks, latency min/p50/p99/max: 10/486/990/1000 µs
Worker 13: 299 tasks, latency min/p50/p99/max: 12/529/1025/2626 µs
Worker 14: 333 tasks, latency min/p50/p99/max: 11/440/990/997 µs
Worker 15: 319 tasks, latency min/p50/p99/max: 15/482/988/1429 µs
Worker 16: 317 tasks, latency min/p50/p99/max: 12/503/993/999 µs
Worker 17: 316 tasks, latency min/p50/p99/max: 10/507/996/1000 µs
Worker 18: 335 tasks, latency min/p50/p99/max: 13/465/996/1000 µs
Worker 19: 318 tasks, latency min/p50/p99/max: 13/497/994/999 µs
Worker 20: 331 tasks, latency min/p50/p99/max: 13/479/983/999 µs
Worker 21: 299 tasks, latency min/p50/p99/max: 11/492/1352/4223 µs
Worker 22: 323 tasks, latency min/p50/p99/max: 13/502/991/997 µs
Worker 23: 296 tasks, latency min/p50/p99/max: 10/559/1000/1453 µs
Worker 24: 325 tasks, latency min/p50/p99/max: 16/475/980/998 µs
Worker 25: 296 tasks, latency min/p50/p99/max: 13/525/998/1355 µs
Worker 26: 325 tasks, latency min/p50/p99/max: 14/464/980/997 µs
Worker 27: 305 tasks, latency min/p50/p99/max: 10/505/982/994 µs
Worker 28: 310 tasks, latency min/p50/p99/max: 16/510/992/1327 µs
Worker 29: 311 tasks, latency min/p50/p99/max: 21/510/990/3600 µs
Worker 30: 299 tasks, latency min/p50/p99/max: 10/529/996/1000 µs
Worker 31: 330 tasks, latency min/p50/p99/max: 13/463/988/1855 µs
```

### Linux

```bash
     Running `target/release/examples/parallel_tasks`
Multi-Threaded Physics/AI Sweep Benchmark
=========================================
CPU Info:
  Model: Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz
  Physical cores: 4
  Performance cores: 4
  Efficiency cores: 0
  Logical cores: 8
Found 4 P-cores and 0 E-cores

--- Running benchmark with 8 workers ---
Worker 0 starting: Normal priority, pin to core: None
Worker 1 starting: Normal priority, pin to core: None
Worker 2 starting: Normal priority, pin to core: None
Worker 3 starting: Normal priority, pin to core: None
Worker 4 starting: Normal priority, pin to core: None
Worker 5 starting: Normal priority, pin to core: None
Worker 6 starting: Normal priority, pin to core: None
Worker 7 starting: Normal priority, pin to core: None

--- Running benchmark with 8 workers ---
Worker 0 starting: AboveNormal priority, pin to core: None
Worker 1 starting: Normal priority, pin to core: None
Worker 2 starting: AboveNormal priority, pin to core: None
Worker 3 starting: Normal priority, pin to core: None
Worker 4 starting: AboveNormal priority, pin to core: None
Worker 5 starting: Normal priority, pin to core: None
Worker 6 starting: AboveNormal priority, pin to core: None
Worker 7 starting: Normal priority, pin to core: None

--- Results Summary ---

Test 1: All workers with Normal priority, no pinning
Total tasks processed: 10000
Tasks per second: 2000.00
Task latency (microseconds):
  Min: 10
  p50: 516
  p99: 1190
  Max: 19639

Per-worker statistics:
Worker 0: 1243 tasks, latency min/p50/p99/max: 10/520/1448/3416 µs
Worker 1: 1266 tasks, latency min/p50/p99/max: 11/515/1357/2052 µs
Worker 2: 1267 tasks, latency min/p50/p99/max: 11/538/993/1052 µs
Worker 3: 1306 tasks, latency min/p50/p99/max: 10/502/991/1192 µs
Worker 4: 1244 tasks, latency min/p50/p99/max: 10/522/1234/2190 µs
Worker 5: 1267 tasks, latency min/p50/p99/max: 10/509/1190/2486 µs
Worker 6: 1186 tasks, latency min/p50/p99/max: 12/519/2645/7049 µs
Worker 7: 1221 tasks, latency min/p50/p99/max: 10/500/997/19639 µs

Test 2: Mixed worker priorities, no pinning
Total tasks processed: 10000
Tasks per second: 2000.00
Task latency (microseconds):
  Min: 10
  p50: 516
  p99: 1151
  Max: 12230

Per-worker statistics:
Worker 0: 1233 tasks, latency min/p50/p99/max: 15/519/1398/5204 µs
Worker 1: 1298 tasks, latency min/p50/p99/max: 10/508/993/2218 µs
Worker 2: 1286 tasks, latency min/p50/p99/max: 10/502/1010/3256 µs
Worker 3: 1232 tasks, latency min/p50/p99/max: 10/537/1268/3581 µs
Worker 4: 1314 tasks, latency min/p50/p99/max: 10/491/990/999 µs
Worker 5: 1245 tasks, latency min/p50/p99/max: 10/510/1419/5228 µs
Worker 6: 1259 tasks, latency min/p50/p99/max: 10/514/997/4321 µs
Worker 7: 1133 tasks, latency min/p50/p99/max: 10/545/1768/12230 µs
```
