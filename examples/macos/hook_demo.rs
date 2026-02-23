//! macOS Function Hook Example
//!
//! Demonstrates how to hook functions in macOS using fishhook-style technique.
//!
//! # Build
//!
//! ```bash
//! cargo build --example hook_demo --release
//! ```
//!
//! # Note
//!
//! This is a simplified example. For production use, consider:
//! - fishhook (https://github.com/facebook/fishhook)
//! - frida-gum
//! - substrate

use std::ffi::{CString, CStr, c_void};
use std::sync::atomic::{AtomicPtr, Ordering};

#[link(name = "c")]
extern "C" {
    fn dlsym(handle: *mut c_void, symbol: *const i8) -> *mut c_void;
    fn printf(format: *const i8, ...) -> i32;
}

const RTLD_DEFAULT: *mut c_void = std::ptr::null_mut();
const RTLD_NEXT: *mut c_void = -1isize as *mut c_void;

// Original function pointer storage
static ORIGINAL_STRLEN: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// Our hooked strlen implementation
#[no_mangle]
pub extern "C" fn hooked_strlen(s: *const i8) -> usize {
    // Call original function
    let original: extern "C" fn(*const i8) -> usize = unsafe {
        std::mem::transmute(ORIGINAL_STRLEN.load(Ordering::SeqCst))
    };

    let result = original(s);

    // Log the call
    let log_msg = CString::new("[injx hook] strlen called, length: %zu\n").unwrap();
    unsafe {
        printf(log_msg.as_ptr(), result);
    }

    result
}

/// Initialize hooks
fn init_hooks() {
    let strlen_sym = CString::new("strlen").unwrap();

    // Get original strlen
    let original = unsafe { dlsym(RTLD_NEXT, strlen_sym.as_ptr()) };
    ORIGINAL_STRLEN.store(original, Ordering::SeqCst);

    let msg = CString::new("[injx] Function hooks initialized\n").unwrap();
    unsafe {
        printf(msg.as_ptr());
    }

    // NOTE: Actually rebinding the symbol requires modifying the Mach-O
    // lazy symbol pointers. This is what fishhook does.
    // For a complete implementation, use fishhook or similar libraries.
}

#[no_mangle]
#[link_section = "__DATA,__mod_init_func"]
pub static INIT: extern "C" fn() = || {
    init_hooks();
};
