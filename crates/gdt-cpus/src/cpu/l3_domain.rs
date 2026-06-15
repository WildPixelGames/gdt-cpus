use crate::AffinityMask;

/// A set of cores sharing one L3 cache instance.
///
/// On chiplet AMD parts a domain is a CCD (a 5950X has two 32 MiB domains on
/// one socket), on hybrid Intel the E-core clusters form their own domains,
/// and on X3D-style parts ONE domain carries the big cache. Cross-domain
/// core-to-core latency is significantly higher than within a domain, so this
/// is the granularity at which games should place cooperating threads.
///
/// Domains are content-keyed during detection (by the lowest member LP of the
/// cache's shared set) - never attributed per socket and never deduplicated by
/// size, both of which silently collapse multi-CCD parts into one domain.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct L3Domain {
    /// Size of this L3 instance in bytes.
    pub size_bytes: u64,
    /// The LPs (OS ids) sharing this L3 instance.
    pub mask: AffinityMask,
    /// Physical cores in this domain (SMT siblings counted once).
    pub core_count: u16,
}
