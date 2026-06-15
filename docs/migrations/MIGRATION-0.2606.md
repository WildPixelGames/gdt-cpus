# Migration Guide: 25.12 -> 0.2606

The topology model is new, the version scheme is new, and if your code walked
`sockets -> cores -> logical_processor_ids` it won't compile until you've been through this page.
We promised we're not shy about breakage - here's the honest map. This guide is meant to be
Ctrl+F'd: search for the symbol the compiler is yelling about.

Why the big break: 25.12 modeled a CPU as a tree (`CpuInfo -> Vec<SocketInfo> -> Vec<CoreInfo>`)
and that tree couldn't say the one thing modern CPUs are about - **L3 domains**. A Ryzen 5950X
has two 32 MiB L3 complexes (CCDs) in one socket; 25.12 reported one L3 per socket and knew
nothing about NUMA. Cross-CCD communication costs ~3.6× the in-CCD round trip (measured -
run `cargo run --release --example l3_domains`), so the domain table is where thread placement
decisions live. 0.2606 flattens the model: one record per logical processor, L3 domains and NUMA
nodes as first-class data, caches per core *kind*.

## Version scheme: `25.12.0` is now `0.2606.0`

Not a downgrade. Bare CalVer lied to semver: cargo treated `25.5 -> 25.12` as a compatible minor
bump and auto-delivered breaking months. With `0.YYMM.patch` every month is a semver-breaking
epoch, so `gdt-cpus = "0.2606"` pins the June 2026 API line. `gdt-cpus-sys` now versions in
lockstep with the lib.

## `cpu_info()` is now `CpuInfo::detect()`

`CpuInfo::detect()` is the entry point and returns an owned `CpuInfo` - no global state.
`cpu_info() -> Result<&'static CpuInfo>` is removed. The count convenience functions
`num_physical_cores()` / `num_logical_cores()` / `num_performance_cores()` /
`num_efficiency_cores()` / `is_hybrid()` still exist, but they perform one-shot detection;
hold a `CpuInfo` if you need more than one query.

```rust
// 25.12
let info = gdt_cpus::cpu_info()?;

// 0.2606
let info = gdt_cpus::CpuInfo::detect()?;
```

## `SocketInfo` and `CoreInfo` are removed - the topology is flat

`CpuInfo::sockets` is gone and has no tree-shaped replacement. Every online logical processor is
one `Lp` record in `CpuInfo::lps`:

```rust
pub struct Lp {
    pub os_id: u16,      // OS LP id - affinity masks address THESE
    pub core: u16,       // dense library-assigned physical core index
    pub socket: u8,
    pub l3_domain: u8,   // index into CpuInfo::l3_domains; Lp::NO_L3 = unknown
    pub numa_node: u8,
    pub kind: CoreKind,
    pub smt_index: u8,   // 0 = first sibling on its core
    pub perf_hint: u16,  // ordinal, machine-local: higher = faster core;
                         // picks the BEST cores within a kind (ARM prime-vs-mid,
                         // Intel favored cores); 0 = no finer signal than kind
    pub cpu_part: u16,   // raw ARM MIDR part (0x0d81 = A720) read per-core from
                         // /proc/cpuinfo; 0 on x86. Distinguishes microarchs; no
                         // name table shipped. NOT a kind signal.
}
```

```rust
// 25.12
for socket in &info.sockets {
    for core in &socket.cores {
        for &lp in &core.logical_processor_ids { /* ... */ }
    }
}

// 0.2606
for lp in &info.lps { /* lp.os_id, lp.core, lp.socket, lp.kind, lp.l3_domain ... */ }

// e.g. P-core primaries (one LP per physical core, no SMT siblings):
let workers: Vec<usize> = info.lps.iter()
    .filter(|lp| lp.kind == CoreKind::Performance && lp.smt_index == 0)
    .map(|lp| lp.os_id as usize)
    .collect();
```

**Relocations** (counts are plain fields or getters now):

