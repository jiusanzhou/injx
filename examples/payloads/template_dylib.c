// macOS dylib Template
// Build: clang -dynamiclib -o payload.dylib template_dylib.c

#include <stdio.h>
#include <unistd.h>
#include <dlfcn.h>

// Constructor - runs when dylib is loaded
__attribute__((constructor))
void on_load(void) {
    printf("[injx] Dylib loaded into PID %d\n", getpid());
    
    // Your initialization code here
    // Examples:
    // - Set up hooks
    // - Start a background thread
    // - Modify process behavior
}

// Destructor - runs when dylib is unloaded
__attribute__((destructor))
void on_unload(void) {
    printf("[injx] Dylib unloaded from PID %d\n", getpid());
    
    // Cleanup code here
}

// Public function that can be called via dlsym
__attribute__((visibility("default")))
int injx_hello(void) {
    printf("[injx] injx_hello() called!\n");
    return 42;
}
