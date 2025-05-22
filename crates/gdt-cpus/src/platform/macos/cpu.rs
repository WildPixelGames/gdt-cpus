//! macOS-specific CPU detection logic.
//!
//! This module implements the core CPU detection capabilities for macOS systems.
//! It primarily utilizes the `sysctl` interface to gather detailed information
//! about the CPU, including vendor, model name, features, core counts (physical,
//! logical, Performance/Efficiency for Apple Silicon), cache hierarchy, and topology.
//!
//! For x86_64 architectures, it also attempts to use `cpuid` instructions as a primary
//! source for basic CPU details, falling back to `sysctl` if needed.
//! The main entry point is the [`detect_cpu_info`] function.

use log::warn;

use crate::{
    CacheInfo, CacheLevel, CacheType, CoreInfo, CoreType, CpuFeatures, CpuInfo, Error, Result,
    SocketInfo, Vendor,
    platform::macos::utils::{sysctlbyname_int, sysctlbyname_string},
};

/// Detects CPU vendor, model name, and features using `sysctl`.
///
/// This function queries various `sysctl` MIBs like `machdep.cpu.brand_string`
/// and `machdep.cpu.vendor`. For Apple Silicon CPUs, it also queries
/// `hw.optional.arm.FEAT_*` and `hw.optional.neon` to determine supported CPU features.
///
/// This serves as a primary detection method on ARM64 (Apple Silicon) and as a
/// fallback on x86_64 if `cpuid` fails or doesn't provide complete information.
///
/// # Arguments
///
/// * `vendor`: A mutable reference to [`Vendor`] to be updated.
/// * `model_name`: A mutable reference to a `String` for the CPU model name.
/// * `features`: A mutable reference to [`CpuFeatures`] to be populated.
fn detect_cpu_via_sysctl(vendor: &mut Vendor, model_name: &mut String, features: &mut CpuFeatures) {
    *model_name = sysctlbyname_string("machdep.cpu.brand_string").unwrap_or_else(|e| {
        warn!("Failed to get model_name, defaulting to Unknown: {}", e);
        "Unknown".to_string()
    });
    let vendor_str =
        sysctlbyname_string("machdep.cpu.vendor").unwrap_or_else(|_| "Unknown".to_string());

    *vendor = match vendor_str.as_str() {
        "GenuineIntel" => Vendor::Intel,
        "AuthenticAMD" => Vendor::Amd,
        "Apple" => Vendor::Apple,
        _ => {
            if model_name.to_lowercase().contains("apple")
                || vendor_str.eq_ignore_ascii_case("apple")
            {
                Vendor::Apple
            } else if model_name.to_lowercase().contains("intel") {
                Vendor::Intel
            } else if model_name.to_lowercase().contains("amd") {
                Vendor::Amd
            } else {
                Vendor::Other(vendor_str)
            }
        }
    };

    *features = CpuFeatures::default();
    if *vendor == Vendor::Apple {
        if sysctlbyname_int::<i32>("hw.optional.neon")
            .map(|v| v == 1)
            .unwrap_or(false)
        {
            features.insert(CpuFeatures::NEON);
        }
        if sysctlbyname_int::<i32>("hw.optional.arm.FEAT_AES")
            .map(|v| v == 1)
            .unwrap_or(false)
        {
            features.insert(CpuFeatures::AES);
        }
        let sha1 = sysctlbyname_int::<i32>("hw.optional.arm.FEAT_SHA1")
            .map(|v| v == 1)
            .unwrap_or(false);
        let sha3 = sysctlbyname_int::<i32>("hw.optional.arm.FEAT_SHA3")
            .map(|v| v == 1)
            .unwrap_or(false);
        let sha256 = sysctlbyname_int::<i32>("hw.optional.arm.FEAT_SHA256")
            .map(|v| v == 1)
            .unwrap_or(false);
        let sha512 = sysctlbyname_int::<i32>("hw.optional.arm.FEAT_SHA512")
            .map(|v| v == 1)
            .unwrap_or(false);
        if sha1 || sha3 || sha256 || sha512 {
            features.insert(CpuFeatures::SHA);
        }
        if sysctlbyname_int::<i32>("hw.optional.arm.FEAT_CRC32")
            .map(|v| v == 1)
            .unwrap_or(false)
        {
            features.insert(CpuFeatures::CRC32);
        }
        // FEAT_SME seems to be for Scalable Matrix Extension, not SVE directly
        if sysctlbyname_int::<i32>("hw.optional.arm.FEAT_SME")
            .map(|v| v == 1)
            .unwrap_or(false)
        {
            features.insert(CpuFeatures::SVE);
        }
    }
}

