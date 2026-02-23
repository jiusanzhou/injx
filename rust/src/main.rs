//! injx CLI - Cross-platform dynamic library injection tool
//!
//! Usage:
//!   injx <PROCESS_NAME|PID> <LIBRARY>...

use std::env;
use std::process::exit;

use injx::{Injector, Process};

#[cfg(target_os = "windows")]
use injx::elevate_privileges;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const USAGE: &str = r#"
USAGE:
    injx <PROCESS_NAME|PID> <LIBRARY>...

EXAMPLES:
    # Windows - Inject DLL into Notepad
    injx notepad.exe payload.dll

    # macOS - Inject dylib into Safari (requires root)
    sudo injx Safari payload.dylib

    # Linux - Inject .so into target process
    sudo injx target_app payload.so

    # Inject multiple libraries by PID
    injx 1234 lib1.dll lib2.dll

NOTES:
    - Windows: Requires same architecture (x86/x64) as target
    - macOS:   Requires root or taskgated entitlement
    - Linux:   Requires CAP_SYS_PTRACE or root
"#;

fn main() {
    println!("injx v{} - Cross-platform library injection", VERSION);

    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("{}", USAGE);
        exit(1);
    }

    let target = &args[1];
    let libraries: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();

    if libraries.is_empty() {
        eprintln!("Error: At least one library path required");
        exit(1);
    }

    // Elevate privileges on Windows
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = elevate_privileges() {
            eprintln!("Warning: Failed to elevate privileges: {}", e);
        }
    }

    // macOS root check
    #[cfg(target_os = "macos")]
    {
        if unsafe { libc::geteuid() } != 0 {
            eprintln!("Warning: Not running as root - task_for_pid may fail");
            eprintln!("         Try: sudo injx {} ...", target);
        }
    }

    // Linux ptrace check
    #[cfg(target_os = "linux")]
    {
        if unsafe { libc::geteuid() } != 0 {
            eprintln!("Warning: Not running as root - ptrace may fail");
            eprintln!("         Try: sudo injx {} ...", target);
            eprintln!("         Or: sudo setcap cap_sys_ptrace=ep $(which injx)");
        }
    }

    // Find or open process
    let process = match target.parse::<u32>() {
        Ok(pid) => {
            println!("Target: PID {}", pid);
            Process::from_pid(pid)
        }
        Err(_) => {
            println!("Target: {}", target);
            Process::find_by_name(target)
        }
    };

    let process = match process {
        Some(p) => {
            println!("Found: {} (PID: {})", p.name(), p.pid());
            p
        }
        None => {
            eprintln!("Error: Process not found: {}", target);
            exit(1);
        }
    };

    // Inject each library
    let mut success_count = 0;
    let mut failed_count = 0;

    println!();
    for lib in &libraries {
        print!("  {} ... ", lib);

        // Check if already injected
        match process.is_injected(lib) {
            Ok(true) => {
                println!("⚠ already injected");
                success_count += 1;
                continue;
            }
            Ok(false) => {}
            Err(e) => {
                eprintln!("⚠ check failed: {}", e);
            }
        }

        // Inject
        match process.inject(lib) {
            Ok(_) => {
                println!("✓ success");
                success_count += 1;
            }
            Err(e) => {
                println!("✗ failed: {}", e);
                failed_count += 1;
            }
        }
    }

    println!();
    println!("Results: {} success, {} failed", success_count, failed_count);

    if failed_count > 0 {
        exit(1);
    }
}
