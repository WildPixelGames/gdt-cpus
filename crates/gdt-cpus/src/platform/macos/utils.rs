//! Utility functions for macOS platform-specific code, primarily for `sysctl` interaction.
//!
//! This module provides helper functions to query system information via the
//! `sysctlbyname` libc function. These utilities are used to retrieve various
//! CPU-related parameters like hardware model, core counts, cache sizes, etc.

use log::debug;

use crate::{Error, Result};

/// Retrieves an integer value from `sysctlbyname`.
///
/// This is a generic helper function to call `libc::sysctlbyname` for various integer types
/// that implement `Default` and `Copy` (e.g., `i32`, `u32`, `u64`, `usize`).
///
/// # Type Parameters
///
/// * `T`: The integer type to retrieve. Must implement `Default` and `Copy`.
///
/// # Arguments
///
/// * `name`: The name of the `sysctl` MIB entry (e.g., "hw.ncpu").
///
/// # Returns
///
/// A `Result<T>` containing the integer value if successful, or an `Error` if:
/// - The `name` is an invalid C string (`Error::Detection`).
/// - `sysctlbyname` fails for reasons other than `ENOENT` (`Error::SystemCall`).
/// - The `sysctl` key specified by `name` is not found (`ENOENT`), resulting in `Error::Detection`.
///   (This can be handled by the caller using `.ok()` or `.unwrap_or_default()` if the key is optional).
/// - The size of the data returned by `sysctlbyname` does not match `std::mem::size_of::<T>()`
///   (`Error::Detection`).
pub(crate) fn sysctlbyname_int<T: Default + Copy>(name: &str) -> Result<T> {
    use std::ffi::CString;

    let c_name = CString::new(name)
        .map_err(|e| Error::Detection(format!("Invalid sysctl name {}: {}", name, e)))?;
    let mut value: T = T::default();
    let mut size = std::mem::size_of::<T>();

    let ret = unsafe {
        libc::sysctlbyname(
            c_name.as_ptr(),
            &mut value as *mut _ as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };

    if ret == -1 {
        // Check if the error is ENOENT (No such file or directory)
        let os_err = std::io::Error::last_os_error();
        if os_err.raw_os_error() == Some(libc::ENOENT) {
            debug!("Sysctl key {} not found (ENOENT)", name);
            // For optional keys, we might want to return Ok(T::default()) or a specific error.
            // For now, let's make it an error that can be handled with .ok() or .unwrap_or_default()
            return Err(Error::Detection(format!("Sysctl key {} not found", name)));
        }
        Err(Error::SystemCall(format!(
            "sysctlbyname for {} failed: {}",
            name, os_err
        )))
    } else if size != std::mem::size_of::<T>() {
        Err(Error::Detection(format!(
            "sysctlbyname for {} returned unexpected size: {} - should be {}",
            name,
            size,
            std::mem::size_of::<T>()
        )))
    } else {
        Ok(value)
    }
}

/// Retrieves a string value from `sysctlbyname`.
///
/// This function calls `libc::sysctlbyname` twice: first to get the required buffer size,
/// and then to retrieve the actual string data. The retrieved byte buffer is then
/// converted to a Rust `String`.
///
/// # Arguments
///
/// * `name`: The name of the `sysctl` MIB entry (e.g., "machdep.cpu.brand_string").
///
/// # Returns
///
/// A `Result<String>` containing the string value if successful, or an `Error` if:
/// - The `name` is an invalid C string (`Error::Detection`).
/// - The `sysctl` key specified by `name` is not found (`ENOENT`), resulting in `Error::Detection`.
/// - `sysctlbyname` fails during the size query or data query (`Error::SystemCall`).
/// - The retrieved data cannot be converted to a UTF-8 string (`Error::Detection`).
///
/// If the `sysctl` key exists but the value is empty, an empty `String` is returned.
pub(crate) fn sysctlbyname_string(name: &str) -> Result<String> {
    use std::ffi::CString;

    let c_name = CString::new(name)
        .map_err(|e| Error::Detection(format!("Invalid sysctl name {}: {}", name, e)))?;
    let mut size: libc::size_t = 0;

    let ret = unsafe {
        libc::sysctlbyname(
            c_name.as_ptr(),
            std::ptr::null_mut(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };

    if ret == -1 {
        let os_err = std::io::Error::last_os_error();
        if os_err.raw_os_error() == Some(libc::ENOENT) {
            debug!("Sysctl key {} not found (ENOENT)", name);
            return Err(Error::Detection(format!("Sysctl key {} not found", name)));
        }
        return Err(Error::SystemCall(format!(
            "sysctlbyname for {} (size query) failed: {}",
            name, os_err
        )));
    }

    if size == 0 {
        return Ok(String::new());
    }

    let mut buf = vec![0u8; size as usize];
    let ret = unsafe {
        libc::sysctlbyname(
            c_name.as_ptr(),
            buf.as_mut_ptr() as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };

    if ret == -1 {
        return Err(Error::SystemCall(format!(
            "sysctlbyname for {} (data query) failed: {}",
            name,
            std::io::Error::last_os_error()
        )));
    }

    // Trim null bytes from the end if any, as size might include it.
    while !buf.is_empty() && buf[buf.len() - 1] == 0 {
        buf.pop();
    }

    String::from_utf8(buf)
        .map_err(|e| Error::Detection(format!("UTF-8 conversion error for {}: {}", name, e)))
}

/// Converts a `u32` value to a `libc::qos_class_t`.
pub(crate) fn u32_to_qos_class_t(value: u32) -> libc::qos_class_t {
    match value {
        0x21 => libc::qos_class_t::QOS_CLASS_USER_INTERACTIVE,
        0x19 => libc::qos_class_t::QOS_CLASS_USER_INITIATED,
        0x15 => libc::qos_class_t::QOS_CLASS_DEFAULT,
        0x11 => libc::qos_class_t::QOS_CLASS_UTILITY,
        0x09 => libc::qos_class_t::QOS_CLASS_BACKGROUND,
        _ => libc::qos_class_t::QOS_CLASS_UNSPECIFIED,
    }
}
