# Payload Templates

This directory contains template payloads for different platforms.

## Structure

```
payloads/
├── template_dll.c      # Windows DLL template (C)
├── template_dylib.c    # macOS dylib template (C)
├── template_so.c       # Linux .so template (C)
└── README.md
```

## Building Payloads

### Windows DLL

```c
// template_dll.c
#include <windows.h>

BOOL WINAPI DllMain(HINSTANCE hinstDLL, DWORD fdwReason, LPVOID lpvReserved) {
    switch (fdwReason) {
        case DLL_PROCESS_ATTACH:
            // Initialization code
            MessageBoxW(NULL, L"Injected!", L"injx", MB_OK);
            break;
        case DLL_PROCESS_DETACH:
            // Cleanup code
            break;
    }
    return TRUE;
}
```

Build:
```bash
# MSVC
cl /LD template_dll.c user32.lib

# MinGW
x86_64-w64-mingw32-gcc -shared -o payload.dll template_dll.c
```

### macOS dylib

```c
// template_dylib.c
#include <stdio.h>

__attribute__((constructor))
void on_load(void) {
    printf("[injx] Dylib loaded!\n");
}

__attribute__((destructor))
void on_unload(void) {
    printf("[injx] Dylib unloaded!\n");
}
```

Build:
```bash
clang -dynamiclib -o payload.dylib template_dylib.c
```

### Linux .so

```c
// template_so.c
#include <stdio.h>

__attribute__((constructor))
void on_load(void) {
    printf("[injx] Shared object loaded!\n");
}

__attribute__((destructor))
void on_unload(void) {
    printf("[injx] Shared object unloaded!\n");
}
```

Build:
```bash
gcc -shared -fPIC -o payload.so template_so.c
```

## Rust Payloads

See the `windows/`, `macos/`, and `linux/` directories for Rust examples.

Build Rust payloads:
```bash
# Windows DLL
cargo build --example messagebox --target x86_64-pc-windows-msvc --release

# macOS dylib
cargo build --example hello_dylib --release

# Linux .so
cargo build --example hello_so --release
```
