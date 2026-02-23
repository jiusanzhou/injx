# Examples

This directory contains example payloads for different platforms.

## Directory Structure

```
examples/
├── windows/           # Windows DLL examples (Rust)
│   ├── messagebox.rs  # Simple MessageBox demo
│   ├── procinfo.rs    # Process info collector
│   └── keylogger_demo.rs # Keyboard hook demo (educational)
├── macos/             # macOS dylib examples (Rust)
│   ├── hello_dylib.rs # Simple dylib
│   └── hook_demo.rs   # Function hooking demo
├── linux/             # Linux .so examples (Rust)
│   └── hello_so.rs    # Simple shared object
└── payloads/          # C templates
    ├── template_dll.c  # Windows DLL template
    ├── template_dylib.c # macOS dylib template
    └── template_so.c   # Linux .so template
```

## Building Examples

### Rust Examples

```bash
# Windows MessageBox DLL
cargo build --example messagebox --target x86_64-pc-windows-msvc --release

# macOS dylib
cargo build --example hello_dylib --release

# Linux .so
cargo build --example hello_so --release
```

### C Templates

```bash
# Windows
cl /LD template_dll.c user32.lib
# or with MinGW
x86_64-w64-mingw32-gcc -shared -o payload.dll template_dll.c -luser32

# macOS
clang -dynamiclib -o payload.dylib template_dylib.c

# Linux
gcc -shared -fPIC -o payload.so template_so.c
```

## Usage

```bash
# Inject into running process
injx notepad.exe ./payload.dll           # Windows
sudo injx Safari ./payload.dylib         # macOS
sudo injx target_app ./payload.so        # Linux

# Alternative: Environment variables
DYLD_INSERT_LIBRARIES=./payload.dylib /path/to/app  # macOS
LD_PRELOAD=./payload.so /path/to/app                # Linux
```

## Security Notice

⚠️ These examples are for **educational purposes only**.

- Do not inject into processes you don't own
- Keyboard logging code is for demonstration only
- Always obtain proper authorization before testing
- Be aware of legal implications in your jurisdiction
