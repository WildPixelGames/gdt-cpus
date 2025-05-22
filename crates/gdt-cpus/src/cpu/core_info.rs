use super::{CacheInfo, CoreType};

/// Represents detailed information about a single physical CPU core.
///
/// This structure provides data about a core's identification, its type (especially
/// in hybrid architectures), the logical processors (hardware threads) it hosts,
/// and information about its dedicated or closely associated caches (L1, L2).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CoreInfo {
    /// A unique identifier for this physical core across the entire system (all sockets).
    ///
    /// This ID is assigned by the library and may not directly correspond to OS-level core IDs.
    pub id: usize,
    /// The identifier of the CPU socket (physical package) to which this core belongs.
    pub socket_id: usize,
    /// The architectural type of this core (e.g., Performance or Efficiency).
    ///
    /// This is particularly relevant for hybrid CPUs. For non-hybrid CPUs,
    /// this defaults to `CoreType::Performance`.
    pub core_type: CoreType,
    /// A list of OS-specific identifiers for the logical processors (hardware threads)
    /// that are executed on this physical core.
    ///
    /// For cores without Hyper-Threading/SMT, this will typically contain one ID.
    /// For cores with Hyper-Threading/SMT, it will contain multiple IDs (e.g., two for HT).
    /// These IDs can be used for setting thread affinity.
    pub logical_processor_ids: Vec<usize>,

    /// Information about the L1 instruction cache specific to this core, if available and detected.
    ///
    /// L1 instruction caches (L1i) store frequently executed instructions.
    pub l1_instruction_cache: Option<CacheInfo>,
    /// Information about the L1 data cache specific to this core, if available and detected.
    ///
    /// L1 data caches (L1d) store frequently accessed data.
    pub l1_data_cache: Option<CacheInfo>,
    /// Information about the L2 cache associated with this core, if available and detected.
    ///
    /// L2 caches are generally larger and slower than L1 caches. They might be exclusive
    /// to this core or shared with a small cluster of other cores, depending on the CPU architecture.
    pub l2_cache: Option<CacheInfo>,
}
