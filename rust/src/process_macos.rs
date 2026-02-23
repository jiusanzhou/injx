//! macOS process handling using Mach APIs
//!
//! Provides process enumeration and task port acquisition.

use std::ffi::CString;
use std::process::Command;

use crate::error::{Error, Result};

// Mach types
type MachPortT = u32;
type KernReturnT = i32;
type TaskT = MachPortT;
type PidT = i32;

const KERN_SUCCESS: KernReturnT = 0;

// External Mach functions
#[link(name = "System", kind = "framework")]
extern "C" {
    fn task_for_pid(target_tport: MachPortT, pid: PidT, task: *mut TaskT) -> KernReturnT;
    fn mach_task_self() -> MachPortT;
}

/// Represents a macOS process with its Mach task port
///
/// # Example
///
/// ```rust,no_run
/// use injx::Process;
///
/// // Find by name (requires root)
/// let proc = Process::find_by_name("Safari").unwrap();
/// println!("PID: {}", proc.pid());
///
/// // Open by PID
/// let proc = Process::from_pid(1234).unwrap();
/// ```
#[derive(Debug)]
pub struct Process {
    /// Process ID
    pid: PidT,
    /// Mach task port
    task: TaskT,
    /// Process name
    process_name: String,
}

impl Process {
    /// Open a process by PID and acquire its task port
    ///
    /// **Note**: Requires root privileges or taskgated entitlement.
    ///
    /// # Arguments
    ///
    /// * `pid` - Process ID to open
    pub fn from_pid(pid: u32) -> Option<Self> {
        let mut task: TaskT = 0;
        let kr = unsafe { task_for_pid(mach_task_self(), pid as PidT, &mut task) };

        if kr != KERN_SUCCESS {
            eprintln!(
                "task_for_pid failed ({}): requires root or taskgated entitlement",
                kr
            );
            return None;
        }

        let name = Self::get_process_name_by_pid(pid as PidT)
            .unwrap_or_else(|| format!("pid:{}", pid));

        Some(Process {
            pid: pid as PidT,
            task,
            process_name: name,
        })
    }

    /// Find the first process matching the given name
    ///
    /// # Arguments
    ///
    /// * `name` - Process name to search for (partial match supported)
    pub fn find_by_name(name: &str) -> Option<Self> {
        let pids = Self::list_pids()?;

        for pid in pids {
            if let Some(proc_name) = Self::get_process_name_by_pid(pid) {
                if proc_name.contains(name) || proc_name == name {
                    return Self::from_pid(pid as u32);
                }
            }
        }

        None
    }

    /// Alias for `find_by_name`
    pub fn find_first_by_name(name: &str) -> Option<Self> {
        Self::find_by_name(name)
    }

    /// Find all processes matching the given name
    pub fn find_all_by_name(name: &str) -> Vec<Self> {
        let pids = match Self::list_pids() {
            Some(p) => p,
            None => return vec![],
        };

        pids.into_iter()
            .filter_map(|pid| {
                let proc_name = Self::get_process_name_by_pid(pid)?;
                if proc_name.contains(name) || proc_name == name {
                    Self::from_pid(pid as u32)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the process ID
    pub fn pid(&self) -> u32 {
        self.pid as u32
    }

    /// Get the process name
    pub fn name(&self) -> &str {
        &self.process_name
    }

    /// Get the Mach task port
    pub fn task(&self) -> TaskT {
        self.task
    }

    /// List all process IDs on the system
    fn list_pids() -> Option<Vec<PidT>> {
        let output = Command::new("ps")
            .args(["-axo", "pid"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let pids: Vec<PidT> = stdout
            .lines()
            .skip(1) // Skip header
            .filter_map(|line| line.trim().parse().ok())
            .collect();

        Some(pids)
    }

    /// Get process name by PID using ps command
    fn get_process_name_by_pid(pid: PidT) -> Option<String> {
        let output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "comm="])
            .output()
            .ok()?;

        let name = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();

        if name.is_empty() {
            None
        } else {
            // Extract just the binary name from path
            Some(
                name.rsplit('/')
                    .next()
                    .unwrap_or(&name)
                    .to_string()
            )
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        // Task ports should ideally be deallocated via mach_port_deallocate,
        // but this can cause issues if other code still holds references.
        // For safety, we don't deallocate here.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_pids() {
        let pids = Process::list_pids();
        assert!(pids.is_some());
        println!("Found {} processes", pids.unwrap().len());
    }

    #[test]
    fn test_find_process() {
        // This should find launchd which always runs
        let proc = Process::find_by_name("launchd");
        // May fail without root
        if let Some(p) = proc {
            println!("Found: {} (PID: {})", p.name(), p.pid());
        }
    }
}