| 25.12                           | 0.2606                           |
| ------------------------------- | -------------------------------- |
| `info.total_sockets`            | `info.socket_count`              |
| `info.total_physical_cores`     | `info.core_count`                |
| `info.total_logical_processors` | `info.lps.len()`                 |
| `info.total_performance_cores`  | `info.num_performance_cores()`   |
| `info.total_efficiency_cores`   | `info.num_efficiency_cores()`    |
| `core.core_type`                | `lp.kind`                        |
| `core.logical_processor_ids`    | `info.lps` filtered by `lp.core` |
| `socket.id` / `core.socket_id`  | `lp.socket`                      |

## `CoreType` is now `CoreKind`

`CoreKind` has four variants: `Performance`, `Efficiency`, `LpEfficiency`, `Unknown` - 3-tier
hybrids (Intel LP E-cores, ARM DynamIQ little+middle+big) classify correctly now. `match` arms
need the new variants. `CpuInfo::cores_by_type_mask(CoreType)` is now
`CpuInfo::kind_mask(CoreKind)`; `CpuInfo::kind_core_counts: [u16; 4]` (indexed by
`CoreKind::index()`) holds physical-core counts per kind.

## Per-core caches are now per-kind: `CpuInfo::l1d` / `l1i` / `l2`

`CoreInfo::{l1_instruction_cache, l1_data_cache, l2_cache}: Option<CacheInfo>` are removed.
All cores of one kind share a cache shape, so caches live on `CpuInfo` as `[CacheInfo; 4]`
indexed by kind:

```rust
// 25.12
let l1d = info.sockets[0].cores[0].l1_data_cache;

// 0.2606
let l1d = info.l1d[CoreKind::Performance.index()]; // zeroed CacheInfo = not detected
```

`CacheInfo` itself reshaped: `{ level, cache_type, size_bytes, line_size_bytes }` is now
`{ size_bytes: u64, line_bytes: u16, shared_by: u16 }` - level and type are implied by which
field of `CpuInfo` you read; `shared_by` is the number of threads sharing one cache instance.
`CacheLevel` / `CacheType` are no longer part of the public surface.

## L3 is now `CpuInfo::l3_domains` - plural, one per CCD/cluster

`SocketInfo::l3_cache: Option<CacheInfo>` is removed. Every L3 complex is an `L3Domain`:

```rust
pub struct L3Domain {
    pub size_bytes: u64,
    pub mask: AffinityMask, // the LPs sharing this L3
    pub core_count: u16,
}
```

Code that assumed one L3 per socket must switch - that assumption lost real chiplet topology. Keeping
cooperating threads inside one domain is the new superpower:

```rust
// 0.2606 - pin a producer/consumer pair inside CCD 0
let in_domain0 = info.performance_core_mask().intersection(&info.l3_domain_mask(0));
gdt_cpus::set_thread_affinity(&in_domain0)?;
```

Mask helpers on `CpuInfo`: `l3_domain_mask(domain)`, `numa_node_mask(node)`,
`primary_thread_mask()` (NEW - one LP per physical core), `lp.perf_hint` (NEW - ordinal
within-kind ranking from Linux `cpu_capacity` / Windows `EfficiencyClass` / macOS perflevel
order; pick the best cores within a kind), plus the surviving
`all_cores_mask()` / `performance_core_mask()` (never empty) / `efficiency_core_mask()` /
`kind_mask(kind)`. Masks address **OS LP ids**, same contract as 25.12.

## `Vendor::Other(String)` is now `Vendor::Other`

`Vendor` is a payload-free `Copy` enum; previously-stringly vendors got named variants:
`Qualcomm`, `Broadcom`, `Nvidia`, `Marvell`. `match` arms on `Vendor::Other(s)` must drop the
binding; use `CpuInfo::model_name` for display strings.

## `SchedulingPolicy` and `get_scheduling_policies()` are removed

The per-OS priority tables are internal implementation detail now. The `ThreadPriority` rustdoc
carries the full mapping table (per OS, with the permission caveats). If you relied on
overriding the mappings: that machinery was never actually wired (the setter was unreachable in
25.12) - open an issue if you genuinely need it.

## `Error::NoCoreOfType`, `Error::Io`, `Error::NotImplemented` are removed

They were never constructed. `match` arms on them are dead code - delete them.

## `set_thread_soft_affinity()` is new

Windows-only CPU Sets placement (`SetThreadSelectedCpuSets`): the scheduler *prefers* the given
LPs but may run the thread elsewhere - Intel's recommended mode for games, and it works across
processor groups. Returns `Error::Unsupported` on Linux/macOS.

