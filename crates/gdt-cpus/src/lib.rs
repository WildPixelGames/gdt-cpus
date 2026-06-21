//! GDT-CPUs: Game Developer's Toolkit for CPU Management
//!
//! This crate provides detailed CPU information and thread management capabilities
//! specifically designed for game developers: CPU topology (including hybrid
//! P/E/LP-E core kinds and L3 cache domains), thread affinity and thread priority.
//!
//! # Key Features
//!
//! *   **Flat topology model**: one [`Lp`] record per logical processor plus a
//!     first-class [`L3Domain`] table - chiplet CPUs (multiple CCDs per socket)
//!     and hybrid designs are represented faithfully.
//! *   **Core kinds**: [`CoreKind::Performance`] / [`CoreKind::Efficiency`] /
//!     [`CoreKind::LpEfficiency`] - modern silicon ships more than two kinds.
//! *   **L3 cache domains**: group cooperating threads by shared L3
//!     ([`CpuInfo::l3_domain_mask`]) - cross-domain latency is the real cliff.
//! *   **Thread Affinity**: pin threads to logical cores or sets of them.
//! *   **Thread Priority**: 7 portable levels mapped to each OS's scheduler.
//! *   **No global state**: [`CpuInfo::detect()`] returns a plain value you own.
//!
//! # Getting Started
//!
//! ```
//! use gdt_cpus::CpuInfo;
//!
//! fn main() -> Result<(), gdt_cpus::Error> {
//!     let info = CpuInfo::detect()?;
//!
//!     println!("CPU: {} ({})", info.model_name, info.vendor);
//!     println!("{} cores / {} threads", info.core_count, info.lps.len());
//!
//!     if info.is_hybrid() {
//!         println!("hybrid: {}P + {}E + {}LP-E",
//!             info.num_performance_cores(),
//!             info.num_efficiency_cores(),
//!             info.num_lp_efficiency_cores());
//!     }
//!
//!     for (i, d) in info.l3_domains.iter().enumerate() {
//!         println!("L3 domain {}: {} MiB, {} cores", i, d.size_bytes >> 20, d.core_count);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Thread placement: what goes where (rules of thumb)
//!
//! Two independent levers exist on Linux/Windows: **placement** (affinity -
//! WHERE a thread may run) and **priority** (WHO wins when threads compete).
//! On macOS QoS fuses both (and Apple Silicon ignores pinning entirely), so
//! treat affinity as best-effort and priority as the portable lever.
//!
//! | Work | Cores | Priority |
//! |---|---|---|
//! | Main / render thread | best Performance core (highest [`Lp::perf_hint`], `smt_index == 0`) | `AboveNormal`-`Highest` |
//! | Simulation / job workers | one per remaining Performance core primary; keep cooperating sets inside ONE L3 domain | `Normal` |
//! | Audio / haptics feeder | any Performance core - do NOT pin it onto the busiest one | `TimeCritical` (dedicate the thread; on macOS it permanently leaves the QoS system). For hard deadlines, [`promote_thread_to_realtime`] |
//! | Asset streaming / decompression | Efficiency cores **if present** - the mask CAN be empty, always fall back to Performance | `BelowNormal` |
//! | Shader/PSO compilation, navmesh & lighting bakes, batch processing | wherever there's room - these want throughput, not latency | `Lowest` |
//! | Telemetry, autosave compression, platform callbacks | LpEfficiency island if present (trickle work only - these islands often have weak interconnects), else unpinned | `Background` |
//!
//! Further rules:
//!
//! *   **Don't pin everything.** Pinning removes the scheduler's freedom; it
//!     pays off only for threads with a real reason - latency (audio, render)
//!     or cache locality (a cooperating producer/consumer set). Leave the
//!     rest soft. On Windows prefer [`set_thread_soft_affinity`] (CPU Sets) -
//!     the scheduler keeps an escape hatch.
//! *   **One heavy thread per physical core**: build worker pools from
//!     [`CpuInfo::primary_thread_mask`] (`smt_index == 0`), not from all LPs -
//!     two heavy threads on SMT siblings share one core's execution
//!     resources. Siblings are fine for light helpers.
//! *   **Group by L3, not by core id**: cross-L3-domain communication costs
//!     multiples of in-domain (3.6× measured on a dual-CCD 5950X - run
//!     `examples/l3_domains.rs`). Place cooperating threads with
//!     [`CpuInfo::l3_domain_mask`]; never assume core ids imply locality.
//! *   **Within a kind, rank with [`Lp::perf_hint`]** - chips ship Performance
//!     tiers spanning several frequency bins (ARM prime-vs-mid, Intel favored
//!     cores). Compare it only within the same detected machine and kind; the
//!     source scale differs per OS. Equal hints = indistinguishable, don't
//!     invent an order.
//! *   **Kinds are classes, not guarantees**: a machine may have NO
//!     Efficiency cores (only P + LP-E), or nothing but Performance. Write
//!     fallbacks: `efficiency_core_mask()` empty -> use Performance at lower
//!     priority.
//! *   **Check what priority can deliver**: on a locked-down Linux box
//!     (no rtkit, default rlimits) every level above `Normal` silently
//!     resolves to `Normal`. [`priority_capabilities`] predicts this up
//!     front; [`promote_thread_to_realtime`] is the explicit escape hatch
//!     for the one thread with a hard deadline.
//!
//! ```no_run
//! use gdt_cpus::{CoreKind, CpuInfo, ThreadPriority, pin_thread_to_core, set_thread_priority};
//!
//! # fn main() -> Result<(), gdt_cpus::Error> {
//! let info = CpuInfo::detect()?;
//!
//! // Best Performance-core primaries first - render thread gets the top one.
//! let mut p_cores: Vec<_> = info.lps.iter()
//!     .filter(|lp| lp.kind == CoreKind::Performance && lp.smt_index == 0)
//!     .collect();
//!
//! p_cores.sort_by_key(|lp| std::cmp::Reverse(lp.perf_hint));
//!
//! if let Some(best) = p_cores.first() {
//!     let _ = pin_thread_to_core(best.os_id as usize); // macOS: Unsupported - fine
//!     let applied = set_thread_priority(ThreadPriority::Highest)?;
//!
//!     eprintln!("render priority: {applied}");
//! }
//!
//! // Background telemetry: LP-E island when it exists, otherwise just priority.
//! let smol = info.kind_mask(CoreKind::LpEfficiency);
//!
//! if !smol.is_empty() {
//!     let _ = gdt_cpus::set_thread_affinity(&smol);
//! }
//!
//! let _applied = set_thread_priority(ThreadPriority::Background)?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! # Cargo Features
//!
//! *   `rtkit` *(default)*: on Linux, negotiate priority through rtkit and
//!     the xdg realtime portal (hand-rolled minimal D-Bus client, no extra
//!     dependencies) when direct syscalls are denied. Opt out with
//!     `default-features = false`.
//! *   `serde`: serialization for the CPU information structures.

