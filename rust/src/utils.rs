//! Utility functions

use std::io;

/// Get the last OS error
#[inline(always)]
pub fn get_last_error() -> io::Error {
    io::Error::last_os_error()
}

/// Convert string to wide string (Windows UTF-16)
#[cfg(target_os = "windows")]
pub fn to_wide_string(s: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

/// Normalize a library path
#[allow(dead_code)]
pub fn normalize_lib_name(lib: &str) -> String {
    std::path::Path::new(lib)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(lib)
        .to_lowercase()
}
