//! Linux shared object injection using ptrace
//!
//! Uses ptrace + dlopen technique.
//!
//! # Requirements
//!
//! - CAP_SYS_PTRACE capability or root privileges
//! - ptrace_scope may need to be set to 0: `echo 0 > /proc/sys/kernel/yama/ptrace_scope`

use std::ffi::CString;
use std::fs;
use std::path::Path;
use std::ptr::null_mut;

use crate::error::{Error, Result};
use crate::inject::Injector;
use crate::process_linux::Process;

type PidT = i32;

// ptrace constants
const PTRACE_ATTACH: i32 = 16;
const PTRACE_DETACH: i32 = 17;
const PTRACE_GETREGS: i32 = 12;
const PTRACE_SETREGS: i32 = 13;
const PTRACE_PEEKTEXT: i32 = 1;
const PTRACE_POKETEXT: i32 = 4;
const PTRACE_CONT: i32 = 7;

#[cfg(target_arch = "x86_64")]
#[repr(C)]
#[derive(Debug, Clone, Default)]
struct UserRegs {
    r15: u64, r14: u64, r13: u64, r12: u64,
    rbp: u64, rbx: u64, r11: u64, r10: u64,
    r9: u64, r8: u64, rax: u64, rcx: u64,
    rdx: u64, rsi: u64, rdi: u64, orig_rax: u64,
    rip: u64, cs: u64, eflags: u64, rsp: u64,
    ss: u64, fs_base: u64, gs_base: u64,
    ds: u64, es: u64, fs: u64, gs: u64,
}

#[cfg(target_arch = "aarch64")]
#[repr(C)]
#[derive(Debug, Clone, Default)]
struct UserRegs {
    regs: [u64; 31],
    sp: u64,
    pc: u64,
    pstate: u64,
}

extern "C" {
    fn ptrace(request: i32, pid: PidT, addr: *mut std::ffi::c_void, data: *mut std::ffi::c_void) -> i64;
    fn waitpid(pid: PidT, status: *mut i32, options: i32) -> PidT;
}

impl Injector for Process {
    fn is_injected(&self, lib: &str) -> Result<bool> {
        let libs = self.loaded_libraries();
        let lib_name = Path::new(lib)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(lib);

        Ok(libs.iter().any(|l| l.contains(lib_name)))
    }

    fn inject(&self, lib: &str) -> Result<()> {
        // Validate library path
        let fullpath = Path::new(lib)
            .canonicalize()
            .map_err(|e| Error::library_not_found(format!("{}: {}", lib, e)))?;

        let lib_path = fullpath
            .to_str()
            .ok_or_else(|| Error::invalid_argument("invalid path encoding"))?;

        // Attach to process
        let ret = unsafe { ptrace(PTRACE_ATTACH, self.pid() as PidT, null_mut(), null_mut()) };
        if ret < 0 {
            return Err(Error::from_errno());
        }

        // Wait for process to stop
        let mut status: i32 = 0;
        unsafe { waitpid(self.pid() as PidT, &mut status, 0) };

        // Save original registers
        let mut orig_regs = UserRegs::default();
        let ret = unsafe {
            ptrace(
                PTRACE_GETREGS,
                self.pid() as PidT,
                null_mut(),
                &mut orig_regs as *mut _ as *mut std::ffi::c_void,
            )
        };
        if ret < 0 {
            unsafe { ptrace(PTRACE_DETACH, self.pid() as PidT, null_mut(), null_mut()) };
            return Err(Error::from_errno());
        }

        // Find dlopen address in target
        let dlopen_addr = find_dlopen_in_process(self.pid() as PidT)?;

        // Execute dlopen
        let result = execute_dlopen(self.pid() as PidT, dlopen_addr, lib_path, &orig_regs);

        // Restore original registers
        unsafe {
            ptrace(
                PTRACE_SETREGS,
                self.pid() as PidT,
                null_mut(),
                &orig_regs as *const _ as *mut std::ffi::c_void,
            )
        };

        // Detach
        unsafe { ptrace(PTRACE_DETACH, self.pid() as PidT, null_mut(), null_mut()) };

        result
    }

    fn eject(&self, _lib: &str) -> Result<()> {
        Err(Error::unsupported("eject not yet implemented for Linux"))
    }
}

