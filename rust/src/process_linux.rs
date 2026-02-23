//! Linux process handling using procfs
//!
//! Provides process enumeration and information via /proc filesystem.

use std::fs;
use std::path::Path;

use crate::error::{Error, Result};

type PidT = i32;

/// Represents a Linux process
///
/// # Example
///
/// ```rust,no_run
/// use injx::Process;
///
/// // Find by name (may require CAP_SYS_PTRACE)
/// let proc = Process::find_by_name("target_app").unwrap();
/// println!("PID: {}", proc.pid());
///
/// // Open by PID
/// let proc = Process::from_pid(1234).unwrap();
/// ```
#[derive(Debug)]
pub struct Process {
    /// Process ID
    pid: PidT,
    /// Process name
    process_name: String,
}

impl Process {
    /// Open a process by PID
    ///
    /// # Arguments
    ///
    /// * `pid` - Process ID to open
    pub fn from_pid(pid: u32) -> Option<Self> {
        let proc_path = format!("/proc/{}", pid);
        if !Path::new(&proc_path).exists() {
            return None;
        }

        let name = Self::get_process_name_by_pid(pid as PidT)
            .unwrap_or_else(|| format!("pid:{}", pid));

        Some(Process {
            pid: pid as PidT,
            process_name: name,
        })
    }

    /// Find the first process matching the given name
    ///
    /// # Arguments
    ///
    /// * `name` - Process name to search for (partial match supported)
    pub fn find_by_name(name: &str) -> Option<Self> {
        Self::find_all_by_name(name).into_iter().next()
    }

    /// Alias for `find_by_name`
    pub fn find_first_by_name(name: &str) -> Option<Self> {
        Self::find_by_name(name)
    }

    /// Find all processes matching the given name
    pub fn find_all_by_name(name: &str) -> Vec<Self> {
        let entries = match fs::read_dir("/proc") {
            Ok(e) => e,
            Err(_) => return vec![],
        };

        entries
            .flatten()
            .filter_map(|entry| {
                let file_name = entry.file_name();
                let pid_str = file_name.to_string_lossy();
                let pid: PidT = pid_str.parse().ok()?;
                
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

    /// Get process name from /proc/[pid]/comm
    fn get_process_name_by_pid(pid: PidT) -> Option<String> {
        let comm_path = format!("/proc/{}/comm", pid);
        fs::read_to_string(&comm_path)
            .ok()
            .map(|s| s.trim().to_string())
    }

    /// Get process command line from /proc/[pid]/cmdline
    pub fn cmdline(&self) -> Option<String> {
        let cmdline_path = format!("/proc/{}/cmdline", self.pid);
        fs::read_to_string(&cmdline_path)
            .ok()
            .map(|s| s.replace('\0', " ").trim().to_string())
    }

    /// Get executable path from /proc/[pid]/exe
    pub fn exe_path(&self) -> Option<String> {
        let exe_path = format!("/proc/{}/exe", self.pid);
        fs::read_link(&exe_path)
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_string()))
    }

    /// List all loaded libraries from /proc/[pid]/maps
    pub fn loaded_libraries(&self) -> Vec<String> {
        let maps_path = format!("/proc/{}/maps", self.pid);
        let content = match fs::read_to_string(&maps_path) {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        let mut libs: Vec<String> = content
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 6 {
                    let path = parts[5];
                    if path.ends_with(".so") || path.contains(".so.") {
                        return Some(path.to_string());
                    }
                }
                None
            })
            .collect();

        libs.sort();
        libs.dedup();
        libs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_processes() {
        let procs = Process::find_all_by_name("");
        println!("Found {} processes", procs.len());
        assert!(!procs.is_empty());
    }

    #[test]
    fn test_process_info() {
        // Find init process (PID 1)
        if let Some(proc) = Process::from_pid(1) {
            println!("PID 1: {}", proc.name());
            println!("Exe: {:?}", proc.exe_path());
            println!("Libs: {:?}", proc.loaded_libraries());
        }
    }
}
