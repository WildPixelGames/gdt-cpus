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
    /// ```rust
    /// use gdt_cpus::{MechanismPolicy, ThreadPriority, set_thread_priority};
    ///
    /// let applied = set_thread_priority(ThreadPriority::Normal)?;
    /// assert_eq!(applied.mechanism().policy, MechanismPolicy::WinPriority);
    /// assert_eq!(applied.mechanism().value, 0);
    /// # Ok::<(), gdt_cpus::Error>(())
    /// ```
    pub const fn default_for(priority: ThreadPriority) -> Self {
        match priority {
            // TODO(windows): consider THREAD_MODE_BACKGROUND_BEGIN for Background -
            // it also lowers I/O and memory priority (truer "don't disturb the
            // game"), but it's a mode toggle: every set_thread_priority would
            // need an unconditional BACKGROUND_END first, and Very Low I/O
            // priority can make background loads glacial. Deferred.
            ThreadPriority::Background => SchedulingPolicy(THREAD_PRIORITY_IDLE.0),
            ThreadPriority::Lowest => SchedulingPolicy(THREAD_PRIORITY_LOWEST.0),
            ThreadPriority::BelowNormal => SchedulingPolicy(THREAD_PRIORITY_BELOW_NORMAL.0),
            ThreadPriority::Normal => SchedulingPolicy(THREAD_PRIORITY_NORMAL.0),
            ThreadPriority::AboveNormal => SchedulingPolicy(THREAD_PRIORITY_ABOVE_NORMAL.0),
            ThreadPriority::Highest => SchedulingPolicy(THREAD_PRIORITY_HIGHEST.0),
            ThreadPriority::TimeCritical => SchedulingPolicy(THREAD_PRIORITY_TIME_CRITICAL.0),
        }
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
