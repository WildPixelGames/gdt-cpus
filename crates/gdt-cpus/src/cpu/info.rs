use crate::{AffinityMask, CoreType, CpuFeatures, Result, SocketInfo, Vendor};

/// Provides a comprehensive overview of the system's CPU capabilities.
///
/// This top-level structure aggregates detailed information about the CPU,
/// including its vendor, model, supported features, socket layout, and core details.
/// It is designed to be the primary source of CPU information for applications
/// using this crate.
///
/// The information is detected once per program execution using a platform-specific
/// approach and then cached for subsequent calls to `gdt_cpus::cpu_info()`.
///
/// # Examples
///
/// ```
/// // Assuming gdt_cpus::cpu_info() returns a Result<&CpuInfo, Error>
/// if let Ok(cpu_info) = gdt_cpus::cpu_info() {
///     println!("CPU Vendor: {}", cpu_info.vendor);
///     println!("CPU Model: {}", cpu_info.model_name);
///     println!("Total Physical Cores: {}", cpu_info.total_physical_cores);
///     #[cfg(target_arch = "x86_64")]
///     if cpu_info.features.contains(gdt_cpus::CpuFeatures::AVX2) {
///         println!("AVX2 is supported!");
///     }
///     #[cfg(target_arch = "aarch64")]
///     if cpu_info.features.contains(gdt_cpus::CpuFeatures::NEON) {
///         println!("NEON is supported!");
///     }
/// }
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CpuInfo {
    /// The manufacturer of the CPU (e.g., Intel, AMD, Apple).
    pub vendor: Vendor,
    /// The specific model name of the CPU as reported by the system
    /// (e.g., "Apple M1 Pro", "Intel(R) Core(TM) i7-13700K").
    pub model_name: String,
    /// A bitfield representing the set of supported CPU features and instruction sets
    /// (e.g., SSE, AVX, NEON).
    pub features: CpuFeatures,

    /// A vector containing detailed information about each physical CPU socket present in the system.
    /// For most consumer systems, this will contain a single `SocketInfo` element.
    pub sockets: Vec<SocketInfo>,

    // Aggregated counts for convenience, derived from the `sockets` data.
    /// Total number of physical CPU sockets in the system.
    pub total_sockets: usize,
    /// Total number of physical cores across all sockets. This does not count logical
    /// processors from Hyper-Threading/SMT.
    pub total_physical_cores: usize,
    /// Total number of logical processors (hardware threads) across all sockets.
    /// This includes threads from Hyper-Threading/SMT.
    pub total_logical_processors: usize,
    /// Total number of performance-type physical cores (e.g., P-cores) if the CPU
    /// has a hybrid architecture or the number of physical cores otherwise.
    pub total_performance_cores: usize,
    /// Total number of efficiency-type physical cores (e.g., E-cores) if the CPU
    /// has a hybrid architecture. Zero if not applicable or not detected.
    pub total_efficiency_cores: usize,
}

impl CpuInfo {
    /// Detects CPU information using platform-specific methods.
    ///
    /// This function is called once internally by `gdt_cpus::cpu_info()`
    /// to initialize the static `CPU_INFO` variable. It dispatches to the
    /// appropriate platform-specific detection logic.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `CpuInfo` on success, or an `Error` if detection fails
    /// or the platform is unsupported.
    pub(crate) fn detect() -> Result<Self> {
        #[cfg(target_os = "linux")]
        {
            crate::platform::linux::cpu::detect_cpu_info()
        }
        #[cfg(target_os = "macos")]
        {
            crate::platform::macos::cpu::detect_cpu_info()
        }
        #[cfg(target_os = "windows")]
        {
            crate::platform::windows::cpu::detect_cpu_info()
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            // Explicitly return an Error for unsupported platforms.
            Err(Error::Unsupported(
                "CPU information detection is not supported on this platform.".to_string(),
            ))
        }
    }