## `promote_thread_to_realtime()` / `demote_thread_from_realtime()` are new

The explicit real-time opt-in (see the Linux behavior change below - `set_thread_priority` no
longer hands out `SCHED_RR`). On Linux it tries direct `SCHED_RR` 85, the xdg realtime portal,
then rtkit - and sets the `RLIMIT_RTTIME` leash the brokered paths require (soft = your budget ->
catchable `SIGXCPU`; hard = the daemon's ceiling, default 200 ms -> **SIGKILL for the whole
process** if an RT thread spins past it without blocking). Read the rustdoc before calling; the
informed consent is the API. On macOS it is the `SCHED_RR` 47 one-way door, on Windows
`THREAD_PRIORITY_TIME_CRITICAL`.

## `priority_capabilities()` is new

Predicts what each `ThreadPriority` level will actually deliver under the current rlimits and
rtkit reachability, as comparable ranks: `caps.distinct(Highest, Normal) == false` means your
render thread will NOT outrank your workers on this box - decide your threading strategy up
front instead of discovering the collapse from frame times.

## `set_thread_priority()` now returns `AppliedPriority`, not `()`

The old `Result<()>` was a lie by omission: a "successful" `set_thread_priority(Highest)` on a
locked-down Linux box means you got `Highest` - OR that every privileged path was denied and you
silently landed on `Normal`. The new return says which:

```rust
let applied = gdt_cpus::set_thread_priority(ThreadPriority::Highest)?;
if applied.degraded() {
    // reason: NoBroker / BrokerTimedOut / BrokerRefused (you're at Normal), or
    // Clamped (kept the level, weaker). Branch on grant / effective / reason.
    log::warn!("priority fell short: {applied}");
}
```

`AppliedPriority` is structured data with accessors: `requested()`, `effective()`, `grant()`,
`reason()`, `broker_error()`, and `mechanism()`. Rebuild one from stored data with
`AppliedPriority::from_parts(...)`, which rejects contradictory parts such as a broker error without
`BrokerRefused`. The `FallbackReason` enum (`NoBroker`, `BrokerTimedOut`, `BrokerRefused`, `Clamped`)
and the `BrokerError` enum (`AccessDenied`, `LimitsExceeded`, `InvalidArgs`, `Failed`, `Other` - the
typed cause of a broker refusal, `Some` only when `reason() == Some(BrokerRefused)`) are **new public
API**. The call still succeeds (no `Err`) whenever the OS accepted *something* - games must not die on
unprivileged boxes - so a denied elevation is reported as **data** (`reason()`), never an error or a
hidden log line. Migration is mechanical: callers that ignored the old `()` add `let _ = ...?;` to
compile; audio/render threads should read `.reason()`/`.degraded()` and react.
`promote_thread_to_realtime` returns the same type.

The applied OS mechanism is the typed `mechanism: Mechanism { policy: MechanismPolicy, value: i8 }`
(the actual `nice` / QoS class / `SCHED_RR` priority as branchable data, `value` read per `policy`) -
there is **no `detail` string and no `.detail()` accessor**. Use the `Display` impl for the human
form (`nice -15`, `QoS UserInteractive`); read `.mechanism()` to branch.

## Behavior changes - same call, different result

- **Windows hard affinity** uses `SetThreadGroupAffinity` (was `SetThreadAffinityMask`). A mask
  spanning multiple processor groups returns `InvalidParameter` for `set_thread_affinity` - use
  `set_thread_soft_affinity` for cross-group placement.
- **Linux `Highest`/`TimeCritical` are no longer real-time.** 25.12 mapped them to `SCHED_RR`
  97/99 - which outranked the threaded IRQ handlers feeding you data, tied the kernel watchdog,
  and on the typical desktop silently fell back to nice 0 anyway. The whole ladder is now pure
  timeshare nice (19/10/5/0/-5/-10/-20): `TimeCritical` (-20) holds a ~9× CFS weight edge over
  `Highest` (-10), which holds ~9× over `Normal` - wins every wake-up race that matters, cannot
  wedge a core. Code that truly needs `SCHED_RR` must say so: `promote_thread_to_realtime()`.
