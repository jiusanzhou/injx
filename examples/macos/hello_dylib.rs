//! macOS Simple dylib Example
//!
//! A minimal dylib that prints a message when loaded.
//!
//! # Build
//!
//! ```bash
//! cargo build --example hello_dylib --release
//! ```
//!
//! # Usage
//!
//! ```bash
//! # Runtime injection (requires root)
//! sudo injx Safari target/release/examples/libhello_dylib.dylib
//!
//! # Or via DYLD_INSERT_LIBRARIES
//! DYLD_INSERT_LIBRARIES=./libhello_dylib.dylib /path/to/app
//! ```

use std::ffi::CString;

#[link(name = "c")]
extern "C" {
    fn printf(format: *const i8, ...) -> i32;
}

/// Constructor - runs when dylib is loaded
#[no_mangle]
#[link_section = "__DATA,__mod_init_func"]
pub static INIT: extern "C" fn() = init;

extern "C" fn init() {
    let msg = CString::new("[injx] Hello from injected dylib!\n").unwrap();
    unsafe {
        printf(msg.as_ptr());
    }
}

/// Destructor - runs when dylib is unloaded
#[no_mangle]
#[link_section = "__DATA,__mod_term_func"]
pub static FINI: extern "C" fn() = fini;

extern "C" fn fini() {
    let msg = CString::new("[injx] Goodbye from injected dylib!\n").unwrap();
    unsafe {
        printf(msg.as_ptr());
    }
}

/// Public function that can be called from the target process
#[no_mangle]
pub extern "C" fn injx_hello() -> i32 {
    let msg = CString::new("[injx] injx_hello() called!\n").unwrap();
    unsafe {
        printf(msg.as_ptr());
    }
    42
}
