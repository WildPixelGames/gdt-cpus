//! Utility functions for macOS platform-specific code, primarily for `sysctl` interaction.
//!
//! This module provides helper functions to query system information via the
//! `sysctlbyname` libc function. These utilities are used to retrieve various
//! CPU-related parameters like hardware model, core counts, cache sizes, etc.

use crate::{Error, Result};

/// Retrieves an integer value from `sysctlbyname`, tolerant of the key's width.
///
/// NOTE(macos): sysctl integer keys are NOT uniformly sized - the
/// `hw.perflevelN.*` keys are 4-byte `CTLTYPE_INT` while legacy `hw.*` keys
/// are often 8-byte QUADs. `sysctlbyname` succeeds whenever the buffer is
/// large enough and reports the value's ACTUAL size, so a strict
/// `size == size_of::<T>()` check silently zeroes every key whose kernel
/// width differs from the requested type. Read into an 8-byte buffer,
/// accept 4 or 8, zero-extend.
///
/// # Returns
///
/// A `Result<T>` containing the integer value if successful, or an `Error` if:
/// - The `name` is an invalid C string (`Error::Detection`).
/// - `sysctlbyname` fails for reasons other than `ENOENT` (`Error::SystemCall`).
/// - The key is not found (`ENOENT` -> `Error::Detection`; callers treat
///   optional keys with `.ok()` / `.unwrap_or…`).
/// - The reported size is neither 4 nor 8, or the value does not fit in `T`
///   (`Error::Detection`).
pub(crate) fn sysctlbyname_int<T: TryFrom<u64>>(name: &str) -> Result<T> {
    use std::ffi::CString;

    let c_name = CString::new(name)
        .map_err(|e| Error::Detection(format!("Invalid sysctl name {}: {}", name, e)))?;

    let mut raw: u64 = 0;
    let mut size = std::mem::size_of::<u64>();

    let ret = unsafe {
        libc::sysctlbyname(
            c_name.as_ptr(),
            &mut raw as *mut _ as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };

    if ret == -1 {
        let os_err = std::io::Error::last_os_error();

        if os_err.raw_os_error() == Some(libc::ENOENT) {
            return Err(Error::Detection(format!("Sysctl key {} not found", name)));
        }

        return Err(Error::SystemCall(format!(
            "sysctlbyname for {} failed: {}",
            name, os_err
        )));
    }

    // aarch64 is little-endian: a 4-byte value occupies the low bytes of `raw`.
    let value: u64 = match size {
        4 => raw & 0xFFFF_FFFF,
        8 => raw,
        n => {
            return Err(Error::Detection(format!(
                "sysctlbyname for {} returned unexpected size: {} (expected 4 or 8)",
                name, n
            )));
        }
    };

    T::try_from(value).map_err(|_| {
        Error::Detection(format!(
            "sysctl {} value {} does not fit the requested integer type",
            name, value
        ))
    })
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
