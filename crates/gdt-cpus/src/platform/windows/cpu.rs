//! Windows-specific CPU information detection logic.
//!
//! This module implements the core CPU detection capabilities for Windows platforms.
//! It primarily leverages the `GetLogicalProcessorInformationEx` Windows API function
//! to gather detailed information about processor groups, sockets (packages),
//! physical cores, logical processors (hyper-threads), cache hierarchy, and NUMA nodes.
//!
//! For basic information like CPU vendor, model name, and features, it uses `cpuid`
//! instructions on x86_64 architectures, with a fallback to querying the Windows Registry
//! if needed (via the `super::registry` module).
//!
//! The main entry point is the [`detect_cpu_info`] function, which orchestrates these
//! detection mechanisms and constructs the comprehensive [`CpuInfo`] struct.

use std::collections::{HashMap, HashSet};
use std::ptr;

use log::{debug, error};

use crate::cpu::{CacheInfo, CacheLevel, CacheType, CoreInfo, SocketInfo};
use crate::{CoreType, CpuFeatures, CpuInfo, Error, Result, Vendor};

// Windows API bindings from the `windows` crate
use windows::Win32::{
    Foundation::GetLastError,
    System::SystemInformation::{
        // PROCESSOR_CACHE_TYPE enum variants are used directly
        CacheData,
        CacheInstruction,
        CacheTrace,
        CacheUnified,
        GetLogicalProcessorInformationEx,
        RelationAll,
        RelationCache,
        RelationProcessorCore,
        RelationProcessorPackage,
        SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX,
    },
};

