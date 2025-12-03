//! GDT-CPUs: Game Developer's Toolkit for CPU Management
//!
//! This crate provides detailed CPU information and thread management capabilities
//! specifically designed for game developers. It aims to simplify tasks such as
//! querying CPU features, understanding core architecture (including hybrid designs
//! like P-cores and E-cores), and managing thread affinity and priority.
//!
//! # Key Features
//!
//! *   **Detailed CPU Information**: Access vendor, model name, supported instruction sets
//!     (e.g., AVX2, NEON), cache details, and core topology via the [`CpuInfo`] struct.
//! *   **Hybrid Architecture Support**: Differentiates between performance and
//!     efficiency cores.
//! *   **Thread Affinity**: (Via the `affinity` module) Pin threads to specific
//!     logical or physical cores.
//! *   **Thread Priority**: (Via the `affinity` module) Set thread priorities for
//!     different operating systems.
//! *   **Platform Abstraction**: Provides a consistent API across Windows, macOS, and Linux.
//! *   **Lazy Initialization**: CPU information is detected once and cached globally.
//!
//! # Getting Started
//!
//! The primary way to get CPU information is through the [`cpu_info()`] function:
//!
//! ```
//! use gdt_cpus::{cpu_info, CpuInfo, Error, CpuFeatures};
//!
//! fn main() -> Result<(), Error> {
//!     let info = cpu_info()?;
//!
//!     println!("CPU Vendor: {}", info.vendor);
//!     println!("CPU Model: {}", info.model_name);
//!     println!("Total Physical Cores: {}", info.total_physical_cores);
//!     println!("Total Logical Processors: {}", info.total_logical_processors);
//!
//!     if info.is_hybrid() {
//!         println!("This is a hybrid CPU with:");
//!         println!("  Performance Cores: {}", info.total_performance_cores);
//!         println!("  Efficiency Cores: {}", info.total_efficiency_cores);
//!     }
//!
//!     #[cfg(target_arch = "x86_64")]
//!     if info.features.contains(CpuFeatures::AVX2) {
//!         println!("AVX2 is supported!");
//!     }
//!     #[cfg(target_arch = "aarch64")]
//!     if info.features.contains(CpuFeatures::NEON) {
//!         println!("NEON is supported!");
//!     }
//!
//!     // You can also use helper functions:
//!     let phys_cores = gdt_cpus::num_physical_cores()?;
//!     println!("Physical cores (via helper): {}", phys_cores);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Cargo Features
//!
//! *   `serde`: Enables serialization and deserialization of CPU information structures
//!     (like [`CpuInfo`], [`CoreInfo`], etc.) using the Serde library.

// #![forbid(unsafe_code)] // Will be selectively allowed in platform-specific modules

// Modules
mod affinity;
mod affinity_mask;
mod cpu;
mod error;
mod platform;
mod priority;

// Re-exports - Public API
pub use affinity::*;
pub use affinity_mask::AffinityMask;
pub use cpu::{
    CacheInfo, CacheLevel, CacheType, CoreInfo, CoreType, CpuFeatures, CpuInfo, SocketInfo, Vendor,
};
pub use error::{Error, Result};
pub use platform::SchedulingPolicy;
pub use priority::ThreadPriority;

/// Retrieves a static reference to the globally detected CPU information.
///
/// This function utilizes a thread-safe lazy initialization pattern. The CPU
/// information is detected only once during the first call to this function
/// (or any related helper function like [`num_physical_cores()`]) within the
/// program's execution. Subsequent calls return a reference to the cached data.
///
/// # Returns
///
/// A `Result<&'static CpuInfo, Error>`. You'll typically want to handle the `Result`
/// to access the `CpuInfo` struct.
///
/// # Examples
///
/// ```
/// use gdt_cpus::{cpu_info, CpuInfo, Error};
///
/// match cpu_info() {
///     Ok(info) => {
///         println!("CPU Model: {}", info.model_name);
///         println!("This system has {} physical cores.", info.num_physical_cores());
///     }
///     Err(e) => {
///         eprintln!("Failed to get CPU info: {:?}", e);
///     }
/// }
/// ```
///
/// For more direct access to the `CpuInfo` struct if successful, you might do:
/// ```
/// # use gdt_cpus::{cpu_info, CpuInfo, Error};
/// # fn main() -> Result<(), Error> {
/// let info = cpu_info()?;
/// println!("Successfully retrieved CPU info for: {}", info.model_name);
/// # Ok(())
/// # }
/// ```
pub fn cpu_info() -> Result<&'static CpuInfo> {
    static CPU_INFO: std::sync::OnceLock<Result<CpuInfo>> = std::sync::OnceLock::new();
    match CPU_INFO.get_or_init(CpuInfo::detect) {
        Ok(cpu_info) => Ok(cpu_info),
        Err(e) => Err(e.clone()),
    }
}

