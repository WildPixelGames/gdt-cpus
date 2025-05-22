//! Linux-specific CPU cache detection logic.
//!
//! This module is responsible for discovering and parsing CPU cache information
//! from the sysfs filesystem (typically under `/sys/devices/system/cpu/cpu*/cache/`).
//! It identifies L1 (Data, Instruction, Unified), L2, and L3 caches, their sizes,
//! line sizes, and their association with physical cores and sockets.
//!
//! The primary function exposed by this module is `detect_cache_topology`, which
//! orchestrates the discovery process. Helper functions parse specific cache
//! levels and process the directory structure within sysfs.

use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use log::{debug, warn};

use crate::{
    CacheInfo, CacheLevel, CacheType, CoreType, platform::linux::utils::read_sysfs_value,
};

use super::CoreToCacheMap;

/// Parses L1 cache information for a specific physical core and updates the cache map.
///
/// L1 cache can be instruction-specific (L1i), data-specific (L1d), or unified.
/// This function handles these cases and assigns the `current_cache_info`
/// to the appropriate L1 slot (L1i or L1d) for the `global_phys_core_id`.
/// If the cache is unified, it's assigned to both L1i and L1d slots if they are empty.
///
/// # Arguments
///
/// * `global_phys_core_id`: The global ID of the physical core this L1 cache belongs to.
/// * `cache_type`: The type of L1 cache (`Instruction`, `Data`, or `Unified`).
/// * `current_cache_info`: The [`CacheInfo`] struct describing the L1 cache.
/// * `core_to_cache_map`: A mutable reference to the map storing cache information per core.
///   The tuple in the map represents `(Option<L1i>, Option<L1d>, Option<L2>)`.
pub(crate) fn parse_cache_l1(
    global_phys_core_id: usize,
    cache_type: CacheType,
    current_cache_info: CacheInfo,
    core_to_cache_map: &mut CoreToCacheMap,
) {
    let entry = core_to_cache_map
        .entry(global_phys_core_id)
        .or_insert((None, None, None));
    if cache_type == CacheType::Instruction && entry.0.is_none() {
        debug!(
            "      Assigning L1i to GPC {}: {:?}",
            global_phys_core_id, current_cache_info
        );
        entry.0 = Some(current_cache_info);
    } else if cache_type == CacheType::Data && entry.1.is_none() {
        debug!(
            "      Assigning L1d to GPC {}: {:?}",
            global_phys_core_id, current_cache_info
        );
        entry.1 = Some(current_cache_info);
    } else if cache_type == CacheType::Unified {
        if entry.0.is_none() {
            debug!(
                "      Assigning Unified L1 as L1i to GPC {}: {:?}",
                global_phys_core_id, current_cache_info
            );
            entry.0 = Some(current_cache_info);
        }
        if entry.1.is_none() {
            debug!(
                "      Assigning Unified L1 as L1d to GPC {}: {:?}",
                global_phys_core_id, current_cache_info
            );
            entry.1 = Some(current_cache_info);
        }
    }
}

/// Parses L2 cache information for a specific physical core and updates the cache map.
///
/// This function assigns the `current_cache_info` to the L2 slot for the
/// `global_phys_core_id` if it's not already set.
///
/// # Arguments
///
/// * `global_phys_core_id`: The global ID of the physical core this L2 cache belongs to.
/// * `current_cache_info`: The [`CacheInfo`] struct describing the L2 cache.
/// * `core_to_cache_map`: A mutable reference to the map storing cache information per core.
///   The tuple in the map represents `(Option<L1i>, Option<L1d>, Option<L2>)`.
pub(crate) fn parse_cache_l2(
    global_phys_core_id: usize,
    current_cache_info: CacheInfo,
    core_to_cache_map: &mut CoreToCacheMap,
) {
    let l2_entry = core_to_cache_map.entry(global_phys_core_id).or_default();
    if l2_entry.2.is_none() {
        debug!(
            "      Assigning L2 to GPC {}: {:?}",
            global_phys_core_id, current_cache_info
        );
        l2_entry.2 = Some(current_cache_info);
    }
}