/// Detects basic CPU information (Vendor, Model Name, Features).
///
/// On `x86_64` architectures, this function first attempts to use `cpuid` instructions
/// (via `crate::platform::common_x86_64::detect_via_cpuid`).
/// If `cpuid` fails, is not available (e.g., on non-x86_64), or does not provide
/// complete information, it falls back to using `detect_cpu_via_sysctl`.
///
/// # Returns
///
/// A `Result` containing a tuple of (`Vendor`, `String` (model name), `CpuFeatures`)
/// if successful, or an `Error` otherwise (though errors from internal sysctl calls
/// are often handled by returning default/unknown values).
fn detect_basic_cpu_info() -> Result<(Vendor, String, CpuFeatures)> {
    let mut vendor = Vendor::Unknown;
    let mut model_name = "Unknown".to_string();
    let mut features = CpuFeatures::default();

    // For x86_64, prioritize raw-cpuid
    #[cfg(target_arch = "x86_64")]
    super::common_x86_64::detect_via_cpuid(&mut vendor, &mut model_name, &mut features);

    // Fallback to /proc/cpuinfo for non-x86_64 or if CPUID failed
    if vendor == Vendor::Unknown || model_name == "Unknown" || features.is_empty() {
        detect_cpu_via_sysctl(&mut vendor, &mut model_name, &mut features);
    }

    Ok((vendor, model_name, features))
}

