//! Linux-specific CPU information detection.
//!
//! This module implements the CPU detection logic for Linux systems. It gathers
//! information about the CPU vendor, model, features, and topology (sockets,
//! physical cores, logical processors, cache hierarchy) primarily by parsing
//! files from the sysfs filesystem (`/sys/devices/system/cpu/`) and supplementing
//! it with information from `/proc/cpuinfo` and direct `cpuid` calls on x86_64
//! architectures.
//!
//! The main entry point is the `detect_cpu_info()` function, which orchestrates
//! the detection process and constructs a [`CpuInfo`] struct.
//!
//! Internal helper modules like `cache`, `features`, `proc`, `sysfs`, and `topology`
//! handle specific aspects of the detection process.

use std::collections::HashMap;
use std::path::Path;

use crate::cpu::CacheInfo;
use crate::{CpuFeatures, CpuInfo, Error, Result, Vendor};

/// Type alias for a map from a global physical core ID to its L1 instruction cache,
/// L1 data cache, and L2 cache information.
///
/// The tuple stores `(Option<L1i_CacheInfo>, Option<L1d_CacheInfo>, Option<L2_CacheInfo>)`.

/// Type alias for a map from a global core ID to its L1d, L1i, and L2 cache information.
type CoreToCacheMap = HashMap<usize, (Option<CacheInfo>, Option<CacheInfo>, Option<CacheInfo>)>;

mod cache;
mod features;
mod proc;
mod sysfs;
mod topology;

use cache::detect_cache_topology;
use proc::detect_via_proc_cpuinfo;
use sysfs::collect_logical_processor_info;
use topology::{build_core_topology, calculate_core_statistics, construct_final_topology};

/// Detects basic CPU information (Vendor, Model Name, Features).
///
/// This function attempts to determine the CPU vendor, model name, and feature set.
///
/// On `x86_64` architectures, it prioritizes using the `cpuid` instruction via the
/// `common_x86_64::detect_via_cpuid` helper.
///
/// If `cpuid` is not available (e.g., non-x86_64) or fails to retrieve complete
/// information (vendor, model name, or features remain unknown/empty), it falls
/// back to parsing `/proc/cpuinfo` using the `proc::detect_via_proc_cpuinfo` helper.
///
/// # Returns
///
/// A `Result` containing a tuple:
/// - `Vendor`: The detected CPU vendor.
/// - `String`: The CPU model name.
/// - `CpuFeatures`: The detected CPU features.
///
/// Or an `Error` if detection fails critically (though fallbacks aim to prevent this).
fn detect_basic_cpu_info() -> Result<(Vendor, String, CpuFeatures)> {
    let mut vendor = Vendor::Unknown;
    let mut model_name = "Unknown".to_string();
    let mut features = CpuFeatures::default();

    // For x86_64, prioritize raw-cpuid
    #[cfg(target_arch = "x86_64")]
    crate::platform::common_x86_64::detect_via_cpuid(&mut vendor, &mut model_name, &mut features);

    // Fallback to /proc/cpuinfo for non-x86_64 or if CPUID failed
    if vendor == Vendor::Unknown || model_name == "Unknown" || features.is_empty() {
        detect_via_proc_cpuinfo(&mut vendor, &mut model_name, &mut features);
    }

    Ok((vendor, model_name, features))
}

/// Detects comprehensive CPU information on Linux systems.
///
/// This function orchestrates the CPU detection process on Linux by:
/// 1. Calling `detect_basic_cpu_info()` to get the vendor, model name, and basic features.
///    This internally uses `cpuid` (on x86_64) and falls back to `/proc/cpuinfo`.
/// 2. Parsing the sysfs filesystem (`/sys/devices/system/cpu/`) to:
///    - Identify online logical processors.
///    - Gather information about physical cores and their mapping to logical processors
///      (using helpers from the `sysfs` module).
///    - Determine socket information and the mapping of cores to sockets.
/// 3. Building the core topology, including identifying core types (Performance/Efficiency)
///    if possible, based on vendor and sysfs data (using helpers from the `topology` module).
/// 4. Detecting cache hierarchy (L1, L2, L3 caches) associated with cores and sockets
///    by parsing relevant sysfs entries (using helpers from the `cache` module).
/// 5. Consolidating all gathered information into a [`CpuInfo`] struct.
///
/// # Errors
///
/// Returns an `Error::Detection` if crucial sysfs paths (like `/sys/devices/system/cpu/`)
/// are not found, or if parsing essential information fails.
///
/// # Returns
///
/// A `Result` containing the populated [`CpuInfo`] struct with detailed CPU information,
/// or an `Error` if detection fails.
pub fn detect_cpu_info() -> Result<CpuInfo> {
    let cpu_base_path = Path::new("/sys/devices/system/cpu/");

    if !cpu_base_path.exists() {
        return Err(Error::Detection(format!(
            "CPU sysfs path not found: {:?}",
            cpu_base_path
        )));
    }

    // --- Vendor, Model, and Features ---
    let (vendor, model_name, features) = detect_basic_cpu_info()?;

    // --- Topology from /sys/devices/system/cpu ---
    let (online_lp_ids, physical_core_info_map, socket_to_core_ids_map) =
        collect_logical_processor_info(cpu_base_path)?;

    let total_logical_processors_count = online_lp_ids.len();

    // --- Building Core Topology ---
    let (
        physical_core_details,
        logical_to_global_physical_core_map,
        _,
        _,
        total_physical_cores_count,
    ) = build_core_topology(
        &vendor,
        &socket_to_core_ids_map,
        &mut physical_core_info_map.clone(),
    );

    // --- Cache Information ---
    let (core_to_cache_map, socket_to_l3_cache_map) = detect_cache_topology(
        cpu_base_path,
        &online_lp_ids,
        &physical_core_details,
        &mut logical_to_global_physical_core_map.clone(),
    );

    // --- Constructing Topology ---
    let sockets_vec = construct_final_topology(
        total_physical_cores_count,
        &socket_to_core_ids_map,
        &online_lp_ids,
        &physical_core_details,
        &core_to_cache_map,
        &socket_to_l3_cache_map,
    );

    // --- Calculating Core Statistics ---
    let (final_total_physical_cores, final_total_performance_cores, final_total_efficiency_cores) =
        calculate_core_statistics(&sockets_vec, &vendor);

    Ok(CpuInfo {
        vendor,
        model_name,
        features,
        sockets: sockets_vec.clone(),
        total_sockets: sockets_vec.len(),
        total_physical_cores: final_total_physical_cores,
        total_logical_processors: total_logical_processors_count,
        total_performance_cores: final_total_performance_cores,
        total_efficiency_cores: final_total_efficiency_cores,
    })
}
