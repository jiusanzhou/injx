//! Windows Process Info Collector DLL
//!
//! When injected, this DLL collects information about the target process
//! and writes it to a file.
//!
//! # Build
//!
//! ```bash
//! cargo build --example procinfo --target x86_64-pc-windows-msvc --release
//! ```

use std::fs::File;
use std::io::Write;
use std::ptr::null_mut;

#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcessId() -> u32;
    fn GetModuleFileNameW(hModule: *mut (), lpFilename: *mut u16, nSize: u32) -> u32;
    fn GetCurrentDirectoryW(nBufferLength: u32, lpBuffer: *mut u16) -> u32;
}

fn wide_to_string(wide: &[u16]) -> String {
    let len = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    String::from_utf16_lossy(&wide[..len])
}

fn collect_process_info() -> String {
    let pid = unsafe { GetCurrentProcessId() };

    let mut exe_path = vec![0u16; 260];
    let exe_len = unsafe { GetModuleFileNameW(null_mut(), exe_path.as_mut_ptr(), 260) };
    let exe = if exe_len > 0 {
        wide_to_string(&exe_path)
    } else {
        "unknown".to_string()
    };

    let mut cwd_path = vec![0u16; 260];
    let cwd_len = unsafe { GetCurrentDirectoryW(260, cwd_path.as_mut_ptr()) };
    let cwd = if cwd_len > 0 {
        wide_to_string(&cwd_path)
    } else {
        "unknown".to_string()
    };

    format!(
        r#"Process Information
===================
PID:        {}
Executable: {}
Working Dir: {}
Collected by: injx process info collector
"#,
        pid, exe, cwd
    )
}

#[no_mangle]
pub extern "system" fn DllMain(_hinst: *mut (), reason: u32, _reserved: *mut ()) -> i32 {
    if reason == 1 {
        // DLL_PROCESS_ATTACH
        let info = collect_process_info();

        // Write to temp file
        if let Ok(mut file) = File::create("C:\\temp\\injx_procinfo.txt") {
            let _ = file.write_all(info.as_bytes());
        }
    }

    1
}
