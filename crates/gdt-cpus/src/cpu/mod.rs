//! Provides detailed information about the system's CPU.
//!
//! This module contains structures and enumerations that describe various aspects
//! of the CPU, including its vendor, model, features, cache hierarchy, core types,
//! and socket layout.
//!
//! The main entry point for CPU information is the [`CpuInfo`] struct, which aggregates
//! all detected data. Other types like [`CacheInfo`], [`CoreInfo`], [`SocketInfo`],
//! [`Vendor`], [`CpuFeatures`], [`CacheLevel`], [`CacheType`], and [`CoreType`]
//! provide specific details.
//!
//! This information is primarily gathered by platform-specific detection logic
//! and is intended to be used by applications requiring fine-grained CPU details,
//! for example, for performance optimization or diagnostics.

mod cache_info;
mod cache_level;
mod cache_type;
mod core_info;
mod core_type;
mod features;
mod info;
mod socket_info;
mod vendor;

pub use cache_info::CacheInfo;
pub use cache_level::CacheLevel;
pub use cache_type::CacheType;
pub use core_info::CoreInfo;
pub use core_type::CoreType;
pub use features::CpuFeatures;
pub use info::CpuInfo;
pub use socket_info::SocketInfo;
pub use vendor::Vendor;
