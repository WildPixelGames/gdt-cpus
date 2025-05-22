//! Final processing and construction of CPU topology for Linux.
//!
//! This module takes the raw and partially processed CPU information gathered from
//! sysfs (`sysfs.rs`), `/proc/cpuinfo` (`proc.rs`), and cache detection (`cache.rs`)
//! to build a structured and comprehensive representation of the CPU topology.
//!
//! It assigns global physical core IDs, resolves core types (Performance/Efficiency),
//! associates logical processors with their respective physical cores, and links
//! cache information (L1, L2, L3) to the appropriate cores and sockets.
//! The final output is typically a list of [`SocketInfo`] structs, each detailing
//! its cores and shared L3 cache.

use std::collections::{HashMap, HashSet};

use log::debug;

use crate::{CacheInfo, CoreInfo, CoreType, SocketInfo, Vendor};

use super::CoreToCacheMap;

/// Processes raw CPU topology data to assign global physical core IDs, determine core types,
/// and map logical processors to physical cores.
///
/// This function iterates through sockets and their cores (as discovered from sysfs, stored
/// in `socket_to_core_ids_map` and `physical_core_info_map`). It performs the following:
/// - Sorts socket IDs and core IDs for consistent processing order.
/// - For each physical core:
///   - If `CoreType` is `Unknown` and the vendor is known (Intel, AMD, Arm, Apple),
///     it defaults the core type to `CoreType::Performance`.
///   - Increments counters for detected Performance and Efficiency cores.
///   - Assigns a unique, sequential `global_physical_core_id`.
///   - Populates `physical_core_details` with a tuple:
///     `(global_phys_core_id, socket_id, CoreType, Vec<lp_id_on_this_core>)`.
///   - Populates `logical_to_global_physical_core_map` mapping each logical processor
///     ID on this core to the `global_physical_core_id`.
///
/// # Arguments
///
/// * `vendor`: The detected CPU [`Vendor`].
/// * `socket_to_core_ids_map`: Maps socket ID to a set of core IDs (relative to socket) on it.
/// * `physical_core_info_map`: Mutable map from `(socket_id, core_id_in_socket)` to `(CoreType, Vec<lp_id>)`.
///   `CoreType` might be updated here.
/// * `physical_core_details`: Mutable vector to store detailed information for each physical core.
/// * `logical_to_global_physical_core_map`: Mutable map to store LP ID to global physical core ID.
/// * `detected_performance_cores`: Mutable counter for P-cores.
/// * `detected_efficiency_cores`: Mutable counter for E-cores.
/// * `global_physical_core_id_counter`: Mutable counter used to assign global physical core IDs.
#[allow(clippy::too_many_arguments)]
pub(crate) fn process_cpu_topology(
    vendor: &Vendor,
    socket_to_core_ids_map: &HashMap<usize, HashSet<usize>>,
    physical_core_info_map: &mut HashMap<(usize, usize), (CoreType, Vec<usize>)>,
    physical_core_details: &mut Vec<(usize, usize, CoreType, Vec<usize>)>,
    logical_to_global_physical_core_map: &mut HashMap<usize, usize>,
    detected_performance_cores: &mut usize,
    detected_efficiency_cores: &mut usize,
    global_physical_core_id_counter: &mut usize,
) {
    let mut sorted_socket_ids: Vec<_> = socket_to_core_ids_map.keys().cloned().collect();
    sorted_socket_ids.sort_unstable();

    for socket_id_val in sorted_socket_ids {
        let mut sorted_core_ids_in_socket: Vec<_> = socket_to_core_ids_map[&socket_id_val]
            .iter()
            .cloned()
            .collect();
        sorted_core_ids_in_socket.sort_unstable();

        for core_id_in_socket_val in sorted_core_ids_in_socket {
            let core_key = (socket_id_val, core_id_in_socket_val);
            if let Some((core_type_val, lp_ids_val)) = physical_core_info_map.get_mut(&core_key) {
                if *core_type_val == CoreType::Unknown
                    && (*vendor == Vendor::Intel
                        || *vendor == Vendor::Amd
                        || *vendor == Vendor::Arm
                        || *vendor == Vendor::Apple)
                {
                    *core_type_val = CoreType::Performance;
                    debug!(
                        "Core type for socket {} core {} was Unknown, defaulted to Performance based on vendor/arch.",
                        socket_id_val, core_id_in_socket_val
                    );
                }

                match core_type_val {
                    CoreType::Performance => *detected_performance_cores += 1,
                    CoreType::Efficiency => *detected_efficiency_cores += 1,
                    CoreType::Unknown => { /* Stays unknown */ }
                }

                lp_ids_val.sort_unstable();

                physical_core_details.push((
                    *global_physical_core_id_counter,
                    socket_id_val,
                    *core_type_val,
                    lp_ids_val.clone(),
                ));
                for lp_id in lp_ids_val {
                    logical_to_global_physical_core_map
                        .insert(*lp_id, *global_physical_core_id_counter);
                }

                *global_physical_core_id_counter += 1;
            }
        }
    }
}

