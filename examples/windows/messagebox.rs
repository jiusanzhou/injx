//! Windows MessageBox DLL Example
//!
//! This DLL shows a message box when injected into a process.
//!
//! # Build
//!
//! ```bash
//! # For 32-bit targets
//! cargo build --example messagebox --target i686-pc-windows-msvc --release
//!
//! # For 64-bit targets
//! cargo build --example messagebox --target x86_64-pc-windows-msvc --release
//! ```
//!
//! # Usage
//!
//! ```bash
//! injx notepad.exe target/release/examples/messagebox.dll
//! ```

use std::ptr::null_mut;

#[link(name = "user32")]
extern "system" {
    fn MessageBoxW(hwnd: *mut (), text: *const u16, caption: *const u16, utype: u32) -> i32;
}

/// Convert &str to wide string (UTF-16)
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// DLL entry point
#[no_mangle]
pub extern "system" fn DllMain(
    _hinst: *mut (),
    reason: u32,
    _reserved: *mut (),
) -> i32 {
    const DLL_PROCESS_ATTACH: u32 = 1;
    const DLL_PROCESS_DETACH: u32 = 0;

    match reason {
        DLL_PROCESS_ATTACH => {
            // Show message box on injection
            let text = to_wide("Hello from injected DLL!\n\ninjx - Cross-platform injection toolkit");
            let caption = to_wide("injx");
            unsafe {
                MessageBoxW(null_mut(), text.as_ptr(), caption.as_ptr(), 0x40); // MB_ICONINFORMATION
            }
        }
        DLL_PROCESS_DETACH => {
            // Optional: cleanup on eject
        }
        _ => {}
    }

    1 // TRUE
}