- **Linux `Lowest`/`BelowNormal` got stronger**: nice 15 -> **10**, nice 10 -> **5**. The old
  `BelowNormal` was ÷9 of `Normal` - Linux-only asset pop-in under load; the new ÷3 keeps
  streaming flowing. `Lowest` (÷9) is the new home for shader/PSO compilation and bakes.
- **Linux negative nice now goes through rtkit when denied** (feature `rtkit`, on by default;
  opt out with `default-features = false`). Unprivileged processes on a systemd desktop get
  real -5/-10/-15 instead of the silent nice-0 fallback - which remains as the last resort.
  Also fixed: 25.12 matched the wrong errno (`EPERM`, actual is `EACCES`), so its documented
  fallback never fired at all.
- **macOS is Apple Silicon only.** `x86_64-apple-darwin` fails at compile time;
  the Intel macOS backend paths were cut rather than shipped untested.
- **macOS `Highest` is QoS now** (`USER_INTERACTIVE`, band 47, timeshare) - the thread stays
  inside the QoS system and keeps its P/E-core routing. `AboveNormal` is `USER_INTERACTIVE`
  relative -4. **`TimeCritical` keeps `SCHED_RR` 47** (fixed priority, no timeshare decay) and -
  per Apple's `qos.h` - **permanently opts that thread out of QoS**. Deliberate: it's the
  dedicated audio/haptics-feeder level. Re-prioritizing an opted-out thread still works (legacy
  `SCHED_OTHER` fallback) but it never rejoins QoS. Dedicate such threads.
- **Multi-CCD/cluster machines report more than one L3 domain.** See the `l3_domains` section
  above; single-domain assumptions silently mismeasure chiplet CPUs.

## gdt-cpus-sys (C ABI)

The socket/core object tree and its ~20 accessors are removed. The flat model crosses the FFI
directly:

| Removed                                                 | Replacement                                                                                                  |
| ------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| `GdtCpusSocketInfo`, `GdtCpusCoreInfo` + tree accessors | `GdtCpusLp` via `gdt_cpus_get_lp(i, *out)`                                                                   |
| per-socket L3 accessors                                 | `GdtCpusL3Domain` via `gdt_cpus_get_l3_domain(d, *out)`, members via `gdt_cpus_get_l3_domain_lp(d, i, *out)` |
| `GdtCpusCoreType`                                       | `GdtCpusCoreKind` (4 values)                                                                                 |
| per-core cache accessors                                | per-kind: `gdt_cpus_get_l1d_cache(kind, *out)`, `_l1i_`, `_l2_`                                              |
| -                                                       | `gdt_cpus_set_thread_affinity` / `_soft_affinity` over LP-id arrays; `GDT_CPUS_NO_L3` sentinel               |

`examples/c/basic_info/main.c` exercises the entire new surface - start there.
`examples/c/priority/main.c` covers `AppliedPriority`, `FallbackReason`, broker
errors, and the realtime consent API from C.

## Examples

The previous release shipped these Rust examples:

- `basic_info`
- `audio_latency`
- `frame_jitter`
- `parallel_tasks`
- `streaming_pipeline`

This release ships:

- `basic_info`
- `thread_priorities`
- `audio_latency`
- `frame_jitter`
- `reserved_core`
- `l3_domains`
- `background_budget`

What changed:

- `parallel_tasks` is removed. Use `frame_jitter` for the worker-pool sizing experiment and
  `reserved_core` for the placement-vs-priority latency framing.
- The old fake-IO `streaming_pipeline` example is removed. Use `background_budget` for the
  CPU-heavy background-work budget experiment. It reserves the render LP, sweeps worker width,
  reports the frame-safe budget, and keeps the output honest about what was measured. Treat
  these as synthetic experiments to rerun on your target hardware, not production workload
  predictions.
- `thread_priorities` is new and prints the priority capability ladder plus the concrete scheduler
  outcome for each request.
- `l3_domains` is new and measures the in-domain vs cross-domain ping-pong latency cliff.
- `audio_latency`, `frame_jitter`, and `reserved_core` were reworked enough that old captured
  output should not be compared line-by-line. They now print structured priority outcomes and
  computed takeaways from the current run.
- `examples/c/priority` is new for the C ABI priority outcome and realtime consent APIs.
