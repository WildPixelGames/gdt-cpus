#![warn(missing_docs)]
//!
//! C-style Foreign Function Interface (FFI) for the `gdt-cpus` crate.
//!
//! This crate provides a C-compatible API to access CPU information and control
//! thread affinity/priority using the underlying `gdt-cpus` Rust library.
//!
//! It is intended for use in applications written in languages other than Rust
//! (e.g., C, C++, C#) that need to integrate with `gdt-cpus` functionality.
//!
//! # Key Features:
//! - **CPU Information:** Retrieve detailed CPU information, including vendor, model name,
//!   core counts (physical, logical, performance, efficiency), socket details, and cache info.
//! - **Thread Management:** Pin threads to specific cores and set thread priorities.
//! - **Error Handling:** Functions typically return an error code (`GdtCpusErrorCode`),
//!   and output parameters are passed as pointers.
//! - **String Handling:** Strings returned by this API (e.g., model name) are `*const c_char`
//!   and are valid for the lifetime of the program.
//!
//! # Usage Example (Conceptual C Code)
//! ```c
//! // #include "gdt_cpus.h"
//! //
//! // GdtCpusCpuInfo cpu_info;
//! // if (gdt_cpus_cpu_info(&cpu_info) == GdtCpusErrorCode_Success) {
//! //     printf("CPU Model: %s\n", cpu_info.model_name);
//! //     printf("Physical Cores: %llu\n", cpu_info.total_physical_cores);
//! // }
//! //
//! // // Pin current thread to logical core 0
//! // gdt_cpus_pin_thread_to_core(0);
//! //
//! // // Set thread priority to high
//! // gdt_cpus_set_thread_priority(GDT_CPUS_THREAD_PRIORITY_TIME_CRITICAL);
//! ```
//!
//! See the individual function and type documentation for more details.

use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::OnceLock;

use gdt_cpus::{CoreType, ThreadPriority};

static CPU_INFO_CONTAINER: OnceLock<CpuInfoContainer> = OnceLock::new();

macro_rules! get_info_validate_out_or_err {
    ($out:ident) => {{
        if $out.is_null() {
            return GdtCpusErrorCode::InvalidParameter as i32;
        }
        match gdt_cpus::cpu_info() {
            Ok(i) => i,
            Err(e) => return GdtCpusErrorCode::from(&e) as i32,
        }
    }};
}

struct CpuInfoContainer {
    model_name_storage: CString,
    vendor_name_storage: CString,
}

/// Top-level structure containing all detected CPU information, exposed via FFI.
///
/// This structure provides a C-compatible representation of the main CPU details
/// gathered by the `gdt-cpus` library. It is simplified for common game development needs.
///
/// Pointers to strings (`vendor_name`, `model_name`) within this struct are valid for the
/// lifetime of the program after `gdt_cpus_cpu_info` has been successfully called, as
/// the underlying data is stored in a static `OnceLock`.
#[repr(C)]
pub struct GdtCpusCpuInfo {
    /// Detected CPU vendor (e.g., Intel, AMD).
    pub vendor: GdtCpusVendor,
    /// Pointer to a null-terminated C string representing the CPU vendor name (e.g., "GenuineIntel").
    pub vendor_name: *const c_char,
    /// Pointer to a null-terminated C string representing the CPU model name (e.g., "Intel(R) Core(TM) i7-8700K CPU @ 3.70GHz").
    pub model_name: *const c_char,
    /// Bitmask of CPU features. The meaning of bits depends on the architecture.
    /// See [`GdtCpusCpuFeatures`] (defined per-architecture).
    pub features: u32,
    /// Number of CPU sockets detected and reported by the library.
    pub sockets_count: u64,
    /// Total number of physical CPU sockets. Often the same as `sockets_count`.
    pub total_sockets: u64,
    /// Total number of physical cores across all sockets.
    pub total_physical_cores: u64,
    /// Total number of logical processors (hardware threads) across all sockets.
    pub total_logical_processors: u64,
    /// Total number of performance-type physical cores (P-cores) if the CPU has a hybrid architecture.
    /// This will be total number of physical cores on non-hybrid CPUs.
    pub total_performance_cores: u64,
    /// Total number of efficiency-type physical cores (E-cores) if the CPU has a hybrid architecture.
    /// This will be 0 on non-hybrid CPUs or if the distinction is not applicable/detectable.
    pub total_efficiency_cores: u64,
}

/// C-compatible structure representing information about a CPU cache level.
#[repr(C)]
pub struct GdtCpusCacheInfo {
    /// The level of the cache (e.g., L1, L2, L3).
    pub level: GdtCpusCacheLevel,
    /// The type of data stored in the cache (e.g., Data, Instruction, Unified).
    pub cache_type: GdtCpusCacheType,
    /// Total size of the cache in bytes.
    pub size_bytes: u64,
    /// Size of a single cache line in bytes.
    pub line_size_bytes: u64,
}

/// C-compatible structure representing information about a single CPU core.
#[repr(C)]
pub struct GdtCpusCoreInfo {
    /// Unique identifier for this core within the system.
    pub id: u64,
    /// Identifier of the CPU socket this core belongs to.
    pub socket_id: u64,
    /// Type of the core (e.g., Performance, Efficiency).
    pub core_type: GdtCpusCoreType,
    /// Number of logical processors (hardware threads) this physical core provides.
    pub logical_processor_ids_count: u64,
    /// True if this core has a dedicated L1 instruction cache, information for which is in `l1_instruction_cache`.
    pub has_l1_instruction_cache: bool,
    /// L1 instruction cache information. Valid if `has_l1_instruction_cache` is true.
    pub l1_instruction_cache: GdtCpusCacheInfo,
    /// True if this core has a dedicated L1 data cache, information for which is in `l1_data_cache`.
    pub has_l1_data_cache: bool,
    /// L1 data cache information. Valid if `has_l1_data_cache` is true.
    pub l1_data_cache: GdtCpusCacheInfo,
    /// True if this core has a dedicated L2 cache, information for which is in `l2_cache`.
    pub has_l2_cache: bool,
    /// L2 cache information. Valid if `has_l2_cache` is true.
    pub l2_cache: GdtCpusCacheInfo,
}

/// C-compatible structure representing information about a CPU socket.
#[repr(C)]
pub struct GdtCpusSocketInfo {
    /// Unique identifier for this CPU socket.
    pub id: u64,
    /// Number of physical cores belonging to this socket.
    pub cores_count: u64,
    /// True if this socket has a shared L3 cache, information for which is in `l3_cache`.
    pub has_l3_cache: bool,
    /// L3 cache information, typically shared by cores on this socket. Valid if `has_l3_cache` is true.
    pub l3_cache: GdtCpusCacheInfo,
}

/// C-compatible enumeration for CPU vendors.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum GdtCpusVendor {
    /// Intel CPU vendor.
    Intel = 0,
    /// AMD CPU vendor.
    Amd = 1,
    /// ARM CPU vendor.
    Arm = 2,
    /// Apple CPU vendor.
    Apple = 3,
    /// Unknown CPU vendor.
    Unknown = 4,
    /// Vendor not recognized by the library.
    Other = 5,
}

/// C-compatible enumeration for CPU cache levels.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum GdtCpusCacheLevel {
    /// L1 cache.
    L1 = 0,
    /// L2 cache.
    L2 = 1,
    /// L3 cache.
    L3 = 2,
    /// L4 cache (rare).
    L4 = 3,
    /// Unknown cache level.
    Unknown = 4,
}

/// C-compatible enumeration for CPU cache types.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum GdtCpusCacheType {
    /// Unified cache (stores both instructions and data).
    Unified = 0,
    /// Instruction cache.
    Instruction = 1,
    /// Data cache.
    Data = 2,
    /// Trace cache (micro-op cache).
    Trace = 3,
    /// Unknown cache type.
    Unknown = 4,
}

