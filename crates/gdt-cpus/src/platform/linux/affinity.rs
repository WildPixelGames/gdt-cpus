//! Linux-specific thread affinity and priority management.
//!
//! This module provides functions to control thread affinity and thread priority
//! (via `nice` values or real-time scheduling policies) on Linux systems.
//!
//! **Important Notes for Linux:**
//! - **Thread Affinity:** Uses `sched_setaffinity` to restrict the current thread
//!   to a set of logical cores specified by an [`AffinityMask`]. The mask indices
//!   are mapped to OS-level logical processor IDs.
//! - **Thread Priority:**
//!   - For standard priorities, this module adjusts the `nice` value of the thread using
//!     `setpriority`. Lower `nice` values (e.g., -20) mean higher priority.
//!   - For real-time priorities, it uses `sched_setscheduler` to apply policies like
//!     `SCHED_RR` or `SCHED_FIFO` with an absolute priority level.
//!   - Setting real-time priorities or negative `nice` values typically requires
//!     `CAP_SYS_NICE` capability or root privileges. The functions include fallback
//!     mechanisms to attempt setting a default `nice` value (0) if permission is denied
//!     for a higher priority.
//!
//! The main functions provided are [`set_thread_affinity`] and [`set_thread_priority`].

use libc::{SYS_gettid, c_int, syscall};
use log::{debug, error, warn};

use crate::{
    AffinityMask, Error, Result, SchedulingPolicy, ThreadPriority, get_scheduling_policies,
    platform::linux::scheduling_policy::NICE_NORMAL,
};

/// Sets the CPU affinity of the current thread on Linux.
///
/// This function restricts the current thread to execute only on the logical
/// cores specified in the [`AffinityMask`]. It uses `libc::sched_setaffinity`
/// to apply the affinity mask.
///
/// # Arguments
///
/// * `mask`: An [`AffinityMask`] specifying which logical cores the thread may run on.
///   Core indices in the mask are mapped to OS-level logical processor IDs.
///
/// # Returns
///
/// - `Ok(())` if the affinity was successfully set.
/// - `Error::Affinity` if the mask is empty, no valid cores could be added, or
///   `sched_setaffinity` fails.
/// - Errors from `crate::cpu_info()` if CPU information cannot be retrieved.
///
/// # Safety
///
/// This function uses `unsafe` blocks for FFI calls to `libc::sched_setaffinity`,
/// `libc::CPU_ZERO`, and `libc::CPU_SET`. These are standard Linux system calls
/// and are safe when provided with valid arguments (a valid `cpu_set_t` and a TID of 0
/// for the current thread).
pub(crate) fn set_thread_affinity(mask: &AffinityMask) -> Result<()> {
    if mask.is_empty() {
        error!("Cannot set thread affinity with an empty mask.");
        return Err(Error::Affinity(
            "Cannot set thread affinity with an empty mask".to_string(),
        ));
    }

    debug!(
        "Attempting to set thread affinity on Linux with mask: {:?}",
        mask
    );

    let cpu_info = crate::cpu_info()?;
    let logical_processor_ids = cpu_info.logical_processor_ids();
    let max_cpus = libc::CPU_SETSIZE as usize;

    // SAFETY: Zero-initializes the cpu_set_t structure.
    // cpu_set_t is POD without non-zeroable invariants; zeroing yields a valid struct.
    let mut cpuset: libc::cpu_set_t = unsafe { std::mem::zeroed() };
    // SAFETY: CPU_ZERO is safe to call with a valid cpu_set_t pointer.
    unsafe {
        libc::CPU_ZERO(&mut cpuset);
    }

    // Set all cores in the mask
    let mut cores_set = 0;
    for core_idx in mask.iter() {
        if let Some(&os_core_id) = logical_processor_ids.get(core_idx) {
            if os_core_id < max_cpus {
                // SAFETY: CPU_SET is safe with a valid cpu_set_t pointer and valid CPU index.
                unsafe {
                    libc::CPU_SET(os_core_id, &mut cpuset);
                }

                cores_set += 1;

                debug!(
                    "Added OS core {} (index {}) to cpuset.",
                    os_core_id, core_idx
                );
            } else {
                warn!(
                    "OS core ID {} exceeds CPU_SETSIZE {}, skipping.",
                    os_core_id, max_cpus
                );
            }
        } else {
            warn!(
                "Core index {} not found in logical_processor_ids, skipping.",
                core_idx
            );
        }
    }

    if cores_set == 0 {
        error!("No valid cores could be added to the affinity mask.");
        return Err(Error::Affinity(
            "No valid cores could be added to the affinity mask".to_string(),
        ));
    }

    debug!("Setting affinity to {} cores.", cores_set);

    // SAFETY: sched_setaffinity is a system call that sets the CPU affinity for the calling thread.
    // pid == 0 means the calling thread, and the size of the cpu_set_t is passed.
    let res =
        unsafe { libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &cpuset) };

    if res == -1 {
        let err = std::io::Error::last_os_error();
        error!("sched_setaffinity failed for mask: {}", err);
        Err(Error::Affinity(format!(
            "sched_setaffinity failed: {}",
            err
        )))
    } else {
        debug!("Successfully set thread affinity to {} cores.", cores_set);
        Ok(())
    }
}

