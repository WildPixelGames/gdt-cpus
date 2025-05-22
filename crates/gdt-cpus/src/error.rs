//! Defines the error types and `Result` alias used throughout the `gdt-cpus` crate.
//!
//! This module provides a centralized way to handle errors that can occur during
//! CPU information detection, thread affinity management, or other operations.
//! The primary error type is [`Error`], and the standard `Result` type is aliased
//! as [`Result<T>`] for convenience.

use std::fmt;

/// A specialized `Result` type for `gdt-cpus` operations.
///
/// This type alias uses [`crate::error::Error`] as its error type.
/// All functions in this crate that can fail will return this `Result` type.
pub type Result<T> = std::result::Result<T, Error>;

/// The primary error enum for all operations within the `gdt-cpus` crate.
///
/// This enum consolidates various error conditions that can arise,
/// such as issues with CPU detection, platform incompatibilities,
/// permission problems, I/O errors, and invalid parameters.
/// It implements `std::error::Error` for interoperability with other Rust error handling mechanisms.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Error {
    /// An error occurred during the process of detecting CPU information.
    /// This could be due to parsing issues, unexpected system responses, or
    /// platform-specific problems.
    /// Contains a descriptive message about the detection failure.
    Detection(String),

    /// An invalid core ID was provided to a function.
    /// Core IDs are typically 0-indexed and should correspond to actual logical processors.
    /// Contains the invalid ID that was used.
    InvalidCoreId(usize),

    /// No CPU core of the requested type (e.g., Performance or Efficiency) could be found.
    /// This can happen on systems without hybrid architectures or if the specified
    /// type doesn't exist or isn't distinguishable on the current platform.
    /// Contains a string describing the requested core type.
    NoCoreOfType(String),

    /// An error occurred during thread affinity operations.
    /// This could involve issues setting or getting thread affinity, such as
    /// the specified core ID being invalid for affinity operations or OS-level restrictions.
    /// Contains a descriptive message about the affinity failure.
    Affinity(String),

    /// The requested operation is not supported on the current operating system
    /// or hardware platform.
    /// Contains a message explaining why the operation is unsupported.
    Unsupported(String),

    /// The operation could not be completed due to insufficient permissions.
    /// For example, setting thread priority or affinity might require
    /// administrator/root privileges on the operating system.
    /// Contains a message detailing the permission issue.
    PermissionDenied(String),

    /// An underlying Input/Output error occurred.
    /// This often wraps `std::io::Error` and is used for file operations or
    /// interactions with system devices that result in I/O failures.
    /// Contains a descriptive message about the I/O failure.
    Io(String),

    /// An error occurred during a system call.
    /// This is often used for platform-specific API call failures not covered
    /// by `std::io::Error`, such as issues with `sysctl` on macOS/BSD or
    /// other low-level OS interactions.
    /// Contains a descriptive message about the system call failure.
    SystemCall(String),

    /// A requested resource or item was not found.
    /// For example, trying to access a non-existent configuration file, registry key,
    /// or a specific piece of information that the system doesn't provide.
    /// Contains a message describing what was not found.
    NotFound(String),

    /// An invalid parameter was supplied to a function.
    /// This is a general error for cases where input validation fails and the
    /// parameter does not fit other more specific error categories.
    /// Contains a message explaining which parameter was invalid and why.
    InvalidParameter(String),

    /// The requested feature or operation is not yet implemented in this version
    /// of the crate.
    /// This is a placeholder for future development or for features that are
    /// planned but not yet available.
    NotImplemented,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Detection(msg) => write!(f, "CPU detection error: {}", msg),
            Error::InvalidCoreId(id) => write!(f, "Invalid core ID: {}", id),
            Error::NoCoreOfType(ty) => write!(f, "No core of type {} found", ty),
            Error::Affinity(msg) => write!(f, "Thread affinity error: {}", msg),
            Error::Unsupported(msg) => write!(f, "Unsupported operation: {}", msg),
            Error::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            Error::Io(msg) => write!(f, "I/O error: {}", msg),
            Error::SystemCall(msg) => write!(f, "System call error: {}", msg),
            Error::NotFound(msg) => write!(f, "Not found: {}", msg),
            Error::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            Error::NotImplemented => write!(f, "Operation not implemented"),
        }
    }
}

impl std::error::Error for Error {}

// Implementations of From for common error conversions
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err.to_string())
    }
}