/// C-compatible enumeration for CPU features on x86_64 architecture.
///
/// This is a bitmask. A CPU might support multiple features.
#[cfg(target_arch = "x86_64")]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum GdtCpusCpuFeatures {
    /// MMX (MultiMedia eXtensions) support.
    MMX = 0x00000001,
    /// SSE (Streaming SIMD Extensions) support.
    SSE = 0x00000002,
    /// SSE2 (Streaming SIMD Extensions 2) support.
    SSE2 = 0x00000004,
    /// SSE3 (Streaming SIMD Extensions 3) support.
    SSE3 = 0x00000008,
    /// SSSE3 (Supplemental Streaming SIMD Extensions 3) support.
    SSSE3 = 0x00000010,
    /// SSE4.1 (Streaming SIMD Extensions 4.1) support.
    SSE4_1 = 0x00000020,
    /// SSE4.2 (Streaming SIMD Extensions 4.2) support.
    SSE4_2 = 0x00000040,
    /// FMA3 (Fused Multiply-Add 3-operand) support.
    FMA3 = 0x00000080,
    /// AVX (Advanced Vector Extensions) support.
    AVX = 0x00000100,
    /// AVX2 (Advanced Vector Extensions 2) support.
    AVX2 = 0x00000200,
    /// AVX-512 Foundation support.
    AVX512F = 0x00000400,
    /// AVX-512 Byte and Word Instructions support.
    AVX512BW = 0x00000800,
    /// AVX-512 Conflict Detection Instructions support.
    AVX512CD = 0x00001000,
    /// AVX-512 Doubleword and Quadword Instructions support.
    AVX512DQ = 0x00002000,
    /// AVX-512 Vector Length Extensions support.
    AVX512VL = 0x00004000,
    /// AES (Advanced Encryption Standard) hardware acceleration support.
    AES = 0x00008000,
    /// SHA (Secure Hash Algorithm) hardware acceleration support.
    SHA = 0x00010000,
    /// CRC32 (Cyclic Redundancy Check) hardware acceleration support.
    CRC32 = 0x00020000,
}

/// C-compatible enumeration for CPU features on aarch64 architecture.
///
/// This is a bitmask. A CPU might support multiple features.
#[cfg(target_arch = "aarch64")]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum GdtCpusCpuFeatures {
    /// NEON (Advanced SIMD) support.
    NEON = 0x00000001,
    /// SVE (Scalable Vector Extension) support.
    SVE = 0x00000002,
    /// AES (Advanced Encryption Standard) hardware acceleration support.
    AES = 0x00000004,
    /// SHA (Secure Hash Algorithm) hardware acceleration support (SHA1, SHA256, SHA512).
    SHA = 0x00000008,
    /// CRC32 (Cyclic Redundancy Check) hardware acceleration support.
    CRC32 = 0x00000010,
}

/// C-compatible enumeration for error codes returned by FFI functions.
///
/// `Success` (0) indicates no error. Negative values indicate errors.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum GdtCpusErrorCode {
    /// No error.
    Success = 0,
    /// Error during CPU detection.
    Detection = -1,
    /// Invalid core ID.
    InvalidCoreId = -2,
    /// No core of the requested type found.
    NoCoreOfType = -3,
    /// Error setting thread affinity.
    Affinity = -4,
    /// Unsupported operation.
    Unsupported = -5,
    /// Permission denied.
    PermissionDenied = -6,
    /// I/O error.
    Io = -7,
    /// System call error.
    SystemCall = -8,
    /// Resource not found.
    NotFound = -9,
    /// Invalid parameter.
    InvalidParameter = -10,
    /// Operation not implemented.
    NotImplemented = -11,
    /// Out of bounds error.
    OutOfBounds = -12,
    /// Unknown error.
    Unknown = -999,
}

/// Core type enumeration.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum GdtCpusCoreType {
    /// Performance core.
    Performance = 0,
    /// Efficiency core.
    Efficiency = 1,
    /// Unknown core type.
    Unknown = 2,
}

/// Thread priority levels.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum GdtCpusThreadPriority {
    /// Background priority.
    ///
    /// Example workload:
    /// - Steam API, achievement sync, cloud saves, etc.
    /// - Absolute background noise.
    ///
    /// **Note (Linux):** Uses SCHED_OTHER + `nice(19)`; under heavy load, p99 latency
    /// can spike into **hundreds of milliseconds** or even seconds.
    Background = 0,
    /// Lowest priority.
    ///
    /// Example workload:
    /// - Analytics, telemetry, stats collection, etc.
    /// - Doesn’t impact gameplay in any way.
    ///
    /// **Note (Linux):** Uses SCHED_OTHER + `nice(15)`; tail‐latencies similar to `Background`.
    Lowest = 1,
    /// Below normal priority.
    ///
    /// Example workload:
    /// - Async workers, secondary systems, AI planning, non-urgent gameplay systems.
    /// - Can be preempted by higher-priority tasks.
    ///
    /// **Note (Linux):** Uses SCHED_OTHER + `nice(10)`; may suffer long tail‐latencies under contention.
    BelowNormal = 2,
    /// Normal priority.
    ///
    /// Example workload:
    /// - Asset loading, streaming, prefetching, etc.
    /// - Typically I/O-bound but latency matters.
    ///
    /// **Note (Linux):** Uses SCHED_OTHER + `nice(0)`; no real RT guarantees under heavy load.
    Normal = 3,
    /// Above normal priority.
    ///
    /// Example workload:
    /// - Main game thread, logic, input, UI thread, etc.
    /// - Needs to be responsive.
    ///
    /// **Note (Linux):** Uses SCHED_OTHER + `nice(-5)`; still not real-time—latency spikes possible.
    AboveNormal = 4,
    /// Highest priority.
    ///
    /// Example workload:
    /// - Render thread, audio thread (deadline-sensitive).
    /// - Needs to finish on time or else!
    ///
    /// **Note:** Maps to real-time SCHED_RR (requires CAP_SYS_NICE or root) with high absolute priority.
    Highest = 5,
    /// Time-critical/real-time priority (use with caution).
    ///
    /// Example workload:
    /// - Worker threads on P-cores, no mercy.
    /// - Full power, minimum latency.
    ///
    /// **Note:** Maps to top-end real-time SCHED_RR (requires CAP_SYS_NICE or root); virtually no tail-latency.
    TimeCritical = 6,
}

impl From<&gdt_cpus::Vendor> for GdtCpusVendor {
    fn from(vendor: &gdt_cpus::Vendor) -> Self {
        match vendor {
            gdt_cpus::Vendor::Intel => GdtCpusVendor::Intel,
            gdt_cpus::Vendor::Amd => GdtCpusVendor::Amd,
            gdt_cpus::Vendor::Arm => GdtCpusVendor::Arm,
            gdt_cpus::Vendor::Apple => GdtCpusVendor::Apple,
            gdt_cpus::Vendor::Unknown => GdtCpusVendor::Unknown,
            gdt_cpus::Vendor::Other(_) => GdtCpusVendor::Other,
        }
    }
}

impl From<&gdt_cpus::CacheLevel> for GdtCpusCacheLevel {
    fn from(level: &gdt_cpus::CacheLevel) -> Self {
        match level {
            gdt_cpus::CacheLevel::L1 => GdtCpusCacheLevel::L1,
            gdt_cpus::CacheLevel::L2 => GdtCpusCacheLevel::L2,
            gdt_cpus::CacheLevel::L3 => GdtCpusCacheLevel::L3,
            gdt_cpus::CacheLevel::L4 => GdtCpusCacheLevel::L4,
            gdt_cpus::CacheLevel::Unknown => GdtCpusCacheLevel::Unknown,
        }
    }
}

