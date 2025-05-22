use super::{CacheLevel, CacheType};

/// Represents detailed information about a specific CPU cache.
///
/// This structure provides insights into the cache's characteristics,
/// such as its level in the memory hierarchy, its designated type
/// (e.g., for data or instructions), its total size, and the size
/// of its cache lines. This information is crucial for performance-sensitive
/// applications that need to optimize memory access patterns.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CacheInfo {
    /// The hierarchical level of the cache (e.g., L1, L2, L3).
    ///
    /// Lower levels (like L1) are smaller, faster, and closer to the CPU core,
    /// while higher levels (like L3) are larger, slower, and typically shared
    /// among multiple cores.
    pub level: CacheLevel,
    /// The designated purpose of the cache.
    ///
    /// This can be:
    /// - `Data`: Cache dedicated to storing data.
    /// - `Instruction`: Cache dedicated to storing executable instructions.
    /// - `Unified`: Cache used for both data and instructions.
    pub cache_type: CacheType,
    /// The total size of the cache, expressed in bytes.
    ///
    /// For example, a value of `32768` would represent a 32KB cache.
    pub size_bytes: u64,
    /// The size of a single cache line (also known as a cache block), in bytes.
    ///
    /// Data is transferred between the cache and main memory in units of cache lines.
    /// Knowing the line size can be important for optimizing data layout to avoid
    /// issues like false sharing in multi-threaded applications.
    pub line_size_bytes: usize,
}
