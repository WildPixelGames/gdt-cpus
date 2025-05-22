//! Windows-specific scheduling policy definitions.
//!
//! This module defines the [`SchedulingPolicy`] struct used for setting thread
//! priority levels on Windows systems. It directly maps to the priority constants
//! defined in the Windows API (e.g., `THREAD_PRIORITY_NORMAL`, `THREAD_PRIORITY_HIGHEST`).

use windows::Win32::System::Threading::{
    THREAD_PRIORITY_ABOVE_NORMAL, THREAD_PRIORITY_BELOW_NORMAL, THREAD_PRIORITY_HIGHEST,
    THREAD_PRIORITY_IDLE, THREAD_PRIORITY_LOWEST, THREAD_PRIORITY_NORMAL,
    THREAD_PRIORITY_TIME_CRITICAL,
};

use crate::ThreadPriority;

/// Represents a Windows thread priority level.
///
/// This struct is a thin wrapper around an `i32` value that corresponds to
/// one of the `THREAD_PRIORITY_*` constants from the Windows API.
///
/// See Microsoft's documentation on "Scheduling Priorities" for more details.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SchedulingPolicy(
    /// The raw Windows thread priority value (e.g., `THREAD_PRIORITY_NORMAL.0`).
    pub i32,
);

impl SchedulingPolicy {
    /// Returns the default Windows `SchedulingPolicy` (priority level)
    /// for a given [`ThreadPriority`].
    ///
    /// This function maps abstract `ThreadPriority` levels to concrete Windows
    /// `THREAD_PRIORITY_*` constants.
    ///
    /// # Examples
    /// ```
    /// # use gdt_cpus::SchedulingPolicy;
    /// # use gdt_cpus::ThreadPriority;
    /// # use windows::Win32::System::Threading::THREAD_PRIORITY_NORMAL;
    /// let policy = SchedulingPolicy::default_for(ThreadPriority::Normal);
    /// assert_eq!(policy.0, THREAD_PRIORITY_NORMAL.0);
    /// ```
    pub const fn default_for(priority: ThreadPriority) -> Self {
        match priority {
            ThreadPriority::Background => SchedulingPolicy(THREAD_PRIORITY_IDLE.0),
            ThreadPriority::Lowest => SchedulingPolicy(THREAD_PRIORITY_LOWEST.0),
            ThreadPriority::BelowNormal => SchedulingPolicy(THREAD_PRIORITY_BELOW_NORMAL.0),
            ThreadPriority::Normal => SchedulingPolicy(THREAD_PRIORITY_NORMAL.0),
            ThreadPriority::AboveNormal => SchedulingPolicy(THREAD_PRIORITY_ABOVE_NORMAL.0),
            ThreadPriority::Highest => SchedulingPolicy(THREAD_PRIORITY_HIGHEST.0),
            ThreadPriority::TimeCritical => SchedulingPolicy(THREAD_PRIORITY_TIME_CRITICAL.0),
        }
    }

    /// Provides a static array of default [`SchedulingPolicy`] mappings
    /// for all [`ThreadPriority`] variants on Windows.
    ///
    /// The order in the array corresponds to the order of variants in `ThreadPriority`.
    /// This is used by `affinity::get_scheduling_policies()` when no other
    /// mappings have been set.
    ///
    /// # Examples
    /// ```
    /// # use gdt_cpus::SchedulingPolicy;
    /// # use windows::Win32::System::Threading::THREAD_PRIORITY_NORMAL;
    /// let mappings = SchedulingPolicy::default_mappings();
    /// assert_eq!(mappings.len(), 7);
    /// assert_eq!(mappings[3].0, THREAD_PRIORITY_NORMAL.0); // Index 3 is Normal
    /// ```
    pub const fn default_mappings() -> &'static [SchedulingPolicy; 7] {
        // Note: This array's order must match the ThreadPriority enum order.
        static DEFAULT_MAPPINGS: [SchedulingPolicy; 7] = [
            SchedulingPolicy::default_for(ThreadPriority::Background),
            SchedulingPolicy::default_for(ThreadPriority::Lowest),
            SchedulingPolicy::default_for(ThreadPriority::BelowNormal),
            SchedulingPolicy::default_for(ThreadPriority::Normal),
            SchedulingPolicy::default_for(ThreadPriority::AboveNormal),
            SchedulingPolicy::default_for(ThreadPriority::Highest),
            SchedulingPolicy::default_for(ThreadPriority::TimeCritical),
        ];

        &DEFAULT_MAPPINGS
    }
}

impl std::fmt::Display for SchedulingPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Match the raw i32 value to its constant name for a more readable display
        let priority_str = match self.0 {
            v if v == THREAD_PRIORITY_IDLE.0 => "Idle",
            v if v == THREAD_PRIORITY_LOWEST.0 => "Lowest",
            v if v == THREAD_PRIORITY_BELOW_NORMAL.0 => "BelowNormal",
            v if v == THREAD_PRIORITY_NORMAL.0 => "Normal",
            v if v == THREAD_PRIORITY_ABOVE_NORMAL.0 => "AboveNormal",
            v if v == THREAD_PRIORITY_HIGHEST.0 => "Highest",
            v if v == THREAD_PRIORITY_TIME_CRITICAL.0 => "TimeCritical",
            _ => "Unknown", // Should not happen with values from default_for
        };
        write!(f, "WindowsPriority({})", priority_str)
    }
}
