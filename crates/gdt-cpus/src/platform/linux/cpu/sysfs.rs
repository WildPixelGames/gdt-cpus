//! Linux-specific CPU topology and core information parsing from sysfs.
//!
//! This module interfaces with the sysfs filesystem (typically mounted at `/sys`)
//! to discover the CPU's physical structure, including sockets, physical cores,
//! logical processors (threads), and core types (e.g., Performance or Efficiency cores).
//!
//! Key information is usually found under `/sys/devices/system/cpu/cpuX/topology/`,
//! where `X` is the logical processor ID. Files like `physical_package_id`, `core_id`,
//! and `core_type` are read to build a comprehensive map of the CPU topology.

use std::{

    collections::{HashMap, HashSet},
    path::Path,
};

use log::{debug, warn};

use crate::{
    CoreType, Error, Result,
    platform::linux::utils::{parse_cpu_range_list, read_sysfs_value},
};

/// Parses CPU topology details for online logical processors from sysfs.
///
/// Iterates through each online logical processor ID provided in `online_lp_ids`.
/// For each logical processor, it reads:
/// - `physical_package_id`: The ID of the socket the LP belongs to.
/// - `core_id`: The ID of the physical core within the socket the LP belongs to.
/// - `core_type`: The type of the core (e.g., "performance", "efficiency"). If not
///   available or unrecognized, defaults to `CoreType::Performance` after a warning.
///
/// This information is used to populate:
/// - `physical_core_info_map`: Maps `(socket_id, core_id_in_socket)` to `(CoreType, Vec<lp_id>)`.
/// - `socket_to_core_ids_map`: Maps `socket_id` to a `HashSet` of `core_id_in_socket` present
///   on that socket.
///
/// # Arguments
///
/// * `cpu_base_path`: The base path to the sysfs CPU directory (e.g., `/sys/devices/system/cpu/`).
/// * `online_lp_ids`: A `HashSet` of online logical processor IDs.
/// * `physical_core_info_map`: A mutable `HashMap` to store physical core information.
///   The key is `(socket_id, core_id_in_socket)`, and the value is `(CoreType, Vec<logical_processor_id>)`.
/// * `socket_to_core_ids_map`: A mutable `HashMap` to map socket IDs to a `HashSet` of
///   physical core IDs (relative to the socket) present on that socket.
pub(crate) fn parse_topology_from_sysfs(
    cpu_base_path: &Path,
    online_lp_ids: &HashSet<usize>,
    physical_core_info_map: &mut HashMap<(usize, usize), (CoreType, Vec<usize>)>,
    socket_to_core_ids_map: &mut HashMap<usize, HashSet<usize>>,
) {
    for &logical_processor_id in online_lp_ids.iter() {
        let path = cpu_base_path.join(format!("cpu{}", logical_processor_id));

        if !path.is_dir() {
            warn!(
                "Directory for online CPU {} not found at {:?}",
                logical_processor_id, path
            );
            continue;
        }

        let topology_path = path.join("topology");
        if !topology_path.exists() {
            warn!(
                "Topology directory missing for online CPU {}",
                logical_processor_id
            );
            continue;
        }

        let physical_package_id =
            read_sysfs_value::<usize>(&topology_path.join("physical_package_id")).unwrap_or(0);
        let core_id_in_package =
            read_sysfs_value::<usize>(&topology_path.join("core_id")).unwrap_or(0);

        socket_to_core_ids_map
            .entry(physical_package_id)
            .or_default()
            .insert(core_id_in_package);

        let core_type_path = topology_path.join("core_type");
        let mut core_type = CoreType::Unknown;
        if core_type_path.exists() {
            if let Ok(type_str) = std::fs::read_to_string(&core_type_path) {
                match type_str.trim().to_lowercase().as_str() {
                    "performance" | "0" => core_type = CoreType::Performance,
                    "efficiency" | "1" => core_type = CoreType::Efficiency,
                    _ => warn!(
                        "Unknown core_type value: '{}' for CPU {}",
                        type_str.trim(),
                        logical_processor_id
                    ),
                }
            } else {
                warn!("Failed to read core_type for CPU {}", logical_processor_id);
            }
        }

        if core_type == CoreType::Unknown {
            core_type = CoreType::Performance;
        }

        let core_key = (physical_package_id, core_id_in_package);
        let entry_data = physical_core_info_map
            .entry(core_key)
            .or_insert_with(|| (core_type, Vec::new()));
        if entry_data.0 == CoreType::Unknown && core_type != CoreType::Unknown {
            entry_data.0 = core_type;
        }
        entry_data.1.push(logical_processor_id);
    }
}

/// Collects comprehensive logical processor and core topology information from sysfs.
///
/// This is the main entry point for discovering CPU topology using sysfs. It performs
/// the following steps:
/// 1. Verifies the existence of `cpu_base_path`.
/// 2. Parses the list of online logical processors using `parse_cpu_range_list` (typically
///    from `/sys/devices/system/cpu/online`).
/// 3. If online LPs are found, it calls `parse_topology_from_sysfs` to populate
///    maps detailing physical core information (type, associated LPs) and socket-to-core mappings.
///
/// # Arguments
///
/// * `cpu_base_path`: The base path to the sysfs CPU directory (e.g., `/sys/devices/system/cpu/`).
///
/// # Returns
///
/// A `Result` containing a tuple with three elements:
/// 1. `HashSet<usize>`: A set of online logical processor IDs.
/// 2. `HashMap<(usize, usize), (CoreType, Vec<usize>)>`: The `physical_core_info_map`
///    populated by `parse_topology_from_sysfs`. Key: `(socket_id, core_id_in_socket)`,
///    Value: `(CoreType, Vec<logical_processor_id_on_this_core>)`.
/// 3. `HashMap<usize, HashSet<usize>>`: The `socket_to_core_ids_map` populated by
///    `parse_topology_from_sysfs`. Key: `socket_id`, Value: `HashSet<core_id_in_socket>`.
///
/// Returns an `Error::Detection` if `cpu_base_path` is not found or no online CPUs
/// are reported.
#[allow(clippy::type_complexity)]
pub(crate) fn collect_logical_processor_info(
    cpu_base_path: &Path,
) -> Result<(
    HashSet<usize>,
    HashMap<(usize, usize), (CoreType, Vec<usize>)>,
    HashMap<usize, HashSet<usize>>,
)> {
    if !cpu_base_path.exists() {
        return Err(Error::Detection(format!(
            "CPU sysfs path not found: {:?}",
            cpu_base_path
        )));
    }

    let online_lp_ids = parse_cpu_range_list(cpu_base_path)?;
    debug!("Online logical processors: {:?}", online_lp_ids);

    if online_lp_ids.is_empty() {
        return Err(Error::Detection(
            "No online CPUs reported by the system.".to_string(),
        ));
    }

    let mut physical_core_info_map = HashMap::new();
    let mut socket_to_core_ids_map = HashMap::new();

    parse_topology_from_sysfs(
        cpu_base_path,
        &online_lp_ids,
        &mut physical_core_info_map,
        &mut socket_to_core_ids_map,
    );

    Ok((
        online_lp_ids,
        physical_core_info_map,
        socket_to_core_ids_map,
    ))
}