/// Detects comprehensive CPU information on macOS systems.
///
/// This function orchestrates the CPU detection process by:
/// 1.  **Fetching Basic CPU Details:** Calls `detect_basic_cpu_info()` to get vendor,
///     model name, and features (using `cpuid` on x86_64 with `sysctl` fallback,
///     or primarily `sysctl` for Apple Silicon).
/// 2.  **Querying Core Counts & Topology via `sysctl`:**
///     - Retrieves total socket count (`hw.packages`), physical core count (`hw.physicalcpu`),
///       and logical processor count (`hw.logicalcpu`).
///     - For Apple Silicon, determines Performance-core (P-core) and Efficiency-core (E-core)
///       counts using `hw.perflevel0.physicalcpu` and `hw.perflevel1.physicalcpu` respectively.
///       Includes logic to handle cases where these specific keys might be unavailable but
///       `hw.nperflevels` indicates a hybrid architecture.
///     - For Intel/AMD, all physical cores are currently assumed to be Performance-cores.
/// 3.  **Gathering Cache Information via `sysctl`:**
///     - Gets the L1 cache line size (`hw.cachelinesize`).
///     - For Apple Silicon: Reads L1I, L1D, and L2 cache sizes per P-core/E-core cluster
///       (e.g., `hw.perflevel0.l1icachesize`, `hw.perflevel0.l2cachesize`, `hw.perflevel0.cpusperl2`).
///       Note: L3 cache for Apple Silicon is typically system-level and its detailed attribution
///       to specific core types or clusters via sysctl is less direct; this function attempts
///       to read `hw.l3cachesize_pkg` for a package-level L3 if available.
///     - For Intel/AMD: Reads `hw.l1icachesize`, `hw.l1dcachesize`, `hw.l2cachesize` (per-core),
///       and `hw.l3cachesize` (shared).
/// 4.  **Constructing Topology:**
///     - Builds `SocketInfo` and `CoreInfo` structures.
///     - Assigns logical processors to their respective physical cores.
///     - Sets `CoreType` (Performance/Efficiency) for each core.
///     - Populates `CacheInfo` for L1, L2 (per core or per cluster for Apple Silicon), and L3 caches.
/// 5.  **NUMA Information:** Currently, NUMA information is simplified, assuming a single NUMA node (ID 0)
///     as macOS does not expose detailed NUMA topology as readily as other platforms.
///
/// # Returns
///
/// A `Result<CpuInfo>` containing the populated `CpuInfo` struct, or an `Error` if
/// critical detection steps fail (e.g., `hw.physicalcpu` returns 0).
///
/// # Panics
///
/// This function may panic if `sysctlbyname_int` calls for essential Apple Silicon cache
/// information (e.g., `hw.perflevel0.l1icachesize`) fail, as these are currently `.expect()`-ed.
/// Other `sysctl` errors are generally handled by falling back to default values or logging warnings.
pub fn detect_cpu_info() -> Result<CpuInfo> {
    // --- Vendor, Model, and Features ---
    let (vendor, model_name, features) = detect_basic_cpu_info()?;

    // --- Core Counts & Topology ---
    let total_sockets_count = sysctlbyname_int::<u32>("hw.packages").unwrap_or(1).max(1) as usize;
    let total_physical_cores_count =
        sysctlbyname_int::<u32>("hw.physicalcpu").unwrap_or(0) as usize;
    let total_logical_processors_count =
        sysctlbyname_int::<u32>("hw.logicalcpu").unwrap_or(0) as usize;

    if total_physical_cores_count == 0 {
        return Err(Error::Detection(
            "Failed to detect any physical cores (hw.physicalcpu returned 0 or error)".to_string(),
        ));
    }

    let mut detected_performance_cores = 0;
    let mut detected_efficiency_cores = 0;

    // Apple Silicon P/E core detection
    if vendor == Vendor::Apple {
        // Try to get P/E core counts directly for Apple Silicon
        if let Ok(p_cores) = sysctlbyname_int::<u32>("hw.perflevel0.physicalcpu") {
            detected_performance_cores = p_cores as usize;
        } else {
            warn!("Could not read hw.perflevel0.physicalcpu for P-core count.");
        }
        if let Ok(e_cores) = sysctlbyname_int::<u32>("hw.perflevel1.physicalcpu") {
            detected_efficiency_cores = e_cores as usize;
        } else {
            warn!("Could not read hw.perflevel1.physicalcpu for E-core count.");
        }

        // If counts are still zero but hw.nperflevels suggests a hybrid architecture,
        // it's an anomaly. We previously defaulted to total_physical_cores for P-cores.
        // Given the new sysctl values, if perflevel counts are zero, it's more likely an issue or non-hybrid.
        if detected_performance_cores == 0
            && detected_efficiency_cores == 0
            && total_physical_cores_count > 0
        {
            if sysctlbyname_int::<i32>("hw.nperflevels").unwrap_or(0) > 1 {
                warn!(
                    "Hybrid architecture indicated by hw.nperflevels, but P/E core counts are zero. Reporting all as P-cores."
                );
                detected_performance_cores = total_physical_cores_count;
            } else {
                // Not hybrid or nperflevels not available, assume all are P-cores
                detected_performance_cores = total_physical_cores_count;
            }
        } else if (detected_performance_cores + detected_efficiency_cores)
            != total_physical_cores_count
            && (detected_performance_cores > 0 || detected_efficiency_cores > 0)
        {
            warn!(
                "Sum of P-cores ({}) and E-cores ({}) does not match total physical cores ({}). Using detected P/E counts.",
                detected_performance_cores, detected_efficiency_cores, total_physical_cores_count
            );
            // Trust the perflevel counts if they are non-zero, even if sum mismatches.
            // The total_physical_cores_count will be the sum of these for consistency in CpuInfo.
            // total_physical_cores_count = detected_performance_cores + detected_efficiency_cores; // This might hide an OS reporting issue.
            // Better to report what OS gives for total and what it gives for P/E separately.
        }
    } else {
        // For Intel/AMD, assume all physical cores are performance if not further specified by OS
        detected_performance_cores = total_physical_cores_count;
    }

    // --- Cache Information ---
    let cache_line_size = sysctlbyname_int::<u64>("hw.cachelinesize").unwrap_or(64) as usize; // Default to 64 if not found

    /// Helper struct to temporarily store cache information for an Apple Silicon core cluster (P or E).
    #[derive(Copy, Clone)]
    struct AppleSiliconCacheInfo {
        l1i_cache_size: u64,
        l1d_cache_size: u64,
        l2_cache_size: u64,
        cores_sharing_l2: u32,
    }

    // Cache sizes per P/E cores of Apple Silicon
    let p_core_cache_info = if vendor == Vendor::Apple {
        let p_cluster_l1i_cache_size = sysctlbyname_int::<u32>("hw.perflevel0.l1icachesize").expect(
            "Failed to get L1i cache size for performance cluster (hw.perflevel0.l1icachesize)",
        ) as u64;
        let p_cluster_l1d_cache_size = sysctlbyname_int::<u32>("hw.perflevel0.l1dcachesize").expect(
            "Failed to get L1d cache size for performance cluster (hw.perflevel0.l1dcachesize)",
        ) as u64;
        let p_cluster_l2_cache_size = sysctlbyname_int::<u32>("hw.perflevel0.l2cachesize").expect(
            "Failed to get L2 cache size for performance cluster (hw.perflevel0.l2cachesize)",
        ) as u64;
        let p_cores_sharing_l2 = sysctlbyname_int::<u32>("hw.perflevel0.cpusperl2").expect(
            "Failed to get cores sharing L2 cache for performance cluster (hw.perflevel0.cpusperl2)",
        );

        Some(AppleSiliconCacheInfo {
            l1i_cache_size: p_cluster_l1i_cache_size,
            l1d_cache_size: p_cluster_l1d_cache_size,
            l2_cache_size: p_cluster_l2_cache_size,
            cores_sharing_l2: p_cores_sharing_l2,
        })
    } else {
        None
    };

    let e_core_cache_info = if vendor == Vendor::Apple {
        let e_cluster_l1i_cache_size = sysctlbyname_int::<u32>("hw.perflevel1.l1icachesize").expect(
            "Failed to get L1i cache size for efficiency cluster (hw.perflevel1.l1icachesize)",
        ) as u64;
        let e_cluster_l1d_cache_size = sysctlbyname_int::<u32>("hw.perflevel1.l1dcachesize").expect(
            "Failed to get L1d cache size for efficiency cluster (hw.perflevel1.l1dcachesize)",
        ) as u64;
        let e_cluster_l2_cache_size = sysctlbyname_int::<u32>("hw.perflevel1.l2cachesize").expect(
            "Failed to get L2 cache size for efficiency cluster (hw.perflevel1.l2cachesize)",
        ) as u64;
        let e_cores_sharing_l2 = sysctlbyname_int::<u32>("hw.perflevel1.cpusperl2").expect(
            "Failed to get cores sharing L2 cache for efficiency cluster (hw.perflevel1.cpusperl2)",
        );

        Some(AppleSiliconCacheInfo {
            l1i_cache_size: e_cluster_l1i_cache_size,
            l1d_cache_size: e_cluster_l1d_cache_size,
            l2_cache_size: e_cluster_l2_cache_size,
            cores_sharing_l2: e_cores_sharing_l2,
        })
    } else {
        None
    };

    // Global L3 cache (typically per socket or system-wide)
    let sys_l3_cache_total_size = sysctlbyname_int::<u64>("hw.l3cachesize").ok();

    // --- Building the Simplified Topology ---
    let mut sockets_vec = Vec::with_capacity(total_sockets_count);
    let physical_cores_per_socket = if total_sockets_count > 0 {
        total_physical_cores_count.div_ceil(total_sockets_count)
    } else {
        0 // Should not happen if total_physical_cores_count > 0 and total_sockets_count is at least 1
    };
    let logical_processors_per_physical_core = if total_physical_cores_count > 0 {
        total_logical_processors_count.div_ceil(total_physical_cores_count)
    } else {
        1 // Avoid division by zero
    };

    let mut current_physical_core_id_counter = 0;
    let mut current_logical_processor_id_counter = 0;

    for socket_idx in 0..total_sockets_count {
        let mut cores_for_this_socket_vec = Vec::new();
        let num_phys_cores_this_socket = if socket_idx == total_sockets_count - 1 {
            total_physical_cores_count
                .saturating_sub(physical_cores_per_socket * (total_sockets_count - 1))
        } else {
            physical_cores_per_socket
        };

        for _core_local_idx in 0..num_phys_cores_this_socket {
            if current_physical_core_id_counter >= total_physical_cores_count {
                break;
            }

            let core_global_id = current_physical_core_id_counter;
            let core_type = if core_global_id < detected_performance_cores {
                CoreType::Performance
            } else {
                CoreType::Efficiency
            };

            let mut logical_processor_ids_for_this_core = Vec::new();
            let num_lps_this_core =
                if current_physical_core_id_counter == total_physical_cores_count - 1 {
                    total_logical_processors_count.saturating_sub(
                        logical_processors_per_physical_core
                            * (total_physical_cores_count.saturating_sub(1)),
                    )
                } else {
                    logical_processors_per_physical_core
                };

            for _ in 0..num_lps_this_core {
                if current_logical_processor_id_counter < total_logical_processors_count {
                    logical_processor_ids_for_this_core.push(current_logical_processor_id_counter);
                    current_logical_processor_id_counter += 1;
                }
            }

            let mut l1i_cache = None;
            let mut l1d_cache = None;
            let mut l2_cache = None;

            if vendor == Vendor::Apple {
                match core_type {
                    CoreType::Performance => {
                        if let Some(cache_info) = p_core_cache_info {
                            if detected_performance_cores > 0 {
                                l1i_cache = Some(CacheInfo {
                                    level: CacheLevel::L1,
                                    cache_type: CacheType::Instruction,
                                    size_bytes: cache_info.l1i_cache_size,
                                    line_size_bytes: cache_line_size,
                                });
                                l1d_cache = Some(CacheInfo {
                                    level: CacheLevel::L1,
                                    cache_type: CacheType::Data,
                                    size_bytes: cache_info.l1d_cache_size,
                                    line_size_bytes: cache_line_size,
                                });
                            }
                            if cache_info.cores_sharing_l2 > 0 && detected_performance_cores > 0 {
                                let num_l2_clusters = (detected_performance_cores as f64
                                    / cache_info.cores_sharing_l2 as f64)
                                    .ceil()
                                    as u64;
                                if num_l2_clusters > 0 {
                                    l2_cache = Some(CacheInfo {
                                        level: CacheLevel::L2,
                                        cache_type: CacheType::Unified,
                                        size_bytes: cache_info.l2_cache_size,
                                        line_size_bytes: cache_line_size,
                                    });
                                }
                            }
                        }
                    }
                    CoreType::Efficiency => {
                        if let Some(cache_info) = e_core_cache_info {
                            if detected_efficiency_cores > 0 {
                                l1i_cache = Some(CacheInfo {
                                    level: CacheLevel::L1,
                                    cache_type: CacheType::Instruction,
                                    size_bytes: cache_info.l1i_cache_size,
                                    line_size_bytes: cache_line_size,
                                });
                                l1d_cache = Some(CacheInfo {
                                    level: CacheLevel::L1,
                                    cache_type: CacheType::Data,
                                    size_bytes: cache_info.l1d_cache_size,
                                    line_size_bytes: cache_line_size,
                                });
                            }
                            if cache_info.cores_sharing_l2 > 0 && detected_efficiency_cores > 0 {
                                let num_l2_clusters = (detected_efficiency_cores as f64
                                    / cache_info.cores_sharing_l2 as f64)
                                    .ceil()
                                    as u64;
                                if num_l2_clusters > 0 {
                                    l2_cache = Some(CacheInfo {
                                        level: CacheLevel::L2,
                                        cache_type: CacheType::Unified,
                                        size_bytes: cache_info.l2_cache_size,
                                        line_size_bytes: cache_line_size,
                                    });
                                }
                            }
                        }
                    }
                    CoreType::Unknown => {} // Should not happen if P/E detection is robust
                }
            } else {
                // Fallback for Intel/AMD or if Apple specific sysctls fail for some reason
                let global_l1i_total = sysctlbyname_int::<u64>("hw.l1icachesize").unwrap_or(0);
                let global_l1d_total = sysctlbyname_int::<u64>("hw.l1dcachesize").unwrap_or(0);
                let global_l2_total = sysctlbyname_int::<u64>("hw.l2cachesize").unwrap_or(0);

                if global_l1i_total > 0 && total_physical_cores_count > 0 {
                    l1i_cache = Some(CacheInfo {
                        level: CacheLevel::L1,
                        cache_type: CacheType::Instruction,
                        size_bytes: global_l1i_total / total_physical_cores_count as u64,
                        line_size_bytes: cache_line_size,
                    });
                }
                if global_l1d_total > 0 && total_physical_cores_count > 0 {
                    l1d_cache = Some(CacheInfo {
                        level: CacheLevel::L1,
                        cache_type: CacheType::Data,
                        size_bytes: global_l1d_total / total_physical_cores_count as u64,
                        line_size_bytes: cache_line_size,
                    });
                }
                if global_l2_total > 0 && total_physical_cores_count > 0 {
                    l2_cache = Some(CacheInfo {
                        level: CacheLevel::L2,
                        cache_type: CacheType::Unified,
                        size_bytes: global_l2_total / total_physical_cores_count as u64,
                        line_size_bytes: cache_line_size,
                    });
                }
            }

            cores_for_this_socket_vec.push(CoreInfo {
                id: core_global_id,
                socket_id: socket_idx,
                core_type,
                logical_processor_ids: logical_processor_ids_for_this_core,
                l1_instruction_cache: l1i_cache,
                l1_data_cache: l1d_cache,
                l2_cache,
            });
            current_physical_core_id_counter += 1;
        }

        let l3_cache_this_socket = if let Some(total_l3_size) = sys_l3_cache_total_size {
            if total_sockets_count > 0 {
                Some(CacheInfo {
                    level: CacheLevel::L3,
                    cache_type: CacheType::Unified,
                    size_bytes: total_l3_size / total_sockets_count as u64, // Assumes L3 is evenly split per socket if multiple sockets
                    line_size_bytes: cache_line_size,
                })
            } else {
                None
            }
        } else {
            None
        };

        sockets_vec.push(SocketInfo {
            id: socket_idx,
            cores: cores_for_this_socket_vec,
            l3_cache: l3_cache_this_socket,
        });
    }

    Ok(CpuInfo {
        vendor,
        model_name,
        features,
        sockets: sockets_vec,
        total_sockets: total_sockets_count,
        total_physical_cores: total_physical_cores_count,
        total_logical_processors: total_logical_processors_count,
        total_performance_cores: detected_performance_cores,
        total_efficiency_cores: detected_efficiency_cores,
    })
}