impl From<&gdt_cpus::CacheType> for GdtCpusCacheType {
    fn from(cache_type: &gdt_cpus::CacheType) -> Self {
        match cache_type {
            gdt_cpus::CacheType::Unified => GdtCpusCacheType::Unified,
            gdt_cpus::CacheType::Instruction => GdtCpusCacheType::Instruction,
            gdt_cpus::CacheType::Data => GdtCpusCacheType::Data,
            gdt_cpus::CacheType::Trace => GdtCpusCacheType::Trace,
            gdt_cpus::CacheType::Unknown => GdtCpusCacheType::Unknown,
        }
    }
}

impl From<&gdt_cpus::Error> for GdtCpusErrorCode {
    fn from(err: &gdt_cpus::Error) -> Self {
        match err {
            gdt_cpus::Error::Detection(_) => GdtCpusErrorCode::Detection,
            gdt_cpus::Error::InvalidCoreId(_) => GdtCpusErrorCode::InvalidCoreId,
            gdt_cpus::Error::NoCoreOfType(_) => GdtCpusErrorCode::NoCoreOfType,
            gdt_cpus::Error::Affinity(_) => GdtCpusErrorCode::Affinity,
            gdt_cpus::Error::Unsupported(_) => GdtCpusErrorCode::Unsupported,
            gdt_cpus::Error::PermissionDenied(_) => GdtCpusErrorCode::PermissionDenied,
            gdt_cpus::Error::Io(_) => GdtCpusErrorCode::Io,
            gdt_cpus::Error::SystemCall(_) => GdtCpusErrorCode::SystemCall,
            gdt_cpus::Error::NotFound(_) => GdtCpusErrorCode::NotFound,
            gdt_cpus::Error::InvalidParameter(_) => GdtCpusErrorCode::InvalidParameter,
            gdt_cpus::Error::NotImplemented => GdtCpusErrorCode::NotImplemented,
        }
    }
}

impl From<&gdt_cpus::CoreType> for GdtCpusCoreType {
    fn from(ct: &CoreType) -> Self {
        match ct {
            CoreType::Performance => GdtCpusCoreType::Performance,
            CoreType::Efficiency => GdtCpusCoreType::Efficiency,
            CoreType::Unknown => GdtCpusCoreType::Unknown,
        }
    }
}

impl From<GdtCpusThreadPriority> for ThreadPriority {
    fn from(tp: GdtCpusThreadPriority) -> Self {
        match tp {
            GdtCpusThreadPriority::Background => ThreadPriority::Background,
            GdtCpusThreadPriority::Lowest => ThreadPriority::Lowest,
            GdtCpusThreadPriority::BelowNormal => ThreadPriority::BelowNormal,
            GdtCpusThreadPriority::Normal => ThreadPriority::Normal,
            GdtCpusThreadPriority::AboveNormal => ThreadPriority::AboveNormal,
            GdtCpusThreadPriority::Highest => ThreadPriority::Highest,
            GdtCpusThreadPriority::TimeCritical => ThreadPriority::TimeCritical,
        }
    }
}

impl From<&gdt_cpus::CacheInfo> for GdtCpusCacheInfo {
    fn from(cache: &gdt_cpus::CacheInfo) -> Self {
        GdtCpusCacheInfo {
            level: GdtCpusCacheLevel::from(&cache.level),
            cache_type: GdtCpusCacheType::from(&cache.cache_type),
            size_bytes: cache.size_bytes,
            line_size_bytes: cache.line_size_bytes as u64,
        }
    }
}

impl From<&gdt_cpus::SocketInfo> for GdtCpusSocketInfo {
    fn from(socket: &gdt_cpus::SocketInfo) -> Self {
        GdtCpusSocketInfo {
            id: socket.id as u64,
            cores_count: socket.cores.len() as u64,
            has_l3_cache: socket.l3_cache.is_some(),
            l3_cache: if let Some(l3_cache) = &socket.l3_cache {
                GdtCpusCacheInfo::from(l3_cache)
            } else {
                GdtCpusCacheInfo::default()
            },
        }
    }
}

impl From<&gdt_cpus::CoreInfo> for GdtCpusCoreInfo {
    fn from(core: &gdt_cpus::CoreInfo) -> Self {
        GdtCpusCoreInfo {
            id: core.id as u64,
            socket_id: core.socket_id as u64,
            core_type: GdtCpusCoreType::from(&core.core_type),
            logical_processor_ids_count: core.logical_processor_ids.len() as u64,
            has_l1_instruction_cache: core.l1_instruction_cache.is_some(),
            l1_instruction_cache: if let Some(l1i_cache) = &core.l1_instruction_cache {
                GdtCpusCacheInfo::from(l1i_cache)
            } else {
                GdtCpusCacheInfo::default()
            },
            has_l1_data_cache: core.l1_data_cache.is_some(),
            l1_data_cache: if let Some(l1d_cache) = &core.l1_data_cache {
                GdtCpusCacheInfo::from(l1d_cache)
            } else {
                GdtCpusCacheInfo::default()
            },
            has_l2_cache: core.l2_cache.is_some(),
            l2_cache: if let Some(l2_cache) = &core.l2_cache {
                GdtCpusCacheInfo::from(l2_cache)
            } else {
                GdtCpusCacheInfo::default()
            },
        }
    }
}

impl Default for GdtCpusCacheInfo {
    fn default() -> Self {
        GdtCpusCacheInfo {
            level: GdtCpusCacheLevel::Unknown,
            cache_type: GdtCpusCacheType::Unknown,
            size_bytes: 0,
            line_size_bytes: 0,
        }
    }
}

/// Returns a description of the error code.
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_error_code_description(error_code: GdtCpusErrorCode) -> *const c_char {
    let description = match error_code {
        GdtCpusErrorCode::Success => c"Success",
        GdtCpusErrorCode::Detection => c"CPU detection error",
        GdtCpusErrorCode::InvalidCoreId => c"Invalid core ID",
        GdtCpusErrorCode::NoCoreOfType => c"No core of the requested type found",
        GdtCpusErrorCode::Affinity => c"Thread affinity error",
        GdtCpusErrorCode::Unsupported => c"Unsupported operation",
        GdtCpusErrorCode::PermissionDenied => c"Permission denied",
        GdtCpusErrorCode::Io => c"I/O error",
        GdtCpusErrorCode::SystemCall => c"System call error",
        GdtCpusErrorCode::NotFound => c"Not found",
        GdtCpusErrorCode::InvalidParameter => c"Invalid parameter",
        GdtCpusErrorCode::NotImplemented => c"Operation not implemented",
        _ => c"Unknown error",
    };

    description.as_ptr()
}

/// Returns a description of the CPU vendor.
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_vendor_description(vendor: GdtCpusVendor) -> *const c_char {
    let description = match vendor {
        GdtCpusVendor::Intel => c"Intel",
        GdtCpusVendor::Amd => c"AMD",
        GdtCpusVendor::Arm => c"ARM",
        GdtCpusVendor::Apple => c"Apple",
        GdtCpusVendor::Unknown => c"Unknown",
        GdtCpusVendor::Other => c"Other",
    };

    description.as_ptr()
}

/// Returns a description of the cache level.
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_cache_level_description(cache_level: GdtCpusCacheLevel) -> *const c_char {
    let description = match cache_level {
        GdtCpusCacheLevel::L1 => c"L1",
        GdtCpusCacheLevel::L2 => c"L2",
        GdtCpusCacheLevel::L3 => c"L3",
        GdtCpusCacheLevel::L4 => c"L4",
        GdtCpusCacheLevel::Unknown => c"Unknown Cache Level",
    };

    description.as_ptr()
}

