//! Utility functions for Windows platform-specific code.
//!
//! This module provides helper functions commonly used by other modules within
//! the `platform::windows` scope, particularly for Windows API interoperability.

use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

/// Converts a Rust string slice (`&str`) into a null-terminated UTF-16 encoded
/// vector of `u16` values, suitable for use with Windows API functions that
/// expect wide strings (PCWSTR).
///
/// The resulting vector includes a null terminator, which is required by many
/// Windows API functions.
///
/// # Arguments
///
/// * `s`: The string slice to convert.
///
/// # Returns
///
/// A `Vec<u16>` containing the UTF-16 representation of the input string,
/// followed by a null terminator.
///
/// # Important
///
/// The `Vec<u16>` returned by this function must live as long as any pointer
/// (e.g., `PCWSTR`) derived from its data is in use. This is crucial to avoid
/// dangling pointers when calling Windows API functions.
///
/// # Examples
///
/// ```
/// fn get_example_vec() -> Vec<u16> {
///   let rust_str = "Hello";
///   super::to_wide_null_vec(rust_str)
/// }
/// let wide_vec = get_example_vec();
///
/// // Example: wide_vec would be [72, 101, 108, 108, 111, 0]
/// // (ASCII values for 'H', 'e', 'l', 'l', 'o', followed by null)
/// assert_eq!(wide_vec.last(), Some(&0));
/// assert_eq!(wide_vec.len(), "Hello".len() + 1);
/// ```
pub(crate) fn to_wide_null_vec(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