/// Sets the `nice` value for the current thread on Linux.
///
/// A lower `nice` value means higher priority. Standard users can only increase
/// the `nice` value (lower priority) or set it back to 0 from a positive value.
/// Setting a negative `nice` value (higher priority) typically requires
/// `CAP_SYS_NICE` capability or root privileges.
///
/// This function attempts to set the given `nice_value`. If `setpriority` fails
/// with `EPERM` (Permission Denied) when trying to set a negative `nice` value,
/// it will attempt to fall back to setting `nice(0)` (i.e., `NICE_NORMAL`).
///
/// # Arguments
///
/// * `nice_value`: The desired `nice` value (typically -20 to 19).
///
/// # Returns
///
/// - `Ok(())` if the `nice` value was successfully set (or successfully fell back to `nice(0)`).
/// - `Error::PermissionDenied` if setting the `nice` value failed due to permissions
///   (and fallback also failed or was not applicable).
/// - `Error::NotFound` if the thread ID (TID) could not be found.
/// - `Error::InvalidParameter` if the `nice_value` is invalid for `setpriority`.
/// - `Error::SystemCall` for other `setpriority` or `gettid` errors.
///
/// # Safety
///
/// This function uses `unsafe` blocks for `syscall(SYS_gettid)` to get the current
/// thread ID and `libc::setpriority` to set the nice value. These are standard
/// Linux system calls. `SYS_gettid` is the correct way to get the TID for `setpriority`
/// when targeting a specific thread. `setpriority` is safe if the TID is valid and
/// `nice_value` is within system-accepted bounds.
fn set_thread_nice_value(nice_value: c_int) -> Result<()> {
    // SAFETY: syscall(SYS_gettid) is the standard way to get the current thread ID on Linux.
    // Returns -1 on error.
    let tid = unsafe { syscall(SYS_gettid) };

    if tid <= 0 {
        let err = std::io::Error::last_os_error();
        error!("Failed to get current thread ID (gettid): {}", err);
        return Err(Error::SystemCall(format!(
            "Failed to get thread ID via gettid(): {}",
            err
        )));
    }

    let tid = tid as libc::id_t;

    debug!(
        "Setting nice value {} for SCHED_OTHER for current thread (TID {}).",
        nice_value, tid
    );

    // SAFETY: setpriority is used to set the nice value for a specific thread (using tid).
    // tid is a valid thread ID, and nice_value is the expected c_int value.
    let res = unsafe { libc::setpriority(libc::PRIO_PROCESS, tid, nice_value) };

    if res == -1 {
        let err = std::io::Error::last_os_error();
        error!(
            "setpriority failed to set nice value {} for TID {}. Error: {}",
            nice_value, tid, err
        );

        match err.raw_os_error() {
            Some(libc::EPERM) => {
                // If we get a permission denied error for a negative nice value, try nice(0)
                if nice_value < NICE_NORMAL {
                    warn!(
                        "Permission denied to set nice value {} for TID {}. Falling back to nice({}).",
                        nice_value, tid, NICE_NORMAL
                    );
                    // Recursive call to set nice(0) - but only one level deep to avoid loops
                    // or directly call setpriority for nice(0)
                    let fallback_res =
                        unsafe { libc::setpriority(libc::PRIO_PROCESS, tid, NICE_NORMAL) };
                    if fallback_res == 0 {
                        debug!(
                            "Successfully fell back to nice({}) for TID {}.",
                            NICE_NORMAL, tid
                        );
                        return Ok(());
                    } else {
                        let fallback_err = std::io::Error::last_os_error();
                        error!(
                            "Fallback to nice({}) also failed for TID {}. Error: {}",
                            NICE_NORMAL, tid, fallback_err
                        );
                        // We return the original PermissionDenied error, as that was the original intent
                        return Err(Error::PermissionDenied(format!(
                            "Setting nice value {} for TID {}: {}",
                            nice_value, tid, err,
                        )));
                    }
                }
                Err(Error::PermissionDenied(format!(
                    "Setting nice value {} for TID {}: {}",
                    nice_value, tid, err
                )))
            }
            Some(libc::ESRCH) => Err(Error::NotFound(format!(
                "Thread with TID {} not found for setpriority: {}",
                tid, err
            ))),
            Some(libc::EINVAL) => Err(Error::InvalidParameter(format!(
                "Invalid nice value {} for setpriority",
                nice_value
            ))),
            _ => Err(Error::SystemCall(format!(
                "setpriority failed for nice value {} for TID {}: {}",
                nice_value, tid, err
            ))),
        }
    } else {
        debug!(
            "Successfully set nice value {} for TID {}.",
            nice_value, tid
        );
        Ok(())
    }
}

