//! Platform-Specific Implementations.
//!
//! This module acts as a dispatcher for platform-specific code. It uses
//! conditional compilation (`#[cfg]`) to include and re-export types and
//! functions relevant to the target operating system (Linux, macOS, Windows)
//! and architecture (e.g., x86_64).
//!
//! The primary goal is to abstract away platform differences, providing a
//! consistent API at the crate level for functionalities like:
//! - CPU information detection (`CpuInfo::detect()`).
//! - Thread affinity and priority settings (via types like [`SchedulingPolicy`]).
//!
//! User-facing types like [`SchedulingPolicy`] are re-exported from the
//! appropriate platform-specific submodule (e.g., `linux::scheduling_policy`,
//! `macos::scheduling_policy`, `windows::scheduling_policy`) ensuring that
//! users of the crate can work with a single type regardless of the target OS.
//!
//! Internal helper modules, such as `common_x86_64`, provide shared logic for
//! specific architectures across different operating systems.

// This module contains platform-specific implementations for CPU information,
// thread affinity, and performance monitoring.

// Conditionally compile and export platform-specific modules.

#[cfg(target_arch = "x86_64")]
pub(crate) mod common_x86_64;

#[cfg(target_os = "linux")]
pub(crate) mod linux;

#[cfg(target_os = "macos")]
pub(crate) mod macos;

#[cfg(target_os = "windows")]
pub(crate) mod windows;

#[cfg(target_os = "linux")]
pub use linux::scheduling_policy::SchedulingPolicy;

#[cfg(target_os = "macos")]
pub use macos::scheduling_policy::SchedulingPolicy;

#[cfg(target_os = "windows")]
pub use windows::scheduling_policy::SchedulingPolicy;

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SchedulingPolicy {}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
impl SchedulingPolicy {
    pub const fn default_for(_priority: crate::ThreadPriority) -> Self {
        SchedulingPolicy {}
    }

    pub const fn default_mappings() -> &'static [SchedulingPolicy; 7] {
        // Note: This array's order must match the ThreadPriority enum order.
        static DEFAULT_MAPPINGS: [SchedulingPolicy; 7] = [
            SchedulingPolicy::default_for(crate::ThreadPriority::Background),
            SchedulingPolicy::default_for(crate::ThreadPriority::Lowest),
            SchedulingPolicy::default_for(crate::ThreadPriority::BelowNormal),
            SchedulingPolicy::default_for(crate::ThreadPriority::Normal),
            SchedulingPolicy::default_for(crate::ThreadPriority::AboveNormal),
            SchedulingPolicy::default_for(crate::ThreadPriority::Highest),
            SchedulingPolicy::default_for(crate::ThreadPriority::TimeCritical),
        ];

        &DEFAULT_MAPPINGS
    }
}
