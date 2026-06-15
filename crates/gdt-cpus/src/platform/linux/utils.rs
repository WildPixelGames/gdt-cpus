//! Utility functions for Linux platform-specific code.
//!
//! This module provides helper functions commonly used by other modules within
//! the `platform::linux` scope, such as parsing sysfs files, CPU range lists,
//! and determining CPU vendor from string identifiers.

use crate::Vendor;

// The kernel range-list parser ("0-3,7,10-11") lives in the platform-neutral
// `ranges` module (the fixture checker shares it); re-exported here for the
// Linux call sites.
pub(crate) use crate::platform::ranges::{parse_range_list_str, parse_range_list_with};

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
    // NOTE(linux): vendor_id and CPU implementer are INDEPENDENT sources -
    // ARM cpuinfo has NO vendor_id line at all, only "CPU implementer".
    // Nesting the implementer table inside a vendor_id-present branch makes
    // every ARM box return Unknown without the table ever being consulted.
    // Check both, in order.
    if let Some(vid_str) = parsed_vendor_id_proc {
        match vid_str.as_str() {
            "GenuineIntel" => return Vendor::Intel,
            "AuthenticAMD" => return Vendor::Amd,
            "Apple" => return Vendor::Apple,
            _ => {}
        }
    }

    if let Some(imp_str) = parsed_cpu_implementer_proc {
        return match imp_str.as_str() {
            "0x41" => Vendor::Arm,
            "0x61" => Vendor::Apple,
            "0x42" => Vendor::Broadcom,
            // Cavium was acquired by Marvell; both implementer codes map there.
            "0x43" => Vendor::Marvell,
            "0x4e" => Vendor::Nvidia,
            "0x51" => Vendor::Qualcomm,
            "0x56" => Vendor::Marvell,
            _ => Vendor::Other,
        };
    }

    if parsed_vendor_id_proc.is_some() {
        Vendor::Other
    } else {
        Vendor::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn implementer_consulted_without_vendor_id() {
        // The ARM reality: no vendor_id line, only CPU implementer.
        let v = parse_vendor_from_string(&None, &Some("0x41".to_string()));
        assert_eq!(v, Vendor::Arm);
        let v = parse_vendor_from_string(&None, &Some("0x61".to_string()));
        assert_eq!(v, Vendor::Apple);
    }

    #[test]
    fn vendor_id_takes_priority() {
        let v =
            parse_vendor_from_string(&Some("GenuineIntel".to_string()), &Some("0x41".to_string()));
        assert_eq!(v, Vendor::Intel);
    }

    #[test]
    fn unknown_only_when_both_absent() {
        assert_eq!(parse_vendor_from_string(&None, &None), Vendor::Unknown);
        assert_eq!(
            parse_vendor_from_string(&Some("WeirdCorp".to_string()), &None),
            Vendor::Other
        );
        assert_eq!(
            parse_vendor_from_string(&None, &Some("0xff".to_string())),
            Vendor::Other
        );
    }
}
