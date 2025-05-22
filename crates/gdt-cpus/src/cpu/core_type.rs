/// Describes the type or class of a CPU core, particularly relevant for hybrid architectures.
///
/// Modern CPUs, especially those with hybrid architectures (like Intel's Performance-cores
/// and Efficient-cores, or ARM's big.LITTLE), feature different types of cores optimized
/// for different tasks. This enum helps classify them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CoreType {
    /// A Performance-core (P-core) or equivalent.
    ///
    /// These cores are designed for high-intensity workloads, providing maximum
    /// single-threaded performance. Examples include Intel's P-cores or ARM's "big" cores
    /// (e.g., Cortex-A7x series).
    Performance,
    /// An Efficiency-core (E-core) or equivalent.
    ///
    /// These cores are designed for power efficiency and handling background tasks
    /// or less demanding workloads. Examples include Intel's E-cores or ARM's "LITTLE"
    /// cores (e.g., Cortex-A5x series).
    Efficiency,
    /// The core type is unknown, not applicable, or cannot be determined.
    Unknown,
}

impl std::fmt::Display for CoreType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreType::Performance => write!(f, "Performance"),
            CoreType::Efficiency => write!(f, "Efficiency"),
            CoreType::Unknown => write!(f, "Unknown"),
        }
    }
}
