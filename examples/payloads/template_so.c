// Linux Shared Object Template
// Build: gcc -shared -fPIC -o payload.so template_so.c

#include <stdio.h>
#include <unistd.h>
#include <dlfcn.h>

// Constructor - runs when .so is loaded
__attribute__((constructor))
void on_load(void) {
    printf("[injx] Shared object loaded into PID %d\n", getpid());
    
    // Your initialization code here
    // Examples:
    // - Hook functions using LD_PRELOAD technique
    // - Set up a monitoring thread
    // - Modify process behavior
}

// Destructor - runs when .so is unloaded
__attribute__((destructor))
void on_unload(void) {
    printf("[injx] Shared object unloaded from PID %d\n", getpid());
    
    // Cleanup code here
}

// Public function that can be called via dlsym
__attribute__((visibility("default")))
int injx_hello(void) {
    printf("[injx] injx_hello() called!\n");
    return 42;
}

// Example: Hook a libc function (works with LD_PRELOAD)
// Uncomment to intercept open() calls
/*
#include <stdarg.h>
#include <fcntl.h>

typedef int (*orig_open_t)(const char *pathname, int flags, ...);

int open(const char *pathname, int flags, ...) {
    printf("[injx] open() intercepted: %s\n", pathname);
    
    orig_open_t orig_open = dlsym(RTLD_NEXT, "open");
    
    if (flags & O_CREAT) {
        va_list args;
        va_start(args, flags);
        mode_t mode = va_arg(args, mode_t);
        va_end(args);
        return orig_open(pathname, flags, mode);
    }
    return orig_open(pathname, flags);
}
*/
