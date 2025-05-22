/// Describes the designated purpose or type of content a CPU cache holds.
///
/// CPU caches can be specialized to store different kinds of information,
/// such as executable instructions, program data, or a combination of both.
/// Understanding the cache type is important for comprehending how a CPU
/// manages and accelerates access to different forms of memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CacheType {
    /// Unified cache, storing both instructions and data.
    ///
    /// A unified cache does not distinguish between instruction and data and can
    /// dynamically allocate its space for either, offering flexibility.
    Unified,
    /// Cache dedicated exclusively to storing executable instructions.
    ///
    /// An instruction cache (I-cache) helps speed up the fetching of instructions
    /// by the CPU's execution units.
    Instruction,
    /// Cache dedicated exclusively to storing data.
    ///
    /// A data cache (D-cache) accelerates access to program data, separate from
    /// executable instructions.
    Data,
    /// Trace cache, typically storing decoded instructions (micro-operations).
    ///
    /// Trace caches are a more specialized type of instruction cache that store
    /// sequences of already decoded instructions, which can improve performance
    /// for certain workloads, especially on CPUs with complex instruction sets.
    Trace,
    /// Represents an unknown or unspecified cache type.
    ///
    /// This variant is used when the cache type cannot be determined or
    /// does not fit into the common classifications.
    Unknown,
}

impl From<u32> for CacheType {
    fn from(cache_type: u32) -> Self {
        match cache_type {
            0 => CacheType::Unified,
            1 => CacheType::Instruction,
            2 => CacheType::Data,
            3 => CacheType::Trace,
            _ => CacheType::Unknown,
        }
    }
}

impl std::fmt::Display for CacheType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheType::Unified => write!(f, "Unified"),
            CacheType::Instruction => write!(f, "Instruction"),
            CacheType::Data => write!(f, "Data"),
            CacheType::Trace => write!(f, "Trace"),
            CacheType::Unknown => write!(f, "Unknown"),
        }
    }
}
