/// Classifies a CPU core by its performance/efficiency role.
///
/// Modern CPUs are heterogeneous in more than one way: Intel ships Performance,
/// Efficiency and Low-Power-Efficiency cores on one die (Meteor Lake onward),
/// AMD mixes full-fat and dense ("c") cores that differ by frequency and cache
/// rather than ISA, and ARM big.LITTLE/DynamIQ has always been multi-kind.
/// A boolean P/E split cannot represent shipping silicon - this enum is N-ary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CoreKind {
    /// A Performance core (Intel P-core, ARM "big", AMD full-fat, Apple P).
    Performance,
    /// An Efficiency core (Intel E-core, ARM "LITTLE", AMD dense/"c", Apple E).
    Efficiency,
    /// A Low-Power Efficiency core (Intel LP-E cores on the SoC tile, or the
    /// lowest capacity tier on a 3-tier ARM design).
    LpEfficiency,
    /// The core kind could not be determined.
    ///
    /// NOTE: detection never *returns* this on a homogeneous machine - the
    /// classification invariant is "homogeneous means all Performance".
    Unknown,
}

impl CoreKind {
    /// Number of variants - sizes the per-kind tables in [`crate::CpuInfo`].
    pub const COUNT: usize = 4;

    /// Stable index for per-kind tables (`l1d[kind.index()]`).
    pub fn index(self) -> usize {
        match self {
            CoreKind::Performance => 0,
            CoreKind::Efficiency => 1,
            CoreKind::LpEfficiency => 2,
            CoreKind::Unknown => 3,
        }
    }
}

impl std::fmt::Display for CoreKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreKind::Performance => write!(f, "Performance"),
            CoreKind::Efficiency => write!(f, "Efficiency"),
            CoreKind::LpEfficiency => write!(f, "LpEfficiency"),
            CoreKind::Unknown => write!(f, "Unknown"),
        }
    }
}
