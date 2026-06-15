//! Windows-specific implementations for CPU information and thread management.
//!
//! This module provides the Windows-specific logic for:
//! - Detecting detailed CPU information, including vendor, model, features,
//!   and topology (sockets, cores, logical processors, caches). This is primarily
//!   handled by the [`cpu`] submodule, which leverages Windows API calls and
//!   may also use the [`registry`] submodule for supplemental information.
//! - Managing thread affinity via the [`affinity`] submodule.
//! - Handling thread scheduling policies and priorities, exposed through the
//!   [`scheduling_policy`] submodule.
//! - Common utility functions specific to Windows platform code, available in
//!   the [`utils`] submodule.
//!
//! The primary interface for CPU information is [`crate::CpuInfo::detect()`].
//! Affinity and priority functions in the crate root use implementations from
//! this module.

pub(crate) mod affinity;
pub(crate) mod cpu;
pub(crate) mod registry;
pub(crate) mod scheduling_policy;
pub(crate) mod utils;
