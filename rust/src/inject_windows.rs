//! Windows DLL injection implementation
//!
//! Uses CreateRemoteThread + LoadLibraryW technique.

use std::mem;
use std::path::Path;
use std::ptr::null_mut;

use crate::error::{Error, Result};
use crate::inject::Injector;
use crate::process_windows::Process;
use crate::utils::to_wide_string;
use crate::winapi::*;

impl Injector for Process {
    fn is_injected(&self, dll: &str) -> Result<bool> {
        // Get module name from path
        let dll_name = Path::new(dll)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(dll)
            .to_lowercase();

        // Enumerate modules
        let handle = unsafe { 
            CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, self.pid()) 
        };
        
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            // Access denied or process not accessible
            return Ok(false);
        }

        let mut entry: MODULEENTRY32 = unsafe { std::mem::zeroed() };
        entry.dwSize = mem::size_of::<MODULEENTRY32>() as u32;

        let mut found = false;
        
        if unsafe { Module32First(handle, &mut entry) } != 0 {
            loop {
                let module_name = char_to_string(&entry.szModule).to_lowercase();
                if module_name.contains(&dll_name) || dll_name.contains(&module_name) {
                    found = true;
                    break;
                }
                
                if unsafe { Module32Next(handle, &mut entry) } == 0 {
                    break;
                }
            }
        }

        unsafe { CloseHandle(handle) };
        Ok(found)
    }

    fn inject(&self, dll: &str) -> Result<()> {
        // Resolve full path
        let fullpath = Path::new(dll)
            .canonicalize()
            .map_err(|e| Error::library_not_found(format!("{}: {}", dll, e)))?;
        
        let dll_path = fullpath
            .to_str()
            .ok_or_else(|| Error::invalid_argument("invalid path encoding"))?;

        // Convert to wide string
        let path_wstr = to_wide_string(dll_path);
        let path_len = path_wstr.len() * 2;

        // Allocate memory in target process
        let remote_path = unsafe {
            VirtualAllocEx(
                self.handle(),
                null_mut(),
                path_len,
                MEM_RESERVE | MEM_COMMIT,
                PAGE_READWRITE,
            )
        };

        if remote_path.is_null() {
            return Err(Error::platform(
                std::io::Error::last_os_error().raw_os_error().unwrap_or(0),
                "failed to allocate memory in target process",
            ));
        }

        // Write DLL path to target process
        let written = unsafe {
            WriteProcessMemory(
                self.handle(),
                remote_path,
                path_wstr.as_ptr() as _,
                path_len,
                null_mut(),
            )
        };

        if written == FALSE {
            unsafe { VirtualFreeEx(self.handle(), remote_path, 0, MEM_RELEASE) };
            return Err(Error::platform(
                std::io::Error::last_os_error().raw_os_error().unwrap_or(0),
                "failed to write path to target memory",
            ));
        }

        // Get LoadLibraryW address
        let load_library = unsafe {
            GetProcAddress(
                GetModuleHandleA("kernel32.dll\0".as_ptr() as _),
                "LoadLibraryW\0".as_ptr() as _,
            )
        };

        if load_library.is_null() {
            unsafe { VirtualFreeEx(self.handle(), remote_path, 0, MEM_RELEASE) };
            return Err(Error::platform(0, "failed to get LoadLibraryW address"));
        }

        // Create remote thread to call LoadLibraryW
        let thread = unsafe {
            CreateRemoteThread(
                self.handle(),
                null_mut(),
                0,
                Some(mem::transmute(load_library)),
                remote_path,
                0,
                null_mut(),
            )
        };

        if thread.is_null() {
            unsafe { VirtualFreeEx(self.handle(), remote_path, 0, MEM_RELEASE) };
            return Err(Error::from_win32());
        }

        // Wait for thread completion
        let wait_result = unsafe { WaitForSingleObject(thread, 5000) }; // 5 second timeout
        
        if wait_result == WAIT_FAILED {
            unsafe {
                CloseHandle(thread);
                VirtualFreeEx(self.handle(), remote_path, 0, MEM_RELEASE);
            }
            return Err(Error::from_win32());
        }

        // Get thread exit code (LoadLibrary return value)
        let mut exit_code: u32 = 0;
        unsafe {
            GetExitCodeThread(thread, &mut exit_code as *mut _ as _);
            CloseHandle(thread);
            VirtualFreeEx(self.handle(), remote_path, 0, MEM_RELEASE);
        }

        // exit_code == 0 means LoadLibrary failed
        if exit_code == 0 {
            return Err(Error::platform(0, "LoadLibrary returned NULL - DLL load failed"));
        }

        Ok(())
    }

    fn eject(&self, dll: &str) -> Result<()> {
        // Get module name from path
        let dll_name = Path::new(dll)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(dll)
            .to_lowercase();

        // Find module handle
        let handle = unsafe { 
            CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, self.pid()) 
        };
        
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            return Err(Error::from_win32());
        }

        let mut entry: MODULEENTRY32 = unsafe { std::mem::zeroed() };
        entry.dwSize = mem::size_of::<MODULEENTRY32>() as u32;

        let mut module_handle: Option<HMODULE> = None;
        
        if unsafe { Module32First(handle, &mut entry) } != 0 {
            loop {
                let module_name = char_to_string(&entry.szModule).to_lowercase();
                if module_name.contains(&dll_name) || dll_name.contains(&module_name) {
                    module_handle = Some(entry.hModule);
                    break;
                }
                
                if unsafe { Module32Next(handle, &mut entry) } == 0 {
                    break;
                }
            }
        }

        unsafe { CloseHandle(handle) };

        let hmodule = module_handle
            .ok_or_else(|| Error::library_not_found(format!("{} not loaded", dll)))?;

        // Get FreeLibrary address
        let free_library = unsafe {
            GetProcAddress(
                GetModuleHandleA("kernel32.dll\0".as_ptr() as _),
                "FreeLibrary\0".as_ptr() as _,
            )
        };

        if free_library.is_null() {
            return Err(Error::platform(0, "failed to get FreeLibrary address"));
        }

        // Create remote thread to call FreeLibrary
        let thread = unsafe {
            CreateRemoteThread(
                self.handle(),
                null_mut(),
                0,
                Some(mem::transmute(free_library)),
                hmodule as _,
                0,
                null_mut(),
            )
        };

        if thread.is_null() {
            return Err(Error::from_win32());
        }

        // Wait for completion
        unsafe {
            WaitForSingleObject(thread, 5000);
            CloseHandle(thread);
        }

        Ok(())
    }
}

/// Convert C char array to String
fn char_to_string(chars: &[i8]) -> String {
    chars.iter()
        .map(|c| *c as u8 as char)
        .collect::<String>()
        .trim_end_matches('\0')
        .to_string()
}

// Additional Windows-specific constants
const TH32CS_SNAPMODULE: u32 = 0x00000008;
const TH32CS_SNAPMODULE32: u32 = 0x00000010;

// Module32First/Next bindings
#[link(name = "kernel32")]
extern "system" {
    fn Module32First(hSnapshot: HANDLE, lpme: *mut MODULEENTRY32) -> i32;
    fn Module32Next(hSnapshot: HANDLE, lpme: *mut MODULEENTRY32) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_api() {
        // This test requires a running process and DLL
        // Just verify the API compiles
        println!("Inject API test - compile check only");
    }
}
