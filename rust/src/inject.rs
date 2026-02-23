//! Cross-platform injection trait
//!
//! This module defines the unified `Injector` trait that all platform-specific
//! implementations must satisfy.

use crate::error::Result;

/// Trait for library injection capabilities
///
/// This trait provides a unified API for injecting and ejecting dynamic
/// libraries across all supported platforms.
///
/// # Example
///
/// ```rust,no_run
/// use injx::{Process, Injector};
///
/// let process = Process::from_pid(1234).unwrap();
///
/// // Inject a library
/// process.inject("./payload.dll")?;
///
/// // Check if injected
/// if process.is_injected("payload.dll")? {
///     println!("Successfully injected!");
/// }
///
/// // Eject the library
/// process.eject("payload.dll")?;
/// # Ok::<(), injx::Error>(())
/// ```
pub trait Injector {
    /// Check if a library is already injected into the process
    ///
    /// # Arguments
    ///
    /// * `lib` - Library name or path to check
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Library is loaded in the process
    /// * `Ok(false)` - Library is not loaded
    /// * `Err(_)` - Failed to check injection status
    fn is_injected(&self, lib: &str) -> Result<bool>;
    
    /// Inject a dynamic library into the process
    ///
    /// # Arguments
    ///
    /// * `lib` - Path to the library file (.dll/.dylib/.so)
    ///
    /// # Platform Notes
    ///
    /// - **Windows**: Uses CreateRemoteThread + LoadLibraryW
    /// - **macOS**: Uses task_for_pid + thread_create_running
    /// - **Linux**: Uses ptrace + dlopen
    ///
    /// # Errors
    ///
    /// * `Error::LibraryNotFound` - Library file doesn't exist
    /// * `Error::PermissionDenied` - Insufficient privileges
    /// * `Error::AlreadyInjected` - Library already loaded (if checking)
    /// * `Error::ArchitectureMismatch` - Wrong architecture
    fn inject(&self, lib: &str) -> Result<()>;
    
    /// Eject/unload a dynamic library from the process
    ///
    /// # Arguments
    ///
    /// * `lib` - Library name or path to eject
    ///
    /// # Platform Notes
    ///
    /// - **Windows**: Uses FreeLibrary via CreateRemoteThread
    /// - **macOS**: Uses dlclose (if supported)
    /// - **Linux**: Uses dlclose via ptrace
    ///
    /// # Errors
    ///
    /// * `Error::Unsupported` - Platform doesn't support ejection
    /// * `Error::LibraryNotFound` - Library not currently loaded
    fn eject(&self, lib: &str) -> Result<()>;
}
