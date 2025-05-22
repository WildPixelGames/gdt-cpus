//! macOS-specific thread affinity and priority management.
//!
//! This module provides functions to control thread affinity (pinning to a core)
//! and thread priority (via Quality of Service classes or absolute scheduling priorities)
//! on macOS systems.
//!
//! **Important Notes for macOS:**
//! - **Thread Affinity (Pinning):**
//!   - On **Apple Silicon (aarch64)**, explicitly pinning threads to cores using
//!     `thread_policy_set` with `THREAD_AFFINITY_POLICY` is **not supported** by the kernel
//!     and will always fail. The system dynamically manages core assignment based on QoS,
//!     power, and thermal considerations.
//!   - On **Intel-based Macs (x86_64)**, thread pinning is technically possible via
//!     `thread_policy_set`. However, its effectiveness can be limited by the system's
//!     Quality of Service (QoS) settings and power management. It should be used judiciously.
//! - **Thread Priority:**
//!   - The primary mechanism for influencing thread scheduling, energy use, and core selection
//!     (P-cores vs E-cores on Apple Silicon) is through **Quality of Service (QoS) classes**
//!     set via `pthread_set_qos_class_self_np`.
//!   - Alternatively, for more direct (but potentially less system-friendly) control,
//!     POSIX real-time scheduling policies like `SCHED_RR` can be used via `pthread_setschedparam`,
//!     which effectively bypasses the QoS system for that thread.
//!
//! The main functions provided are [`pin_thread_to_core`] and [`set_thread_priority`].

use log::{debug, error};

use crate::{
    Error, Result, ThreadPriority, get_scheduling_policies,
    platform::macos::{scheduling_policy::SchedulingPolicy, utils::u32_to_qos_class_t},
};

/// Attempts to pin the current thread to a specific logical core ID on macOS.
///
/// The `logical_core_id` is the OS-level identifier for a logical processor.
///
/// # Behavior per Architecture:
/// - **Apple Silicon (`aarch64`):** This function will **always return `Error::Unsupported`**.
///   Explicit thread pinning is not supported by the macOS kernel on Apple Silicon.
///   Thread placement is managed by the system based on QoS, power, and thermal state.
///   Use [`set_thread_priority`] to influence core selection indirectly via QoS classes.
/// - **Intel-based Macs (`x86_64`):** This function uses `mach_sys::thread_policy_set` with
///   `THREAD_AFFINITY_POLICY`. While technically possible, the actual enforcement of this
///   pinning can be influenced or overridden by the system's Quality of Service (QoS)
///   settings and power management policies. It should be considered a hint to the scheduler.
///
/// # Arguments
///
/// * `logical_core_id`: The OS-level ID of the logical core to pin the current thread to.
///
/// # Returns
///
/// - `Ok(())` if the `thread_policy_set` call was successful (on x86_64, implies a hint was set).
/// - `Error::Unsupported` on Apple Silicon, or if `thread_policy_set` returns `KERN_NOT_SUPPORTED` on x86_64.
/// - `Error::Affinity` for other `thread_policy_set` failures on x86_64.
/// - `Error::SystemCall` if `mach_thread_self` fails on x86_64.
#[cfg(target_arch = "aarch64")]
pub fn pin_thread_to_core(_logical_core_id: usize) -> Result<()> {
    Err(Error::Unsupported(
        "Thread affinity (pinning) is not supported on Apple Silicon, use `set_thread_priority`"
            .into(),
    ))
}

