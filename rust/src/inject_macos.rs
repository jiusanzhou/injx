//! macOS dylib injection using Mach APIs
//!
//! Uses task_for_pid + thread_create_running technique.
//!
//! # Requirements
//!
//! - Root privileges or taskgated entitlement
//! - SIP (System Integrity Protection) may need to be disabled for some targets

use std::ffi::CString;
use std::path::Path;
use std::ptr::null_mut;

use crate::error::{Error, Result};
use crate::inject::Injector;
use crate::process_macos::Process;

// Mach types
type MachPortT = u32;
type KernReturnT = i32;
type TaskT = MachPortT;
type ThreadT = MachPortT;
type MachVmAddressT = u64;
type MachVmSizeT = u64;
type ThreadStateFlavor = i32;
type NaturalT = u32;

const KERN_SUCCESS: KernReturnT = 0;
const VM_FLAGS_ANYWHERE: i32 = 0x0001;
const VM_PROT_READ: i32 = 0x01;
const VM_PROT_WRITE: i32 = 0x02;

#[cfg(target_arch = "aarch64")]
const ARM_THREAD_STATE64: ThreadStateFlavor = 6;
#[cfg(target_arch = "aarch64")]
const ARM_THREAD_STATE64_COUNT: u32 = 68;

#[cfg(target_arch = "x86_64")]
const X86_THREAD_STATE64: ThreadStateFlavor = 4;
#[cfg(target_arch = "x86_64")]
const X86_THREAD_STATE64_COUNT: u32 = 42;

// RTLD_NOW constant
const RTLD_NOW: i32 = 0x2;

#[link(name = "System", kind = "framework")]
extern "C" {
    fn mach_task_self() -> MachPortT;
    fn mach_vm_allocate(
        target_task: TaskT,
        address: *mut MachVmAddressT,
        size: MachVmSizeT,
        flags: i32,
    ) -> KernReturnT;
    fn mach_vm_write(
        target_task: TaskT,
        address: MachVmAddressT,
        data: *const u8,
        data_cnt: u32,
    ) -> KernReturnT;
    fn mach_vm_protect(
        target_task: TaskT,
        address: MachVmAddressT,
        size: MachVmSizeT,
        set_maximum: i32,
        new_protection: i32,
    ) -> KernReturnT;
    fn mach_vm_deallocate(
        target_task: TaskT,
        address: MachVmAddressT,
        size: MachVmSizeT,
    ) -> KernReturnT;
    fn thread_create_running(
        parent_task: TaskT,
        flavor: ThreadStateFlavor,
        new_state: *const NaturalT,
        new_state_count: u32,
        child_thread: *mut ThreadT,
    ) -> KernReturnT;
    fn thread_terminate(thread: ThreadT) -> KernReturnT;
}

#[link(name = "dl")]
extern "C" {
    fn dlsym(handle: *mut std::ffi::c_void, symbol: *const i8) -> *mut std::ffi::c_void;
}

impl Injector for Process {
    fn is_injected(&self, lib: &str) -> Result<bool> {
        // Use vmmap to check loaded libraries
        // This is a simplified check - full implementation would parse vmmap output
        let lib_name = Path::new(lib)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(lib);

        let output = std::process::Command::new("vmmap")
            .arg(self.pid().to_string())
            .output();
     match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                Ok(stdout.contains(lib_name))
            }
            Err(_) => Ok(false), // vmmap not available or access denied
        }
    }

    fn inject(&self, lib: &str) -> Result<()> {
        // Validate dylib path
        let fullpath = Path::new(lib)
            .canonicalize()
            .map_err(|e| Error::library_not_found(format!("{}: {}", lib, e)))?;

        let lib_path = fullpath
            .to_str()
            .ok_or_else(|| Error::invalid_argument("invalid path encoding"))?;

        let lib_cstr = CString::new(lib_path)
            .map_err(|_| Error::invalid_argument("path contains null byte"))?;

        let lib_bytes = lib_cstr.as_bytes_with_nul();
        let lib_len = lib_bytes.len();

        // Allocate memory in target for dylib path
        let mut remote_path: MachVmAddressT = 0;
        let kr = unsafe {
            mach_vm_allocate(
                self.task(),
                &mut remote_path,
                lib_len as MachVmSizeT,
                VM_FLAGS_ANYWHERE,
            )
        };
        if kr != KERN_SUCCESS {
            return Err(Error::from_mach(kr));
        }

        // Write path to target
        let kr = unsafe {
            mach_vm_write(self.task(), remote_path, lib_bytes.as_ptr(), lib_len as u32)
        };
        if kr != KERN_SUCCESS {
            unsafe { machate(self.task(), remote_path, lib_len as MachVmSizeT) };
            return Err(Error::from_mach(kr));
        }

        // Make memory readable
        let kr = unsafe {
            mach_vm_protect(
                self.task(),
                remote_path,
                lib_len as MachVmSizeT,
                0,
                VM_PROT_READ,
            )
        };
        if kr != KERN_SUCCESS {
            eprintln!("Warning: mach_vm_protect failed: {}", kr);
        }

        // Get dlopen address
        let dlopen_addr = get_dlopen_address()?;

        // Allocate stack
        let stack_size: MachVmSizeT = 0x10000; // 64KB
        let mut stack_addr: MachVmAddressT = 0;
        let kr = unsafe {
            mach_vm_allocate(self.task(), &mut stack_addr, stack_size, VM_FLAGS_ANYWHERE)
        };
        if kr != KERN_SUCCESS {
            unsafe { mach_vm_deallocate(self.task(), remote_path, lib_len as MachVmSizeT) };
            return Err(Error::from_mach(kr));
        }

        // Make stack RW
        unsafe {
            mach_vm_protect(
                self.task(),
                stack_addr,
                stack_size,
                0,
                VM_PROT_READ | VM_PROT_WRITE,
            )
        };

        // Create thread
        let result = self.create_injection_thread(
            dlopen_addr,
            remote_path,
            stack_addr + stack_size,
        );

        // Note: We don't cleanup remote_path immediately as the thread needs it
        // In a production implementation, we'd wait for thread completion

        result
    }

    fn eject(&self, _lib: &str) -> Result<()> {
        Err(Error::unsupported("eject not yet implemented for macOS"))
    }
}

