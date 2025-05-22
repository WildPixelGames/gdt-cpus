//! macOS-specific CPU detection and thread management.
//!
//! This module encapsulates all platform-specific logic for macOS. It provides
//! functionalities for detecting detailed CPU information, managing thread affinity,
//! and setting thread scheduling policies tailored for the macOS environment.
//!
//! ## Submodules
//!
//! - [`affinity`]: Handles pinning threads to specific logical cores and managing
//!   thread affinity settings.
//! - [`cpu`]: Contains the core logic for detecting CPU features, topology (sockets,
//!   physical cores, logical processors), cache information, and distinguishing
//!   between Performance and Efficiency cores using macOS-specific APIs like `sysctl`.
//! - [`scheduling_policy`]: Defines how abstract thread priorities map to
//!   macOS-specific scheduling policies (e.g., QoS classes).
//! - [`utils`]: Provides utility functions used across the macOS platform-specific modules,
//!   often for interacting with system calls or parsing system information.

pub(crate) mod affinity;
pub(crate) mod cpu;
pub(crate) mod scheduling_policy;
pub(crate) mod utils;
