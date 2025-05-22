/// Represents the hierarchical level of a CPU cache within the memory system.
///
/// CPU caches are organized in levels, typically denoted as L1, L2, L3, and sometimes L4.
/// These levels indicate the cache's proximity to the CPU core, its size, and its speed.
/// Generally, lower levels are faster, smaller, and closer to the core, while higher
/// levels are slower, larger, and often shared among multiple cores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CacheLevel {
    /// Level 1 (L1) cache.
    ///
    /// This is the fastest and smallest cache, located closest to the CPU core.
    /// L1 caches are typically split into separate caches for instructions (L1i)
    /// and data (L1d) for each core.
    L1,
    /// Level 2 (L2) cache.
    ///
    /// L2 cache is larger and slower than L1 but faster than L3. It's often
    /// dedicated to a single CPU core, though some architectures might share it
    /// between a small group of cores.
    L2,
    /// Level 3 (L3) cache.
    ///
    /// L3 cache, also known as the Last-Level Cache (LLC) in many systems,
    /// is larger and slower than L2. It is typically shared among all cores
    /// on a CPU socket or die. It serves as a common pool of frequently accessed
    /// data for all cores.
    L3,
    /// Level 4 (L4) cache.
    ///
    /// L4 cache is less common but can be found in some high-end CPUs or
    /// specific architectures (e.g., as an eDRAM cache). It acts as an additional
    /// layer beyond L3, often shared across multiple sockets or as a victim cache.
    L4,
    /// Represents an unknown or unspecified cache level.
    ///
    /// This variant is used when the cache level cannot be determined
    /// or does not fit into the standard L1-L4 classification.
    Unknown,
}

impl std::fmt::Display for CacheLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheLevel::L1 => write!(f, "L1"),
            CacheLevel::L2 => write!(f, "L2"),
            CacheLevel::L3 => write!(f, "L3"),
            CacheLevel::L4 => write!(f, "L4"),
            CacheLevel::Unknown => write!(f, "Unknown"),
        }
    }
}

impl From<u32> for CacheLevel {
    fn from(level: u32) -> Self {
        match level {
            1 => CacheLevel::L1,
            2 => CacheLevel::L2,
            3 => CacheLevel::L3,
            4 => CacheLevel::L4,
            _ => CacheLevel::Unknown,
        }
    }
}
