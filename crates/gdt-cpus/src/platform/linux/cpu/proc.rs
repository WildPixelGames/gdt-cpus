//! Linux-specific CPU information detection by parsing `/proc/cpuinfo`.
//!
//! This module is responsible for reading and interpreting the contents of the
//! `/proc/cpuinfo` file, a common method on Linux systems to obtain details
//! about the CPU, such as its vendor, model name, and supported features.
//!
//! The functions herein are often used as a fallback mechanism when `cpuid`-based
//! detection (common on x86_64) is unavailable or insufficient, or as the primary
//! method on architectures like aarch64 where `/proc/cpuinfo` is a standard source
//! for this information.

use std::{
    collections::HashSet,
    io::{BufRead, BufReader},
};

use log::{debug, warn};

use crate::{
    CpuFeatures, Vendor,
    platform::linux::{
        cpu::features::detect_features_from_hashmap, utils::parse_vendor_from_string,
    },
};

/// Parses the `/proc/cpuinfo` file to extract raw strings for CPU vendor, model name, and features.
///
/// It reads `/proc/cpuinfo` line by line, looking for specific fields primarily
/// within the first processor's information block.
///
/// The fields searched are:
/// - For vendor: "vendor_id" (common on x86) or "CPU implementer" (common on ARM).
/// - For model name: "model name" (x86) or "Processor" (ARM).
/// - For features: "flags" (x86) or "Features" (ARM).
///
/// # Returns
///
/// A tuple containing four `Option<String>`:
/// 1. `parsed_vendor_id_proc`: The value of the "vendor_id" field, if found.
/// 2. `parsed_cpu_implementer_proc`: The value of the "CPU implementer" field, if found.
/// 3. `parsed_model_name_proc`: The value of the "model name" or "Processor" field, if found.
/// 4. `parsed_features_line_proc`: The content of the "flags" or "Features" line, if found.
///
/// Returns `(None, None, None, None)` if `/proc/cpuinfo` cannot be opened.
pub(crate) fn parse_proc_cpuinfo() -> (
    Option<String>, // vendor_id
    Option<String>, // cpu_implementer
    Option<String>, // model_name
    Option<String>, // features_line
) {
    let cpuinfo_file = if let Ok(file) = std::fs::File::open("/proc/cpuinfo") {
        file
    } else {
        warn!("Failed to open /proc/cpuinfo for fallback vendor/model/features.");
        return (None, None, None, None);
    };

    let mut parsed_vendor_id_proc: Option<String> = None;
    let mut parsed_cpu_implementer_proc: Option<String> = None;
    let mut parsed_model_name_proc: Option<String> = None;
    let mut parsed_features_line_proc: Option<String> = None;

    let reader = BufReader::new(cpuinfo_file);
    let mut first_processor_block = true;

    for line_res in reader.lines() {
        let line = match line_res {
            Ok(l) => l,
            Err(_) => continue,
        };
        if line.trim().is_empty() {
            if first_processor_block {
                first_processor_block = false;
                if (parsed_vendor_id_proc.is_some() || parsed_cpu_implementer_proc.is_some())
                    && parsed_model_name_proc.is_some()
                    && parsed_features_line_proc.is_some()
                {
                    break;
                }
            }
            continue;
        }
        if !first_processor_block {
            continue;
        }

        let parts: Vec<&str> = line.split(':').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            match parts[0] {
                "vendor_id" => {
                    if parsed_vendor_id_proc.is_none() {
                        parsed_vendor_id_proc = Some(parts[1].to_string());
                    }
                }
                "CPU implementer" => {
                    if parsed_cpu_implementer_proc.is_none() {
                        parsed_cpu_implementer_proc = Some(parts[1].to_string());
                    }
                }
                "model name" | "Processor" => {
                    if parsed_model_name_proc.is_none() {
                        parsed_model_name_proc = Some(parts[1].to_string());
                    }
                }
                "flags" | "Features" => {
                    if parsed_features_line_proc.is_none() {
                        parsed_features_line_proc = Some(parts[1].to_string());
                    }
                }
                _ => {}
            }
        }
    }

    (
        parsed_vendor_id_proc,
        parsed_cpu_implementer_proc,
        parsed_model_name_proc,
        parsed_features_line_proc,
    )
}

/// Detects and updates CPU vendor, model name, and features using data from `/proc/cpuinfo`.
///
/// This function calls `parse_proc_cpuinfo` to get raw data and then processes this
/// data to update the provided `vendor`, `model_name`, and `features` arguments.
/// It's often used as a fallback if `cpuid` detection didn't yield complete results
/// (e.g., `Vendor::Unknown` or a generic model name like "Family X Model Y") or as the
/// primary source on non-x86_64 architectures.
///
/// - The `vendor` is updated if it's currently `Vendor::Unknown`, using `parse_vendor_from_string`.
/// - The `model_name` is updated if it's "Unknown" or a generic family/model string.
/// - The `features` are updated by parsing the features line (if available) and using
///   `detect_features_from_hashmap` to set the corresponding flags in the `CpuFeatures` struct.
///
/// # Arguments
///
/// * `vendor`: A mutable reference to a [`Vendor`] enum to be updated.
/// * `model_name`: A mutable reference to a String holding the CPU model name, to be updated.
/// * `features`: A mutable reference to a [`CpuFeatures`] struct to be updated.
pub(crate) fn detect_via_proc_cpuinfo(
    vendor: &mut Vendor,
    model_name: &mut String,
    features: &mut CpuFeatures,
) {
    debug!("Using /proc/cpuinfo for vendor, model, or features (fallback or non-x86_64).");

    let (
        parsed_vendor_id_proc,
        parsed_cpu_implementer_proc,
        parsed_model_name_proc,
        parsed_features_line_proc,
    ) = parse_proc_cpuinfo();

    if parsed_vendor_id_proc.is_none()
        && parsed_cpu_implementer_proc.is_none()
        && parsed_features_line_proc.is_none()
        && parsed_model_name_proc.is_none()
    {
        warn!("No vendor/model/features information found in /proc/cpuinfo.");
        return;
    }

    if *vendor == Vendor::Unknown {
        *vendor = parse_vendor_from_string(&parsed_vendor_id_proc, &parsed_cpu_implementer_proc);
    }

    debug!(
        "Determined vendor from /proc/cpuinfo (fallback): {:?}",
        vendor
    );

    if model_name == "Unknown" || model_name.starts_with("Family") {
        // If CPUID gave generic model
        *model_name = parsed_model_name_proc.unwrap_or_else(|| "Unknown".to_string());

        debug!(
            "Determined model_name from /proc/cpuinfo (fallback): {}",
            model_name
        );
    }

    let mut cpu_features_from_proc_cpuinfo: HashSet<String> = HashSet::new();

    if let Some(f_line) = parsed_features_line_proc {
        cpu_features_from_proc_cpuinfo = f_line
            .to_lowercase()
            .split_whitespace()
            .map(String::from)
            .collect();

        debug!(
            "Features from /proc/cpuinfo (non-x86_64 or CPUID feature fallback): {:?}",
            cpu_features_from_proc_cpuinfo
        );
    }

    detect_features_from_hashmap(features, &cpu_features_from_proc_cpuinfo);
}