impl Process {
    #[cfg(target_arch = "aarch64")]
    fn create_injection_thread(
        &self,
        dlopen_addr: u64,
        path_addr: MachVmAddressT,
        stack_top: MachVmAddressT,
    ) -> Result<()> {
        // ARM64 thread state
        // x0 = path (first argument)
        // x1 = RTLD_NOW (second argument)
        // sp = stack pointer
        // pc = dlopen address
        let mut state = [0u64; 34]; // arm_thread_state64_t
        state[0] = path_addr;       // x0 = path
        state[1] = RTLD_NOW as u64; // x1 = RTLD_NOW
        state[31] = stack_top;      // sp
        state[32] = dlopen_addr;    // pc

        let mut thread: ThreadT = 0;
        let kr = unsafe {
            thread_create_running(
                self.task(),
                ARM_THREAD_STATE64,
                state.as_ptr() as *const NaturalT,
                ARM_THREAD_STATE64_COUNT,
                &mut thread,
            )
        };

        if kr != KERN_SUCCESS {
            return Err(Error::from_mach(kr));
        }

        eprintln!("Created injection thread {} in target process", thread);
        Ok(())
    }

    #[cfg(target_arch = "x86_64")]
    fn create_injection_thread(
        &self,
        dlopen_addr: u64,
        path_addr: MachVmAddressT,
        stack_top: MachVmAddressT,
    ) -> Result<()> {
        // x86_64 thread state
        // rdi = path (first argument)
        // rsi = RTLD_NOW (second argument)
        // rsp = stack pointer (16-byte aligned)
        // rip = dlopen address
        let mut state = [0u64; 21]; // x86_thread_state64_t
        state[4] = path_addr;           // rdi
        state[5] = RTLD_NOW as u64;     // rsi
        state[7] = stack_top & !0xF;    // rsp (aligned)
        state[16] = dlopen_addr;        // rip

        let mut thread: ThreadT = 0;
        let kr = unsafe {
            thread_create_running(
                self.task(),
                X86_THREAD_STATE64,
                state.as_ptr() as *const NaturalT,
                X86_THREAD_STATE64_COUNT,
                &mut thread,
            )
        };

        if kr != KERN_SUCCESS {
            return Err(Error::from_mach(kr));
        }

        eprintln!("Created injection thread {} in target process", thread);
        Ok(())
    }
}

/// Get dlopen address from current process
///
/// Assumes same shared cache in target (works for most userspace apps)
fn get_dlopen_address() -> Result<u64> {
    const RTLD_DEFAULT: *mut std::ffi::c_void = std::ptr::null_mut();

    let sym = CString::new("dlopen").unwrap();
    let addr = unsafe { dlsym(RTLD_DEFAULT, sym.as_ptr()) };

    if addr.is_null() {
        return Err(Error::platform(0, "dlopen symbol not found"));
    }

    Ok(addr as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_dlopen() {
        let addr = get_dlopen_address();
        assert!(addr.is_ok());
        println!("dlopen @ 0x{:x}", addr.unwrap());
    }
}