/// Returns the total number of physical cores in the system.
///
/// This is a convenience function that calls [`cpu_info()`] and extracts
/// `total_physical_cores` from the [`CpuInfo`] struct.
///
/// # Returns
///
/// A `Result<usize, Error>` containing the number of physical cores on success.
///
/// # Examples
///
/// ```
/// use gdt_cpus::num_physical_cores;
///
/// match num_physical_cores() {
///     Ok(count) => println!("Number of physical cores: {}", count),
///     Err(e) => eprintln!("Error getting physical core count: {:?}", e),
/// }
/// ```
pub fn num_physical_cores() -> Result<usize> {
    cpu_info().map(|info| info.num_physical_cores())
}

/// Returns the total number of logical cores (hardware threads) in the system.
///
/// This count includes threads from technologies like Intel's Hyper-Threading or AMD's SMT.
/// This is a convenience function that calls [`cpu_info()`] and extracts
/// `total_logical_processors` from the [`CpuInfo`] struct.
///
/// # Returns
///
/// A `Result<usize, Error>` containing the number of logical cores on success.
///
/// # Examples
///
/// ```
/// use gdt_cpus::num_logical_cores;
///
/// match num_logical_cores() {
///     Ok(count) => println!("Number of logical cores: {}", count),
///     Err(e) => eprintln!("Error getting logical core count: {:?}", e),
/// }
/// ```
pub fn num_logical_cores() -> Result<usize> {
    cpu_info().map(|info| info.num_logical_cores())
}

/// Returns the total number of performance-type physical cores in the system.
///
/// This is relevant for hybrid architectures (e.g., Intel P-cores, ARM big cores).
/// Returns number of physical cores if the system does not have a hybrid architecture.
/// This is a convenience function that calls [`cpu_info()`] and extracts
/// `total_performance_cores` from the [`CpuInfo`] struct.
///
/// # Returns
///
/// A `Result<usize, Error>` containing the number of performance cores on success.
///
/// # Examples
///
/// ```
/// use gdt_cpus::num_performance_cores;
///
/// match num_performance_cores() {
///     Ok(count) => println!("Number of performance cores: {}", count),
///     Err(e) => eprintln!("Error getting performance core count: {:?}", e),
/// }
/// ```
pub fn num_performance_cores() -> Result<usize> {
    cpu_info().map(|info| info.num_performance_cores())
}

/// Returns the total number of efficiency-type physical cores in the system.
///
/// This is relevant for hybrid architectures (e.g., Intel E-cores, ARM LITTLE cores).
/// Returns 0 if the system does not have a hybrid architecture or if this information
/// could not be determined.
/// This is a convenience function that calls [`cpu_info()`] and extracts
/// `total_efficiency_cores` from the [`CpuInfo`] struct.
///
/// # Returns
///
/// A `Result<usize, Error>` containing the number of efficiency cores on success.
///
/// # Examples
///
/// ```
/// use gdt_cpus::num_efficiency_cores;
///
/// match num_efficiency_cores() {
///     Ok(count) => println!("Number of efficiency cores: {}", count),
///     Err(e) => eprintln!("Error getting efficiency core count: {:?}", e),
/// }
/// ```
pub fn num_efficiency_cores() -> Result<usize> {
    cpu_info().map(|info| info.num_efficiency_cores())
}

/// Returns `true` if the system CPU has a hybrid architecture (e.g., both P-cores and E-cores).
///
/// This is determined by checking if both `total_performance_cores` and
/// `total_efficiency_cores` (from [`CpuInfo`]) are greater than zero.
/// This is a convenience function that calls [`cpu_info()`].
///
/// # Returns
///
/// A `Result<bool, Error>` which is `Ok(true)` if hybrid, `Ok(false)` if not, or an `Err`
/// if CPU information could not be obtained.
///
/// # Examples
///
/// ```
/// use gdt_cpus::is_hybrid;
///
/// match is_hybrid() {
///     Ok(hybrid) => if hybrid {
///         println!("This system has a hybrid CPU architecture.");
///     } else {
///         println!("This system does not have a hybrid CPU architecture.");
///     }
///     Err(e) => eprintln!("Error determining if CPU is hybrid: {:?}", e),
/// }
/// ```
pub fn is_hybrid() -> Result<bool> {
    cpu_info().map(|info| info.is_hybrid())
}