/// Constructs the final, structured CPU topology as a vector of [`SocketInfo`].
///
/// This function takes all processed information about physical cores, their types,
/// associated logical processors, and cache details to build a hierarchical
/// representation of the CPU, starting from sockets, then cores, then logical processors.
///
/// It ensures that only online logical processors are included in the final `CoreInfo`.
/// L1i, L1d, and L2 caches are assigned per core, and L3 cache per socket.
///
/// # Arguments
///
/// * `total_physical_cores_count`: Total number of physical cores detected. Used for a fallback if no sockets were explicitly found.
/// * `socket_to_core_ids_map`: Maps socket ID to a set of core IDs on it.
/// * `online_lp_ids`: A `HashSet` of online logical processor IDs.
/// * `physical_core_details`: A slice containing processed details for each physical core:
///   `(global_phys_core_id, socket_id, CoreType, Vec<lp_id_on_this_core>)`.
/// * `core_to_cache_map`: Map from global physical core ID to its L1/L2 cache information.
/// * `socket_to_l3_cache_map`: Map from socket ID to its L3 cache information.
///
/// # Returns
///
/// A `Vec<SocketInfo>` representing the complete CPU topology.
pub(crate) fn construct_final_topology(
    total_physical_cores_count: usize,
    socket_to_core_ids_map: &HashMap<usize, HashSet<usize>>,
    online_lp_ids: &HashSet<usize>,
    physical_core_details: &[(usize, usize, CoreType, Vec<usize>)],
    core_to_cache_map: &CoreToCacheMap,
    socket_to_l3_cache_map: &HashMap<usize, Option<CacheInfo>>,
) -> Vec<SocketInfo> {
    let mut sockets_vec = Vec::new();

    let mut final_sorted_socket_ids_for_topology: Vec<_> =
        socket_to_core_ids_map.keys().cloned().collect();

    if final_sorted_socket_ids_for_topology.is_empty() && total_physical_cores_count > 0 {
        final_sorted_socket_ids_for_topology.push(0);
    }

    final_sorted_socket_ids_for_topology.sort_unstable();

    for socket_id_val in final_sorted_socket_ids_for_topology {
        let mut cores_for_this_socket_vec = Vec::new();
        for (global_core_id, s_id, core_type, lp_ids) in
            physical_core_details
                .iter()
                .filter(|(_, _, _, core_lp_ids)| {
                    core_lp_ids
                        .iter()
                        .any(|lp_id| online_lp_ids.contains(lp_id))
                })
        {
            if *s_id == socket_id_val {
                let (l1i, l1d, l2) = core_to_cache_map
                    .get(global_core_id)
                    .cloned()
                    .unwrap_or((None, None, None));
                let online_lp_ids_for_core: Vec<usize> = lp_ids
                    .iter()
                    .cloned()
                    .filter(|lp_id| online_lp_ids.contains(lp_id))
                    .collect();
                if !online_lp_ids_for_core.is_empty() {
                    cores_for_this_socket_vec.push(CoreInfo {
                        id: *global_core_id,
                        socket_id: *s_id,
                        core_type: *core_type,
                        logical_processor_ids: online_lp_ids_for_core,
                        l1_instruction_cache: l1i,
                        l1_data_cache: l1d,
                        l2_cache: l2,
                    });
                }
            }
        }
        if !cores_for_this_socket_vec.is_empty() {
            cores_for_this_socket_vec.sort_by_key(|c| c.id);
            sockets_vec.push(SocketInfo {
                id: socket_id_val,
                cores: cores_for_this_socket_vec,
                l3_cache: socket_to_l3_cache_map
                    .get(&socket_id_val)
                    .cloned()
                    .unwrap_or(None),
            });
        }
    }

    sockets_vec
}

