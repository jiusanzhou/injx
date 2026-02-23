<div align="center">

# 🔧 injx

**Cross-platform Dynamic Library Injection Toolkit**

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Go](https://img.shields.io/badge/Go-1.21+-00ADD8?style=flat-square&logo=go)](https://golang.org/)
[![License](https://img.shields.io/badge/License-Apache--2.0-blue?style=flat-square)](./LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey?style=flat-square)]()

[Features](#features) • [Installation](#installation) • [Usage](#usage) • [API](#api-reference) • [Examples](#examples) • [中文文档](./README_CN.md)

</div>

---

## Overview

**injx** is a cross-platform toolkit for injecting dynamic libraries into running processes. It supports Windows DLLs, macOS dylibs, and Linux shared objects (.so files).

Available as:
- 🖥️ **CLI Tool** - Command-line interface for quick injection
- 📦 **Library** - Rust and Go APIs for integration into your projects

## Features

- 🦀 **Rust Implementation** - Safe, fast, zero-dependency core
- 🐹 **Go Implementation** - Pure Go with Windows support
- 🖥️ **Cross-Platform** - Windows, macOS, and Linux support
- 📦 **Dual Interface** - CLI tool and library API
- 🔍 **Process Discovery** - Find processes by name or PID
- 💉 **Inject/Eject** - Load and unload libraries at runtime
- ✅ **Status Check** - Verify if a library is already injected

## Platform Support

| Platform | Architecture | Rust | Go | Technique |
|----------|-------------|------|-----|-----------|
| Windows  | x86         | ✅   | ✅  | CreateRemoteThread + LoadLibrary |
| Windows  | x64         | ✅   | ✅  | CreateRemoteThread + LoadLibrary |
| macOS    | ARM64       | ✅   | 🚧  | task_for_pid + thread_create_running |
| macOS    | x64         | ✅   | 🚧  | task_for_pid + thread_create_running |
| Linux    | x64         | ✅   | 🚧  | ptrace + dlopen |
| Linux    | ARM64       | ✅   | 🚧  | ptrace + dlopen |

**Legend:** ✅ Full support | 🚧 In development

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/user/injx.git
cd injx

# Build everything
make build

# Or build individually
make build-rust    # Rust CLI and library
make build-go      # Go CLI (Windows)
```

### Rust (Cargo)

```bash
# Install CLI
cargo install --path rust

# Or add as dependency
# Cargo.toml:
# [dependencies]
# injx = { git = "https://github.com/user/injx" }
```

### Go

```bash
go get go.zoe.im/injx
```

## Usage

### Command Line

```bash
# Basic injection
injx <PROCESS_NAME|PID> <LIBRARY>...

# Windows - Inject DLL into Notepad
injx notepad.exe payload.dll

# macOS - Inject dylib (requires root)
sudo injx Safari payload.dylib

# Linux - Inject .so (requires CAP_SYS_PTRACE or root)
sudo injx firefox payload.so

# Inject multiple libraries
injx 1234 lib1.dll lib2.dll lib3.dll

# Inject by PID
injx 12345 ./myhook.dll
```

### Rust Library

```rust
use injx::{Process, Injector};

fn main() -> injx::Result<()> {
    // Find process by name
    let process = Process::find_by_name("target.exe")
        .ok_or("Process not found")?;
    
    println!("Found: {} (PID: {})", process.name(), process.pid());

    // Check if already injected
    if process.is_injected("payload.dll")? {
        println!("Already injected!");
        return Ok(());
    }

    // Inject library
    process.inject("./payload.dll")?;
    println!("Injection successful!");

    // Eject library (Windows only currently)
    process.eject("payload.dll")?;

    Ok(())
}
```

### Go Library

```go
package main

import (
    "fmt"
    "log"

    "go.zoe.im/injgo"
)

func main() {
    // Find process by name
    proc, err := injgo.FindProcessByName("target.exe")
    if err != nil {
        log.Fatal(err)
    }

    fmt.Printf("Found: %s (PID: %d)\n", proc.Name, proc.PID)

    // Check if already injected
    injected, _ := proc.IsInjected("payload.dll")
    if injected {
        fmt.Println("Already injected!")
        return
    }

    // Inject library
    err = proc.Inject("./payload.dll")
    if err != nil {
        log.Fatal(err)
    }

    fmt.Println("Injection successful!")
}
```

## API Reference

### Rust API

#### `Process`

```rust
// Find process by name (returns first match)
Process::find_by_name(name: &str) -> Option<Process>

// Find all processes matching name
Process::find_all_by_name(name: &str) -> Vec<Process>

// Open process by PID
Process::from_pid(pid: u32) -> Option<Process>

// Get process info
process.pid() -> u32
process.name() -> &str
```

#### `Injector` Trait

```rust
// Check if library is loaded
process.is_injected(lib: &str) -> Result<bool>

// Inject library
process.inject(lib: &str) -> Result<()>

// Eject library
process.eject(lib: &str) -> Result<()>
```

### Go API

```go
// Find process by name
injgo.FindProcessByName(name string) (*Process, error)

// Find process by PID
injgo.FindProcessByPID(pid int) (*Process, error)

// Process methods
proc.IsInjected(lib string) (bool, error)
proc.Inject(lib string) error
proc.Eject(lib string) error

// Convenience functions
injgo.InjectByPID(pid int, lib string, replace bool) error
injgo.InjectByProcessName(name, lib string, replace bool) error
```

## Examples

### Example Payloads

The `examples/` directory contains ready-to-use payload examples:

| Example | Platform | Description |
|---------|----------|-------------|
| `messagebox.rs` | Windows | Shows a MessageBox when injected |
| `procinfo.rs` | Windows | Collects and saves process info |
| `keylogger_demo.rs` | Windows | Keyboard hook demo (educational) |
| `hello_dylib.rs` | macOS | Simple dylib with constructor |
| `hook_demo.rs` | macOS | Function hooking example |
| `hello_so.rs` | Linux | Simple .so with constructor |

### Building Examples

```bash
# Windows MessageBox DLL
cargo build --example messagebox --target x86_64-pc-windows-msvc --release

# macOS dylib
cargo build --example hello_dylib --release

# Linux .so
cargo build --example hello_so --release
```

### C/C++ Payloads

See `examples/payloads/` for C templates:
- `template_dll.c` - Windows DLL
- `template_dylib.c` - macOS dylib
- `template_so.c` - Linux .so

## Technical Details

### Windows (CreateRemoteThread + LoadLibrary)

1. Open target process with `PROCESS_ALL_ACCESS`
2. Allocate memory in target for DLL path (`VirtualAllocEx`)
3. Write DLL path to allocated memory (`WriteProcessMemory`)
4. Get `LoadLibraryW` address from kernel32.dll
5. Create remote thread calling `LoadLibraryW` (`CreateRemoteThread`)
6. Wait for thread completion

### macOS (task_for_pid + Mach APIs)

1. Get task port via `task_for_pid` (requires root)
2. Allocate memory in target (`mach_vm_allocate`)
3. Write dylib path and set up registers
4. Create thread via `thread_create_running` calling `dlopen`

### Linux (ptrace + dlopen)

1. Attach to target via `PTRACE_ATTACH`
2. Save original registers
3. Find `dlopen` address in target's libc
4. Write library path to process memory
5. Set up registers for dlopen call
6. Continue execution, restore registers
7. Detach from process

## Requirements

### Build Requirements

- **Rust**: 1.70+ with platform-specific targets
- **Go**: 1.21+

### Runtime Requirements

| Platform | Requirements |
|----------|-------------|
| Windows | Same architecture as target (x86→x86, x64→x64) |
| macOS | Root privileges or taskgated entitlement |
| Linux | CAP_SYS_PTRACE capability or root |

## Development

```bash
# Format code
make fmt

# Run lints
make lint

# Run tests
make test

# Clean build artifacts
make clean
```

## Security Notice

⚠️ **This tool is for educational and authorized security testing only.**

- Only use on processes you own or have explicit permission to test
- Be aware of legal implications in your jurisdiction
- Some antivirus software may flag injection tools

## Project Structure

```
injx/
├── rust/                   # Rust implementation
│   ├── src/
│   │   ├── lib.rs          # Library entry point
│   │   ├── main.rs         # CLI entry point
│   │   ├── error.rs        # Error types
│   │   ├── inject.rs       # Injector trait
│   │   ├── process_*.rs    # Platform process handling
│   │   └── inject_*.rs     # Platform injection logic
│   └── Cargo.toml
├── go/                     # Go implementation
│   ├── inject.go           # Core types and interface
│   ├── inject_windows.go   # Windows implementation
│   ├── inject_darwin.go    # macOS implementation
│   ├── inject_linux.go     # Linux implementation
│   └── cmd/injgo/          # CLI entry point
├── examples/               # Example payloads
│   ├── windows/
│   ├── macos/
│   ├── linux/
│   └── payloads/           # C templates
├── Makefile
└── README.md
```

## License

[Apache-2.0](./LICENSE)

## Credits

- Originally derived from [injgo](https://github.com/jiusanzhou/injgo) and [injrs](https://github.com/jiusanzhou/injrs)
- Author: [Zoe](https://zoe.im)

---

<div align="center">
Made with ❤️ by <a href="https://zoe.im">Zoe</a>
</div>