/// Parses L3 cache information for a specific socket and updates the cache map.
///
/// L3 cache is typically shared per socket. This function assigns the
/// `current_cache_info` to the L3 slot for the `socket_id_of_lp` if it's
/// not already set.
///
/// # Arguments
///
/// * `current_cache_info`: The [`CacheInfo`] struct describing the L3 cache.
/// * `socket_id_of_lp`: The ID of the socket this L3 cache belongs to.
/// * `socket_to_l3_cache_map`: A mutable reference to the map storing L3 cache information per socket.
pub(crate) fn parse_cache_l3(
    current_cache_info: CacheInfo,
    socket_id_of_lp: usize,
    socket_to_l3_cache_map: &mut HashMap<usize, Option<CacheInfo>>,
) {
    let l3_entry = socket_to_l3_cache_map.entry(socket_id_of_lp).or_default();
    if l3_entry.is_none() {
        debug!(
            "      Assigning L3 to Socket {}: {:?}",
            socket_id_of_lp, current_cache_info
        );
        *l3_entry = Some(current_cache_info);
    }
}

/// Processes a cache index directory (e.g., `/sys/devices/system/cpu/cpuX/cache/indexY/`)
/// for a specific logical processor.
///
/// This function reads cache properties (level, type, size, line size) from the
/// sysfs files within a given cache index directory. It dedupes cache instances
/// using the `unique_caches` map to avoid redundant `CacheInfo` objects for
/// shared caches. It then calls `parse_cache_l1`, `parse_cache_l2`, or `parse_cache_l3`
/// based on the detected cache level.
///
/// # Arguments
///
/// * `cpu_cache_path`: Path to the logical processor's cache directory (e.g., `/sys/devices/system/cpu/cpu0/cache/`).
/// * `logical_processor_id`: The ID of the logical processor whose cache is being processed.
/// * `global_phys_core_id`: The global physical core ID that this logical processor belongs to.
/// * `socket_id_of_lp`: The socket ID that this logical processor belongs to.
/// * `core_to_cache_map`: Mutable map to store L1/L2 cache info per physical core.
/// * `socket_to_l3_cache_map`: Mutable map to store L3 cache info per socket.
/// * `unique_caches`: Mutable map to store unique `CacheInfo` instances to avoid duplication.
///
/// # Returns
///
/// Returns `true` if any cache information was successfully found and processed
/// within this index directory, `false` otherwise.
pub(crate) fn process_cache_index(
    cpu_cache_path: &Path,
    logical_processor_id: usize,
    global_phys_core_id: usize,
    socket_id_of_lp: usize,
    core_to_cache_map: &mut CoreToCacheMap,
    socket_to_l3_cache_map: &mut HashMap<usize, Option<CacheInfo>>,
    unique_caches: &mut HashMap<(CacheLevel, CacheType, u64, usize), CacheInfo>,
) -> bool {
    let mut found_any_cache_info = false;

    let cache_indices = if let Ok(cache_indices) = std::fs::read_dir(cpu_cache_path) {
        cache_indices
    } else {
        debug!(
            "Could not read cache directory for LP {}: {:?}",
            logical_processor_id, cpu_cache_path
        );
        return found_any_cache_info;
    };

    for cache_index_entry in cache_indices.flatten() {
        let cache_path = cache_index_entry.path();
        let index_name = cache_index_entry.file_name().to_string_lossy().into_owned();

        if !(cache_path.is_dir() && index_name.starts_with("index")) {
            debug!(
                "  Skipping non-directory or non-index entry: {:?} index name: {:?}",
                cache_path, index_name
            );
            continue;
        }

        debug!("  Processing cache index dir: {:?}", index_name);

        let level_path = cache_path.join("level");
        let type_path = cache_path.join("type");
        let size_path = cache_path.join("size");
        let line_size_path = cache_path.join("coherency_line_size");

        if !level_path.exists()
            || !type_path.exists()
            || !size_path.exists()
            || !line_size_path.exists()
        {
            debug!(
                "    Cache index {:?} for LP {} is missing one or more required files. Skipping this index.",
                index_name, logical_processor_id
            );
            continue;
        }

        let level = read_sysfs_value::<u32>(&level_path)
            .map(CacheLevel::from)
            .unwrap_or(CacheLevel::Unknown);
        let type_str = std::fs::read_to_string(&type_path).unwrap_or_default();
        let cache_type = match type_str.trim() {
            "Data" => CacheType::Data,
            "Instruction" => CacheType::Instruction,
            "Unified" => CacheType::Unified,
            _ => CacheType::Unknown,
        };

        // Corrected cache size parsing to handle "K" suffix
        let size_str_with_k = std::fs::read_to_string(&size_path).unwrap_or_default();
        let size_kib = size_str_with_k
            .trim()
            .trim_end_matches('K')
            .trim_end_matches('k')
            .parse::<u64>()
            .unwrap_or(0);
        let size_bytes = size_kib * 1024;

        let line_size_bytes = read_sysfs_value::<usize>(&line_size_path).unwrap_or(0);

        debug!(
            "    LP {}: Index {:?}: Level {:?}, Type {:?}, Size {} KiB ({} bytes), Line {} bytes",
            logical_processor_id,
            index_name,
            level,
            cache_type,
            size_kib,
            size_bytes,
            line_size_bytes
        );

        if size_bytes == 0 || line_size_bytes == 0 {
            debug!(
                "    Skipping cache index {:?} for LP {} due to zero size or line size.",
                index_name, logical_processor_id
            );
            continue;
        }

        found_any_cache_info = true;

        let cache_key = (level, cache_type, size_bytes, line_size_bytes);
        let current_cache_info = *unique_caches.entry(cache_key).or_insert_with(|| {
            debug!(
                "      New unique cache identified: {:?}, {:?}, {} bytes, line {} bytes",
                level, cache_type, size_bytes, line_size_bytes
            );
            CacheInfo {
                level,
                cache_type,
                size_bytes,
                line_size_bytes,
            }
        });

        match level {
            CacheLevel::L1 => parse_cache_l1(
                global_phys_core_id,
                cache_type,
                current_cache_info,
                core_to_cache_map,
            ),
            CacheLevel::L2 => {
                parse_cache_l2(global_phys_core_id, current_cache_info, core_to_cache_map)
            }
            CacheLevel::L3 => {
                parse_cache_l3(current_cache_info, socket_id_of_lp, socket_to_l3_cache_map)
            }
            _ => debug!("    Ignoring cache with unhandled level: {:?}", level),
        }
    }

    found_any_cache_info
}

