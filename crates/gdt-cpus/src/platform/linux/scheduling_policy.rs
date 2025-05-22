//! Linux-specific scheduling policy definitions.
//!
//! This module defines the [`SchedulingPolicy`] enum and related constants
//! used for setting thread scheduling parameters on Linux systems.
//! It maps [`ThreadPriority`](crate::ThreadPriority) levels to either `nice` values
//! for the `SCHED_OTHER` policy or absolute priorities for real-time policies
//! like `SCHED_RR`.

use libc::c_int;

use crate::ThreadPriority;

/// `nice` value for `SCHED_OTHER` corresponding to `ThreadPriority::Background`.
pub const NICE_BACKGROUND: c_int = 19;
/// `nice` value for `SCHED_OTHER` corresponding to `ThreadPriority::Lowest`.
pub const NICE_LOWEST: c_int = 15;
/// `nice` value for `SCHED_OTHER` corresponding to `ThreadPriority::BelowNormal`.
pub const NICE_BELOW_NORMAL: c_int = 10;
/// `nice` value for `SCHED_OTHER` corresponding to `ThreadPriority::Normal`.
pub const NICE_NORMAL: c_int = 0;
/// `nice` value for `SCHED_OTHER` corresponding to `ThreadPriority::AboveNormal`.
pub const NICE_ABOVE_NORMAL: c_int = -5;
// Note: Real-time priorities (Highest, TimeCritical) use absolute values, not 'nice'.

/// Represents a Linux scheduling policy and its associated parameter.
///
/// On Linux, thread scheduling can be controlled using different policies.
/// This enum encapsulates the two main approaches used by this crate:
/// - `SCHED_OTHER`: The default time-sharing scheduler, whose behavior is influenced
///   by `nice` values. Lower `nice` values (more negative) mean higher priority.
/// - `SCHED_RR` (Round Robin): Real-time policies that use absolute priority values (1-99).
///   Higher values mean higher priority.
///   Using these policies typically requires `CAP_SYS_NICE` capability.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SchedulingPolicy {
    /// Uses the standard `SCHED_OTHER` scheduling policy with a specified `nice` value.
    ///
    /// `nice` values range from -20 (highest priority) to 19 (lowest priority).
    /// See `man 7 sched` for more details on `SCHED_OTHER` and `nice`.
    Nice {
        /// The `nice` value to apply.
        value: c_int,
    },
    /// Uses a real-time scheduling policy (typically `SCHED_RR` in this crate)
    /// with an absolute priority level.
    ///
    /// Absolute priorities for real-time policies range from 1 (lowest) to 99 (highest).
    /// See `man 7 sched` for more details on `SCHED_RR`.
    Absolute {
        /// The absolute priority value (1-99).
        priority: c_int,
    },
}

impl SchedulingPolicy {
    /// Returns the default Linux `SchedulingPolicy` for a given [`ThreadPriority`].
    ///
    /// This function maps the abstract `ThreadPriority` levels to concrete
    /// Linux scheduling parameters (`nice` values for normal priorities,
    /// absolute priorities for real-time ones).
    ///
    /// # Examples
    /// ```
    /// # use gdt_cpus::platform::linux::scheduling_policy::{SchedulingPolicy, NICE_NORMAL};
    /// # use gdt_cpus::ThreadPriority;
    /// let policy = SchedulingPolicy::default_for(ThreadPriority::Normal);
    /// assert_eq!(policy, SchedulingPolicy::Nice { value: NICE_NORMAL });
    ///
    /// let rt_policy = SchedulingPolicy::default_for(ThreadPriority::TimeCritical);
    /// assert_eq!(rt_policy, SchedulingPolicy::Absolute { priority: 99 });
    /// ```
    pub const fn default_for(priority: ThreadPriority) -> Self {
        match priority {
            ThreadPriority::Background => SchedulingPolicy::Nice {
                value: NICE_BACKGROUND,
            },
            ThreadPriority::Lowest => SchedulingPolicy::Nice { value: NICE_LOWEST },
            ThreadPriority::BelowNormal => SchedulingPolicy::Nice {
                value: NICE_BELOW_NORMAL,
            },
            ThreadPriority::Normal => SchedulingPolicy::Nice { value: NICE_NORMAL },
            ThreadPriority::AboveNormal => SchedulingPolicy::Nice {
                value: NICE_ABOVE_NORMAL,
            },
            // For SCHED_RR, priority is 1 (low) to 99 (high).
            // We map Highest to 97 and TimeCritical to 99.
            // These values are somewhat arbitrary but provide distinct high-priority levels.
            ThreadPriority::Highest => SchedulingPolicy::Absolute { priority: 97 },
            ThreadPriority::TimeCritical => SchedulingPolicy::Absolute { priority: 99 },
        }
    }

    /// Provides a static array of default [`SchedulingPolicy`] mappings
    /// for all [`ThreadPriority`] variants.
    ///
    /// The order in the array corresponds to the order of variants in `ThreadPriority`:
    /// Background, Lowest, BelowNormal, Normal, AboveNormal, Highest, TimeCritical.
    ///
    /// This is used by `affinity::get_scheduling_policies()` when no other
    /// mappings have been set.
    ///
    /// # Examples
    /// ```
    /// # use gdt_cpus::SchedulingPolicy;
    /// let mappings = SchedulingPolicy::default_mappings();
    /// assert_eq!(mappings.len(), 7);
    /// assert_eq!(mappings[3], SchedulingPolicy::default_for(gdt_cpus::ThreadPriority::Normal));
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
