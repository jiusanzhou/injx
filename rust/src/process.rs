/// Cross-platform process abstraction
/// 
/// Platform-specific implementations:
/// - Windows: process_windows.rs
/// - macOS: process_macos.rs  
/// - Linux: process_linux.rs

#[cfg(target_os = "windows")]
mod platform {
    pub use crate::process_windows::*;
}

#[cfg(target_os = "macos")]
mod platform {
    pub use crate::process_macos::*;
}

#[cfg(target_os = "linux")]
mod platform {
    pub use crate::process_linux::*;
}

pub use platform::*;
