//! Defines thread priority levels used for scheduling.
//!
//! This module contains the [`ThreadPriority`] enum, which specifies various
//! priority levels that can be assigned to a thread. These levels help the
//! operating system determine how to schedule threads, which is crucial for
//! performance-sensitive applications like games.
//!
//! Each priority level is described with an example workload and notes on its
//! typical behavior, especially on Linux systems. Higher priority levels generally
//! mean that a thread is more likely to be run and less likely to be preempted,
//! but the exact behavior is OS-dependent. Using very high (real-time) priorities
//! often requires special permissions.

/// Represents different priority levels that can be assigned to a thread.
///
/// These priority levels are hints to the operating system's scheduler.
/// The actual behavior can vary based on the OS, system load, and other factors.
///
/// On Linux, some priority levels map to `nice` values for `SCHED_OTHER` policy,
/// while `Highest` and `TimeCritical` typically map to real-time policies like
/// `SCHED_RR` and may require `CAP_SYS_NICE` capabilities or root privileges.
///
/// The enum derives common traits like `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`,
/// `PartialOrd`, `Ord`, `Hash`, and `Default` (where `Normal` is the default).
/// It also implements `Display` for easy printing of priority level names.
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ThreadPriority {
    /// Background priority: For tasks that should only run when CPU is idle.
    ///
    /// Ideal for non-critical background operations that should have minimal impact
    /// on foreground tasks.
    ///
    /// # Example Workloads
    /// *   Steam API synchronization, achievement updates, cloud saves.
    /// *   Other absolute background noise processes.
    ///
    /// # Platform Notes
    /// *   **Linux:** Typically uses `SCHED_OTHER` policy with a high `nice` value (e.g., 19).
    ///     Under heavy system load, p99 latency can spike significantly, potentially
    ///     into hundreds of milliseconds or even seconds.
    Background,

    /// Lowest priority: For tasks that are not time-sensitive but more important than background.
    ///
    /// Suitable for analytics, telemetry, or statistics collection that should not
    /// interfere with main application performance.
    ///
    /// # Example Workloads
    /// *   Analytics, telemetry data collection.
    /// *   Low-importance tasks that don't impact gameplay.
    ///
    /// # Platform Notes
    /// *   **Linux:** Typically uses `SCHED_OTHER` with a `nice` value (e.g., 15).
    ///     Tail-latencies can be long, similar to `Background` priority.
    Lowest,

    /// Below normal priority: For tasks that are less critical than normal operations.
    ///
    /// Use for asynchronous workers, secondary game systems, AI planning, or other
    /// non-urgent gameplay systems that can be preempted by more critical tasks.
    ///
    /// # Example Workloads
    /// *   Asynchronous worker threads, secondary game systems.
    /// *   AI pathfinding or planning.
    /// *   Non-urgent gameplay logic.
    ///
    /// # Platform Notes
    /// *   **Linux:** Typically uses `SCHED_OTHER` with a `nice` value (e.g., 10).
    ///     May suffer from long tail-latencies under heavy contention.
    BelowNormal,

    /// Normal priority: The default priority for most threads.
    ///
    /// Suitable for general tasks like asset loading, streaming, or prefetching,
    /// where I/O is often the bottleneck but latency still matters.
    ///
    /// # Example Workloads
    /// *   Asset loading and streaming.
    /// *   Prefetching game data.
    /// *   Standard application threads.
    ///
    /// # Platform Notes
    /// *   **Linux:** Typically uses `SCHED_OTHER` with a `nice` value of 0.
    ///     Offers no real-time guarantees and can experience latency spikes under heavy load.
    #[default]
    Normal,

    /// Above normal priority: For tasks that are more important than normal but not critical.
    ///
    /// Use for main game logic, input processing, or UI threads that need to be responsive
    /// but don't require hard real-time guarantees.
    ///
    /// # Example Workloads
    /// *   Main game loop, primary game logic.
    /// *   User input processing.
    /// *   UI rendering and interaction thread.
    ///
    /// # Platform Notes
    /// *   **Linux:** Typically uses `SCHED_OTHER` with a negative `nice` value (e.g., -5).
    ///     Still not a real-time priority; latency spikes are possible.
    AboveNormal,

    /// Highest priority: For critical tasks that are deadline-sensitive.
    ///
    /// Recommended for render threads or audio processing threads where meeting deadlines
    /// is crucial for smooth user experience.
    ///
    /// # Example Workloads
    /// *   Main render thread.
    /// *   Audio processing and mixing thread.
    ///
    /// # Platform Notes
    /// *   **General:** Often maps to a real-time scheduling policy.
    /// *   **Linux:** Typically maps to `SCHED_RR` (Round Robin) with a high real-time priority.
    ///     Requires `CAP_SYS_NICE` capability or root privileges.
    Highest,

    /// Time-critical priority: For extremely sensitive tasks requiring minimum latency.
    ///
    /// **Use with extreme caution.** This level gives threads the highest possible precedence
    /// and can potentially starve other system processes if not managed carefully.
    /// Ideal for short, critical bursts of work on performance cores.
    ///
    /// # Example Workloads
    /// *   Highly critical worker threads pinned to Performance-cores (P-cores).
    /// *   Tasks demanding absolute minimum latency.
    ///
    /// # Platform Notes
    /// *   **General:** Maps to the highest available real-time scheduling priority.
    /// *   **Linux:** Typically maps to `SCHED_RR` with a very high (often maximum) real-time priority.
    ///     Requires `CAP_SYS_NICE` capability or root privileges. Offers virtually no tail-latency.
    TimeCritical,
}

impl std::fmt::Display for ThreadPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreadPriority::Background => write!(f, "Background"),
            ThreadPriority::Lowest => write!(f, "Lowest"),
            ThreadPriority::BelowNormal => write!(f, "BelowNormal"),
            ThreadPriority::Normal => write!(f, "Normal"),
            ThreadPriority::AboveNormal => write!(f, "AboveNormal"),
            ThreadPriority::Highest => write!(f, "Highest"),
            ThreadPriority::TimeCritical => write!(f, "TimeCritical"),
        }
    }
}
