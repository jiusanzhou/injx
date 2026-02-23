//! Error types for injx
//!
//! This module provides unified error handling across all platforms.

use std::error::Error as StdError;
use std::fmt;
use std::io;

/// Result type alias using injx Error
pub type Result<T> = std::result::Result<T, Error>;

/// Unified error type for injection operations
#[derive(Debug)]
pub enum Error {
    /// Process not found
    ProcessNotFound(String),
    
    /// Library/module not found
    LibraryNotFound(String),
    
    /// Insufficient permissions
    PermissionDenied(String),
    
    /// Library already injected
    AlreadyInjected(String),
    
    /// Architecture mismatch (e.g., x86 vs x64)
    ArchitectureMismatch(String),
    
    /// Platform-specific API error
    PlatformError {
        /// Error code (platform-specific)
        code: i32,
        /// Error message
        message: String,
    },
    
    /// Operation not supported on this platform
    Unsupported(String),
    
    /// Generic IO error
    Io(io::Error),
    
    /// Invalid argument
    InvalidArgument(String),
}

impl Error {
    /// Create a process not found error
    pub fn process_not_found(name: impl Into<String>) -> Self {
        Error::ProcessNotFound(name.into())
    }
    
    /// Create a library not found error
    pub fn library_not_found(path: impl Into<String>) -> Self {
        Error::LibraryNotFound(path.into())
    }
    
    /// Create a permission denied error
    pub fn permission_denied(msg: impl Into<String>) -> Self {
        Error::PermissionDenied(msg.into())
    }
    
    /// Create an already injected error
    pub fn already_injected(lib: impl Into<String>) -> Self {
        Error::AlreadyInjected(lib.into())
    }
    
    /// Create an unsupported error
    pub fn unsupported(feature: impl Into<String>) -> Self {
        Error::Unsupported(feature.into())
    }
    
    /// Create a platform-specific error
    pub fn platform(code: i32, message: impl Into<String>) -> Self {
        Error::PlatformError {
            code,
            message: message.into(),
        }
    }
    
    /// Create an invalid argument error
    pub fn invalid_argument(msg: impl Into<String>) -> Self {
        Error::InvalidArgument(msg.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ProcessNotFound(name) => {
                write!(f, "process not found: {}", name)
            }
            Error::LibraryNotFound(path) => {
                write!(f, "library not found: {}", path)
            }
            Error::PermissionDenied(msg) => {
                write!(f, "permission denied: {}", msg)
            }
            Error::AlreadyInjected(lib) => {
                write!(f, "library already injected: {}", lib)
            }
            Error::ArchitectureMismatch(msg) => {
                write!(f, "architecture mismatch: {}", msg)
            }
            Error::PlatformError { code, message } => {
                write!(f, "platform error ({}): {}", code, message)
            }
            Error::Unsupported(feature) => {
                write!(f, "unsupported: {}", feature)
            }
            Error::Io(err) => {
                write!(f, "io error: {}", err)
            }
            Error::InvalidArgument(msg) => {
                write!(f, "invalid argument: {}", msg)
            }
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<Error> for io::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::Io(e) => e,
            Error::ProcessNotFound(msg) => io::Error::new(io::ErrorKind::NotFound, msg),
            Error::LibraryNotFound(msg) => io::Error::new(io::ErrorKind::NotFound, msg),
            Error::PermissionDenied(msg) => io::Error::new(io::ErrorKind::PermissionDenied, msg),
            Error::Unsupported(msg) => io::Error::new(io::ErrorKind::Unsupported, msg),
            _ => io::Error::new(io::ErrorKind::Other, err.to_string()),
        }
    }
}

// Windows-specific error conversion
#[cfg(target_os = "windows")]
impl Error {
    /// Create error from Windows last error code
    pub fn from_win32() -> Self {
        let err = io::Error::last_os_error();
        let code = err.raw_os_error().unwrap_or(0);
        Error::PlatformError {
            code,
            message: err.to_string(),
        }
    }
}

// macOS-specific error conversion
#[cfg(target_os = "macos")]
impl Error {
    /// Create error from Mach kernel return code
    pub fn from_mach(kr: i32) -> Self {
        Error::PlatformError {
            code: kr,
            message: format!("mach error: {}", kr),
        }
    }
}

// Linux-specific error conversion
#[cfg(target_os = "linux")]
impl Error {
    /// Create error from errno
    pub fn from_errno() -> Self {
        let err = io::Error::last_os_error();
        let code = err.raw_os_error().unwrap_or(0);
        Error::PlatformError {
            code,
            message: err.to_string(),
        }
    }
}
