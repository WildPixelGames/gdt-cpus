//! macOS-specific CPU detection and thread management - Apple Silicon only.
//!
//! x86_64 macOS is deliberately unsupported so the backend can stay focused on
//! the Apple Silicon topology and QoS behavior we test.
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

#[cfg(all(target_os = "macos", not(target_arch = "aarch64")))]
compile_error!(
    "gdt-cpus supports macOS on Apple Silicon (aarch64) only; \
     x86_64-apple-darwin is intentionally unsupported"
);

// Thread control + the live sysctl reader are genuinely macOS-only. The
// detection pipeline (`cpu`) is pure logic behind the SysctlSource seam and
// also compiles for tests on other platforms (fixture-driven CI coverage).
#[cfg(target_os = "macos")]
pub(crate) mod affinity;
pub(crate) mod cpu;
#[cfg(target_os = "macos")]
pub(crate) mod scheduling_policy;
#[cfg(target_os = "macos")]
pub(crate) mod utils;
