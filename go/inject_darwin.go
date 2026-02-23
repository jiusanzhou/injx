//go:build darwin
// +build darwin

package injgo

// macOS implementation using Mach APIs
// NOTE: Requires root privileges or taskgated entitlement

/*
#cgo LDFLAGS: -framework CoreFoundation

#include <sys/types.h>
#include <sys/sysctl.h>
#include <stdlib.h>
#include <string.h>

// Get process name by PID
static int get_proc_name(int pid, char *name, size_t size) {
    int mib[4] = {CTL_KERN, KERN_PROC, KERN_PROC_PID, pid};
    struct kinfo_proc info;
    size_t info_size = sizeof(info);

    if (sysctl(mib, 4, &info, &info_size, NULL, 0) < 0) {
        return -1;
    }

    strncpy(name, info.kp_proc.p_comm, size - 1);
    name[size - 1] = '\0';
    return 0;
}
*/
import "C"
import (
	"bufio"
	"fmt"
	"os/exec"
	"strconv"
	"strings"
	"unsafe"
)

// findProcessByName finds a process by name (macOS implementation)
func findProcessByName(name string) (*Process, error) {
	processes, err := findAllProcessesByName(name)
	if err != nil {
		return nil, err
	}
	if len(processes) == 0 {
		return nil, ErrProcessNotFound
	}
	return processes[0], nil
}

// findProcessByPID opens a process by PID (macOS implementation)
func findProcessByPID(pid int) (*Process, error) {
	name := C.CString("")
	defer C.free(unsafe.Pointer(name))

	cName := make([]byte, 256)
	result := C.get_proc_name(C.int(pid), (*C.char)(unsafe.Pointer(&cName[0])), C.size_t(len(cName)))
	if result < 0 {
		return nil, ErrProcessNotFound
	}

	procName := string(cName[:cStringLen(cName)])

	return &Process{
		PID:  pid,
		Name: procName,
	}, nil
}

// findAllProcessesByName finds all processes matching the name
func findAllProcessesByName(name string) ([]*Process, error) {
	cmd := exec.Command("ps", "-axo", "pid,comm")
	output, err := cmd.Output()
	if err != nil {
		return nil, err
	}

	var processes []*Process
	scanner := bufio.NewScanner(strings.NewReader(string(output)))

	// Skip header
	scanner.Scan()

	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		parts := strings.Fields(line)
		if len(parts) < 2 {
			continue
		}

		pid, err := strconv.Atoi(parts[0])
		if err != nil {
			continue
		}

		procName := parts[len(parts)-1]
		// Extract just the binary name
		if idx := strings.LastIndex(procName, "/"); idx >= 0 {
			procName = procName[idx+1:]
		}

		if name == "" || strings.Contains(procName, name) || procName == name {
			processes = append(processes, &Process{
				PID:  pid,
				Name: procName,
			})
		}
	}

	return processes, nil
}

// isInjected checks if a dylib is loaded in the process
func isInjected(pid int, lib string) (bool, error) {
	// Use vmmap to check loaded libraries
	cmd := exec.Command("vmmap", strconv.Itoa(pid))
	output, _ := cmd.Output()

	return strings.Contains(string(output), lib), nil
}

// inject loads a dylib into the target process
// NOTE: This is a placeholder - full implementation requires Mach APIs
func inject(pid int, lib string) error {
	// TODO: Implement using task_for_pid + thread_create_running
	// This requires:
	// 1. Get task port via task_for_pid (needs root)
	// 2. Allocate memory in target
	// 3. Write dylib path
	// 4. Create thread calling dlopen

	return fmt.Errorf("macOS injection not yet implemented in Go - use Rust CLI or: DYLD_INSERT_LIBRARIES=%s /path/to/app", lib)
}

// eject unloads a dylib from the target process
func eject(pid int, lib string) error {
	return ErrUnsupported
}

// cStringLen returns the length of a null-terminated C string
func cStringLen(b []byte) int {
	for i, c := range b {
		if c == 0 {
			return i
		}
	}
	return len(b)
}
