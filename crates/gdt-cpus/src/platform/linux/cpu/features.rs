//! Linux-specific CPU feature detection from parsed `/proc/cpuinfo` data.
//!
//! This module provides helper functions to populate the [`CpuFeatures`] bitflags
//! based on a set of feature strings obtained by parsing the `flags` (on x86_64)
//! or `Features` (on aarch64) lines from `/proc/cpuinfo`.
//!
//! It contains architecture-specific implementations (`x86_64`, `aarch64`) for
//! mapping known feature strings to their corresponding [`CpuFeatures`] variants.
//! These functions are typically used as a fallback or supplement to `cpuid` based
//! detection.

use std::collections::HashSet;

use crate::CpuFeatures;

/// Populates [`CpuFeatures`] based on a HashSet of feature strings from `/proc/cpuinfo` on `x86_64`.
///
/// This function checks for the presence of known x86_64 feature strings (e.g., "mmx",
/// "sse", "avx", "aes") in the `cpu_features_from_proc_cpuinfo` set and sets the
/// corresponding flags in the mutable `features` argument.
///
/// Note: The feature strings in `/proc/cpuinfo` (like "pni" for SSE3) might differ
/// from the canonical names. This function handles known aliases.
///
/// # Arguments
///
/// * `features`: A mutable reference to a [`CpuFeatures`] struct to be updated.
/// * `cpu_features_from_proc_cpuinfo`: A HashSet containing feature strings parsed
///   from the 'flags' line of `/proc/cpuinfo` for an x86_64 CPU.
#[cfg(target_arch = "x86_64")]
pub(crate) fn detect_features_from_hashmap(
    features: &mut CpuFeatures,
    cpu_features_from_proc_cpuinfo: &HashSet<String>,
) {
    if cpu_features_from_proc_cpuinfo.contains("mmx") {
        features.insert(CpuFeatures::MMX);
    }
    if cpu_features_from_proc_cpuinfo.contains("sse") {
        features.insert(CpuFeatures::SSE);
    }
    if cpu_features_from_proc_cpuinfo.contains("sse2") {
        features.insert(CpuFeatures::SSE2);
    }
    if cpu_features_from_proc_cpuinfo.contains("pni") {
        features.insert(CpuFeatures::SSE3);
    }
    if cpu_features_from_proc_cpuinfo.contains("ssse3") {
        features.insert(CpuFeatures::SSSE3);
    }
    if cpu_features_from_proc_cpuinfo.contains("sse4_1") {
        features.insert(CpuFeatures::SSE4_1);
    }
    if cpu_features_from_proc_cpuinfo.contains("sse4_2") {
        features.insert(CpuFeatures::SSE4_2);
    }
    if cpu_features_from_proc_cpuinfo.contains("fma") {
        features.insert(CpuFeatures::FMA3);
    }
    if cpu_features_from_proc_cpuinfo.contains("avx") {
        features.insert(CpuFeatures::AVX);
    }
    if cpu_features_from_proc_cpuinfo.contains("avx2") {
        features.insert(CpuFeatures::AVX2);
    }
    if cpu_features_from_proc_cpuinfo.contains("avx512f") {
        features.insert(CpuFeatures::AVX512F);
    }
    if cpu_features_from_proc_cpuinfo.contains("avx512bw") {
        features.insert(CpuFeatures::AVX512BW);
    }
    if cpu_features_from_proc_cpuinfo.contains("avx512cd") {
        features.insert(CpuFeatures::AVX512CD);
    }
    if cpu_features_from_proc_cpuinfo.contains("avx512dq") {
        features.insert(CpuFeatures::AVX512DQ);
    }
    if cpu_features_from_proc_cpuinfo.contains("avx512vl") {
        features.insert(CpuFeatures::AVX512VL);
    }
    if cpu_features_from_proc_cpuinfo.contains("aes") {
        features.insert(CpuFeatures::AES);
    }
    if cpu_features_from_proc_cpuinfo.contains("sha_ni") {
        features.insert(CpuFeatures::SHA);
    }
    if cpu_features_from_proc_cpuinfo.contains("crc32") {
        features.insert(CpuFeatures::CRC32);
    }
}

/// Populates [`CpuFeatures`] based on a HashSet of feature strings from `/proc/cpuinfo` on `aarch64`.
///
/// This function checks for the presence of known aarch64 feature strings (e.g., "asimd",
/// "neon", "aes", "sha2", "sve") in the `cpu_features_from_proc_cpuinfo` set and
/// sets the corresponding flags in the mutable `features` argument.
///
/// # Arguments
///
/// * `features`: A mutable reference to a [`CpuFeatures`] struct to be updated.
/// * `cpu_features_from_proc_cpuinfo`: A HashSet containing feature strings parsed
///   from the 'Features' line of `/proc/cpuinfo` for an aarch64 CPU.
#[cfg(target_arch = "aarch64")]
pub(crate) fn detect_features_from_hashmap(
    features: &mut CpuFeatures,
    cpu_features_from_proc_cpuinfo: &HashSet<String>,
) {
    if cpu_features_from_proc_cpuinfo.contains("asimd")
        || cpu_features_from_proc_cpuinfo.contains("neon")
        || cpu_features_from_proc_cpuinfo.contains("fp")
    {
        features.insert(CpuFeatures::NEON);
    }
    if cpu_features_from_proc_cpuinfo.contains("aes") {
        features.insert(CpuFeatures::AES);
    }
    if cpu_features_from_proc_cpuinfo.contains("sha1")
        || cpu_features_from_proc_cpuinfo.contains("sha2")
        || cpu_features_from_proc_cpuinfo.contains("sha256")
        || cpu_features_from_proc_cpuinfo.contains("sha512")
        || cpu_features_from_proc_cpuinfo.contains("sha3")
    {
        features.insert(CpuFeatures::SHA);
    }
    if cpu_features_from_proc_cpuinfo.contains("crc32") {
        features.insert(CpuFeatures::CRC32);
    }
    if cpu_features_from_proc_cpuinfo.contains("sve") {
        features.insert(CpuFeatures::SVE);
    }
}

// Fallback for other architectures - currently does nothing.
// Consider adding a generic fallback or specific implementations if other architectures
// expose features in /proc/cpuinfo in a parsable way.
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub(crate) fn detect_features_from_hashmap(
    _features: &mut CpuFeatures,
    _cpu_features_from_proc_cpuinfo: &HashSet<String>,
) {
    // No standard feature flags known to be parsed this way for other architectures.
}
