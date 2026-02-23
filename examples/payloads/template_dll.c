// Windows DLL Template
// Build: cl /LD template_dll.c user32.lib
//    or: x86_64-w64-mingw32-gcc -shared -o payload.dll template_dll.c -luser32

#include <windows.h>

// Optional: Export functions for the target to call
__declspec(dllexport) int hello(void) {
    MessageBoxW(NULL, L"hello() called!", L"injx", MB_OK);
    return 42;
}

BOOL WINAPI DllMain(HINSTANCE hinstDLL, DWORD fdwReason, LPVOID lpvReserved) {
    switch (fdwReason) {
        case DLL_PROCESS_ATTACH:
            // Runs when DLL is loaded
            // Avoid heavy work here - spawn a thread if needed
            MessageBoxW(NULL, L"DLL Injected!", L"injx", MB_ICONINFORMATION);
            break;

        case DLL_THREAD_ATTACH:
            // Runs when a new thread is created in the process
            break;

        case DLL_THREAD_DETACH:
            // Runs when a thread exits
            break;

        case DLL_PROCESS_DETACH:
            // Runs when DLL is unloaded
            // Cleanup resources here
            break;
    }
    return TRUE;
}
