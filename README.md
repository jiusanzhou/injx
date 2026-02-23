<div align="center">

# `jinx`

**Cross-language DLL/dylib injection toolkit**

[![Go](https://img.shields.io/badge/Go-00ADD8?style=flat&logo=go&logoColor=white)](./go)
[![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)](./rust)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](./LICENSE)

</div>

## Overview

**Jinx** is a cross-language dynamic library injection toolkit. It provides both CLI tools and libraries for injecting DLLs (Windows) / dylibs (macOS) / shared objects (Linux) into running processes.

## Features

- 🦀 **Rust implementation** - Safe, fast, with example DLLs
- 🐹 **Go implementation** - Pure Go, zero dependencies
- 🖥️ **Cross-platform** - Windows, macOS, Linux (WIP)
- 📦 **CLI & Library** - Use as tool or integrate into your project

## Project Structure

```
jinx/
├── go/           # Go implementation (from injgo)
├── rust/         # Rust implementation (from injrs)
├── examples/     # Example DLLs for testing
├── scripts/      # Build & release scripts
└── README.md
```

## Quick Start

### Rust

```bash
cd rust
cargo build --release
./target/release/jinx <PROCESS_NAME|PID> <DLL_PATH>
```

### Go

```bash
cd go
go build -o jinx ./cmd/injgo
./jinx <PROCESS_NAME|PID> <DLL_PATH>
```

## Platform Support

| Platform | Rust | Go | Status |
|----------|------|-----|--------|
| Windows x86 | ✅ | ✅ | Stable |
| Windows x64 | ✅ | ⚠️ | Partial |
| macOS | 🚧 | 🚧 | WIP |
| Linux | 🚧 | 🚧 | WIP |

## Usage Examples

### Inject DLL to Calculator

```bash
# Using Rust
jinx Calc.exe test.dll

# Using Go
jinx Calculator.exe test.dll
```

### As Library (Rust)

```rust
use jinx::process::Process;
use jinx::inject::InjectorExt;

let process = Process::find_first_by_name("target.exe").unwrap();
process.inject("./payload.dll").unwrap();
```

### As Library (Go)

```go
import "go.zoe.im/jinx"

err := jinx.InjectByProcessName("target.exe", "./payload.dll", false)
```

## Building

### Prerequisites

- Rust 1.70+ (for Rust implementation)
- Go 1.21+ (for Go implementation)

### Build All

```bash
./scripts/build.sh
```

## License

Apache-2.0

## Credits

- Originally from [injgo](https://github.com/jiusanzhou/injgo) and [injrs](https://github.com/jiusanzhou/injrs)
- Author: [Zoe](https://zoe.im)