    /// Returns the total number of physical cores in the system.
    ///
    /// This is a convenience method equivalent to accessing `self.total_physical_cores`.
    /// It sums the number of physical cores across all sockets.
    ///
    /// # Examples
    ///
    /// ```
    /// # use gdt_cpus::{CpuInfo, Vendor, CpuFeatures, SocketInfo};
    /// # let cpu_info = CpuInfo {
    /// #     vendor: Vendor::Unknown, model_name: String::new(), features: CpuFeatures::empty(),
    /// #     sockets: Vec::new(), total_sockets: 0, total_physical_cores: 8,
    /// #     total_logical_processors: 16, total_performance_cores: 4, total_efficiency_cores: 4,
    /// # };
    /// assert_eq!(cpu_info.num_physical_cores(), 8);
    /// ```
    pub fn num_physical_cores(&self) -> usize {
        self.total_physical_cores
    }

    /// Returns the total number of logical cores (hardware threads) in the system.
    ///
    /// This is a convenience method equivalent to accessing `self.total_logical_processors`.
    /// This count includes threads from technologies like Intel's Hyper-Threading or AMD's SMT.
    ///
    /// # Examples
    ///
    /// ```
    /// # use gdt_cpus::{CpuInfo, Vendor, CpuFeatures, SocketInfo};
    /// # let cpu_info = CpuInfo {
    /// #     vendor: Vendor::Unknown, model_name: String::new(), features: CpuFeatures::empty(),
    /// #     sockets: Vec::new(), total_sockets: 0, total_physical_cores: 8,
    /// #     total_logical_processors: 16, total_performance_cores: 4, total_efficiency_cores: 4,
    /// # };
    /// assert_eq!(cpu_info.num_logical_cores(), 16);
    /// ```
    pub fn num_logical_cores(&self) -> usize {
        self.total_logical_processors
    }

    /// Returns the total number of performance-type physical cores in the system.
    ///
    /// This is relevant for hybrid architectures (e.g., Intel P-cores, ARM big cores).
    /// Returns the number of physical cores if the system does not have a hybrid architecture.
    /// This is a convenience method equivalent to accessing `self.total_performance_cores`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use gdt_cpus::{CpuInfo, Vendor, CpuFeatures, SocketInfo};
    /// # let cpu_info = CpuInfo {
    /// #     vendor: Vendor::Unknown, model_name: String::new(), features: CpuFeatures::empty(),
    /// #     sockets: Vec::new(), total_sockets: 0, total_physical_cores: 8,
    /// #     total_logical_processors: 16, total_performance_cores: 4, total_efficiency_cores: 4,
    /// # };
    /// assert_eq!(cpu_info.num_performance_cores(), 4);
    /// ```
    pub fn num_performance_cores(&self) -> usize {
        self.total_performance_cores
    }

    /// Returns the total number of efficiency-type physical cores in the system.
    ///
    /// This is relevant for hybrid architectures (e.g., Intel E-cores, ARM LITTLE cores).
    /// Returns 0 if the system does not have a hybrid architecture or if this information
    /// could not be determined.
    /// This is a convenience method equivalent to accessing `self.total_efficiency_cores`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use gdt_cpus::{CpuInfo, Vendor, CpuFeatures, SocketInfo};
    /// # let cpu_info = CpuInfo {
    /// #     vendor: Vendor::Unknown, model_name: String::new(), features: CpuFeatures::empty(),
    /// #     sockets: Vec::new(), total_sockets: 0, total_physical_cores: 8,
    /// #     total_logical_processors: 16, total_performance_cores: 4, total_efficiency_cores: 4,
    /// # };
    /// assert_eq!(cpu_info.num_efficiency_cores(), 4);
    /// ```
    pub fn num_efficiency_cores(&self) -> usize {
        self.total_efficiency_cores
    }

