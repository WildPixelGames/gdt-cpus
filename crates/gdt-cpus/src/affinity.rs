//! Manages thread affinity and priority for the current thread.
//!
//! This module provides functions to control which CPU cores a thread runs on
//! (`pin_thread_to_core`) and to set the scheduling priority of the current thread
//! (`set_thread_priority`). These operations are often crucial in game development
//! for optimizing performance and ensuring that critical tasks receive sufficient
//! CPU resources.
//!
//! The behavior of these functions is platform-specific, with implementations
//! provided for Windows, Linux, and macOS. For unsupported platforms, a
//! `Error::Unsupported` will be returned.
//!
//! It also provides a way to retrieve a default set of scheduling policies via
//! `get_scheduling_policies`.

use crate::{SchedulingPolicy, ThreadPriority, error::Result};

// This static variable is internal and does not require public Rustdoc.
// It's used by `get_scheduling_policies` to cache scheduling policy mappings if ever set.
static SCHEDULING_POLICIES_MAPPINGS: std::sync::OnceLock<[SchedulingPolicy; 7]> =
    std::sync::OnceLock::new();

/// Pins the current thread to a specific logical core ID.
///
/// This function attempts to set the CPU affinity for the currently executing thread
/// to the specified logical core. Logical core IDs are OS-specific.
/// You can get a list of all logical core IDs using `cpu_info().logical_processor_ids()`.
///
/// # Parameters
///
/// *   `logical_core_id`: The OS-specific ID of the logical core to pin the thread to.
///
/// # Returns
///
/// *   `Ok(())` if the thread was successfully pinned.
/// *   `Err(Error)` if pinning failed. This could be due to an invalid `logical_core_id`,
///     insufficient permissions, or if the operation is unsupported on the current platform.
///
/// # Examples
///
/// ```
/// use gdt_cpus::{pin_thread_to_core, cpu_info};
///
/// // Attempt to pin the current thread to the first available logical core.
/// if let Ok(info) = cpu_info() {
///     if let Some(first_core_id) = info.logical_processor_ids().first() {
///         match pin_thread_to_core(*first_core_id) {
///             Ok(_) => println!("Successfully pinned thread to logical core {}", first_core_id),
///             Err(e) => eprintln!("Failed to pin thread to core {}: {}", first_core_id, e),
///         }
///     } else {
///         eprintln!("No logical cores found to pin to.");
///     }
/// } else {
///     eprintln!("Could not retrieve CPU info to determine a valid core ID.");
/// }
/// ```
///
/// Pinning to a specific known core (e.g., core 0 if known to be valid):
/// ```
/// # use gdt_cpus::pin_thread_to_core;
/// // Pin the current thread to logical core 0 (ensure this ID is valid for your system)
/// match pin_thread_to_core(0) {
///     Ok(_) => println!("Attempted to pin thread to logical core 0"),
///     Err(e) => eprintln!("Failed to pin thread to logical core 0: {}", e),
/// }
/// ```
pub fn pin_thread_to_core(logical_core_id: usize) -> Result<()> {
    // Platform-specific implementation
    #[cfg(target_os = "windows")]
    {
        crate::platform::windows::affinity::pin_thread_to_core(logical_core_id)
    }
    #[cfg(target_os = "linux")]
    {
        crate::platform::linux::affinity::pin_thread_to_core(logical_core_id)
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = logical_core_id; // suppress unused variable warning

        Err(crate::Error::Unsupported(
            "Thread pinning is not supported on this platform.".to_string(),
        ))
    }
}

/// Sets the priority of the current thread.
///
/// The interpretation of priority levels can vary between operating systems.
/// Refer to the `ThreadPriority` enum for available levels.
///
/// # Parameters
///
/// *   `priority`: The desired priority level from the [`ThreadPriority`] enum.
///
/// # Returns
///
/// *   `Ok(())` if the thread priority was successfully set.
/// *   `Err(Error)` if setting the priority failed. This could be due to insufficient
///     permissions or an unsupported priority level on the current platform.
///
/// # Examples
///
/// ```
/// use gdt_cpus::{set_thread_priority, ThreadPriority};
///
/// // Set the current thread to a higher priority
/// match set_thread_priority(ThreadPriority::Highest) {
///     Ok(_) => println!("Thread priority set to Highest."),
///     Err(e) => eprintln!("Failed to set thread priority: {}", e),
/// }
///
/// // Set the current thread to a lower priority
/// match set_thread_priority(ThreadPriority::Lowest) {
///     Ok(_) => println!("Thread priority set to Lowest."),
///     Err(e) => eprintln!("Failed to set thread priority: {}", e),
/// }
/// ```
pub fn set_thread_priority(priority: ThreadPriority) -> Result<()> {
    // Platform-specific implementation
    #[cfg(target_os = "windows")]
    {
        crate::platform::windows::affinity::set_thread_priority(priority)
    }
    #[cfg(target_os = "linux")]
    {
        crate::platform::linux::affinity::set_thread_priority(priority)
    }
    #[cfg(target_os = "macos")]
    {
        crate::platform::macos::affinity::set_thread_priority(priority)
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err(Error::Unsupported(
            "Setting thread priority is not supported on this platform.".to_string(),
        ))
    }
}

/// Retrieves a static slice representing the system's or default scheduling policies.
///
/// This function returns a predefined array of 7 [`SchedulingPolicy`] items.
/// If platform-specific mappings for scheduling policies have been initialized
/// (this crate does not currently expose a public way to do this), those mappings are returned.
/// Otherwise, it returns a default set of mappings defined by
/// [`SchedulingPolicy::default_mappings()`].
///
/// The exact meaning and order of these 7 policies might depend on the
/// `SchedulingPolicy` definition and its default mappings.
///
/// # Returns
///
/// A `&'static [SchedulingPolicy; 7]` array.
///
/// # Examples
///
/// ```
/// use gdt_cpus::{get_scheduling_policies, SchedulingPolicy};
///
/// let policies: &[SchedulingPolicy; 7] = get_scheduling_policies();
/// println!("Available/Default Scheduling Policies:");
/// for (index, policy) in policies.iter().enumerate() {
///     // Assuming SchedulingPolicy implements Debug or Display
///     println!("Policy {}: {}", index, policy);
/// }
/// ```
pub fn get_scheduling_policies() -> &'static [SchedulingPolicy; 7] {
    SCHEDULING_POLICIES_MAPPINGS
        .get()
        .unwrap_or_else(|| SchedulingPolicy::default_mappings())
}