/// Builds the initial core topology by processing sysfs data.
///
/// This function orchestrates the initial phase of topology construction. It initializes
/// data structures and calls `process_cpu_topology` to populate them. This step
/// focuses on identifying physical cores, their types, associated logical processors,
/// and assigning global IDs.
///
/// If no specific P-core/E-core distinction is found after processing, and there are
/// physical cores, it assumes all physical cores are `CoreType::Performance`.
///
/// # Arguments
///
/// * `vendor`: The detected CPU [`Vendor`].
/// * `socket_to_core_ids_map`: Maps socket ID to a set of core IDs (relative to socket) on it.
/// * `physical_core_info_map`: Mutable map from `(socket_id, core_id_in_socket)` to `(CoreType, Vec<lp_id>)`.
///
/// # Returns
///
/// A tuple containing:
/// - `Vec<(usize, usize, CoreType, Vec<usize>)>`: The `physical_core_details`.
/// - `HashMap<usize, usize>`: The `logical_to_global_physical_core_map`.
/// - `usize`: Count of `detected_performance_cores`.
/// - `usize`: Count of `detected_efficiency_cores`.
/// - `usize`: `total_physical_cores_count` (derived from the global ID counter).
#[allow(clippy::type_complexity)]
pub(crate) fn build_core_topology(
    vendor: &Vendor,
    socket_to_core_ids_map: &HashMap<usize, HashSet<usize>>,
    physical_core_info_map: &mut HashMap<(usize, usize), (CoreType, Vec<usize>)>,
) -> (
    Vec<(usize, usize, CoreType, Vec<usize>)>,
    HashMap<usize, usize>,
    usize,
    usize,
    usize,
) {
    let mut detected_performance_cores = 0;
    let mut detected_efficiency_cores = 0;
    let mut global_physical_core_id_counter = 0;
    let mut physical_core_details = Vec::new();
    let mut logical_to_global_physical_core_map = HashMap::new();

    process_cpu_topology(
        vendor,
        socket_to_core_ids_map,
        physical_core_info_map,
        &mut physical_core_details,
        &mut logical_to_global_physical_core_map,
        &mut detected_performance_cores,
        &mut detected_efficiency_cores,
        &mut global_physical_core_id_counter,
    );

    let total_physical_cores_count = global_physical_core_id_counter;

    if detected_performance_cores == 0
        && detected_efficiency_cores == 0
        && total_physical_cores_count > 0
    {
        debug!(
            "No P/E core distinction found, assuming all {} physical cores are Performance cores.",
            total_physical_cores_count
        );
        detected_performance_cores = total_physical_cores_count;
    }

    (
        physical_core_details,
        logical_to_global_physical_core_map,
        detected_performance_cores,
        detected_efficiency_cores,
        total_physical_cores_count,
    )
}

/// Calculates final core statistics based on the fully constructed [`SocketInfo`] vector.
///
/// This function iterates through the final topology (`sockets_vec`) to count the
/// total number of physical cores, Performance cores, and Efficiency cores.
/// It includes a fallback logic: if a core's type is `Unknown`, its type is
/// inferred based on the CPU vendor (Intel/AMD defaults to Performance, Apple/Arm
/// might have specific logic or also default to Performance if no E-cores are present).
///
/// # Arguments
///
/// * `sockets_vec`: A slice of [`SocketInfo`] structs representing the final CPU topology.
/// * `vendor`: The detected CPU [`Vendor`].
///
/// # Returns
///
/// A tuple `(final_total_physical_cores, final_performance_cores, final_efficiency_cores)`.
pub(crate) fn calculate_core_statistics(
    sockets_vec: &[SocketInfo],
    vendor: &Vendor,
) -> (usize, usize, usize) {
    let final_total_physical_cores = sockets_vec.iter().map(|s| s.cores.len()).sum();

    let resolve_core_type = |core: &CoreInfo| -> CoreType {
        match core.core_type {
            CoreType::Performance => CoreType::Performance,
            CoreType::Efficiency => CoreType::Efficiency,
            CoreType::Unknown
                if *vendor == Vendor::Intel
                    || *vendor == Vendor::Amd
                    || *vendor == Vendor::Arm
                    || *vendor == Vendor::Apple =>
            {
                CoreType::Performance
            }
            _ => CoreType::Unknown,
        }
    };

    let (mut final_total_performance_cores, final_total_efficiency_cores) = sockets_vec
        .iter()
        .flat_map(|socket| &socket.cores)
        .filter(|core| !core.logical_processor_ids.is_empty())
        .map(resolve_core_type)
        .fold((0, 0), |(perf, eff), core_type| match core_type {
            CoreType::Performance => (perf + 1, eff),
            CoreType::Efficiency => (perf, eff + 1),
            CoreType::Unknown => (perf, eff),
        });

    if final_total_performance_cores == 0
        && final_total_efficiency_cores == 0
        && final_total_physical_cores > 0
    {
        final_total_performance_cores = final_total_physical_cores;
    }

    (
        final_total_physical_cores,
        final_total_performance_cores,
        final_total_efficiency_cores,
    )
}
