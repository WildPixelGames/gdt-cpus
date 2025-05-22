//! Windows-specific thread affinity and priority management.
//!
//! This module provides functions to control the CPU affinity and priority
//! of the current thread on Windows systems. It uses Windows API calls like
//! `SetThreadAffinityMask` and `SetThreadPriority`.

use log::{debug, error};
use windows::Win32::Foundation::{GetLastError, HANDLE};
use windows::Win32::System::Threading::{
    GetCurrentThread, SetThreadAffinityMask, SetThreadPriority, THREAD_PRIORITY,
};

use crate::{Error, Result, ThreadPriority, get_scheduling_policies};

/// Pins the current thread to a specific logical core ID on Windows.
///
/// This function uses `SetThreadAffinityMask` to restrict the execution of the
/// current thread to the specified logical core.
///
/// # Arguments
///
/// * `logical_core_id`: The zero-based ID of the logical core to pin the thread to.
///   This ID must be less than the number of bits in `usize` (e.g., < 64 on a
///   64-bit system).
///
/// # Returns
///
/// A `Result<()>` which is `Ok(())` on success, or an `Error` if:
/// - `logical_core_id` is too large (see `Error::InvalidCoreId`).
/// - `SetThreadAffinityMask` fails (see `Error::Affinity`).
///
/// # Remarks
///
/// For systems with more than 64 logical processors, `SetThreadAffinityMask` is
/// insufficient. Such systems require the use of `SetThreadGroupAffinity` and related
/// processor group APIs, which are not currently implemented by this function.
pub(crate) fn pin_thread_to_core(logical_core_id: usize) -> Result<()> {
    debug!(
        "Attempting to pin thread to logical_core_id: {} on Windows.",
        logical_core_id
    );

    if logical_core_id >= (std::mem::size_of::<usize>() * 8) {
        error!(
            "Logical core ID {} is too large for SetThreadAffinityMask (max {}). Systems with >64 LPs require SetThreadGroupAffinity.",
            logical_core_id,
            (std::mem::size_of::<usize>() * 8) - 1
        );
        return Err(Error::InvalidCoreId(logical_core_id));
    }

    let affinity_mask: usize = 1usize << logical_core_id;
    let thread_handle: HANDLE = unsafe { GetCurrentThread() };
    let old_mask = unsafe { SetThreadAffinityMask(thread_handle, affinity_mask) };

    if old_mask == 0 {
        let error_code = unsafe { GetLastError() };
        error!(
            "SetThreadAffinityMask failed for logical_core_id {}. Error: {:?}",
            logical_core_id, error_code
        );
        Err(Error::Affinity(format!(
            "SetThreadAffinityMask failed with error code: {:?}",
            error_code
        )))
    } else {
        debug!(
            "Successfully pinned thread to logical_core_id: {} (old mask: {:#x}).",
            logical_core_id, old_mask
        );
        Ok(())
    }
}

/// Sets the priority of the current thread on Windows.
///
/// This function maps the abstract `ThreadPriority` enum to a Windows-specific
/// thread priority value using `SetThreadPriority`.
/// The mapping from `ThreadPriority` to Windows `THREAD_PRIORITY` values is
/// defined by the `get_scheduling_policies()` function.
///
/// # Arguments
///
/// * `priority`: The desired [`ThreadPriority`] for the current thread.
///
/// # Returns
///
/// A `Result<()>` which is `Ok(())` on success, or an `Error` if:
/// - The `priority` variant is invalid or cannot be mapped (see `Error::Affinity`).
/// - `SetThreadPriority` fails (see `Error::Affinity`).
pub(crate) fn set_thread_priority(priority: ThreadPriority) -> Result<()> {
    let priority_idx = priority as usize;
    let sched_policy = get_scheduling_policies().get(priority_idx).ok_or_else(|| {
        Error::Affinity(format!("Invalid ThreadPriority variant: {}", priority_idx))
    })?;

    debug!("Setting Windows thread priority: {}", sched_policy);

    let thread_handle = unsafe { GetCurrentThread() };
    let success_result =
        unsafe { SetThreadPriority(thread_handle, THREAD_PRIORITY(sched_policy.0)) };

    match success_result {
        Ok(_) => {
            debug!(
                "Successfully set Windows thread priority to {}.",
                sched_policy
            );
            Ok(())
        }
        Err(e) => {
            // GetLastError() might not be relevant here if .ok() already processed it.
            // The error `e` from `.ok()` is windows_core::Error.
            error!(
                "SetThreadPriority failed to set priority {}. Error: {:?}",
                sched_policy, e
            );
            Err(Error::Affinity(format!(
                "SetThreadPriority failed with error: {:?}",
                e
            )))
        }
    }
}
