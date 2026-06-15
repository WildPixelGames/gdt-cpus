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
//!
//! These are used by the `gdt-cpus` crate to determine how to configure thread
//! priorities on macOS.

use crate::{QosClass, ThreadPriority};

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
        /// The QoS class as a stable [`QosClass`] ordinal, converted to the
        /// darwin `qos_class_t` at the call site (a total map - no re-decoding a
        /// raw `u32` through a parallel table that could yield `UNSPECIFIED`).
        class: QosClass,
        /// The relative priority within the QoS class. This is an offset from the
        /// default priority of the QoS class. Negative values indicate lower priority
        /// relative to the class's default, positive values higher.
        /// For most gdt-cpus use cases, this will be 0.
        relative_priority: libc::c_int,
    },
    /// Uses an absolute fixed priority via `pthread_setschedparam(SCHED_RR)`.
    ///
    /// Used only by `TimeCritical`. The user band caps at 47 (`MAXPRI_USER`);
    /// no privileges are required (unlike Linux), but per `qos.h` the call
    /// PERMANENTLY opts the thread out of the QoS system.
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
    /// NOTE(macos): the ladder is DELIBERATELY one notch "inflated" vs Apple's
    /// nominal tier names (BelowNormal -> DEFAULT, Normal -> USER_INITIATED).
    /// On Apple Silicon the QoS class is also the P/E-core routing lever: UTILITY
    /// and below get routed to E-cores, DEFAULT and above stay P-eligible. An
    /// E-core is a ~3-4x perf cliff, so mapping BelowNormal to UTILITY made
    /// "slightly less important" work run on a different machine (measured -
    /// too slow). Only Background/Lowest are meant to land on E-cores. Don't
    /// "fix" this to the textbook ladder.
    ///
    /// NOTE(macos): the top end is QoS too, except TimeCritical. The legacy
    /// API (pthread_setschedparam) caps at MAXPRI_USER = 47 - the SAME base
    /// priority QoS USER_INTERACTIVE already runs at - and per qos.h it
    /// PERMANENTLY opts the thread out of the QoS system (every later QoS
    /// call returns EPERM). The only thing it buys is FIXED priority: no
    /// timeshare decay while burning CPU. So AboveNormal/Highest stay inside
    /// the QoS world (UI -4 ≈ 43 / UI 0 = 47, timeshare), and only
    /// TimeCritical - the dedicated audio/haptics-feeder level whose threads
    /// never come back down - takes RR 47 fixed and walks through the
    /// one-way door knowingly.
    pub const fn default_for(priority: ThreadPriority) -> Self {
        match priority {
            ThreadPriority::Background => SchedulingPolicy::QoS {
                class: QosClass::Background,
                relative_priority: 0,
            },
            ThreadPriority::Lowest => SchedulingPolicy::QoS {
                class: QosClass::Utility,
                relative_priority: 0,
            },
            ThreadPriority::BelowNormal => SchedulingPolicy::QoS {
                class: QosClass::Default,
                relative_priority: 0,
            },
            ThreadPriority::Normal => SchedulingPolicy::QoS {
                class: QosClass::UserInitiated,
                relative_priority: 0,
            },
            ThreadPriority::AboveNormal => SchedulingPolicy::QoS {
                class: QosClass::UserInteractive,
                relative_priority: -4,
            },
            ThreadPriority::Highest => SchedulingPolicy::QoS {
                class: QosClass::UserInteractive,
                relative_priority: 0,
            },
            ThreadPriority::TimeCritical => SchedulingPolicy::Absolute { priority: 47 },
        }
    }
}

impl std::fmt::Display for SchedulingPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchedulingPolicy::QoS {
                class,
                relative_priority,
            } => {
                let qos_class_str = match class {
                    QosClass::UserInteractive => "User Interactive",
                    QosClass::UserInitiated => "User Initiated",
                    QosClass::Utility => "Utility",
                    QosClass::Background => "Background",
                    QosClass::Default => "Default",
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
