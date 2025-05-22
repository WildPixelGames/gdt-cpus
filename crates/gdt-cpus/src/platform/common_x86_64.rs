//! Common x86_64 CPU detection logic using the `cpuid` instruction.
//!
//! This module provides functions to query CPU features, vendor, and model name
//! on x86_64 architectures by directly using the `cpuid` instruction.
//! It relies on the `raw_cpuid` crate to perform these low-level queries.
//!
//! The functions here are `pub(crate)` and are intended to be used by the
//! platform-specific modules (e.g., `linux.rs`, `windows.rs`, `macos.rs`)
//! when they are compiled for an x86_64 target.

use log::debug;

use crate::{CpuFeatures, Vendor};

/// Detects CPU features available on x86_64 using `cpuid`.
///
/// This function queries various `cpuid` leaves and bits to determine
/// the presence of common x86_64 instruction set extensions like MMX, SSE,
/// AVX, AVX2, AVX512, AES, SHA, etc.
///
/// The detected features are added to the mutable `features` argument.
///
/// # Arguments
///
/// * `features`: A mutable reference to a [`CpuFeatures`] bitflags struct
///   where detected features will be set.
pub(crate) fn detect_features_via_cpuid(features: &mut CpuFeatures) {
    let cpuid = raw_cpuid::CpuId::new();

    if let Some(fi) = cpuid.get_feature_info() {
        if fi.has_mmx() {
            features.insert(CpuFeatures::MMX);
        }
        if fi.has_sse() {
            features.insert(CpuFeatures::SSE);
        }
        if fi.has_sse2() {
            features.insert(CpuFeatures::SSE2);
        }
        if fi.has_sse3() {
            features.insert(CpuFeatures::SSE3);
        }
        if fi.has_ssse3() {
            features.insert(CpuFeatures::SSSE3);
        }
        if fi.has_sse41() {
            features.insert(CpuFeatures::SSE4_1);
        }
        if fi.has_sse42() {
            features.insert(CpuFeatures::SSE4_2);
        }
        if fi.has_avx() {
            features.insert(CpuFeatures::AVX);
        }
        if fi.has_aesni() {
            features.insert(CpuFeatures::AES);
        }
        if fi.has_fma() {
            features.insert(CpuFeatures::FMA3);
        }
    }

    // Extended features (typically Leaf 7, Sub-leaf 0)
    if let Some(ext_fi) = cpuid.get_extended_feature_info() {
        if ext_fi.has_sha() {
            features.insert(CpuFeatures::SHA);
        }
        if ext_fi.has_avx2() {
            features.insert(CpuFeatures::AVX2);
        }
        if ext_fi.has_avx512f() {
            features.insert(CpuFeatures::AVX512F);
        }
        if ext_fi.has_avx512bw() {
            features.insert(CpuFeatures::AVX512BW);
        }
        if ext_fi.has_avx512cd() {
            features.insert(CpuFeatures::AVX512CD);
        }
        if ext_fi.has_avx512dq() {
            features.insert(CpuFeatures::AVX512DQ);
        }
        if ext_fi.has_avx512vl() {
            features.insert(CpuFeatures::AVX512VL);
        }
    }
}

/// Detects CPU vendor, model name, and features on x86_64 using `cpuid`.
///
/// This function populates the provided mutable references with the CPU vendor
/// (Intel, AMD, or Other), the processor brand string (model name), and
/// calls `detect_features_via_cpuid` to populate the CPU features.
///
/// # Arguments
///
/// * `vendor`: A mutable reference to a [`Vendor`] enum.
/// * `model_name`: A mutable reference to a String that will hold the CPU model name.
/// * `features`: A mutable reference to a [`CpuFeatures`] bitflags struct.
pub(crate) fn detect_via_cpuid(
    vendor: &mut Vendor,
    model_name: &mut String,
    features: &mut CpuFeatures,
) {
    debug!("Attempting to use raw-cpuid for x86_64 vendor, model, and features.");
    let cpuid = raw_cpuid::CpuId::new();

    if let Some(vf) = cpuid.get_vendor_info() {
        let vendor_str = vf.as_str();
        debug!("CPUID Vendor: {}", vendor_str);
        *vendor = match vendor_str {
            "GenuineIntel" => Vendor::Intel,
            "AuthenticAMD" => Vendor::Amd,
            _ => Vendor::Other(vendor_str.to_string()),
        };
    }

    if let Some(pbs) = cpuid.get_processor_brand_string() {
        *model_name = pbs.as_str().trim().to_string();
        debug!("CPUID Model Name: {}", model_name);
    } else if let Some(fi) = cpuid.get_feature_info() {
        *model_name = format!(
            "Family {} Model {} Stepping {}",
            fi.family_id(),
            fi.model_id(),
            fi.stepping_id()
        );
        debug!("CPUID Model Name (fallback): {}", model_name);
    } else {
        *model_name = "Unknown x86_64".to_string();
        debug!("CPUID Model Name: Could not determine.");
    }

    detect_features_via_cpuid(features);
}