#![deny(missing_docs)]

// Modules
mod affinity;
mod affinity_mask;
mod capabilities;
mod cpu;
mod error;
mod platform;
mod priority;
mod realtime;

// Re-exports - Public API
pub use affinity::*;
pub use affinity_mask::AffinityMask;
pub use capabilities::{PriorityCaps, priority_capabilities};
pub use cpu::{CacheInfo, CoreKind, CpuFeatures, CpuInfo, L2Domain, L3Domain, Lp, Vendor};
pub use error::{Error, Result};
pub use priority::{
    AppliedPriority, BrokerError, FallbackReason, Grant, Mechanism, MechanismPolicy, QosClass,
    ThreadPriority,
};
pub use realtime::{demote_thread_from_realtime, promote_thread_to_realtime};

/// Total number of physical cores (SMT siblings counted once).
///
/// Convenience detection path; prefer holding a [`CpuInfo`] from
/// [`CpuInfo::detect()`] and reading `core_count`.
pub fn num_physical_cores() -> Result<usize> {
    CpuInfo::detect().map(|info| info.num_physical_cores())
}

/// Total number of logical processors (hardware threads).
pub fn num_logical_cores() -> Result<usize> {
    CpuInfo::detect().map(|info| info.num_logical_cores())
}

/// Number of physical cores classified as Performance.
///
/// Equals `num_physical_cores()` on homogeneous machines (the classification
/// invariant: homogeneous means all Performance).
pub fn num_performance_cores() -> Result<usize> {
    CpuInfo::detect().map(|info| info.num_performance_cores())
}

/// Number of physical cores classified as Efficiency (0 on non-hybrid machines).
pub fn num_efficiency_cores() -> Result<usize> {
    CpuInfo::detect().map(|info| info.num_efficiency_cores())
}

/// Number of physical cores classified as LpEfficiency (0 on non-hybrid machines).
pub fn num_lp_efficiency_cores() -> Result<usize> {
    CpuInfo::detect().map(|info| info.num_lp_efficiency_cores())
}

/// `true` if more than one core kind is present (P/E/LP-E).
pub fn is_hybrid() -> Result<bool> {
    CpuInfo::detect().map(|info| info.is_hybrid())
}
