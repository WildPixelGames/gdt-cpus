//! Utility functions for Linux platform-specific code.
//!
//! This module provides helper functions commonly used by other modules within
//! the `platform::linux` scope, such as parsing sysfs files, CPU range lists,
//! and determining CPU vendor from string identifiers.

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::str::FromStr;

use crate::{Error, Result, Vendor};

/// Reads a value from a sysfs file and parses it into a specified type.
///
/// Sysfs files often contain single values (e.g., a number, a string). This function
/// reads the entire content of the file at the given `path`, trims whitespace,
/// and then attempts to parse it into type `T`.
///
/// # Type Parameters
///
/// * `T`: The type to parse the file content into. Must implement `std::str::FromStr`.
///
/// # Arguments
///
/// * `path`: A `Path` to the sysfs file to read.
///
/// # Errors
///
/// Returns `Error::Detection` if:
/// - The file cannot be read (e.g., it doesn't exist, permissions error).
/// - The content cannot be parsed into type `T`.
///
/// # Examples
///
/// ```ignore
/// // Assuming a sysfs file `/tmp/my_value` contains "123"
/// let value: Result<u32> = read_sysfs_value(Path::new("/tmp/my_value"));
/// assert_eq!(value.unwrap(), 123);
/// ```
pub(crate) fn read_sysfs_value<T: FromStr>(path: &Path) -> Result<T> {
    let content = fs::read_to_string(path)
        .map_err(|e| Error::Detection(format!("Failed to read sysfs file {:?}: {}", path, e)))?;
    content
        .trim()
        .parse::<T>()
        .map_err(|_| Error::Detection(format!("Failed to parse value from {:?}", path)))
}

/// Parses a list of CPU IDs from a string, typically from `/sys/devices/system/cpu/online`.
///
/// The string format is a comma-separated list of CPU IDs or ranges.
/// For example: "0-3,7,10-11".
/// This function reads the `online` file relative to `cpu_base_path` to get this string.
///
/// # Arguments
///
/// * `cpu_base_path`: The base path to the CPU sysfs directory (e.g., `/sys/devices/system/cpu/`).
///   The function will look for an `online` file within this directory.
///
/// # Returns
///
/// A `Result` containing a `HashSet<usize>` of all CPU IDs parsed from the list.
///
/// # Errors
///
/// Returns `Error::Detection` if:
/// - The `online` file cannot be read.
/// - The content has an invalid format (e.g., malformed range, non-numeric ID).
///
/// # Examples
///
/// ```ignore
/// // Assuming /sys/devices/system/cpu/online contains "0-1,3"
/// let online_cpus = parse_cpu_range_list(Path::new("/sys/devices/system/cpu/"));
/// assert!(online_cpus.is_ok());
/// let cpus = online_cpus.unwrap();
/// assert!(cpus.contains(&0));
/// assert!(cpus.contains(&1));
/// assert!(!cpus.contains(&2));
/// assert!(cpus.contains(&3));
/// ```
pub(crate) fn parse_cpu_range_list(cpu_base_path: &Path) -> Result<HashSet<usize>> {
    let range_str = fs::read_to_string(cpu_base_path.join("online")).map_err(|e| {
        Error::Detection(format!(
            "Failed to read /sys/devices/system/cpu/online: {}",
            e
        ))
    })?;
    let range_str = range_str.trim();

    let mut cpus = HashSet::new();

    for part in range_str.split(',') {
        if part.contains('-') {
            let mut iter = part.splitn(2, '-');
            let start_str = iter
                .next()
                .ok_or_else(|| Error::Detection(format!("Invalid CPU range format: {}", part)))?;
            let end_str = iter
                .next()
                .ok_or_else(|| Error::Detection(format!("Invalid CPU range format: {}", part)))?;
            let start = start_str
                .parse::<usize>()
                .map_err(|_| Error::Detection(format!("Invalid CPU range start: {}", start_str)))?;
            let end = end_str
                .parse::<usize>()
                .map_err(|_| Error::Detection(format!("Invalid CPU range end: {}", end_str)))?;
            if start > end {
                return Err(Error::Detection(format!(
                    "Invalid CPU range order: {}-{}",
                    start, end
                )));
            }
            for cpu_id in start..=end {
                cpus.insert(cpu_id);
            }
        } else {
            let cpu_id = part
                .parse::<usize>()
                .map_err(|_| Error::Detection(format!("Invalid CPU ID in range list: {}", part)))?;
            cpus.insert(cpu_id);
        }
    }
    Ok(cpus)
}

/// Determines the CPU [`Vendor`] based on string identifiers, typically from `/proc/cpuinfo`.
///
/// This function uses the `vendor_id` string (e.g., "GenuineIntel", "AuthenticAMD") and,
/// for ARM-based CPUs, the `CPU implementer` hex code (e.g., "0x41" for ARM Ltd.).
///
/// # Arguments
///
/// * `parsed_vendor_id_proc`: An `Option<String>` containing the `vendor_id` string.
/// * `parsed_cpu_implementer_proc`: An `Option<String>` containing the `CPU implementer` string.
///
/// # Returns
///
/// The determined [`Vendor`]. If the vendor cannot be specifically identified from the
/// provided strings, it may return `Vendor::Unknown` or `Vendor::Other` with the
/// original string.
pub(crate) fn parse_vendor_from_string(
    parsed_vendor_id_proc: &Option<String>,
    parsed_cpu_implementer_proc: &Option<String>,
) -> Vendor {
    if let Some(vid_str) = &parsed_vendor_id_proc {
        match vid_str.as_str() {
            "GenuineIntel" => Vendor::Intel,
            "AuthenticAMD" => Vendor::Amd,
            "Apple" => Vendor::Apple,
            _ => {
                if let Some(imp_str) = &parsed_cpu_implementer_proc {
                    match imp_str.as_str() {
                        "0x41" => Vendor::Arm,
                        "0x61" => Vendor::Apple,
                        "0x42" => Vendor::Other("Broadcom".to_string()),
                        "0x43" => Vendor::Other("Cavium".to_string()),
                        "0x44" => Vendor::Other("DEC".to_string()),
                        "0x4e" => Vendor::Other("Nvidia".to_string()),
                        "0x51" => Vendor::Other("Qualcomm".to_string()),
                        "0x56" => Vendor::Other("Marvell".to_string()),
                        "Apple"
                            if parsed_vendor_id_proc.is_none()
                                || parsed_vendor_id_proc.as_deref() != Some("Apple") =>
                        {
                            Vendor::Apple
                        }
                        _ => Vendor::Other(imp_str.clone()),
                    }
                } else {
                    Vendor::Other(vid_str.clone())
                }
            }
        }
    } else {
        Vendor::Unknown
    }
}