/// Sets the priority of the current thread on Linux.
///
/// This function maps the abstract [`ThreadPriority`] to Linux-specific scheduling parameters:
///
/// 1.  **If `SchedulingPolicy::Nice { value }` (for standard priorities):**
///     It calls the internal `set_thread_nice_value()` helper to adjust the thread's `nice`
///     value using `libc::setpriority`. Threads under `SCHED_OTHER` (the default policy)
///     are prioritized based on their `nice` value (lower is higher priority).
///     Setting negative `nice` values usually requires `CAP_SYS_NICE` or root privileges.
///     If permission is denied, it attempts to fall back to `nice(0)`.
///
/// 2.  **If `SchedulingPolicy::RealTime { policy, priority }` (for real-time priorities):**
///     It uses `libc::sched_setscheduler()` to apply a real-time scheduling policy
///     (e.g., `SCHED_RR`, `SCHED_FIFO`) and an absolute priority level. This requires
///     `CAP_SYS_NICE` or root privileges. If `sched_setscheduler` fails with `EPERM`
///     (Permission Denied), it attempts to fall back to setting `nice(0)` using
///     `set_thread_nice_value(NICE_NORMAL)`.
///
/// # Arguments
///
/// * `priority`: A [`ThreadPriority`] enum variant indicating the desired priority level.
///
/// # Returns
///
/// - `Ok(())` if the priority was successfully set (or successfully fell back to `nice(0)`).
/// - `Error::PermissionDenied` if setting the priority failed due to permissions and fallback also failed.
/// - `Error::SystemCall` for other system call errors.
/// - `Error::InvalidParameter` if scheduling parameters are invalid.
/// - `Error::NotFound` if the TID is not found during `set_thread_nice_value`.
///
/// # Safety
///
/// This function uses `unsafe` blocks for `libc::sched_setscheduler()` and relies on
/// the internal (also `unsafe`) `set_thread_nice_value()`. These are standard Linux
/// system calls. `sched_setscheduler` is safe if the TID (0 for current thread),
/// policy, and `sched_param` are valid. Permissions are the primary concern, handled by fallbacks.
pub(crate) fn set_thread_priority(priority: ThreadPriority) -> Result<()> {
    // 1. Get scheduling policy
    let priority_idx = priority as usize;
    let sched_policy = get_scheduling_policies().get(priority_idx).ok_or_else(|| {
        Error::Affinity(format!("Invalid ThreadPriority variant: {}", priority_idx))
    })?;

    // 2. Apply the policy
    match *sched_policy {
        SchedulingPolicy::Nice { value } => {
            set_thread_nice_value(value) // Call internal function
        }
        SchedulingPolicy::Absolute { priority } => {
            debug!(
                "Attempting to set thread scheduling policy to SCHED_RR with priority {} (per-thread).",
                priority
            );

            // SAFETY: These functions are safe to call; SCHED_RR is a valid policy.
            // They return -1 on error (e.g., if the policy is not supported, which is unlikely for SCHED_RR).
            let (rt_min, rt_max) = unsafe {
                (
                    libc::sched_get_priority_min(libc::SCHED_RR),
                    libc::sched_get_priority_max(libc::SCHED_RR),
                )
            };

            if rt_min == -1 || rt_max == -1 {
                let err = std::io::Error::last_os_error();
                error!(
                    "Failed to get SCHED_RR priority range (min: {}, max: {}). Error: {}",
                    rt_min, rt_max, err
                );
                return Err(Error::SystemCall(format!(
                    "Failed to get SCHED_RR priority range: {}",
                    err
                )));
            }

            // Validate the absolute priority against the min and max values.
            assert!(
                priority >= rt_min && priority <= rt_max,
                "Absolute priority {} is out of range [{}, {}] for SCHED_RR.",
                priority,
                rt_min,
                rt_max
            );

            // SAFETY: pthread_self() always returns a valid handle to the current thread.
            let current_thread = unsafe { libc::pthread_self() };

            // SAFETY: sched_param is a POD structure; zeroing it is safe.
            // sched_priority is then set to the validated value absolute_priority.
            let mut param: libc::sched_param = unsafe { std::mem::zeroed() };
            param.sched_priority = priority;

            // SAFETY: Sets the SCHED_RR policy for the current thread.
            // current_thread is valid, param.sched_priority is within the range [rt_min, rt_max].
            let res =
                unsafe { libc::pthread_setschedparam(current_thread, libc::SCHED_RR, &param) };

            if res != 0 {
                let err = std::io::Error::from_raw_os_error(res); // res is errno in this case
                error!(
                    "pthread_setschedparam failed for SCHED_RR with priority {}. Error code: {} ({})",
                    priority, res, err
                );

                match res {
                    libc::EPERM => {
                        warn!(
                            "Permission denied for SCHED_RR with priority {}. Falling back to SCHED_OTHER with nice({}).",
                            priority, NICE_NORMAL
                        );

                        // Try to set nice(0) as a fallback
                        // We use the internal function that already handles the fallback logic for nice
                        match set_thread_nice_value(NICE_NORMAL) {
                            Ok(_) => {
                                debug!(
                                    "Successfully fell back to SCHED_OTHER with nice({}).",
                                    NICE_NORMAL
                                );
                                Ok(())
                            }
                            Err(fallback_err) => {
                                error!(
                                    "Fallback to SCHED_OTHER with nice({}) also failed. Original EPERM for SCHED_RR stands. Fallback error: {:?}",
                                    NICE_NORMAL, fallback_err
                                );
                                // Return PermissionDenied for the original SCHED_RR attempt
                                Err(Error::PermissionDenied(format!(
                                    "Setting SCHED_RR with priority {}: {}",
                                    priority, err,
                                )))
                            }
                        }
                    }
                    libc::EINVAL => Err(Error::InvalidParameter(format!(
                        "Invalid parameters for SCHED_RR: priority={}, policy=SCHED_RR. Error: {}",
                        priority, err
                    ))),
                    libc::ESRCH => Err(Error::NotFound(format!(
                        "Thread not found for pthread_setschedparam. Error: {}",
                        err
                    ))),
                    _ => Err(Error::SystemCall(format!(
                        "pthread_setschedparam failed for SCHED_RR with priority {}. Error: {}",
                        priority, err
                    ))),
                }
            } else {
                debug!(
                    "Successfully set thread scheduling policy to SCHED_RR with priority {}.",
                    priority
                );
                Ok(())
            }
        }
    }
}
