//! Windows process handling
//!
//! Provides process enumeration and manipulation using Windows APIs.

#![allow(non_snake_case)]
#![allow(unused_imports)]

use std::ffi::OsString;
use std::mem::{self, MaybeUninit};
use std::os::windows::ffi::OsStringExt;

use crate::error::{Error, Result};
use crate::inject::Injector;
use crate::winapi::*;

/// Windows process handle type
pub type ProcessHandle = HANDLE;

/// Represents a Windows process
///
/// # Example
///
/// ```rust,no_run
/// use injx::Process;
///
/// // Find by name
/// let proc = Process::find_by_name("notepad.exe").unwrap();
/// println!("PID: {}", proc.pid());
///
/// // Open by PID
/// let proc = Process::from_pid(1234).unwrap();
/// ```
#[derive(Debug)]
pub struct Process {
    pid: u32,
    process_name: String,
    handle: ProcessHandle,
}

impl Process {
    /// Create a new Process instance
    fn new(handle: ProcessHandle, pid: u32, name: &str) -> Self {
        Self {
            pid,
            process_name: name.into(),
            handle,
        }
    }

    /// Open a process by PID
    ///
    /// # Arguments
    ///
    /// * `pid` - Process ID to open
    ///
    /// # Returns
    ///
    /// * `Some(Process)` - Successfully opened process
    /// * `None` - Process not found or access denied
    pub fn from_pid(pid: u32) -> Option<Self> {
        let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, FALSE, pid) };
        if handle.is_null() {
            return None;
        }

        let name = get_process_name(handle);
        Some(Self::new(handle, pid, &name))
    }

    /// Find the first process matching the given name
    ///
    /// # Arguments
    ///
    /// * `name` - Process name to search for (partial match supported)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use injx::Process;
    ///
    /// let proc = Process::find_by_name("notepad").unwrap();
    /// ```
    pub fn find_by_name(name: &str) -> Option<Self> {
        Self::find_all_by_name(name).into_iter().next()
    }

    /// Alias for `find_by_name` - Find the first matching process
    pub fn find_first_by_name(name: &str) -> Option<Self> {
        Self::find_by_name(name)
    }

    /// Find all processes matching the given name
    ///
    /// # Arguments
    ///
    /// * `name` - Process name to search for (partial match supported)
    pub fn find_all_by_name(name: &str) -> Vec<Self> {
        find_processes_by_name(name).unwrap_or_default()
    }

    /// Get the process ID
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Get the process name
    pub fn name(&self) -> &str {
        &self.process_name
    }

    /// Get the raw process handle
    pub fn handle(&self) -> ProcessHandle {
        self.handle
    }

    /// Check if the process is running under WOW64 (32-bit on 64-bit Windows)
    pub fn is_wow64(&self) -> Result<bool> {
        let mut is_wow64 = MaybeUninit::uninit();
        let r = unsafe { IsWow64Process(self.handle, is_wow64.as_mut_ptr()) };
        if r == FALSE {
            return Err(Error::from_win32());
        }
        Ok(unsafe { is_wow64.assume_init() } == TRUE)
    }

    /// Close the process handle
    fn close(&self) -> Result<()> {
        if self.handle.is_null() {
            return Ok(());
        }
        let result = unsafe { CloseHandle(self.handle) };
        if result != 0 {
            return Ok(());
        }
        Err(Error::from_win32())
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

/// Get process name from handle
fn get_process_name(handle: ProcessHandle) -> String {
    let mut buf = [0u16; MAX_PATH + 1];
    unsafe {
        GetModuleBaseNameW(handle, 0 as _, buf.as_mut_ptr(), MAX_PATH as DWORD + 1);
        wchar_to_string(&buf)
    }
}

/// Get process executable path from handle
#[allow(dead_code)]
fn get_process_path(handle: ProcessHandle) -> String {
    let mut buf = [0u16; MAX_PATH + 1];
    unsafe {
        GetModuleFileNameExW(handle, 0 as _, buf.as_mut_ptr(), MAX_PATH as DWORD + 1);
        wchar_to_string(&buf)
    }
}

/// Find processes by name
fn find_processes_by_name(name: &str) -> Result<Vec<Process>> {
    let handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };

    if handle.is_null() || handle == INVALID_HANDLE_VALUE {
        return Err(Error::from_win32());
    }

    let mut result: Vec<Process> = Vec::new();
    let mut entry: PROCESSENTRY32 = unsafe { std::mem::zeroed() };
    entry.dwSize = mem::size_of::<PROCESSENTRY32>() as u32;

    while unsafe { Process32Next(handle, &mut entry) } != 0 {
        let exe_name = char_to_string(&entry.szExeFile);
        entry.szExeFile = unsafe { std::mem::zeroed() };

        if !name.is_empty() && !exe_name.to_lowercase().contains(&name.to_lowercase()) {
            continue;
        }

        if let Some(proc) = Process::from_pid(entry.th32ProcessID) {
            result.push(proc);
        }
    }

    unsafe { CloseHandle(handle) };

    Ok(result)
}

/// Convert C char array to String
fn char_to_string(chars: &[i8]) -> String {
    chars.iter().map(|c| *c as u8 as char).collect::<String>()
        .trim_end_matches('\0')
        .to_string()
}

/// Convert wide char array to String
fn wchar_to_string(slice: &[u16]) -> String {
    match slice.iter().position(|&x| x == 0) {
        Some(pos) => OsString::from_wide(&slice[..pos])
            .to_string_lossy()
            .into_owned(),
        None => OsString::from_wide(slice).to_string_lossy().into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_processes() {
        let processes = find_processes_by_name("");
        assert!(processes.is_ok());
        println!("Found {} processes", processes.unwrap().len());
    }
}