/// Attempts to pin the current thread to a specific logical core ID on macOS.
///
/// The `logical_core_id` is the OS-level identifier for a logical processor.
///
/// # Behavior per Architecture:
/// - **Apple Silicon (`aarch64`):** This function will **always return `Error::Unsupported`**.
///   Explicit thread pinning is not supported by the macOS kernel on Apple Silicon.
///   Thread placement is managed by the system based on QoS, power, and thermal state.
///   Use [`set_thread_priority`] to influence core selection indirectly via QoS classes.
/// - **Intel-based Macs (`x86_64`):** This function uses `mach_sys::thread_policy_set` with
///   `THREAD_AFFINITY_POLICY`. While technically possible, the actual enforcement of this
///   pinning can be influenced or overridden by the system's Quality of Service (QoS)
///   settings and power management policies. It should be considered a hint to the scheduler.
///
/// # Arguments
///
/// * `logical_core_id`: The OS-level ID of the logical core to pin the current thread to.
///
/// # Returns
///
/// - `Ok(())` if the `thread_policy_set` call was successful (on x86_64, implies a hint was set).
/// - `Error::Unsupported` on Apple Silicon, or if `thread_policy_set` returns `KERN_NOT_SUPPORTED` on x86_64.
/// - `Error::Affinity` for other `thread_policy_set` failures on x86_64.
/// - `Error::SystemCall` if `mach_thread_self` fails on x86_64.
#[cfg(target_arch = "x86_64")]
pub fn pin_thread_to_core(logical_core_id: usize) -> Result<()> {
    use mach_sys::kern_return::{KERN_NOT_SUPPORTED, KERN_RETURN, KERN_SUCCESS};
    use mach_sys::mach_init::mach_thread_self;
    use mach_sys::mach_types::thread_t;
    use mach_sys::port::MACH_PORT_NULL;
    use mach_sys::thread_policy::{
        THREAD_AFFINITY_POLICY, THREAD_AFFINITY_POLICY_COUNT, thread_affinity_policy_data_t,
        thread_policy_set, thread_policy_t,
    };
    use mach_sys::vm_types::integer_t;

    debug!(
        "Attempting to pin thread to logical_core_id: {} on macOS.",
        logical_core_id
    );
    unsafe {
        debug!(
            "Calling thread_policy_set with logical_core_id: {}",
            logical_core_id
        );
        let thread_port: thread_t = mach_thread_self();
        debug!(" thread_port: {:?}", thread_port);
        if thread_port == MACH_PORT_NULL {
            error!("Failed to get current mach thread port (thread_self returned MACH_PORT_NULL).");
            return Err(Error::SystemCall(
                "Failed to get current mach thread port (thread_self)".to_string(),
            ));
        }

        let mut policy_data: thread_affinity_policy_data_t = std::mem::zeroed();
        policy_data.affinity_tag = logical_core_id as integer_t;

        let policy_count = THREAD_AFFINITY_POLICY_COUNT;
        debug!(" policy_count: {}", policy_count);

        let kret = thread_policy_set(
            thread_port,
            THREAD_AFFINITY_POLICY,
            &policy_data as *const _ as thread_policy_t,
            policy_count,
        );

        let kreturn = KERN_RETURN::from_int(kret);

        debug!(" thread_policy_set returned: {}", kreturn);

        match kret {
            KERN_SUCCESS => {
                debug!(
                    "Successfully called thread_policy_set for logical_core_id: {}. OS behavior may vary.",
                    logical_core_id
                );
                Ok(())
            }
            KERN_NOT_SUPPORTED => {
                error!("thread_policy_set failed with KERN_NOT_SUPPORTED.");
                Err(Error::Unsupported(
                    "thread_policy_set failed with KERN_NOT_SUPPORTED".to_string(),
                ))
            }
            _ => {
                error!(
                    "thread_policy_set failed with kernel error: {}. This is often a hint and may not be strictly enforced by the OS, especially if it conflicts with QoS or power management.",
                    kreturn
                );
                Err(Error::Affinity(format!(
                    "thread_policy_set (THREAD_AFFINITY_POLICY) failed with kernel error: {}. See system logs for more details.",
                    kreturn
                )))
            }
        }
    }
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
/// # Panics
///
/// Panics if `SchedulingPolicy::Absolute` is used and the associated `priority` value
/// is outside the valid range for `SCHED_RR` on the system.
pub fn set_thread_priority(priority: ThreadPriority) -> Result<()> {
    let priority_idx = priority as usize;
    let sched_policy = get_scheduling_policies().get(priority_idx).ok_or_else(|| {
        Error::Affinity(format!("Invalid ThreadPriority variant: {}", priority_idx))
    })?;

    match *sched_policy {
        SchedulingPolicy::QoS {
            class,
            relative_priority,
        } => {
            let class = u32_to_qos_class_t(class);
            let qos_class_str = match class {
                libc::qos_class_t::QOS_CLASS_USER_INTERACTIVE => "User Interactive",
                libc::qos_class_t::QOS_CLASS_USER_INITIATED => "User Initiated",
                libc::qos_class_t::QOS_CLASS_UTILITY => "Utility",
                libc::qos_class_t::QOS_CLASS_BACKGROUND => "Background",
                libc::qos_class_t::QOS_CLASS_DEFAULT => "Default",
                _ => "Unknown",
            };

            debug!(
                "Setting thread QoS class to {} with relative priority {}.",
                qos_class_str, relative_priority
            );

            // SAFETY: Sets the QoS class for the current thread.
            // qos_class and relative_priority come from a trusted static map.
            // Both are valid values per macOS docs.
            let err = unsafe { libc::pthread_set_qos_class_self_np(class, relative_priority) };

            if err != 0 {
                error!(
                    "Failed to set thread QoS class {} with relative priority {}. Error code: {}",
                    qos_class_str,
                    relative_priority,
                    std::io::Error::from_raw_os_error(err)
                );
                Err(Error::Affinity(format!(
                    "pthread_set_qos_class_self_np failed with error code: {}",
                    std::io::Error::from_raw_os_error(err)
                )))
            } else {
                debug!("Successfully set QoS class for current thread.");
                Ok(())
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

            // Validate the absolute priority against the min and max values.
            assert!(
                priority >= min && priority <= max,
                "Absolute priority {} is out of range [{}, {}] for SCHED_RR.",
                priority,
                min,
                max
            );

            // SAFETY: Zero-initializes the sched_param structure.
            // sched_param is POD without non-zeroable invariants; zeroing yields a valid struct.
            let mut param: libc::sched_param = unsafe { std::mem::zeroed() };
            param.sched_priority = priority;

            debug!(
                "Setting thread scheduling policy to SCHED_RR with absolute priority {}.",
                priority
            );

            // SAFETY: Sets the scheduling policy and priority for the current thread.
            // current_thread from pthread_self is valid, param has valid priority in [min, max],
            // and SCHED_RR is supported on this OS.
            let err =
                unsafe { libc::pthread_setschedparam(current_thread, libc::SCHED_RR, &param) };

            if err != 0 {
                error!(
                    "Failed to set thread scheduling policy SCHED_RR with absolute priority {}. Error code: {}",
                    priority,
                    std::io::Error::from_raw_os_error(err)
                );
                Err(Error::Affinity(format!(
                    "pthread_setschedparam failed with error code: {}",
                    std::io::Error::from_raw_os_error(err)
                )))
            } else {
                debug!("Successfully set thread scheduling policy.");
                Ok(())
            }
        }
    }
}