/// Iterates through all online logical processors, determines their physical core and
/// socket affiliations, and processes their cache information using `process_cache_index`.
///
/// This function serves as an intermediate step, looping through each logical processor
/// identified in `online_lp_ids` and delegating the actual sysfs cache directory parsing
/// to `process_cache_index`. It also handles mapping logical processors to their
/// global physical core IDs and socket IDs.
///
/// # Arguments
///
/// * `cpu_base_path`: Base path to the sysfs CPU devices (e.g., `/sys/devices/system/cpu/`).
/// * `online_lp_ids`: A set of IDs for all online logical processors.
/// * `physical_core_details`: A slice containing details for each physical core,
///   including its global ID, socket ID, core type, and a list of its logical processor IDs.
/// * `core_to_cache_map`: Mutable map to store L1/L2 cache info per physical core.
/// * `socket_to_l3_cache_map`: Mutable map to store L3 cache info per socket.
/// * `logical_to_global_physical_core_map`: Map from logical processor ID to global physical core ID.
///
/// # Returns
///
/// Returns `true` if cache information was found for at least one logical processor,
/// `false` otherwise.
pub(crate) fn process_cache_for_logical_processor(
    cpu_base_path: &Path,
    online_lp_ids: &HashSet<usize>,
    physical_core_details: &[(usize, usize, CoreType, Vec<usize>)], // global_phys_core_id, socket_id, core_type, lps_on_this_core
    core_to_cache_map: &mut CoreToCacheMap,
    socket_to_l3_cache_map: &mut HashMap<usize, Option<CacheInfo>>,
    logical_to_global_physical_core_map: &mut HashMap<usize, usize>,
) -> bool {
    let mut found_any_cache_info = false;

    let mut unique_caches: HashMap<(CacheLevel, CacheType, u64, usize), CacheInfo> = HashMap::new();

    for logical_processor_id in online_lp_ids.iter() {
        let cpu_cache_path = cpu_base_path
            .join(format!("cpu{}", logical_processor_id))
            .join("cache");
        debug!(
            "Checking cache for online LP {}: path {:?}",
            logical_processor_id, cpu_cache_path
        );
        if !cpu_cache_path.exists() {
            debug!(
                "Cache path for online LP {} does not exist. Skipping.",
                logical_processor_id
            );
            continue;
        }

        let global_phys_core_id =
            match logical_to_global_physical_core_map.get(logical_processor_id) {
                Some(id) => *id,
                None => {
                    debug!(
                        "Online LP {} not mapped to any physical core. Skipping cache.",
                        logical_processor_id
                    );
                    continue;
                }
            };

        let socket_id_of_lp = physical_core_details.iter()
            .find(|(gpc_id, _, _, _)| *gpc_id == global_phys_core_id)
            .map(|(_, sid, _, _)| *sid)
            .unwrap_or_else(|| {
                warn!("Could not determine socket ID for online LP {} (phys core {}). Defaulting to 0 for cache.", logical_processor_id, global_phys_core_id);
                0
            });

        found_any_cache_info |= process_cache_index(
            &cpu_cache_path,
            *logical_processor_id,
            global_phys_core_id,
            socket_id_of_lp,
            core_to_cache_map,
            socket_to_l3_cache_map,
            &mut unique_caches,
        );
    }

    found_any_cache_info
}

