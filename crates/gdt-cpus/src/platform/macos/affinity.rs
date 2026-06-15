//! macOS-specific thread affinity and priority management.
//!
//! This module provides functions to control thread affinity (pinning to a core)
//! and thread priority (via Quality of Service classes or absolute scheduling priorities)
//! on macOS systems.
//!
//! **Important Notes for macOS (Apple Silicon only):**
//! - **Thread Affinity (Pinning):**
//!   - Explicitly pinning threads to cores using `thread_policy_set` with
//!     `THREAD_AFFINITY_POLICY` is **not supported** by the kernel on Apple Silicon
//!     and will always fail. The system dynamically manages core assignment based on
//!     QoS, power, and thermal considerations.
//! - **Thread Priority:**
//!   - The primary mechanism for influencing thread scheduling, energy use, and core selection
//!     (P-cores vs E-cores on Apple Silicon) is through **Quality of Service (QoS) classes**
//!     set via `pthread_set_qos_class_self_np`. Every level except `TimeCritical` uses QoS.
//!   - `TimeCritical` uses `pthread_setschedparam(SCHED_RR, 47)` - fixed priority, no timeshare
//!     decay, no privileges required. Per `qos.h` this PERMANENTLY opts the thread out of the
//!     QoS system (later QoS calls return EPERM); the QoS arm below detects that and falls back
//!     to legacy `SCHED_OTHER`, so re-prioritizing an opted-out thread still works.
//!
//! The main function provided is [`set_thread_priority`].

use crate::{
    AppliedPriority, Error, Grant, Mechanism, MechanismPolicy, QosClass, Result, ThreadPriority,
    platform::macos::scheduling_policy::SchedulingPolicy,
};

/// Total map from the crate's stable [`QosClass`] ordinal to the darwin
/// `qos_class_t` - no `UNSPECIFIED` fallback, unlike re-decoding a raw `u32`.
fn qos_class_t_of(c: QosClass) -> libc::qos_class_t {
    match c {
        QosClass::Background => libc::qos_class_t::QOS_CLASS_BACKGROUND,
        QosClass::Utility => libc::qos_class_t::QOS_CLASS_UTILITY,
        QosClass::Default => libc::qos_class_t::QOS_CLASS_DEFAULT,
        QosClass::UserInitiated => libc::qos_class_t::QOS_CLASS_USER_INITIATED,
        QosClass::UserInteractive => libc::qos_class_t::QOS_CLASS_USER_INTERACTIVE,
    }
}

/// Saturating cast of an OS scheduling value into `Mechanism.value` (`i8`).
/// Every shipping value fits (SCHED_RR <= 47, SCHED_OTHER band ~31), but a
/// surprising OS max must not silently wrap the reported diagnostic.
fn band_i8(v: libc::c_int) -> i8 {
    v.clamp(i8::MIN as libc::c_int, i8::MAX as libc::c_int) as i8
}

pub(crate) fn promote_thread_to_realtime() -> Result<AppliedPriority> {
    match SchedulingPolicy::default_for(ThreadPriority::TimeCritical) {
        SchedulingPolicy::Absolute { priority } => {
            set_sched_rr(ThreadPriority::TimeCritical, priority)
        }
        SchedulingPolicy::QoS { .. } => Err(Error::Unsupported(
            "TimeCritical does not map to SCHED_RR on this macOS target".to_string(),
        )),
    }
}

fn set_sched_rr(requested: ThreadPriority, priority: libc::c_int) -> Result<AppliedPriority> {
    let current_thread = unsafe { libc::pthread_self() };

    let (min, max) = unsafe {
        (
            libc::sched_get_priority_min(libc::SCHED_RR),
            libc::sched_get_priority_max(libc::SCHED_RR),
        )
    };

    let priority = priority.clamp(min, max);

    let mut param: libc::sched_param = unsafe { std::mem::zeroed() };

    param.sched_priority = priority;

    let err = unsafe { libc::pthread_setschedparam(current_thread, libc::SCHED_RR, &param) };

    if err != 0 {
        if err == libc::EPERM {
            return Err(Error::PermissionDenied(format!(
                "SCHED_RR priority {} denied",
                priority
            )));
        }

        return Err(Error::Affinity(format!(
            "pthread_setschedparam failed with error code: {}",
            std::io::Error::from_raw_os_error(err)
        )));
    }

    Ok(AppliedPriority::new(
        requested,
        requested,
        Grant::Realtime,
        Mechanism {
            policy: MechanismPolicy::SchedRr,
            value: band_i8(priority),
        },
    ))
}

