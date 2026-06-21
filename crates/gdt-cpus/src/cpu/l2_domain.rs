use crate::AffinityMask;

/// A set of cores sharing one L2 cache instance.
///
/// Where an [`L3Domain`](crate::L3Domain) is the last-level-cache sharing group (a CCD on chiplet AMD,
/// a cluster on hybrid Intel), an L2 domain is the much finer group that shares one L2: on most desktop
/// parts that is a single physical core plus its SMT siblings; on hybrid Intel the efficiency cores
/// share an L2 in clusters. Sharing L2 is the shortest core-to-core path, so this is the granularity at
/// which two threads are "closest" - the unit for slicing a few cooperating threads out of a larger L3
/// domain.
///
/// Domains are content-keyed during detection (by the lowest member LP of the cache's shared set), and
/// each carries its own [`size_bytes`](Self::size_bytes), so heterogeneous L2 sizes (mixed core kinds)
/// are represented exactly rather than collapsed to a per-kind average.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct L2Domain {
    /// Size of this L2 instance in bytes.
    pub size_bytes: u64,
    /// The LPs (OS ids) sharing this L2 instance.
    pub mask: AffinityMask,
    /// Physical cores in this domain (SMT siblings counted once).
    pub core_count: u16,
    /// Index of the parent [`L3Domain`](crate::L3Domain) these cores share, or
    /// [`Lp::NO_L3`](crate::Lp::NO_L3) when the machine reports no L3 (e.g. Apple Silicon). Every LP in
    /// an L2 domain shares one L3, so this lets a caller walk the L2 groups inside a single L3 domain
    /// without intersecting masks.
    pub l3_domain: u8,
}