/// Detects the complete cache topology for the CPU on Linux.
///
/// This is the main entry point for cache detection within this module. It initializes
/// the necessary data structures and calls `process_cache_for_logical_processor`
/// to populate them by iterating through sysfs.
///
/// # Arguments
///
/// * `cpu_base_path`: Base path to the sysfs CPU devices (e.g., `/sys/devices/system/cpu/`).
/// * `online_lp_ids`: A set of IDs for all online logical processors.
/// * `physical_core_details`: A slice containing details for each physical core.
/// * `logical_to_global_physical_core_map`: Map from logical processor ID to global physical core ID.
///
/// # Returns
///
/// A tuple containing:
/// - `CoreToCacheMap`: Map from global physical core ID to its L1/L2 cache info.
/// - `HashMap<usize, Option<CacheInfo>>`: Map from socket ID to its L3 cache info.
pub(crate) fn detect_cache_topology(
    cpu_base_path: &Path,
    online_lp_ids: &HashSet<usize>,
    physical_core_details: &[(usize, usize, CoreType, Vec<usize>)],
    logical_to_global_physical_core_map: &mut HashMap<usize, usize>,
) -> (CoreToCacheMap, HashMap<usize, Option<CacheInfo>>) {
    debug!("Starting cache detection for online LPs...");
    let mut core_to_cache_map = HashMap::new();
    let mut socket_to_l3_cache_map = HashMap::new();

    let found_any_cache_info = process_cache_for_logical_processor(
        cpu_base_path,
        online_lp_ids,
        physical_core_details,
        &mut core_to_cache_map,
        &mut socket_to_l3_cache_map,
        logical_to_global_physical_core_map,
    );

    if !found_any_cache_info && !online_lp_ids.is_empty() {
        warn!(
            "No valid cache information found in sysfs for any online logical processors. Cache data may be unavailable in this environment (e.g., container/VM)."
        );
    }

    debug!(
        "Cache detection finished. Core map: {:?}, Socket map: {:?}",
        core_to_cache_map, socket_to_l3_cache_map
    );

    (core_to_cache_map, socket_to_l3_cache_map)
}