/// Returns a description of the cache type.
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_cache_type_description(cache_type: GdtCpusCacheType) -> *const c_char {
    let description = match cache_type {
        GdtCpusCacheType::Unified => c"Unified",
        GdtCpusCacheType::Instruction => c"Instruction",
        GdtCpusCacheType::Data => c"Data",
        GdtCpusCacheType::Trace => c"Trace",
        GdtCpusCacheType::Unknown => c"Unknown Cache Type",
    };

    description.as_ptr()
}

/// Returns a description of the core type.
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_core_type_description(core_type: GdtCpusCoreType) -> *const c_char {
    let description = match core_type {
        GdtCpusCoreType::Performance => c"Performance",
        GdtCpusCoreType::Efficiency => c"Efficiency",
        GdtCpusCoreType::Unknown => c"Unknown Core Type",
    };

    description.as_ptr()
}

/// Returns a description of the core type.
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_thread_priority_description(
    priority: GdtCpusThreadPriority,
) -> *const c_char {
    let description = match priority {
        GdtCpusThreadPriority::Background => c"Background",
        GdtCpusThreadPriority::Lowest => c"Lowest",
        GdtCpusThreadPriority::BelowNormal => c"Below Normal",
        GdtCpusThreadPriority::Normal => c"Normal",
        GdtCpusThreadPriority::AboveNormal => c"Above Normal",
        GdtCpusThreadPriority::Highest => c"Highest",
        GdtCpusThreadPriority::TimeCritical => c"Time Critical",
    };

    description.as_ptr()
}

/// Retrieves comprehensive information about the CPU(s) in the system.
///
/// This function populates the `out_info` output parameter with a `GdtCpusCpuInfo` struct,
/// which contains details such as CPU vendor, model name, feature flags, and counts of
/// various core types (physical, logical, performance, efficiency) and sockets.
///
/// The `model_name` and `vendor_name` fields within the `GdtCpusCpuInfo` struct are pointers
/// to C strings. These strings are managed internally and are guaranteed to be valid for the
/// lifetime of the program after the first call to this function. Subsequent calls will return
/// pointers to the same statically allocated strings.
///
/// # Arguments
///
/// * `out_info`: A mutable pointer to a `GdtCpusCpuInfo` struct where the CPU information will be written.
///               The data stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful retrieval.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on failure.
///
/// # Safety
///
/// The caller must ensure that `out_info` is a valid pointer to a mutable `GdtCpusCpuInfo` memory location.
/// The memory pointed to by `out_info` must be writable and properly aligned.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_cpu_info(out_info: *mut GdtCpusCpuInfo) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_info);

    let container = CPU_INFO_CONTAINER.get_or_init(|| {
        let model_name_cstring = CString::new(rust_info.model_name.clone())
            .unwrap_or_else(|_| CString::new("Unknown").unwrap());

        let vendor_name_cstring = CString::new(format!("{}", rust_info.vendor))
            .unwrap_or_else(|_| CString::new("Unknown").unwrap());

        CpuInfoContainer {
            model_name_storage: model_name_cstring,
            vendor_name_storage: vendor_name_cstring,
        }
    });

    unsafe {
        *out_info = GdtCpusCpuInfo {
            vendor: GdtCpusVendor::from(&rust_info.vendor),
            vendor_name: container.vendor_name_storage.as_ptr(),
            model_name: container.model_name_storage.as_ptr(),
            features: rust_info.features.bits() as u32,
            sockets_count: rust_info.sockets.len() as u64,
            total_sockets: rust_info.total_sockets as u64,
            total_physical_cores: rust_info.total_physical_cores as u64,
            total_logical_processors: rust_info.total_logical_processors as u64,
            total_performance_cores: rust_info.total_performance_cores as u64,
            total_efficiency_cores: rust_info.total_efficiency_cores as u64,
        };
    }

    GdtCpusErrorCode::Success as i32
}

/// Returns the CPU vendor.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_get_vendor(vendor: *mut GdtCpusVendor) -> i32 {
    let rust_info = get_info_validate_out_or_err!(vendor);

    unsafe {
        *vendor = GdtCpusVendor::from(&rust_info.vendor);
    }

    GdtCpusErrorCode::Success as i32
}

/// Retrieves the CPU feature flags as a bitmask.
///
/// This function populates the `out_features` output parameter with the CPU feature flags.
/// The specific meaning of each bit depends on the CPU architecture and is defined by `gdt_cpus::Features`.
///
/// # Arguments
///
/// * `out_features`: A mutable pointer to a `u32` where the CPU feature flags will be written.
///                   The value stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful retrieval.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on failure.
///
/// # Safety
///
/// The caller must ensure that `out_features` is a valid pointer to a mutable `u32` memory location.
/// The memory pointed to by `out_features` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_get_features(out_features: *mut u32) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_features);

    unsafe {
        *out_features = rust_info.features.bits() as u32;
    }

    GdtCpusErrorCode::Success as i32
}

/// Retrieves the total number of physical CPU cores available on the system.
///
/// This function populates the `out_num_physical_cores` output parameter with the count of physical cores.
///
/// # Arguments
///
/// * `out_num_physical_cores`: A mutable pointer to a `u64` where the total number of physical CPU cores will be written.
///                               The value stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful retrieval.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on failure.
///
/// # Safety
///
/// The caller must ensure that `out_num_physical_cores` is a valid pointer to a mutable `u64` memory location.
/// The memory pointed to by `out_num_physical_cores` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_num_physical_cores(out_num_physical_cores: *mut u64) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_num_physical_cores);

    unsafe {
        *out_num_physical_cores = rust_info.total_physical_cores as u64;
    }

    GdtCpusErrorCode::Success as i32
}

/// Retrieves the total number of logical CPUs (threads) available on the system.
///
/// This function populates the `out_num_logical_cores` output parameter with the count of logical CPUs.
///
/// # Arguments
///
/// * `out_num_logical_cores`: A mutable pointer to a `u64` where the total number of logical CPUs will be written.
///                            The value stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful retrieval.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on failure.
///
/// # Safety
///
/// The caller must ensure that `out_num_logical_cores` is a valid pointer to a mutable `u64` memory location.
/// The memory pointed to by `out_num_logical_cores` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_num_logical_cores(out_num_logical_cores: *mut u64) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_num_logical_cores);

    unsafe {
        *out_num_logical_cores = rust_info.total_logical_processors as u64;
    }

    GdtCpusErrorCode::Success as i32
}

/// Retrieves the number of logical CPUs (threads) per physical core.
///
/// This function assumes a symmetrical multiprocessing (SMP) system where each core has the same number of logical CPUs.
/// It populates the `out_num_logical_cpus_per_core` output parameter with this count.
///
/// # Arguments
///
/// * `out_num_logical_cpus_per_core`: A mutable pointer to a `u64` where the number of logical CPUs per core will be written.
///                                    The value stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful retrieval.
/// * `GdtCpusErrorCode::NotSupported` if the system is not SMP or the information cannot be determined reliably for a single value.
/// * Other error codes from `GdtCpusErrorCode` (as `i32`) on failure.
///
/// # Safety
///
/// The caller must ensure that `out_num_logical_cpus_per_core` is a valid pointer to a mutable `u64` memory location.
/// The memory pointed to by `out_num_logical_cpus_per_core` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_num_logical_cpus_per_core(out_num_logical_cpus_per_core: *mut u64) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_num_logical_cpus_per_core);

    let logical_cpus_per_core = rust_info.total_logical_processors / rust_info.total_physical_cores;

    unsafe {
        *out_num_logical_cpus_per_core = logical_cpus_per_core as u64;
    }

    GdtCpusErrorCode::Success as i32
}