#[cfg(target_arch = "x86_64")]
fn execute_dlopen(pid: PidT, dlopen_addr: u64, lib_path: &str, orig_regs: &UserRegs) -> Result<()> {
    // Write path to stack
    let stack_addr = orig_regs.rsp - 0x100;
    write_string_to_memory(pid, stack_addr, lib_path)?;

    // Set up registers for dlopen call
    // rdi = path, rsi = RTLD_NOW (2)
    let mut new_regs = orig_regs.clone();
    new_regs.rdi = stack_addr;
    new_regs.rsi = 2; // RTLD_NOW
    new_regs.rip = dlopen_addr;

    let ret = unsafe {
        ptrace(
            PTRACE_SETREGS,
            pid,
            null_mut(),
            &new_regs as *const _ as *mut std::ffi::c_void,
        )
    };
    if ret < 0 {
        return Err(Error::from_errno());
    }

    // Continue execution
    unsafe { ptrace(PTRACE_CONT, pid, null_mut(), null_mut()) };

    // Wait briefly for dlopen to complete
    // In production, we'd use a breakpoint and wait properly
    std::thread::sleep(std::time::Duration::from_millis(100));

    Ok(())
}

#[cfg(target_arch = "aarch64")]
fn execute_dlopen(pid: PidT, dlopen_addr: u64, lib_path: &str, orig_regs: &UserRegs) -> Result<()> {
    let stack_addr = orig_regs.sp - 0x100;
    write_string_to_memory(pid, stack_addr, lib_path)?;

    // x0 = path, x1 = RTLD_NOW (2)
    let mut new_regs = orig_regs.clone();
    new_regs.regs[0] = stack_addr;
    new_regs.regs[1] = 2; // RTLD_NOW
    new_regs.pc = dlopen_addr;

    let ret = unsafe {
        ptrace(
            PTRACE_SETREGS,
            pid,
            null_mut(),
            &new_regs as *const _ as *mut std::ffi::c_void,
        )
    };
    if ret < 0 {
        return Err(Error::from_errno());
    }

    unsafe { ptrace(PTRACE_CONT, pid, null_mut(), null_mut()) };
    std::thread::sleep(std::time::Duration::from_millis(100));

    Ok(())
}

/// Write a null-terminated string to process memory
fn write_string_to_memory(pid: PidT, addr: u64, data: &str) -> Result<()> {
    let bytes = data.as_bytes();
    let mut offset = 0;

    while offset <= bytes.len() {
        let mut word: u64 = 0;
        let chunk_end = std::cmp::min(offset + 8, bytes.len());

        // Build word from bytes
        for i in offset..chunk_end {
            word |= (bytes[i] as u64) << ((i - offset) * 8);
        }

        // Write word
        let ret = unsafe {
            ptrace(
                PTRACE_POKETEXT,
                pid,
                (addr + offset as u64) as *mut std::ffi::c_void,
                word as *mut std::ffi::c_void,
            )
        };
        if ret < 0 {
            return Err(Error::from_errno());
        }

        offset += 8;
    }

    Ok(())
}

/// Find dlopen address in target process
fn find_dlopen_in_process(pid: PidT) -> Result<u64> {
    // Parse /proc/[pid]/maps to find libc/libdl
    let maps_path = format!("/proc/{}/maps", pid);
    let maps_content = fs::read_to_string(&maps_path)?;

    // Find executable region of libc or libdl
    let mut libc_base: Option<u64> = None;
    for line in maps_content.lines() {
        if (line.contains("libc") || line.contains("libdl")) && line.contains("r-xp") {
            let addr_str = line.split('-').next().unwrap_or("");
            if let Ok(addr) = u64::from_str_radix(addr_str, 16) {
                libc_base = Some(addr);
                break;
            }
        }
    }

    let _base = libc_base.ok_or_else(|| {
        Error::platform(0, "libc/libdl not found in target process")
    })?;

    // Get dlopen address from our process
    // This assumes same libc version - simplified approach
    let sym = CString::new("dlopen").unwrap();
    let addr = unsafe { libc::dlsym(libc::RTLD_DEFAULT, sym.as_ptr()) };

    if addr.is_null() {
        return Err(Error::platform(0, "dlopen not found"));
    }

    // Note: This only works if libc is at the same address in both processes
    // A proper implementation would parse ELF symbols
    Ok(addr as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_dlopen() {
        // This test requires root or CAP_SYS_PTRACE
        if let Some(proc) = Process::from_pid(1) {
            let result = find_dlopen_in_process(proc.pid() as PidT);
            println!("dlopen lookup result: {:?}", result);
        }
    }
}
