//! Windows API bindings
//!
//! Re-exports commonly used Windows API types and functions.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

pub use winapi::shared::minwindef::{
    DWORD, FALSE, HMODULE, MAX_PATH, TRUE,
};

pub use winapi::shared::ntdef::{HANDLE, NULL};

pub use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};

pub use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};

pub use winapi::um::memoryapi::{
    ReadProcessMemory, VirtualAllocEx, VirtualFreeEx, WriteProcessMemory,
};

pub use winapi::um::processthreadsapi::{
    CreateRemoteThread, GetCurrentProcess, GetExitCodeThread, GetProcessId,
    OpenProcess, OpenProcessToken,
};

pub use winapi::um::psapi::{GetModuleBaseNameW, GetModuleFileNameExW};

pub use winapi::um::securitybaseapi::AdjustTokenPrivileges;

pub use winapi::um::synchapi::WaitForSingleObject;

pub use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Process32Next, MODULEENTRY32, PROCESSENTRY32,
    TH32CS_SNAPPROCESS,
};

pub use winapi::um::winbase::{
    LookupPrivilegeValueA, CREATE_NEW_CONSOLE, CREATE_SUSPENDED, INFINITE,
    WAIT_FAILED,
};

pub use winapi::um::winnt::{
    LUID_AND_ATTRIBUTES, MEM_COMMIT, MEM_DECOMMIT, MEM_RELEASE, MEM_RESERVE,
    PAGE_EXECUTE_READWRITE, PAGE_READWRITE, PROCESS_ALL_ACCESS,
    SE_DEBUG_NAME, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES,
    TOKEN_PRIVILEGES, TOKEN_QUERY,
};

pub use winapi::um::wow64apiset::IsWow64Process;