/// Retrieves the total number of performance cores (P-cores) available on the system.
///
/// This function is typically relevant for hybrid architecture CPUs (e.g., Intel Alder Lake and newer).
/// It populates the `out_num_performance_cores` output parameter with the count.
///
/// # Arguments
///
/// * `out_num_performance_cores`: A mutable pointer to a `u64` where the total number of performance cores will be written.
///                                  The value stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful retrieval.
/// * `GdtCpusErrorCode::NotSupported` if the system does not differentiate core types or the count is zero.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on other failures.
///
/// # Safety
///
/// The caller must ensure that `out_num_performance_cores` is a valid pointer to a mutable `u64` memory location.
/// The memory pointed to by `out_num_performance_cores` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_num_performance_cores(out_num_performance_cores: *mut u64) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_num_performance_cores);

    unsafe {
        *out_num_performance_cores = rust_info.total_performance_cores as u64;
    }

    GdtCpusErrorCode::Success as i32
}

/// Retrieves the total number of efficiency cores (E-cores) available on the system.
///
/// This function is typically relevant for hybrid architecture CPUs (e.g., Intel Alder Lake and newer).
/// It populates the `out_num_efficiency_cores` output parameter with the count.
///
/// # Arguments
///
/// * `out_num_efficiency_cores`: A mutable pointer to a `u64` where the total number of efficiency cores will be written.
///                                 The value stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful retrieval.
/// * `GdtCpusErrorCode::NotSupported` if the system does not differentiate core types or the count is zero.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on other failures.
///
/// # Safety
///
/// The caller must ensure that `out_num_efficiency_cores` is a valid pointer to a mutable `u64` memory location.
/// The memory pointed to by `out_num_efficiency_cores` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_num_efficiency_cores(out_num_efficiency_cores: *mut u64) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_num_efficiency_cores);

    unsafe {
        *out_num_efficiency_cores = rust_info.total_efficiency_cores as u64;
    }

    GdtCpusErrorCode::Success as i32
}

/// Checks if the CPU has a hybrid architecture (e.g., containing both P-cores and E-cores).
///
/// This function populates the `out_is_hybrid` output parameter with the result.
///
/// # Arguments
///
/// * `out_is_hybrid`: A mutable pointer to a `bool` where the result of the hybrid check will be written.
///                    `true` if the CPU is hybrid, `false` otherwise.
///                    The value stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful determination.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on failure.
///
/// # Safety
///
/// The caller must ensure that `out_is_hybrid` is a valid pointer to a mutable `bool` memory location.
/// The memory pointed to by `out_is_hybrid` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_is_hybrid(out_is_hybrid: *mut bool) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_is_hybrid);

    unsafe {
        *out_is_hybrid = rust_info.is_hybrid();
    }

    GdtCpusErrorCode::Success as i32
}

/// Retrieves the total number of CPU sockets (physical CPU packages) in the system.
///
/// This function populates the `out_num_sockets` output parameter with the count.
/// For most consumer systems, this will be 1.
///
/// # Arguments
///
/// * `out_num_sockets`: A mutable pointer to a `u64` where the total number of CPU sockets will be written.
///                      The value stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful retrieval.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on failure.
///
/// # Safety
///
/// The caller must ensure that `out_num_sockets` is a valid pointer to a mutable `u64` memory location.
/// The memory pointed to by `out_num_sockets` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_num_sockets(out_num_sockets: *mut u64) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_num_sockets);

    unsafe {
        *out_num_sockets = rust_info.sockets.len() as u64;
    }

    GdtCpusErrorCode::Success as i32
}

/// Retrieves detailed information for a specific CPU socket, by its index.
///
/// This function populates the `out_socket_info` output parameter with the details of the specified socket.
/// The socket index must be less than the total number of sockets reported by `gdt_cpus_num_sockets`.
///
/// # Arguments
///
/// * `socket_index`: The 0-based index of the CPU socket to query.
/// * `out_socket_info`: A mutable pointer to a `GdtCpusSocketInfo` struct where the socket information will be written.
///                      The data stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful retrieval.
/// * `GdtCpusErrorCode::OutOfBounds` if `socket_index` is invalid.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on other failures.
///
/// # Safety
///
/// The caller must ensure that `out_socket_info` is a valid pointer to a mutable `GdtCpusSocketInfo` memory location.
/// The memory pointed to by `out_socket_info` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_get_socket_info(
    socket_index: u64,
    out_socket_info: *mut GdtCpusSocketInfo,
) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_socket_info);

    let socket = match rust_info.sockets.get(socket_index as usize) {
        Some(socket) => socket,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    unsafe {
        *out_socket_info = GdtCpusSocketInfo::from(socket);
    }

    GdtCpusErrorCode::Success as i32
}

/// Retrieves the system-specific ID for a CPU socket, by its index.
///
/// This function populates the `out_socket_id` output parameter with the ID of the specified socket.
/// The socket index must be less than the total number of sockets reported by `gdt_cpus_num_sockets`.
///
/// # Arguments
///
/// * `socket_index`: The 0-based index of the CPU socket to query.
/// * `out_socket_id`: A mutable pointer to a `u64` where the socket ID will be written.
///                    The value stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful retrieval.
/// * `GdtCpusErrorCode::OutOfBounds` if `socket_index` is invalid.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on other failures.
///
/// # Safety
///
/// The caller must ensure that `out_socket_id` is a valid pointer to a mutable `u64` memory location.
/// The memory pointed to by `out_socket_id` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_get_socket_id_for_socket(socket_index: u64, out_socket_id: *mut u64) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_socket_id);

    let socket = match rust_info.sockets.get(socket_index as usize) {
        Some(socket) => socket,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    unsafe {
        *out_socket_id = socket.id as u64;
    }

    GdtCpusErrorCode::Success as i32
}

/// Checks if a specific CPU socket has L3 cache information available.
///
/// This function populates the `out_has_l3_cache_info` output parameter.
/// The socket index must be less than the total number of sockets reported by `gdt_cpus_num_sockets`.
///
/// # Arguments
///
/// * `socket_index`: The 0-based index of the CPU socket to query.
/// * `out_has_l3_cache_info`: A mutable pointer to a `bool` where the result will be written.
///                            `true` if L3 cache information is available for the socket, `false` otherwise.
///                            The value stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful determination.
/// * `GdtCpusErrorCode::OutOfBounds` if `socket_index` is invalid.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on other failures.
///
/// # Safety
///
/// The caller must ensure that `out_has_l3_cache_info` is a valid pointer to a mutable `bool` memory location.
/// The memory pointed to by `out_has_l3_cache_info` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_has_l3_cache_info(
    socket_index: u64,
    out_has_l3_cache_info: *mut bool,
) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_has_l3_cache_info);

    let socket = match rust_info.sockets.get(socket_index as usize) {
        Some(socket) => socket,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    unsafe {
        *out_has_l3_cache_info = socket.l3_cache.is_some();
    }

    GdtCpusErrorCode::Success as i32
}

