//! macOS-specific scheduling policy definitions and mappings.
//!
//! This module defines the [`SchedulingPolicy`] enum, which represents how abstract
//! [`ThreadPriority`](crate::ThreadPriority) levels are mapped to concrete macOS scheduling
//! parameters. It primarily leverages macOS's Quality of Service (QoS) classes
//! for most priority levels and supports setting absolute real-time priorities
//! (e.g., for `SCHED_RR`) for the highest levels.
//!
//! The key public items are:
//! - The [`SchedulingPolicy`] enum itself.
//! - The [`SchedulingPolicy::default_for()`] method to get a policy for a specific priority.
//! - The [`SchedulingPolicy::default_mappings()`] method to get an array of all default policies.
//!
//! These are used by the `gdt-cpus` crate to determine how to configure thread
//! priorities on macOS.

use crate::{ThreadPriority, platform::macos::utils::u32_to_qos_class_t};

/// Represents a macOS scheduling policy and its associated parameters.
///
/// On macOS, thread scheduling is largely managed by Quality of Service (QoS) classes,
/// which indicate the nature of the work being performed. For very high priority,
/// real-time scheduling can also be used with absolute priority values.
///
/// See Apple's documentation on "Energy Efficiency Guide for Mac Apps" and
/// "Prioritizing Work at the Task Level" for more details on QoS.
/// For absolute priorities, refer to `pthread_setschedparam` and `TH_POLICY_FIFO` or `TH_POLICY_RR`.
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SchedulingPolicy {
    /// Uses a Quality of Service (QoS) class and a relative priority within that class.
    ///
    /// QoS classes help the system manage resources like CPU time and power based on
    /// the importance and nature of the work.
    QoS {
        /// The QoS class (e.g., `QOS_CLASS_USER_INTERACTIVE`, `QOS_CLASS_BACKGROUND`).
        class: u32,
        /// The relative priority within the QoS class. This is an offset from the
        /// default priority of the QoS class. Negative values indicate lower priority
        /// relative to the class's default, positive values higher.
        /// For most gdt-cpus use cases, this will be 0.
        relative_priority: libc::c_int,
    },
    /// Uses an absolute real-time priority (typically for `TH_POLICY_FIFO` or `TH_POLICY_RR`).
    ///
    /// These are used for the highest priority levels and require appropriate permissions.
    /// The range for these priorities on macOS is typically 1 through 47 for FIFO/RR,
    /// but system policies can affect this.
    Absolute {
        /// The absolute priority value. Higher values mean higher priority.
        priority: libc::c_int,
    },
}

impl SchedulingPolicy {
    /// Returns the default macOS `SchedulingPolicy` for a given [`ThreadPriority`].
    ///
    /// This function maps abstract `ThreadPriority` levels to concrete macOS
    /// scheduling parameters, primarily using QoS classes.
    ///
    /// # Examples
    /// ```
    /// # use gdt_cpus::SchedulingPolicy;
    /// # use gdt_cpus::ThreadPriority;
    /// let policy = SchedulingPolicy::default_for(ThreadPriority::Normal);
    /// // Expected: QoS User Initiated, relative priority 0
    /// match policy {
    ///     SchedulingPolicy::QoS { class, relative_priority } => {
    ///         assert_eq!(relative_priority, 0);
    ///     },
    ///     _ => panic!("Expected QoS policy for Normal priority"),
    /// }
    /// ```
    pub const fn default_for(priority: ThreadPriority) -> Self {
        match priority {
            ThreadPriority::Background => SchedulingPolicy::QoS {
                class: libc::qos_class_t::QOS_CLASS_BACKGROUND as u32,
                relative_priority: 0,
            },
            ThreadPriority::Lowest => SchedulingPolicy::QoS {
                class: libc::qos_class_t::QOS_CLASS_UTILITY as u32,
                relative_priority: 0,
            },
            ThreadPriority::BelowNormal => SchedulingPolicy::QoS {
                class: libc::qos_class_t::QOS_CLASS_DEFAULT as u32,
                relative_priority: 0,
            },
            ThreadPriority::Normal => SchedulingPolicy::QoS {
                class: libc::qos_class_t::QOS_CLASS_USER_INITIATED as u32,
                relative_priority: 0,
            },
            ThreadPriority::AboveNormal => SchedulingPolicy::QoS {
                class: libc::qos_class_t::QOS_CLASS_USER_INTERACTIVE as u32,
                relative_priority: 0,
            },
            // macOS real-time priorities (e.g., for pthread_setschedparam with THREAD_TIME_CONSTRAINT_POLICY)
            // are typically in a range like 1-47 or 1-63 depending on policy and system.
            ThreadPriority::Highest => SchedulingPolicy::Absolute { priority: 43 },
            ThreadPriority::TimeCritical => SchedulingPolicy::Absolute { priority: 47 },
        }
    }

    /// Provides a static array of default [`SchedulingPolicy`] mappings
    /// for all [`ThreadPriority`] variants on macOS.
    ///
    /// The order in the array corresponds to the order of variants in `ThreadPriority`.
    /// This is used by `affinity::get_scheduling_policies()` when no other
    /// mappings have been set.
    ///
    /// # Examples
    /// ```
    /// # use gdt_cpus::SchedulingPolicy;
    /// let mappings = SchedulingPolicy::default_mappings();
    /// assert_eq!(mappings.len(), 7);
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
        match self {
            SchedulingPolicy::QoS {
                class,
                relative_priority,
            } => {
                // Using direct constants for matching as qos_class_t is often a type alias for u32 or similar
                let qos_class_str = match u32_to_qos_class_t(*class) {
                    libc::qos_class_t::QOS_CLASS_USER_INTERACTIVE => "User Interactive",
                    libc::qos_class_t::QOS_CLASS_USER_INITIATED => "User Initiated",
                    libc::qos_class_t::QOS_CLASS_UTILITY => "Utility",
                    libc::qos_class_t::QOS_CLASS_BACKGROUND => "Background",
                    libc::qos_class_t::QOS_CLASS_DEFAULT => "Default",
                    _ => "Unknown", // Or handle more gracefully if qos_class_t has other unnamed consts
                };
                write!(
                    f,
                    "QoS Class: {}, Relative Priority: {}",
                    qos_class_str, relative_priority
                )
            }
            SchedulingPolicy::Absolute { priority } => {
                write!(f, "Absolute Priority: {}", priority)
            }
        }
    }
}
