//! # injx - Cross-platform Dynamic Library Injection Toolkit
//!
//! `injx` provides a unified API for injecting dynamic libraries into running processes
//! across Windows, macOS, and Linux platforms.
//!
//! ## Platform Support
//!
//! | Platform | Library Type | Technique |
//! |----------|--------------|-----------|
//! | Windows  | DLL          | CreateRemoteThread + LoadLibrary |
//! | macOS    | dylib        | task_for_pid + thread_create_running |
//! | Linux    | .so          | ptrace + dlopen |
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use injx::{Process, Injector};
//!
//! // Find process by name
//! let process = Process::find_by_name("target_app").unwrap();
//! println!("Found: {} (PID: {})", process.name(), process.pid());
//!
//! // Inject library
//! process.inject("./payload.dll").unwrap();
//!
//! // Check injection status
//! if process.is_injected("payload.dll").unwrap() {
//!     println!("Injection successful!");
//! }
//!
//! // Eject library (if supported)
//! process.eject("payload.dll").unwrap();
//! ```
//!
//! ## Using PID
//!
//! ```rust,no_run
//! use injx::Process;
//!
//! let process = Process::from_pid(1234).unwrap();
//! process.inject("./payload.dll").unwrap();
//! ```
//!
//! ## Requirements
//!
//! - **Windows**: Same architecture as target (x86 inject x86, x64 inject x64)
//! - **macOS**: Root privileges or taskgated entitlement
//! - **Linux**: CAP_SYS_PTRACE capability or root

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![allow(dead_code)]

pub mod error;
pub mod inject;

mod utils;

// Platform-specific modules
#[cfg(target_os = "windows")]
mod winapi;
#[cfg(target_os = "windows")]
mod process_windows;
#[cfg(target_os = "windows")]
mod inject_windows;
#[cfg(target_os = "windows")]
mod elevate_windows;

#[cfg(target_os = "macos")]
mod process_macos;
#[cfg(target_os = "macos")]
mod inject_macos;

#[cfg(target_os = "linux")]
mod process_linux;
#[cfg(target_os = "linux")]
mod inject_linux;

// Re-exports for unified API
pub use error::{Error, Result};
pub use inject::Injector;

#[cfg(target_os = "windows")]
pub use process_windows::Process;
#[cfg(target_os = "windows")]
pub use elevate_windows::elevate_privileges;

#[cfg(target_os = "macos")]
pub use process_macos::Process;

#[cfg(target_os = "linux")]
pub use process_linux::Process;

/// Prelude module for convenient imports
pub mod prelude {
    //! Convenient re-exports for common usage.
    //!
    //! ```rust
    //! use injx::prelude::*;
    //! ```
    
    pub use crate::error::{Error, Result};
    pub use crate::inject::Injector;
    pub use crate::Process;
}