/// Sets the priority of the current thread on macOS.
///
/// This function adapts its behavior based on the `SchedulingPolicy` associated with the
/// provided `ThreadPriority` (obtained via `get_scheduling_policies()`):
///
/// 1.  **If `SchedulingPolicy::QoS`:**
///     Uses `libc::pthread_set_qos_class_self_np()` to set the Quality of Service (QoS)
///     class (e.g., User Interactive, Utility) and a relative priority within that class.
///     This is the **recommended method** for influencing thread scheduling, energy
///     consumption, and core selection (P-cores vs. E-cores on Apple Silicon) on macOS,
///     as it allows the system to make optimal decisions.
///
/// 2.  **If `SchedulingPolicy::Absolute` (maps to `SCHED_RR`):**
///     Uses `libc::pthread_setschedparam()` to set a POSIX real-time scheduling policy
///     (`SCHED_RR`) and an absolute priority level. This gives more direct control but
///     effectively opts the thread out of the system's QoS management, which might lead
///     to suboptimal system performance or energy use if not managed carefully.
///     The absolute priority value must be within the range allowed by `SCHED_RR`
///     (obtained via `sched_get_priority_min/max`).
///
/// # Arguments
///
/// * `priority`: A [`ThreadPriority`] enum variant indicating the desired priority level.
///
/// # Returns
///
/// - `Ok(())` if the priority was successfully set.
/// - `Error::Affinity` if `pthread_set_qos_class_self_np` or `pthread_setschedparam` fails,
///   or if an invalid `ThreadPriority` variant is provided.
///
/// Real-time levels denied with `EPERM` fall back to QoS `USER_INTERACTIVE`
/// (unprivileged processes still get a sane state).
pub fn set_thread_priority(priority: ThreadPriority) -> Result<AppliedPriority> {
    // The Absolute arm below shadows `priority` with the RR int, so capture the
    // requested level up front for the AppliedPriority report.
    let requested = priority;

    // Map the portable level to the macOS policy (table in scheduling_policy.rs).
    let sched_policy = SchedulingPolicy::default_for(priority);

    match sched_policy {
        SchedulingPolicy::QoS {
            class,
            relative_priority,
        } => {
            // `class` is a stable QosClass ordinal; convert to the darwin
            // qos_class_t for the syscall (a total map - no UNSPECIFIED fallback).
            let qos_class = qos_class_t_of(class);
            let qos_class_str = match class {
                QosClass::UserInteractive => "User Interactive",
                QosClass::UserInitiated => "User Initiated",
                QosClass::Utility => "Utility",
                QosClass::Background => "Background",
                QosClass::Default => "Default",
            };

            // SAFETY: Sets the QoS class for the current thread.
            // qos_class and relative_priority come from a trusted static map.
            // Both are valid values per macOS docs.
            let err = unsafe { libc::pthread_set_qos_class_self_np(qos_class, relative_priority) };

            if err != 0 {
                if err == libc::EPERM {
                    // NOTE(macos): per qos.h, a thread that ever used
                    // pthread_setschedparam (our TimeCritical) is PERMANENTLY
                    // opted out of QoS - every QoS call returns EPERM forever.
                    // Re-prioritizing such a thread must go through the legacy
                    // API: SCHED_OTHER with the level scaled into the user
                    // band (levels 0..6 linear over [min, max]; Normal lands
                    // on Darwin's default base 31 when the band is [15, 47]).
                    let (min, max) = unsafe {
                        (
                            libc::sched_get_priority_min(libc::SCHED_OTHER),
                            libc::sched_get_priority_max(libc::SCHED_OTHER),
                        )
                    };

                    // SAFETY: sched_param is POD; zeroing it is safe.
                    let mut param: libc::sched_param = unsafe { std::mem::zeroed() };

                    param.sched_priority = min + (max - min) * (priority as libc::c_int) / 6;

                    // SAFETY: pthread_self() is always valid; priority is
                    // within [min, max] by construction.
                    let fb = unsafe {
                        libc::pthread_setschedparam(libc::pthread_self(), libc::SCHED_OTHER, &param)
                    };

                    if fb == 0 {
                        return Ok(AppliedPriority::new(
                            priority,
                            priority,
                            Grant::Direct,
                            Mechanism {
                                policy: MechanismPolicy::SchedOther,
                                value: band_i8(param.sched_priority),
                            },
                        ));
                    }

                    return Err(Error::PermissionDenied(format!(
                        "QoS class {} denied and SCHED_OTHER fallback failed (error {})",
                        qos_class_str, fb
                    )));
                }

                Err(Error::Affinity(format!(
                    "pthread_set_qos_class_self_np failed with error code: {}",
                    std::io::Error::from_raw_os_error(err)
                )))
            } else {
                Ok(AppliedPriority::new(
                    priority,
                    priority,
                    Grant::Direct,
                    Mechanism {
                        policy: MechanismPolicy::Qos,
                        // The applied class is known directly (no re-decoding).
                        value: class as i8,
                    },
                ))
            }
        }
        SchedulingPolicy::Absolute { priority } => {
            // SAFETY: Fetches the current thread ID.
            // pthread_self always returns a valid thread handle for the calling thread.
            let current_thread = unsafe { libc::pthread_self() };

            // SAFETY: Fetches the minimum and maximum scheduling priority for SCHED_RR.
            // SCHED_RR is a valid scheduling policy constant; these calls cannot fail per POSIX spec.
            let (min, max) = unsafe {
                (
                    libc::sched_get_priority_min(libc::SCHED_RR),
                    libc::sched_get_priority_max(libc::SCHED_RR),
                )
            };

            // Clamp into the SCHED_RR range instead of erroring - best-effort,
            // like the Linux cascade. On shipping Apple Silicon max == 47, so the
            // requested 47 passes unchanged; a future/sandboxed OS reporting a
            // lower max degrades gracefully (the applied value is reported in
            // Mechanism) rather than returning InvalidParameter.
            let priority = priority.clamp(min, max);

            // SAFETY: Zero-initializes the sched_param structure.
            // sched_param is POD without non-zeroable invariants; zeroing yields a valid struct.
            let mut param: libc::sched_param = unsafe { std::mem::zeroed() };

            param.sched_priority = priority;

            // SAFETY: Sets the scheduling policy and priority for the current thread.
            // current_thread from pthread_self is valid, param has valid priority in [min, max],
            // and SCHED_RR is supported on this OS.
            let err =
                unsafe { libc::pthread_setschedparam(current_thread, libc::SCHED_RR, &param) };

            if err != 0 {
                if err == libc::EPERM {
                    // Defensive only: libpthread's pthread_setschedparam has NO
                    // privilege check (verified in apple-oss-distributions
                    // source), so EPERM is not expected here - but sandboxed
                    // environments may surprise, and the QoS fallback is cheap.
                    let fb = unsafe {
                        libc::pthread_set_qos_class_self_np(
                            libc::qos_class_t::QOS_CLASS_USER_INTERACTIVE,
                            0,
                        )
                    };

                    if fb == 0 {
                        return Ok(AppliedPriority::new(
                            requested,
                            requested,
                            Grant::Direct,
                            Mechanism {
                                policy: MechanismPolicy::Qos,
                                value: QosClass::UserInteractive as i8,
                            },
                        ));
                    }

                    return Err(Error::PermissionDenied(format!(
                        "SCHED_RR priority {} denied and QoS fallback failed",
                        priority
                    )));
                }

                Err(Error::Affinity(format!(
                    "pthread_setschedparam failed with error code: {}",
                    std::io::Error::from_raw_os_error(err)
                )))
            } else {
                Ok(AppliedPriority::new(
                    requested,
                    requested,
                    Grant::Realtime,
                    Mechanism {
                        policy: MechanismPolicy::SchedRr,
                        value: band_i8(priority),
                    },
                ))
            }
        }
    }
}