/// Retrieves L3 cache information for a specific CPU socket.
///
/// This function populates the `out_cache_info` output parameter with the L3 cache details if available.
/// The socket index must be less than the total number of sockets reported by `gdt_cpus_num_sockets`.
/// Call `gdt_cpus_has_l3_cache_info` first to check for L3 cache presence.
///
/// # Arguments
///
/// * `socket_index`: The 0-based index of the CPU socket to query.
/// * `out_cache_info`: A mutable pointer to a `GdtCpusCacheInfo` struct where the L3 cache information will be written.
///                     The data stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful retrieval.
/// * `GdtCpusErrorCode::OutOfBounds` if `socket_index` is invalid.
/// * `GdtCpusErrorCode::NotApplicable` if the specified socket does not have an L3 cache.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on other failures.
///
/// # Safety
///
/// The caller must ensure that `out_cache_info` is a valid pointer to a mutable `GdtCpusCacheInfo` memory location.
/// The memory pointed to by `out_cache_info` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_get_l3_cache_info(
    socket_index: u64,
    out_cache_info: *mut GdtCpusCacheInfo,
) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_cache_info);

    let socket = match rust_info.sockets.get(socket_index as usize) {
        Some(socket) => socket,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    unsafe {
        *out_cache_info = if let Some(l3_cache) = &socket.l3_cache {
            GdtCpusCacheInfo::from(l3_cache)
        } else {
            return GdtCpusErrorCode::NotFound as i32;
        };
    }

    GdtCpusErrorCode::Success as i32
}

/// Retrieves the number of physical cores within a specific CPU socket.
///
/// This function populates the `out_num_cores` output parameter with the count of physical cores for the given socket.
/// The socket index must be less than the total number of sockets reported by `gdt_cpus_num_sockets`.
///
/// # Arguments
///
/// * `socket_index`: The 0-based index of the CPU socket to query.
/// * `out_num_cores`: A mutable pointer to a `u64` where the number of cores will be written.
///                    The value stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful retrieval.
/// * `GdtCpusErrorCode::OutOfBounds` if `socket_index` is invalid.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on other failures.
///
/// # Safety
///
/// The caller must ensure that `out_num_cores` is a valid pointer to a mutable `u64` memory location.
/// The memory pointed to by `out_num_cores` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_num_cores(socket_index: u64, out_num_cores: *mut u64) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_num_cores);

    let socket = match rust_info.sockets.get(socket_index as usize) {
        Some(socket) => socket,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    unsafe {
        *out_num_cores = socket.cores.len() as u64;
    }

    GdtCpusErrorCode::Success as i32
}

