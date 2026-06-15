//! Error types and the `Result` alias used throughout the `gdt-cpus` crate.

use std::fmt;

/// A specialized `Result` type for `gdt-cpus` operations.
pub type Result<T> = std::result::Result<T, Error>;

/// The error enum for all operations within the `gdt-cpus` crate.
///
/// Payloads are descriptive strings; callers that need machine-readable
/// scheduler outcomes should use [`crate::AppliedPriority`] for priority calls.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Error {
    /// CPU information detection failed (parsing, unexpected system response,
    /// platform-specific trouble). Contains a descriptive message.
    Detection(String),

    /// An invalid logical-core ID was provided. Contains the offending ID.
    InvalidCoreId(usize),

    /// A thread affinity operation failed. Contains a descriptive message.
    Affinity(String),

    /// The requested operation is not supported on this OS or hardware.
    /// Contains a message explaining why.
    Unsupported(String),

    /// The operation requires permissions the process doesn't have (e.g.
    /// raising thread priority unprivileged). Contains the detail message.
    PermissionDenied(String),

    /// A system call failed (sysctl, Win32, raw syscalls). Contains a
    /// descriptive message.
    SystemCall(String),

    /// A requested resource was not found (file, registry key, sysctl key).
    /// Contains a message describing what was missing.
    NotFound(String),

    /// An invalid parameter was supplied. Contains the explanation.
    InvalidParameter(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Detection(msg) => write!(f, "CPU detection error: {}", msg),
            Error::InvalidCoreId(id) => write!(f, "Invalid core ID: {}", id),
            Error::Affinity(msg) => write!(f, "Thread affinity error: {}", msg),
            Error::Unsupported(msg) => write!(f, "Unsupported operation: {}", msg),
            Error::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            Error::SystemCall(msg) => write!(f, "System call error: {}", msg),
            Error::NotFound(msg) => write!(f, "Not found: {}", msg),
            Error::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
        }
    }
}

impl std::error::Error for Error {}
