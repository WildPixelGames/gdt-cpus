//! The CPU data model: flat per-LP topology, L3 domains, core kinds.
//!
//! Entry point: [`CpuInfo::detect()`]. The model is deliberately flat - one
//! [`Lp`] record per online logical processor, a first-class [`L3Domain`]
//! table (CCDs/clusters), and per-kind caches. Socket membership is a field on
//! each LP, not a container: per-socket hierarchies cannot represent chiplet
//! CPUs, where one socket carries several L3 domains.

mod cache_info;
mod core_kind;
mod features;
mod info;
mod l2_domain;
mod l3_domain;
mod lp;
mod vendor;

pub use cache_info::CacheInfo;
pub use core_kind::CoreKind;
pub use features::CpuFeatures;
pub use info::CpuInfo;
pub use l2_domain::L2Domain;
pub use l3_domain::L3Domain;
pub use lp::Lp;
pub use vendor::Vendor;
