// Package injgo provides cross-platform library injection capabilities.
//
// This package offers a unified API for injecting dynamic libraries (DLLs, dylibs, shared objects)
// into running processes across Windows, macOS, and Linux.
//
// # Quick Start
//
//	// Find process by name
//	proc, err := injgo.FindProcessByName("target.exe")
//	if err != nil {
//	    log.Fatal(err)
//	}
//
//	// Inject library
//	err = proc.Inject("./payload.dll")
//	if err != nil {
//	    log.Fatal(err)
//	}
//
// # Platform Support
//
//   - Windows: Full support via CreateRemoteThread + LoadLibrary
//   - macOS: Partial (requires root, task_for_pid)
//   - Linux: Partial (requires CAP_SYS_PTRACE or root)
package injgo

import "errors"

// Common errors
var (
	// ErrProcessNotFound is returned when the target process cannot be found
	ErrProcessNotFound = errors.New("process not found")

	// ErrLibraryNotFound is returned when the library file doesn't exist
	ErrLibraryNotFound = errors.New("library not found")

	// ErrAlreadyInjected is returned when the library is already loaded
	ErrAlreadyInjected = errors.New("library already injected")

	// ErrPermissionDenied is returned when there are insufficient privileges
	ErrPermissionDenied = errors.New("permission denied")

	// ErrUnsupported is returned for unsupported operations on the current platform
	ErrUnsupported = errors.New("operation not supported on this platform")
)

// Process represents a running process that can be injected into
type Process struct {
	// PID is the process identifier
	PID int

	// Name is the process name
	Name string

	// ExePath is the full path to the executable (if available)
	ExePath string
}

// Injector defines the interface for library injection operations
type Injector interface {
	// IsInjected checks if a library is already loaded in the process
	IsInjected(lib string) (bool, error)

	// Inject loads a library into the process
	Inject(lib string) error

	// Eject unloads a library from the process
	Eject(lib string) error
}

// FindProcessByName finds a process by its name
// Returns the first matching process or ErrProcessNotFound
func FindProcessByName(name string) (*Process, error) {
	return findProcessByName(name)
}

// FindProcessByPID opens a process by its PID
func FindProcessByPID(pid int) (*Process, error) {
	return findProcessByPID(pid)
}

// FindAllProcessesByName finds all processes matching the given name
func FindAllProcessesByName(name string) ([]*Process, error) {
	return findAllProcessesByName(name)
}

// IsInjected checks if a library is already loaded in the process
func (p *Process) IsInjected(lib string) (bool, error) {
	return isInjected(p.PID, lib)
}

// Inject loads a library into the process
func (p *Process) Inject(lib string) error {
	return inject(p.PID, lib)
}

// Eject unloads a library from the process
func (p *Process) Eject(lib string) error {
	return eject(p.PID, lib)
}

// InjectByPID is a convenience function to inject by PID directly
func InjectByPID(pid int, lib string, replace bool) error {
	if !replace {
		injected, _ := isInjected(pid, lib)
		if injected {
			return ErrAlreadyInjected
		}
	}
	return inject(pid, lib)
}

// InjectByProcessName is a convenience function to inject by process name
func InjectByProcessName(name string, lib string, replace bool) error {
	proc, err := FindProcessByName(name)
	if err != nil {
		return err
	}
	return InjectByPID(proc.PID, lib, replace)
}
