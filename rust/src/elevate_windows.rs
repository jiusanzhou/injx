//! Windows privilege elevation utilities

use std::ptr::null_mut;

use crate::error::{Error, Result};
use crate::winapi::*;

/// Elevate current process to debug privileges
///
/// This enables debugging other processes, which is required for injection.
///
/// # Example
///
/// ```rust,no_run
/// use injx::elevate_privileges;
///
/// elevate_privileges().expect("Failed to elevate privileges");
/// ```
pub fn elevate_privileges() -> Result<()> {
    unsafe {
        // Get current process token
        let mut token: HANDLE = null_mut();
        let result = OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        );

        if result == FALSE {
            return Err(Error::from_win32());
        }

        // Lookup SeDebugPrivilege LUID
        let mut luid: LUID = std::mem::zeroed();
        let result = LookupPrivilegeValueA(
            null_mut(),
            SE_DEBUG_NAME.as_ptr() as _,
            &mut luid,
        );

        if result == FALSE {
            CloseHandle(token);
            return Err(Error::from_win32());
        }

        // Build TOKEN_PRIVILEGES structure
        let mut privileges: TOKEN_PRIVILEGES = std::mem::zeroed();
        privileges.PrivilegeCount = 1;
        privileges.Privileges[0] = LUID_AND_ATTRIBUTES {
            Luid: luid,
            Attributes: SE_PRIVILEGE_ENABLED,
        };

        // Adjust token privileges
        let result = AdjustTokenPrivileges(
            token,
            FALSE,
            &mut privileges,
            0,
            null_mut(),
            null_mut(),
        );

        CloseHandle(token);

        if result == FALSE {
            return Err(Error::from_win32());
        }

        // Check if the privilege was actually enabled
        let last_error = std::io::Error::last_os_error();
        if last_error.raw_os_error() == Some(ERROR_NOT_ALL_ASSIGNED as i32) {
            return Err(Error::permission_denied(
                "SeDebugPrivilege not available - run as Administrator",
            ));
        }

        Ok(())
    }
}

// Additional constants
const ERROR_NOT_ALL_ASSIGNED: u32 = 1300;

// LUID structure
#[repr(C)]
#[derive(Copy, Clone, Default)]
struct LUID {
    LowPart: u32,
    HighPart: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elevate() {
        // This may fail without admin privileges, which is expected
        let result = elevate_privileges();
        println!("Elevation result: {:?}", result);
    }
}
