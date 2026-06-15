/// Size, line size and sharing degree of one cache instance.
///
/// The library stores caches per CORE KIND (L1d/L1i/L2 are uniform within a
/// kind on all shipping silicon) and per L3 DOMAIN - never per core, which
/// only duplicates identical data, and never per socket, which cannot
/// represent chiplet parts. Cache level/type enums remain internal parsing
/// vocabulary for the platform detectors.
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CacheInfo {
    /// Total size in bytes. 0 = not detected.
    pub size_bytes: u64,
    /// Cache line size in bytes (typically 64).
    pub line_bytes: u16,
    /// Number of LPs sharing ONE instance of this cache
    /// (2 = core-private with SMT; >2 = cluster-shared, e.g. Intel E-core L2).
    pub shared_by: u16,
}
