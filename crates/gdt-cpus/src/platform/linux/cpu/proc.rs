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
pub(crate) fn parse_proc_cpuinfo(
    procfs_root: &std::path::Path,
) -> (
    Option<String>, // vendor_id
    Option<String>, // cpu_implementer
    Option<String>, // model_name
    Option<String>, // features_line
) {
    let cpuinfo_file = if let Ok(file) = std::fs::File::open(procfs_root.join("cpuinfo")) {
        file
    } else {
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
                "flags" | "Features" if parsed_features_line_proc.is_none() => {
                    parsed_features_line_proc = Some(parts[1].to_string());
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
    procfs_root: &std::path::Path,
    vendor: &mut Vendor,
    model_name: &mut String,
    features: &mut CpuFeatures,
) {
    let (
        parsed_vendor_id_proc,
        parsed_cpu_implementer_proc,
        parsed_model_name_proc,
        parsed_features_line_proc,
    ) = parse_proc_cpuinfo(procfs_root);

    if parsed_vendor_id_proc.is_none()
        && parsed_cpu_implementer_proc.is_none()
        && parsed_features_line_proc.is_none()
        && parsed_model_name_proc.is_none()
    {
        return;
    }

    if *vendor == Vendor::Unknown {
        *vendor = parse_vendor_from_string(&parsed_vendor_id_proc, &parsed_cpu_implementer_proc);
    }

    if model_name == "Unknown" || model_name.starts_with("Family") {
        // If CPUID gave generic model
        *model_name = parsed_model_name_proc.unwrap_or_else(|| "Unknown".to_string());
    }

    let mut cpu_features_from_proc_cpuinfo: HashSet<String> = HashSet::new();

    if let Some(f_line) = parsed_features_line_proc {
        cpu_features_from_proc_cpuinfo = f_line
            .to_lowercase()
            .split_whitespace()
            .map(String::from)
            .collect();
    }

    detect_features_from_hashmap(features, &cpu_features_from_proc_cpuinfo);
}

/// Parses a hex field value like `0xd81` (or bare `d81`) into a `u16`.
fn parse_hex_u16(s: &str) -> Option<u16> {
    let s = s.trim();
    let hex = s
        .strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s);
    u16::from_str_radix(hex, 16).ok()
}

/// Walks EVERY processor block of `/proc/cpuinfo` content and returns each
/// block's `(processor id, CPU part)` pair.
///
/// Unlike [`parse_proc_cpuinfo`] (which reads identity from the first block
/// only, since vendor/model/features are uniform), the `CPU part` is per-core
/// on heterogeneous ARM: a big.LITTLE chip reports e.g. `0xd81` (Cortex-A720)
/// on its big cores and `0xd80` (Cortex-A520) on its little cores. Blocks
/// without a `CPU part` line (x86, where the field does not exist) contribute
/// nothing, so x86 content yields an empty result. The raw part number is kept
/// verbatim; combined with the chip vendor (the MIDR implementer) it names the
/// microarchitecture, but no implementer/part name table is shipped.
pub(crate) fn parse_cpu_parts(content: &str) -> Vec<(u16, u16)> {
    let mut out = Vec::new();
    let mut cur_proc: Option<u16> = None;
    let mut cur_part: Option<u16> = None;

    for line in content.lines() {
        if line.trim().is_empty() {
            if let (Some(p), Some(part)) = (cur_proc, cur_part) {
                out.push((p, part));
            }

            cur_proc = None;
            cur_part = None;

            continue;
        }

        let mut it = line.splitn(2, ':');
        let key = it.next().map(str::trim).unwrap_or("");
        let Some(val) = it.next().map(str::trim) else {
            continue;
        };

        match key {
            "processor" => cur_proc = val.parse::<u16>().ok(),
            "CPU part" => cur_part = parse_hex_u16(val),
            _ => {}
        }
    }

    // Flush the final block: the last record has no trailing blank line.
    if let (Some(p), Some(part)) = (cur_proc, cur_part) {
        out.push((p, part));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_parts_walks_every_block() {
        // cix-p1 shape: the CPU part differs per core (A720 0xd81 vs A520
        // 0xd80), so a first-block-only parse would miss the heterogeneity.
        let cpuinfo = "\
processor\t: 0
CPU implementer\t: 0x41
CPU part\t: 0xd81

processor\t: 1
CPU part\t: 0xd81

processor\t: 2
CPU part\t: 0xd80

processor\t: 3
CPU part\t: 0xd80
";
        assert_eq!(
            parse_cpu_parts(cpuinfo),
            vec![(0, 0xd81), (1, 0xd81), (2, 0xd80), (3, 0xd80)]
        );
    }

    #[test]
    fn cpu_parts_empty_when_no_part_field() {
        // x86 /proc/cpuinfo carries no "CPU part" line at all.
        let cpuinfo = "\
processor\t: 0
vendor_id\t: AuthenticAMD
model name\t: AMD Ryzen 9 5950X 16-Core Processor

processor\t: 1
vendor_id\t: AuthenticAMD
";
        assert!(parse_cpu_parts(cpuinfo).is_empty());
    }
}