/// Retrieves detailed information for a specific core within a socket.
///
/// This function populates the `out_core_info` output parameter with details of the specified core.
/// Both `socket_index` and `core_index` must be valid (i.e., less than the counts returned by
/// `gdt_cpus_num_sockets` and `gdt_cpus_num_cores` respectively).
///
/// # Arguments
///
/// * `socket_index`: The 0-based index of the CPU socket.
/// * `core_index`: The 0-based index of the core within the specified socket.
/// * `out_core_info`: A mutable pointer to a `GdtCpusCoreInfo` struct where the core information will be written.
///                    The data stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful retrieval.
/// * `GdtCpusErrorCode::OutOfBounds` if `socket_index` or `core_index` is invalid.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on other failures.
///
/// # Safety
///
/// The caller must ensure that `out_core_info` is a valid pointer to a mutable `GdtCpusCoreInfo` memory location.
/// The memory pointed to by `out_core_info` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_get_core_info(
    socket_index: u64,
    core_index: u64,
    out_core_info: *mut GdtCpusCoreInfo,
) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_core_info);

    let socket = match rust_info.sockets.get(socket_index as usize) {
        Some(socket) => socket,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    let core = match socket.cores.get(core_index as usize) {
        Some(core) => core,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    unsafe {
        *out_core_info = GdtCpusCoreInfo::from(core);
    }

    GdtCpusErrorCode::Success as i32
}

/// Retrieves the system-specific ID for a core within a socket.
///
/// This function populates the `out_core_id` output parameter with the ID of the specified core.
/// Both `socket_index` and `core_index` must be valid.
///
/// # Arguments
///
/// * `socket_index`: The 0-based index of the CPU socket.
/// * `core_index`: The 0-based index of the core within the specified socket.
/// * `out_core_id`: A mutable pointer to a `u64` where the core ID will be written.
///                  The value stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful retrieval.
/// * `GdtCpusErrorCode::OutOfBounds` if `socket_index` or `core_index` is invalid.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on other failures.
///
/// # Safety
///
/// The caller must ensure that `out_core_id` is a valid pointer to a mutable `u64` memory location.
/// The memory pointed to by `out_core_id` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_get_core_id(
    socket_index: u64,
    core_index: u64,
    out_core_id: *mut u64,
) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_core_id);

    let socket = match rust_info.sockets.get(socket_index as usize) {
        Some(socket) => socket,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    let core = match socket.cores.get(core_index as usize) {
        Some(core) => core,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    unsafe {
        *out_core_id = core.id as u64;
    }

    GdtCpusErrorCode::Success as i32
}

/// Retrieves the system-specific ID of the socket to which a specific core belongs.
///
/// This function populates the `out_socket_id` output parameter with the ID of the socket containing the specified core.
/// Both `socket_index` (for selecting the socket to look within) and `core_index` must be valid.
/// This can be useful to confirm a core's parent socket ID if iterating through cores directly.
///
/// # Arguments
///
/// * `socket_index`: The 0-based index of the CPU socket where the core is assumed to be located.
/// * `core_index`: The 0-based index of the core within the specified socket.
/// * `out_socket_id`: A mutable pointer to a `u64` where the parent socket ID will be written.
///                    The value stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful retrieval.
/// * `GdtCpusErrorCode::OutOfBounds` if `socket_index` or `core_index` is invalid.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on other failures.
///
/// # Safety
///
/// The caller must ensure that `out_socket_id` is a valid pointer to a mutable `u64` memory location.
/// The memory pointed to by `out_socket_id` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_get_socket_id_for_core(
    socket_index: u64,
    core_index: u64,
    out_socket_id: *mut u64,
) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_socket_id);

    let socket = match rust_info.sockets.get(socket_index as usize) {
        Some(socket) => socket,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    let core = match socket.cores.get(core_index as usize) {
        Some(core) => core,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    unsafe {
        *out_socket_id = core.socket_id as u64;
    }

    GdtCpusErrorCode::Success as i32
}

/// Retrieves the type of a specific core (e.g., Performance or Efficiency core).
///
/// This function populates the `out_core_type` output parameter with the type of the specified core.
/// Both `socket_index` and `core_index` must be valid.
/// This is particularly relevant for hybrid architecture CPUs.
///
/// # Arguments
///
/// * `socket_index`: The 0-based index of the CPU socket.
/// * `core_index`: The 0-based index of the core within the specified socket.
/// * `out_core_type`: A mutable pointer to a `GdtCpusCoreType` enum where the core type will be written.
///                    The value stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful retrieval.
/// * `GdtCpusErrorCode::OutOfBounds` if `socket_index` or `core_index` is invalid.
/// * `GdtCpusErrorCode::NotSupported` if core types are not differentiated or identifiable.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on other failures.
///
/// # Safety
///
/// The caller must ensure that `out_core_type` is a valid pointer to a mutable `GdtCpusCoreType` memory location.
/// The memory pointed to by `out_core_type` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_get_core_type(
    socket_index: u64,
    core_index: u64,
    out_core_type: *mut GdtCpusCoreType,
) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_core_type);

    let socket = match rust_info.sockets.get(socket_index as usize) {
        Some(socket) => socket,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    let core = match socket.cores.get(core_index as usize) {
        Some(core) => core,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    unsafe {
        *out_core_type = GdtCpusCoreType::from(&core.core_type);
    }

    GdtCpusErrorCode::Success as i32
}

/// Checks if a specific core has L1 instruction (L1i) cache information available.
///
/// This function populates the `out_has_l1i_cache_info` output parameter.
/// Both `socket_index` and `core_index` must be valid (i.e., less than the counts returned by
/// `gdt_cpus_num_sockets` and `gdt_cpus_num_cores` respectively).
///
/// # Arguments
///
/// * `socket_index`: The 0-based index of the CPU socket.
/// * `core_index`: The 0-based index of the core within the specified socket.
/// * `out_has_l1i_cache_info`: A mutable pointer to a `bool` where the result will be written.
///                             `true` if L1i cache information is available for the core, `false` otherwise.
///                             The value stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful determination.
/// * `GdtCpusErrorCode::OutOfBounds` if `socket_index` or `core_index` is invalid.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on other failures.
///
/// # Safety
///
/// The caller must ensure that `out_has_l1i_cache_info` is a valid pointer to a mutable `bool` memory location.
/// The memory pointed to by `out_has_l1i_cache_info` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_has_l1i_cache_info(
    socket_index: u64,
    core_index: u64,
    out_has_l1i_cache_info: *mut bool,
) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_has_l1i_cache_info);

    let socket = match rust_info.sockets.get(socket_index as usize) {
        Some(socket) => socket,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    let core = match socket.cores.get(core_index as usize) {
        Some(core) => core,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    unsafe {
        *out_has_l1i_cache_info = core.l1_instruction_cache.is_some();
    }

    GdtCpusErrorCode::Success as i32
}

/// Retrieves L1 instruction (L1i) cache information for a specific core.
///
/// This function populates the `out_cache_info` output parameter with the L1i cache details if available.
/// Both `socket_index` and `core_index` must be valid.
/// It's recommended to call `gdt_cpus_has_l1i_cache_info` first to check for L1i cache presence for the core.
///
/// # Arguments
///
/// * `socket_index`: The 0-based index of the CPU socket.
/// * `core_index`: The 0-based index of the core within the specified socket.
/// * `out_cache_info`: A mutable pointer to a `GdtCpusCacheInfo` struct where the L1i cache information will be written.
///                     The data stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful retrieval.
/// * `GdtCpusErrorCode::OutOfBounds` if `socket_index` or `core_index` is invalid.
/// * `GdtCpusErrorCode::NotFound` if the specified core does not have an L1i cache.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on other failures.
///
/// # Safety
///
/// The caller must ensure that `out_cache_info` is a valid pointer to a mutable `GdtCpusCacheInfo` memory location.
/// The memory pointed to by `out_cache_info` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_get_l1i_cache_info(
    socket_index: u64,
    core_index: u64,
    out_cache_info: *mut GdtCpusCacheInfo,
) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_cache_info);

    let socket = match rust_info.sockets.get(socket_index as usize) {
        Some(socket) => socket,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    let core = match socket.cores.get(core_index as usize) {
        Some(core) => core,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    unsafe {
        *out_cache_info = if let Some(l1i_cache) = &core.l1_instruction_cache {
            GdtCpusCacheInfo::from(l1i_cache)
        } else {
            return GdtCpusErrorCode::NotFound as i32;
        };
    }

    GdtCpusErrorCode::Success as i32
}

/// Checks if a specific core has L1 data (L1d) cache information available.
///
/// This function populates the `out_has_l1d_cache_info` output parameter.
/// Both `socket_index` and `core_index` must be valid (i.e., less than the counts returned by
/// `gdt_cpus_num_sockets` and `gdt_cpus_num_cores` respectively).
///
/// # Arguments
///
/// * `socket_index`: The 0-based index of the CPU socket.
/// * `core_index`: The 0-based index of the core within the specified socket.
/// * `out_has_l1d_cache_info`: A mutable pointer to a `bool` where the result will be written.
///                             `true` if L1d cache information is available for the core, `false` otherwise.
///                             The value stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful determination.
/// * `GdtCpusErrorCode::OutOfBounds` if `socket_index` or `core_index` is invalid.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on other failures.
///
/// # Safety
///
/// The caller must ensure that `out_has_l1d_cache_info` is a valid pointer to a mutable `bool` memory location.
/// The memory pointed to by `out_has_l1d_cache_info` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_has_l1d_cache_info(
    socket_index: u64,
    core_index: u64,
    out_has_l1d_cache_info: *mut bool,
) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_has_l1d_cache_info);

    let socket = match rust_info.sockets.get(socket_index as usize) {
        Some(socket) => socket,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    let core = match socket.cores.get(core_index as usize) {
        Some(core) => core,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    unsafe {
        *out_has_l1d_cache_info = core.l1_data_cache.is_some();
    }

    GdtCpusErrorCode::Success as i32
}

/// Retrieves L1 data (L1d) cache information for a specific core.
///
/// This function populates the `out_cache_info` output parameter with the L1d cache details if available.
/// Both `socket_index` and `core_index` must be valid.
/// It's recommended to call `gdt_cpus_has_l1d_cache_info` first to check for L1d cache presence for the core.
///
/// # Arguments
///
/// * `socket_index`: The 0-based index of the CPU socket.
/// * `core_index`: The 0-based index of the core within the specified socket.
/// * `out_cache_info`: A mutable pointer to a `GdtCpusCacheInfo` struct where the L1d cache information will be written.
///                     The data stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful retrieval.
/// * `GdtCpusErrorCode::OutOfBounds` if `socket_index` or `core_index` is invalid.
/// * `GdtCpusErrorCode::NotFound` if the specified core does not have an L1d cache.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on other failures.
///
/// # Safety
///
/// The caller must ensure that `out_cache_info` is a valid pointer to a mutable `GdtCpusCacheInfo` memory location.
/// The memory pointed to by `out_cache_info` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_get_l1d_cache_info(
    socket_index: u64,
    core_index: u64,
    out_cache_info: *mut GdtCpusCacheInfo,
) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_cache_info);

    let socket = match rust_info.sockets.get(socket_index as usize) {
        Some(socket) => socket,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    let core = match socket.cores.get(core_index as usize) {
        Some(core) => core,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    unsafe {
        *out_cache_info = if let Some(l1d_cache) = &core.l1_data_cache {
            GdtCpusCacheInfo::from(l1d_cache)
        } else {
            return GdtCpusErrorCode::NotFound as i32;
        };
    }

    GdtCpusErrorCode::Success as i32
}

/// Checks if a specific core has L2 cache information available.
///
/// This function populates the `out_has_l2_cache_info` output parameter.
/// Both `socket_index` and `core_index` must be valid (i.e., less than the counts returned by
/// `gdt_cpus_num_sockets` and `gdt_cpus_num_cores` respectively).
///
/// # Arguments
///
/// * `socket_index`: The 0-based index of the CPU socket.
/// * `core_index`: The 0-based index of the core within the specified socket.
/// * `out_has_l2_cache_info`: A mutable pointer to a `bool` where the result will be written.
///                              `true` if L2 cache information is available for the core, `false` otherwise.
///                              The value stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful determination.
/// * `GdtCpusErrorCode::OutOfBounds` if `socket_index` or `core_index` is invalid.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on other failures.
///
/// # Safety
///
/// The caller must ensure that `out_has_l2_cache_info` is a valid pointer to a mutable `bool` memory location.
/// The memory pointed to by `out_has_l2_cache_info` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_has_l2_cache_info(
    socket_index: u64,
    core_index: u64,
    out_has_l2_cache_info: *mut bool,
) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_has_l2_cache_info);

    let socket = match rust_info.sockets.get(socket_index as usize) {
        Some(socket) => socket,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    let core = match socket.cores.get(core_index as usize) {
        Some(core) => core,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    unsafe {
        *out_has_l2_cache_info = core.l2_cache.is_some();
    }

    GdtCpusErrorCode::Success as i32
}

/// Retrieves L2 cache information for a specific core.
///
/// This function populates the `out_cache_info` output parameter with the L2 cache details if available.
/// Both `socket_index` and `core_index` must be valid.
/// It's recommended to call `gdt_cpus_has_l2_cache_info` first to check for L2 cache presence for the core.
///
/// # Arguments
///
/// * `socket_index`: The 0-based index of the CPU socket.
/// * `core_index`: The 0-based index of the core within the specified socket.
/// * `out_cache_info`: A mutable pointer to a `GdtCpusCacheInfo` struct where the L2 cache information will be written.
///                     The data stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful retrieval.
/// * `GdtCpusErrorCode::OutOfBounds` if `socket_index` or `core_index` is invalid.
/// * `GdtCpusErrorCode::NotFound` if the specified core does not have an L2 cache.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on other failures.
///
/// # Safety
///
/// The caller must ensure that `out_cache_info` is a valid pointer to a mutable `GdtCpusCacheInfo` memory location.
/// The memory pointed to by `out_cache_info` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_get_l2_cache_info(
    socket_index: u64,
    core_index: u64,
    out_cache_info: *mut GdtCpusCacheInfo,
) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_cache_info);

    let socket = match rust_info.sockets.get(socket_index as usize) {
        Some(socket) => socket,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    let core = match socket.cores.get(core_index as usize) {
        Some(core) => core,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    unsafe {
        *out_cache_info = if let Some(l2_cache) = &core.l2_cache {
            GdtCpusCacheInfo::from(l2_cache)
        } else {
            return GdtCpusErrorCode::NotFound as i32;
        };
    }

    GdtCpusErrorCode::Success as i32
}

/// Retrieves the number of logical processor IDs associated with a specific physical core.
///
/// This function populates the `out_num_logical_ids` output parameter with the count.
/// This typically corresponds to the number of hardware threads (e.g., via Hyper-Threading or SMT) for that core.
/// Both `socket_index` and `core_index` must be valid.
///
/// # Arguments
///
/// * `socket_index`: The 0-based index of the CPU socket.
/// * `core_index`: The 0-based index of the core within the specified socket.
/// * `out_num_logical_ids`: A mutable pointer to a `u64` where the number of logical processor IDs will be written.
///                            The value stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful retrieval.
/// * `GdtCpusErrorCode::OutOfBounds` if `socket_index` or `core_index` is invalid.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on other failures.
///
/// # Safety
///
/// The caller must ensure that `out_num_logical_ids` is a valid pointer to a mutable `u64` memory location.
/// The memory pointed to by `out_num_logical_ids` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_num_logical_processor_ids(
    socket_index: u64,
    core_index: u64,
    out_num_logical_ids: *mut u64,
) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_num_logical_ids);

    let socket = match rust_info.sockets.get(socket_index as usize) {
        Some(socket) => socket,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    let core = match socket.cores.get(core_index as usize) {
        Some(core) => core,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    unsafe {
        *out_num_logical_ids = core.logical_processor_ids.len() as u64;
    }

    GdtCpusErrorCode::Success as i32
}

/// Retrieves a specific logical processor ID for a given physical core.
///
/// This function populates the `out_logical_id` output parameter with the system-specific ID of a logical processor (thread).
/// `socket_index`, `core_index`, and `logical_processor_index` must all be valid.
/// The `logical_processor_index` should be less than the count returned by `gdt_cpus_num_logical_processor_ids` for that core.
///
/// # Arguments
///
/// * `socket_index`: The 0-based index of the CPU socket.
/// * `core_index`: The 0-based index of the core within the specified socket.
/// * `logical_processor_index`: The 0-based index of the logical processor within the specified core.
/// * `out_logical_id`: A mutable pointer to a `u64` where the logical processor ID will be written.
///                       The value stored is only valid if the function returns `GdtCpusErrorCode::Success`.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) on successful retrieval.
/// * `GdtCpusErrorCode::OutOfBounds` if `socket_index`, `core_index`, or `logical_processor_index` is invalid.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on other failures.
///
/// # Safety
///
/// The caller must ensure that `out_logical_id` is a valid pointer to a mutable `u64` memory location.
/// The memory pointed to by `out_logical_id` must be writable.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_get_logical_processor_id(
    socket_index: u64,
    core_index: u64,
    logical_processor_index: u64,
    out_logical_id: *mut u64,
) -> i32 {
    let rust_info = get_info_validate_out_or_err!(out_logical_id);

    let socket = match rust_info.sockets.get(socket_index as usize) {
        Some(socket) => socket,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    let core = match socket.cores.get(core_index as usize) {
        Some(core) => core,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    let core_logical_processor_id = match core
        .logical_processor_ids
        .get(logical_processor_index as usize)
    {
        Some(id) => id,
        None => return GdtCpusErrorCode::OutOfBounds as i32,
    };

    unsafe {
        *out_logical_id = *core_logical_processor_id as u64;
    }

    GdtCpusErrorCode::Success as i32
}

/// Pins the current thread to a specific logical core (hardware thread).
///
/// This function attempts to set the affinity of the calling thread to the specified logical core ID.
/// The `logical_core_id` should be a valid ID obtained from functions like `gdt_cpus_get_logical_processor_id`.
/// Pinning a thread can be useful for performance-critical tasks to ensure a thread runs on a specific core,
/// potentially improving cache utilization and reducing context switching.
///
/// # Arguments
///
/// * `logical_core_id`: The system-specific ID of the logical core to pin the current thread to.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) if the thread was successfully pinned.
/// * `GdtCpusErrorCode::InvalidParameter` if `logical_core_id` is invalid or out of range for the system.
/// * `GdtCpusErrorCode::NotSupported` if thread pinning is not supported on the current platform.
/// * `GdtCpusErrorCode::OsError` if an underlying OS-level error occurred.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on other failures.
///
/// # Notes
///
/// - The behavior and success of thread pinning can be platform-dependent.
/// - Incorrectly pinning threads can sometimes lead to performance degradation, so use with understanding.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_pin_thread_to_core(logical_core_id: u64) -> i32 {
    match gdt_cpus::pin_thread_to_core(logical_core_id as usize) {
        Ok(_) => GdtCpusErrorCode::Success as i32,
        Err(err) => GdtCpusErrorCode::from(&err) as i32,
    }
}

/// Sets the priority of the current thread.
///
/// This function attempts to change the scheduling priority of the calling thread.
/// The `priority` argument should be one of the values from the `GdtCpusThreadPriority` enum.
/// Setting thread priority can influence how the operating system schedules the thread relative to others.
///
/// # Arguments
///
/// * `priority`: A `GdtCpusThreadPriority` enum value specifying the desired priority level.
///
/// # Returns
///
/// * `GdtCpusErrorCode::Success` (0) if the thread priority was successfully set.
/// * `GdtCpusErrorCode::InvalidParameter` if the `priority` value is not a valid member of `GdtCpusThreadPriority`.
/// * `GdtCpusErrorCode::NotSupported` if setting thread priority is not supported or the requested level is not available on the current platform.
/// * `GdtCpusErrorCode::PermissionDenied` if the caller does not have sufficient privileges to change the thread priority.
/// * `GdtCpusErrorCode::OsError` if an underlying OS-level error occurred.
/// * An error code from `GdtCpusErrorCode` (as `i32`) on other failures.
///
/// # Notes
///
/// - The interpretation and effect of thread priorities are highly platform-dependent.
/// - Setting very high priorities might require special privileges and can potentially starve other system processes if not used carefully.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
extern "C" fn gdt_cpus_set_thread_priority(priority: GdtCpusThreadPriority) -> i32 {
    match gdt_cpus::set_thread_priority(priority.into()) {
        Ok(_) => GdtCpusErrorCode::Success as i32,
        Err(err) => GdtCpusErrorCode::from(&err) as i32,
    }
}
