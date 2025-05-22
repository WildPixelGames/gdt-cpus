//! Linux-specific implementations for CPU information and thread management.
//!
//! This module provides the Linux-specific logic for:
//! - Detecting detailed CPU information, including vendor, model, features,
//!   and topology (sockets, cores, logical processors, caches). This is primarily
//!   handled by the [`cpu`] submodule.
//! - Managing thread affinity via the [`affinity`] submodule, allowing threads
//!   to be pinned to specific CPUs.
//! - Handling thread scheduling policies, exposed through the [`scheduling_policy`]
//!   submodule.
//! - Common utility functions used across the Linux platform-specific code,
//!   available in the [`utils`] submodule.
//!
//! The primary interface for CPU information is through `gdt_cpus::cpu_info()`,
//! which will delegate to `cpu::detect_cpu_info()` when compiled for Linux.
//! Similarly, affinity and priority functions in the crate root will use
//! implementations from this module.

pub(crate) mod affinity;
pub(crate) mod cpu;
pub(crate) mod scheduling_policy;
pub(crate) mod utils;