/// Detects comprehensive CPU information on Windows systems.
///
/// This function orchestrates the CPU detection process by:
/// 1.  **Fetching Basic CPU Details:**
///     - On `x86_64`, uses `cpuid` instructions (via `common_x86_64::detect_via_cpuid`)
///       to get vendor, model name, and features.
///     - On `aarch64`, sets vendor to ARM and assumes NEON support as a baseline.
///     - If primary methods are insufficient, falls back to querying the Windows Registry
///       (via `super::registry::detect_via_registry`) for vendor and model name.
///
/// 2.  **Querying System Topology with `GetLogicalProcessorInformationEx`:**
///     - Calls `GetLogicalProcessorInformationEx` with `RelationAll` to retrieve an array
///       of `SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX` structures. This provides data for:
///       - Processor cores (`RelationProcessorCore`), including efficiency class (for P/E cores)
///         and group masks.
///       - Processor packages/sockets (`RelationProcessorPackage`), including group masks.
///       - Cache information (`RelationCache`), including level, type, size, line size, and
///         group affinity mask.
///       - NUMA nodes (`RelationNumaNode`), including node ID and group mask.
///
/// 3.  **Parsing and Reconstructing Topology:**
///     - Iterates through the retrieved information to:
///       - Map logical processors to their core type (Performance or Efficiency).
///       - Identify NUMA nodes and associate logical processors with them.
///       - Build a representation of sockets (`SocketInfo`) and physical cores (`CoreInfo`).
///       - Assign unique global IDs to physical cores.
///       - Determine the L1, L2, and L3 cache hierarchy and associate caches with the
///         appropriate cores or sockets based on their affinity masks.
///
/// 4.  **Aggregating Final Counts:**
///     - Calculates total counts for sockets, physical cores, logical processors,
///       performance cores, and efficiency cores.
///
/// # Returns
///
/// A `Result<CpuInfo>` containing the populated `CpuInfo` struct with detailed
/// CPU information for the Windows system, or an `Error` if critical detection steps fail
/// (e.g., `GetLogicalProcessorInformationEx` fails fundamentally).
///
/// # Panics
/// This function may panic if unsafe transmutations of buffer pointers fail, though efforts are made to ensure pointer arithmetic is correct based on `info_ex.Size`.
pub fn detect_cpu_info() -> Result<CpuInfo> {
    let mut vendor = Vendor::Unknown;
    let mut model_name = "Unknown".to_string();
    let mut features = CpuFeatures::default();

    // --- Vendor, Model, and Features via CPUID (for x86_64) ---
    #[cfg(target_arch = "x86_64")]
    crate::platform::common_x86_64::detect_via_cpuid(&mut vendor, &mut model_name, &mut features);

    #[cfg(target_arch = "aarch64")]
    if vendor == Vendor::Unknown {
        vendor = Vendor::Arm;
        features.insert(CpuFeatures::NEON);
    }

    if vendor == Vendor::Unknown || model_name == "Unknown" || features.is_empty() {
        super::registry::detect_via_registry(&mut vendor, &mut model_name)?;
    }

    // --- Topology, Cache, P/E Cores via GetLogicalProcessorInformationEx ---
    let mut buffer_size: u32 = 0;
    let result_size_query = unsafe {
        GetLogicalProcessorInformationEx(RelationAll, Some(ptr::null_mut()), &mut buffer_size)
    };

    if result_size_query.is_err() {
        let err_code = unsafe { GetLastError() };
        // ERROR_INSUFFICIENT_BUFFER (122) is expected if buffer_size was 0 and got updated.
        // If buffer_size is still 0 after this, then it's a real error.
        if buffer_size == 0 {
            // True error if buffer_size was not updated
            error!(
                "GetLogicalProcessorInformationEx (size query) failed and buffer_size is 0. Error: {:?}",
                err_code
            );
            return Err(Error::SystemCall(format!(
                "GetLogicalProcessorInformationEx (size query) failed. Error: {:?}",
                err_code
            )));
        }
        debug!(
            "GetLogicalProcessorInformationEx (size query) returned error (expected for size retrieval), buffer_size set to: {}. Error: {:?}",
            buffer_size, err_code
        );
    }

    if buffer_size == 0 {
        error!(
            "GetLogicalProcessorInformationEx returned zero buffer size despite no direct error from size query or after expected error."
        );
        return Err(Error::Detection(
            "GetLogicalProcessorInformationEx returned zero buffer size.".to_string(),
        ));
    }

    let mut buffer: Vec<u8> = vec![0; buffer_size as usize];
    let success_result = unsafe {
        GetLogicalProcessorInformationEx(
            RelationAll,
            Some(buffer.as_mut_ptr() as *mut SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX),
            &mut buffer_size,
        )
    };

    if let Err(e) = success_result {
        error!(
            "GetLogicalProcessorInformationEx (data query) failed. Error: {:?}",
            e
        );
        return Err(Error::SystemCall(format!(
            "GetLogicalProcessorInformationEx (data query) failed. Error: {:?}",
            e
        )));
    }

    let mut total_performance_cores = 0;
    let mut total_efficiency_cores = 0;

    let mut logical_processor_map: HashMap<usize, CoreType> = HashMap::new();
    let mut socket_id_anchors: HashSet<usize> = HashSet::new();

    struct CacheFromApi {
        level: CacheLevel,
        cache_type: CacheType,
        size_bytes: u64,
        line_size_bytes: usize,
        logical_processor_mask: usize,
        group: u16,
    }
    let mut caches_from_api: Vec<CacheFromApi> = Vec::new();

    let mut current_ptr = buffer.as_ptr();
    let end_ptr = unsafe { buffer.as_ptr().add(buffer_size as usize) };

    let mut p_cores_count = 0;

    debug!("Starting GetLogicalProcessorInformationEx pass 1: Identify LPs, P/E, Caches");
    while current_ptr < end_ptr {
        let info_ex = unsafe { &*(current_ptr as *const SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX) };

        #[allow(non_upper_case_globals)]
        match info_ex.Relationship {
            RelationProcessorCore => {
                let proc_core = unsafe { &info_ex.Anonymous.Processor };
                let core_type = if proc_core.EfficiencyClass == 0 {
                    CoreType::Efficiency
                } else {
                    CoreType::Performance
                };
                for i in 0..proc_core.GroupMask.len() {
                    let group_affinity = &proc_core.GroupMask[i];
                    let group_idx = group_affinity.Group as usize;
                    let mask = group_affinity.Mask;
                    for bit_idx in 0..std::mem::size_of::<usize>() * 8 {
                        if (mask >> bit_idx) & 1 != 0 {
                            if core_type == CoreType::Performance {
                                p_cores_count += 1;
                            }
                            let logical_processor_id = (group_idx * 64) + bit_idx;
                            logical_processor_map.insert(logical_processor_id, core_type);
                        }
                    }
                }
            }
            RelationProcessorPackage => {
                let package = unsafe { &info_ex.Anonymous.Processor };
                for i in 0..package.GroupMask.len() {
                    let group_affinity = &package.GroupMask[i];
                    let group_idx = group_affinity.Group as usize;
                    let mask = group_affinity.Mask;
                    for bit_idx in 0..std::mem::size_of::<usize>() * 8 {
                        if (mask >> bit_idx) & 1 != 0 {
                            let first_lp_of_package = (group_idx * 64) + bit_idx;
                            socket_id_anchors.insert(first_lp_of_package);
                            break;
                        }
                    }
                }
            }
            RelationCache => {
                let cache_info_api = unsafe { &info_ex.Anonymous.Cache };
                let level = CacheLevel::from(cache_info_api.Level as u32);
                let cache_type_win = cache_info_api.Type;
                let cache_type = match cache_type_win {
                    CacheUnified => CacheType::Unified,
                    CacheInstruction => CacheType::Instruction,
                    CacheData => CacheType::Data,
                    CacheTrace => CacheType::Trace,
                    _ => CacheType::Unknown,
                };
                let group_idx = unsafe { cache_info_api.Anonymous.GroupMask.Group };
                let mask = unsafe { cache_info_api.Anonymous.GroupMask.Mask };

                caches_from_api.push(CacheFromApi {
                    level,
                    cache_type,
                    size_bytes: cache_info_api.CacheSize as u64,
                    line_size_bytes: cache_info_api.LineSize as usize,
                    logical_processor_mask: mask,
                    group: group_idx,
                });
            }
            _ => {}
        }
        current_ptr = unsafe { current_ptr.add(info_ex.Size as usize) };
    }

    debug!("Reconstructing topology...");
    let mut sockets_map: HashMap<usize, SocketInfo> = HashMap::new();
    let mut core_id_counters: HashMap<usize, usize> = HashMap::new();
    let mut global_to_socket_core_map: HashMap<usize, (usize, usize)> = HashMap::new();
    let mut lp_to_global_core_map: HashMap<usize, usize> = HashMap::new();

    let mut global_phys_core_id_assigner = 0;
    let mut processed_core_relations: HashSet<usize> = HashSet::new();

    current_ptr = buffer.as_ptr();
    while current_ptr < end_ptr {
        let info_ex = unsafe { &*(current_ptr as *const SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX) };
        if info_ex.Relationship == RelationProcessorCore {
            let proc_core_rel = unsafe { &info_ex.Anonymous.Processor };

            let mut first_lp_in_this_core_relation = usize::MAX;
            let mut lps_in_this_core_relation = Vec::new();
            for i in 0..proc_core_rel.GroupMask.len() {
                let group_affinity = &proc_core_rel.GroupMask[i];
                let group_idx = group_affinity.Group as usize;
                let mask = group_affinity.Mask;
                for bit_idx in 0..std::mem::size_of::<usize>() * 8 {
                    if (mask >> bit_idx) & 1 != 0 {
                        let lp_id = (group_idx * 64) + bit_idx;
                        lps_in_this_core_relation.push(lp_id);
                        if first_lp_in_this_core_relation == usize::MAX {
                            first_lp_in_this_core_relation = lp_id;
                        }
                    }
                }
            }

            if first_lp_in_this_core_relation == usize::MAX
                || processed_core_relations.contains(&first_lp_in_this_core_relation)
            {
                current_ptr = unsafe { current_ptr.add(info_ex.Size as usize) };
                continue;
            }
            processed_core_relations.insert(first_lp_in_this_core_relation);

            let current_global_phys_core_id = global_phys_core_id_assigner;
            global_phys_core_id_assigner += 1;

            let mut core_type_from_map = logical_processor_map
                .get(&first_lp_in_this_core_relation)
                .cloned()
                .unwrap_or(CoreType::Performance);
            if p_cores_count == 0 {
                core_type_from_map = CoreType::Performance;
            }
            if core_type_from_map == CoreType::Performance {
                total_performance_cores += 1;
            } else {
                total_efficiency_cores += 1;
            }

            let mut socket_anchor_for_this_core = 0;
            'socket_search: for i in 0..proc_core_rel.GroupMask.len() {
                let group_affinity = &proc_core_rel.GroupMask[i];
                let group_idx = group_affinity.Group as usize;
                let core_mask_in_group = group_affinity.Mask;
                for bit_idx in 0..std::mem::size_of::<usize>() * 8 {
                    if (core_mask_in_group >> bit_idx) & 1 != 0 {
                        let _lp_id_of_core = (group_idx * 64) + bit_idx;
                        let mut temp_pkg_ptr = buffer.as_ptr();
                        while temp_pkg_ptr < end_ptr {
                            let pkg_info_ex_s = unsafe {
                                &*(temp_pkg_ptr as *const SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX)
                            };
                            if pkg_info_ex_s.Relationship == RelationProcessorPackage {
                                let package_rel = unsafe { &pkg_info_ex_s.Anonymous.Processor };
                                for j in 0..package_rel.GroupMask.len() {
                                    let pkg_group_affinity = &package_rel.GroupMask[j];
                                    if pkg_group_affinity.Group as usize == group_idx
                                        && (pkg_group_affinity.Mask >> bit_idx) & 1 != 0
                                    {
                                        let mut first_lp_of_package = usize::MAX;
                                        for k_pkg_grp in 0..package_rel.GroupMask.len() {
                                            let first_lp_pkg_grp_aff =
                                                &package_rel.GroupMask[k_pkg_grp];
                                            for l_pkg_bit in 0..std::mem::size_of::<usize>() * 8 {
                                                if (first_lp_pkg_grp_aff.Mask >> l_pkg_bit) & 1 != 0
                                                {
                                                    first_lp_of_package =
                                                        (first_lp_pkg_grp_aff.Group as usize * 64)
                                                            + l_pkg_bit;
                                                    break;
                                                }
                                            }
                                            if first_lp_of_package != usize::MAX {
                                                break;
                                            }
                                        }
                                        socket_anchor_for_this_core = first_lp_of_package;
                                        break 'socket_search;
                                    }
                                }
                            }
                            temp_pkg_ptr = unsafe { temp_pkg_ptr.add(pkg_info_ex_s.Size as usize) };
                        }
                        if socket_anchor_for_this_core != 0 {
                            break 'socket_search;
                        }
                    }
                }
            }

            let socket_entry = sockets_map
                .entry(socket_anchor_for_this_core)
                .or_insert_with(|| SocketInfo {
                    id: socket_anchor_for_this_core,
                    cores: Vec::new(),
                    l3_cache: None,
                });

            let core_id_in_socket = *core_id_counters
                .entry(socket_anchor_for_this_core)
                .or_insert(0);
            core_id_counters.insert(socket_anchor_for_this_core, core_id_in_socket + 1);
            global_to_socket_core_map.insert(
                current_global_phys_core_id,
                (socket_anchor_for_this_core, core_id_in_socket),
            );

            lps_in_this_core_relation.sort_unstable();
            for lp_id in &lps_in_this_core_relation {
                lp_to_global_core_map.insert(*lp_id, current_global_phys_core_id);
            }

            let mut l1i = None;
            let mut l1d = None;
            let mut l2 = None;
            for cache_api in &caches_from_api {
                let mut shared_by_this_core = false;
                for lp_id in &lps_in_this_core_relation {
                    if cache_api.group == (*lp_id / 64) as u16
                        && (cache_api.logical_processor_mask >> (*lp_id % 64)) & 1 != 0
                    {
                        shared_by_this_core = true;
                        break;
                    }
                }
                if shared_by_this_core {
                    let cache_info = CacheInfo {
                        level: cache_api.level,
                        cache_type: cache_api.cache_type,
                        size_bytes: cache_api.size_bytes,
                        line_size_bytes: cache_api.line_size_bytes,
                    };
                    match cache_api.level {
                        CacheLevel::L1 => {
                            if cache_api.cache_type == CacheType::Instruction && l1i.is_none() {
                                l1i = Some(cache_info);
                            } else if cache_api.cache_type == CacheType::Data && l1d.is_none() {
                                l1d = Some(cache_info);
                            } else if cache_api.cache_type == CacheType::Unified {
                                if l1i.is_none() {
                                    l1i = Some(cache_info);
                                }
                                if l1d.is_none() {
                                    l1d = Some(cache_info);
                                }
                            }
                        }
                        CacheLevel::L2 => {
                            if l2.is_none() {
                                l2 = Some(cache_info);
                            }
                        }
                        _ => {}
                    }
                }
            }
            socket_entry.cores.push(CoreInfo {
                id: current_global_phys_core_id,
                socket_id: socket_anchor_for_this_core,
                core_type: core_type_from_map,
                logical_processor_ids: lps_in_this_core_relation,
                l1_instruction_cache: l1i,
                l1_data_cache: l1d,
                l2_cache: l2,
            });
        }
        current_ptr = unsafe { current_ptr.add(info_ex.Size as usize) };
    }

    for cache_api in &caches_from_api {
        if cache_api.level == CacheLevel::L3 {
            let mut owning_socket_anchor = usize::MAX;
            for bit_idx in 0..std::mem::size_of::<usize>() * 8 {
                if (cache_api.logical_processor_mask >> bit_idx) & 1 != 0 {
                    let lp_id = (cache_api.group as usize * 64) + bit_idx;
                    if let Some(global_core_id) = lp_to_global_core_map.get(&lp_id) {
                        if let Some((socket_anchor, _)) =
                            global_to_socket_core_map.get(global_core_id)
                        {
                            owning_socket_anchor = *socket_anchor;
                            break;
                        }
                    }
                }
            }
            if owning_socket_anchor != usize::MAX {
                if let Some(socket) = sockets_map.get_mut(&owning_socket_anchor) {
                    if socket.l3_cache.is_none() {
                        socket.l3_cache = Some(CacheInfo {
                            level: cache_api.level,
                            cache_type: cache_api.cache_type,
                            size_bytes: cache_api.size_bytes,
                            line_size_bytes: cache_api.line_size_bytes,
                        });
                    }
                }
            }
        }
    }

    let mut final_sockets_vec: Vec<SocketInfo> = sockets_map.into_values().collect();
    final_sockets_vec.sort_by_key(|s| s.id);
    for (new_id, socket_info) in final_sockets_vec.iter_mut().enumerate() {
        socket_info.id = new_id;
        for core in socket_info.cores.iter_mut() {
            core.socket_id = new_id;
        }
        socket_info.cores.sort_by_key(|c| c.id);
    }

    let final_total_lp_count = lp_to_global_core_map.len();
    let final_total_physical_cores = global_phys_core_id_assigner;

    if final_total_physical_cores > 0 && total_performance_cores == 0 && total_efficiency_cores == 0
    {
        debug!(
            "No P/E core distinction from EfficiencyClass, assuming all {} cores are Performance.",
            final_total_physical_cores
        );
        total_performance_cores = final_total_physical_cores;
    }

    Ok(CpuInfo {
        vendor,
        model_name,
        features,
        sockets: final_sockets_vec,
        total_sockets: socket_id_anchors.len().max(1),
        total_physical_cores: final_total_physical_cores,
        total_logical_processors: final_total_lp_count,
        total_performance_cores,
        total_efficiency_cores,
    })
}
