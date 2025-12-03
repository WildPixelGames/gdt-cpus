//! Windows-specific thread affinity and priority management.
//!
//! This module provides functions to control the CPU affinity and priority
//! of the current thread on Windows systems. It uses Windows API calls like
//! `SetThreadAffinityMask` and `SetThreadPriority`.

use log::{debug, error};
use windows::Win32::{
    Foundation::{GetLastError, HANDLE},
    System::Threading::{
        GetCurrentThread, SetThreadAffinityMask, SetThreadPriority, THREAD_PRIORITY,
    },
};

use crate::{AffinityMask, Error, Result, ThreadPriority, get_scheduling_policies};

/// Sets the CPU affinity of the current thread on Windows.
///
/// This function uses `SetThreadAffinityMask` to restrict the execution of the
/// current thread to the logical cores specified in the [`AffinityMask`].
///
/// # Arguments
///
/// * `mask`: An [`AffinityMask`] specifying which logical cores the thread may run on.
///
/// # Returns
///
/// A `Result<()>` which is `Ok(())` on success, or an `Error` if:
/// - The mask is empty or computes to zero (see `Error::Affinity`).
/// - `SetThreadAffinityMask` fails (see `Error::Affinity`).
///
/// # Remarks
///
/// Currently only the first 64 cores (bits 0-63) of the mask are used due to
/// `SetThreadAffinityMask` limitations. For systems with more than 64 logical
/// processors, `SetThreadGroupAffinity` and related processor group APIs would
/// be required, which are not currently implemented.
pub(crate) fn set_thread_affinity(mask: &AffinityMask) -> Result<()> {
    if mask.is_empty() {
        error!("Cannot set thread affinity with an empty mask.");
        return Err(Error::Affinity(
            "Cannot set thread affinity with an empty mask".to_string(),
        ));
    }

    debug!(
        "Attempting to set thread affinity on Windows with mask: {:?}",
        mask
    );

    let affinity_mask: usize = mask.as_raw_u64() as usize;

    if affinity_mask == 0 {
        error!("Affinity mask is computed as zero for mask {:?}", mask);
        return Err(Error::Affinity(format!(
            "Affinity mask is computed as zero for mask {:?}",
            mask
        )));
    }

    let thread_handle: HANDLE = unsafe { GetCurrentThread() };
    let old_mask = unsafe { SetThreadAffinityMask(thread_handle, affinity_mask) };

    if old_mask == 0 {
        let error_code = unsafe { GetLastError() };
        error!(
            "SetThreadAffinityMask failed for mask {}. Error: {:?}",
            mask, error_code
        );
        Err(Error::Affinity(format!(
            "SetThreadAffinityMask failed with error code: {:?}",
            error_code
        )))
    } else {
        debug!(
            "Successfully set thread affinity to {:#x} (old mask: {:#x}).",
            affinity_mask, old_mask
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
