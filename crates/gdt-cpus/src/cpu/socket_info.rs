use super::{CacheInfo, CoreInfo};

/// Represents information about a single CPU socket (physical CPU package).
///
/// A CPU socket is a physical connector on a motherboard that houses a CPU.
/// Multi-socket systems have more than one physical CPU. This structure
/// details the cores contained within a socket and any caches shared at the socket level (e.g., L3 cache).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SocketInfo {
    /// A unique identifier for this CPU socket.
    ///
    /// For single-socket systems, this will typically be 0.
    pub id: usize,
    /// A list of all physical cores belonging to this socket.
    ///
    /// Each `CoreInfo` in this vector provides detailed information about a specific core.
    pub cores: Vec<CoreInfo>,
    /// Information about the L3 cache, if present and detected for this socket.
    ///
    /// The L3 cache (Last-Level Cache or LLC in many contexts) is typically shared
    /// among all cores within the same physical CPU socket.
    pub l3_cache: Option<CacheInfo>,
}
