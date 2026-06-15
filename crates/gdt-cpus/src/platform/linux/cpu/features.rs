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
        // NOTE(x86): the CRC32 instructions ARE part of SSE4.2 - there is no
        // separate cpuinfo flag on x86 ("crc32" is the ARM flag name), so
        // CRC32 must be set here or it is never set on x86 at all.
        features.insert(CpuFeatures::CRC32);
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
    if cpu_features_from_proc_cpuinfo.contains("popcnt") {
        features.insert(CpuFeatures::POPCNT);
    }
    if cpu_features_from_proc_cpuinfo.contains("bmi1") {
        features.insert(CpuFeatures::BMI1);
    }
    if cpu_features_from_proc_cpuinfo.contains("bmi2") {
        features.insert(CpuFeatures::BMI2);
    }
    if cpu_features_from_proc_cpuinfo.contains("f16c") {
        features.insert(CpuFeatures::F16C);
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
    if cpu_features_from_proc_cpuinfo.contains("fphp")
        || cpu_features_from_proc_cpuinfo.contains("asimdhp")
    {
        features.insert(CpuFeatures::FP16);
    }
    if cpu_features_from_proc_cpuinfo.contains("asimddp") {
        features.insert(CpuFeatures::DOTPROD);
    }
    if cpu_features_from_proc_cpuinfo.contains("i8mm") {
        features.insert(CpuFeatures::I8MM);
    }
    if cpu_features_from_proc_cpuinfo.contains("bf16") {
        features.insert(CpuFeatures::BF16);
    }
    if cpu_features_from_proc_cpuinfo.contains("sve2") {
        features.insert(CpuFeatures::SVE2);
    }
    if cpu_features_from_proc_cpuinfo.contains("atomics") {
        features.insert(CpuFeatures::LSE);
    }
    if cpu_features_from_proc_cpuinfo.contains("jscvt") {
        features.insert(CpuFeatures::JSCVT);
    }
    if cpu_features_from_proc_cpuinfo.contains("lrcpc") {
        features.insert(CpuFeatures::LRCPC);
    }
    if cpu_features_from_proc_cpuinfo.contains("pmull") {
        features.insert(CpuFeatures::PMULL);
    }
    if cpu_features_from_proc_cpuinfo.contains("asimdrdm") {
        features.insert(CpuFeatures::RDM);
    }
    if cpu_features_from_proc_cpuinfo.contains("asimdfhm") {
        features.insert(CpuFeatures::FHM);
    }
    if cpu_features_from_proc_cpuinfo.contains("fcma") {
        features.insert(CpuFeatures::FCMA);
    }
    if cpu_features_from_proc_cpuinfo.contains("lse2") {
        features.insert(CpuFeatures::LSE2);
    }
    if cpu_features_from_proc_cpuinfo.contains("ilrcpc") {
        features.insert(CpuFeatures::LRCPC2);
    }
    if cpu_features_from_proc_cpuinfo.contains("sm3") {
        features.insert(CpuFeatures::SM3);
    }
    if cpu_features_from_proc_cpuinfo.contains("sm4") {
        features.insert(CpuFeatures::SM4);
    }
    if cpu_features_from_proc_cpuinfo.contains("sveaes") {
        features.insert(CpuFeatures::SVEAES);
    }
    if cpu_features_from_proc_cpuinfo.contains("svepmull") {
        features.insert(CpuFeatures::SVEPMULL);
    }
    if cpu_features_from_proc_cpuinfo.contains("svebitperm") {
        features.insert(CpuFeatures::SVEBITPERM);
    }
    if cpu_features_from_proc_cpuinfo.contains("svesha3") {
        features.insert(CpuFeatures::SVESHA3);
    }
    if cpu_features_from_proc_cpuinfo.contains("svesm4") {
        features.insert(CpuFeatures::SVESM4);
    }
    if cpu_features_from_proc_cpuinfo.contains("svei8mm") {
        features.insert(CpuFeatures::SVEI8MM);
    }
    if cpu_features_from_proc_cpuinfo.contains("svebf16") {
        features.insert(CpuFeatures::SVEBF16);
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