    /// Returns `true` if the system CPU has a hybrid architecture (e.g., both P-cores and E-cores).
    ///
    /// This is determined by checking if both `total_performance_cores` and
    /// `total_efficiency_cores` are greater than zero.
    ///
    /// # Examples
    ///
    /// ```
    /// # use gdt_cpus::{CpuInfo, Vendor, CpuFeatures, SocketInfo};
    /// # let hybrid_cpu_info = CpuInfo {
    /// #     vendor: Vendor::Unknown, model_name: String::new(), features: CpuFeatures::empty(),
    /// #     sockets: Vec::new(), total_sockets: 0, total_physical_cores: 8,
    /// #     total_logical_processors: 16, total_performance_cores: 4, total_efficiency_cores: 4,
    /// # };
    /// # let non_hybrid_cpu_info = CpuInfo {
    /// #     vendor: Vendor::Unknown, model_name: String::new(), features: CpuFeatures::empty(),
    /// #     sockets: Vec::new(), total_sockets: 0, total_physical_cores: 4,
    /// #     total_logical_processors: 8, total_performance_cores: 4, total_efficiency_cores: 0,
    /// # };
    /// assert!(hybrid_cpu_info.is_hybrid());
    /// assert!(!non_hybrid_cpu_info.is_hybrid());
    /// ```
    pub fn is_hybrid(&self) -> bool {
        self.total_performance_cores > 0 && self.total_efficiency_cores > 0
    }

    /// Returns a flat list of all OS-specific logical processor IDs in the system.
    ///
    /// These IDs can be used, for example, when setting thread affinity. The order
    /// of IDs is generally not guaranteed but often follows socket and core enumeration.
    ///
    /// # Examples
    ///
    /// ```
    /// # use gdt_cpus::{CpuInfo, Vendor, CpuFeatures, SocketInfo, CoreInfo, CoreType};
    /// # let core1 = CoreInfo { id: 0, socket_id: 0, core_type: CoreType::Performance, logical_processor_ids: vec![0, 1], l1_instruction_cache: None, l1_data_cache: None, l2_cache: None };
    /// # let core2 = CoreInfo { id: 1, socket_id: 0, core_type: CoreType::Performance, logical_processor_ids: vec![2, 3], l1_instruction_cache: None, l1_data_cache: None, l2_cache: None };
    /// # let socket0 = SocketInfo { id: 0, cores: vec![core1, core2], l3_cache: None };
    /// # let cpu_info = CpuInfo {
    /// #     vendor: Vendor::Unknown, model_name: String::new(), features: CpuFeatures::empty(),
    /// #     sockets: vec![socket0], total_sockets: 1, total_physical_cores: 2,
    /// #     total_logical_processors: 4, total_performance_cores: 2, total_efficiency_cores: 0,
    /// # };
    /// let ids = cpu_info.logical_processor_ids();
    /// assert_eq!(ids, vec![0, 1, 2, 3]);
    /// ```
    pub fn logical_processor_ids(&self) -> Vec<usize> {
        self.sockets
            .iter()
            .flat_map(|socket| {
                socket
                    .cores
                    .iter()
                    .flat_map(|core| core.logical_processor_ids.clone())
            })
            .collect()
    }

    pub fn all_cores_mask(&self) -> AffinityMask {
        AffinityMask::from_cores(&self.logical_processor_ids())
    }

    pub fn performance_core_mask(&self) -> AffinityMask {
        self.cores_by_type_mask(CoreType::Performance)
    }

    pub fn efficiency_core_mask(&self) -> AffinityMask {
        self.cores_by_type_mask(CoreType::Efficiency)
    }

    pub fn cores_by_type_mask(&self, core_type: CoreType) -> AffinityMask {
        let core_ids: Vec<usize> = self
            .sockets
            .iter()
            .flat_map(|socket| {
                socket.cores.iter().filter_map(|core| {
                    if core.core_type == core_type {
                        Some(core.logical_processor_ids.clone())
                    } else {
                        None
                    }
                })
            })
            .flatten()
            .collect();

        AffinityMask::from_cores(&core_ids)
    }
}
