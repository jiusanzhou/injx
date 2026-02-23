//! Linux Shared Object Injection Example
//!
//! A minimal .so that prints a message when loaded.
//!
//! # Build
//!
//! ```bash
//! cargo build --example hello_so --release
//! ```
//!
//! # Usage
//!
//! ```bash
//! # Runtime injection (requires CAP_SYS_PTRACE or root)
//! sudo injx target_app target/release/examples/libhello_so.so
//!
//! # Or via LD_PRELOAD
//! LD_PRELOAD=./libhello_so.so /path/to/app
//! ```

use std::ffi::CString;

#[link(name = "c")]
extern "C" {
    fn printf(format: *const i8, ...) -> i32;
    fn getpid() -> i32;
    fn getppid() -> i32;
}

/// Constructor - runs when .so is loaded
/// __attribute__((constructor))
#[no_mangle]
#[link_section = ".init_array"]
pub static INIT: extern "C" fn() = init;

extern "C" fn init() {
    let pid = unsafe { getpid() };
    let ppid = unsafe { getppid() };

    let msg = CString::new("[injx] Hello from injected .so! PID: %d, PPID: %d\n").unwrap();
    unsafe {
        printf(msg.as_ptr(), pid, ppid);
    }
}

/// Destructor - runs when .so is unloaded
/// __attribute__((destructor))
#[no_mangle]
#[link_section = ".fini_array"]
pub static FINI: extern "C" fn() = fini;

extern "C" fn fini() {
    let msg = CString::new("[injx] Goodbye from injected .so!\n").unwrap();
    unsafe {
        printf(msg.as_ptr());
    }
}

/// Public function that can be called
#[no_mangle]
pub extern "C" fn injx_hello() -> i32 {
    let msg = CString::new("[injx] injx_hello() called from injected .so!\n").unwrap();
    unsafe {
        printf(msg.as_ptr());
    }
    42
}
