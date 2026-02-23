//go:build windows
// +build windows

package injgo

import (
	"errors"
	"path/filepath"
	"syscall"
	"unsafe"

	"go.zoe.im/injgo/pkg/w32"
)

// Platform-specific errors
var (
	ErrCreateSnapshot = errors.New("failed to create process snapshot")
	ErrModuleSnapshot = errors.New("failed to create module snapshot")
	ErrModuleNotFound = errors.New("module not found in process")
)

// findProcessByName finds a process by name (Windows implementation)
func findProcessByName(name string) (*Process, error) {
	handle, err := syscall.CreateToolhelp32Snapshot(syscall.TH32CS_SNAPPROCESS, 0)
	if err != nil {
		return nil, ErrCreateSnapshot
	}
	defer syscall.CloseHandle(handle)

	var entry syscall.ProcessEntry32
	entry.Size = uint32(unsafe.Sizeof(entry))

	err = syscall.Process32First(handle, &entry)
	if err != nil {
		return nil, ErrProcessNotFound
	}

	for {
		exeFile := w32.UTF16PtrToString(&entry.ExeFile[0])
		if exeFile == name || filepath.Base(exeFile) == name {
			return &Process{
				PID:     int(entry.ProcessID),
				Name:    exeFile,
				ExePath: exeFile, // TODO: get full path
			}, nil
		}

		err = syscall.Process32Next(handle, &entry)
		if err != nil {
			break
		}
	}

	return nil, ErrProcessNotFound
}

// findProcessByPID opens a process by PID (Windows implementation)
func findProcessByPID(pid int) (*Process, error) {
	handle, err := syscall.OpenProcess(syscall.PROCESS_QUERY_INFORMATION, false, uint32(pid))
	if err != nil {
		return nil, ErrProcessNotFound
	}
	syscall.CloseHandle(handle)

	// Get process name
	name := ""
	snapshot, _ := syscall.CreateToolhelp32Snapshot(syscall.TH32CS_SNAPPROCESS, 0)
	if snapshot != 0 {
		defer syscall.CloseHandle(snapshot)
		var entry syscall.ProcessEntry32
		entry.Size = uint32(unsafe.Sizeof(entry))
		syscall.Process32First(snapshot, &entry)
		for {
			if int(entry.ProcessID) == pid {
				name = w32.UTF16PtrToString(&entry.ExeFile[0])
				break
			}
			if syscall.Process32Next(snapshot, &entry) != nil {
				break
			}
		}
	}

	return &Process{
		PID:  pid,
		Name: name,
	}, nil
}

// findAllProcessesByName finds all processes matching the name
func findAllProcessesByName(name string) ([]*Process, error) {
	handle, err := syscall.CreateToolhelp32Snapshot(syscall.TH32CS_SNAPPROCESS, 0)
	if err != nil {
		return nil, ErrCreateSnapshot
	}
	defer syscall.CloseHandle(handle)

	var processes []*Process
	var entry syscall.ProcessEntry32
	entry.Size = uint32(unsafe.Sizeof(entry))

	err = syscall.Process32First(handle, &entry)
	if err != nil {
		return processes, nil
	}

	for {
		exeFile := w32.UTF16PtrToString(&entry.ExeFile[0])
		if name == "" || exeFile == name || filepath.Base(exeFile) == name {
			processes = append(processes, &Process{
				PID:     int(entry.ProcessID),
				Name:    exeFile,
				ExePath: exeFile,
			})
		}

		err = syscall.Process32Next(handle, &entry)
		if err != nil {
			break
		}
	}

	return processes, nil
}

// isInjected checks if a DLL is loaded in the process
func isInjected(pid int, lib string) (bool, error) {
	dllName := filepath.Base(lib)

	handle := w32.CreateToolhelp32Snapshot(w32.TH32CS_SNAPMODULE|w32.TH32CS_SNAPMODULE32, uint32(pid))
	if handle == 0 {
		return false, nil // Can't check, assume not injected
	}
	defer w32.CloseHandle(handle)

	var entry w32.MODULEENTRY32
	entry.Size = uint32(unsafe.Sizeof(entry))

	if !w32.Module32First(handle, &entry) {
		return false, nil
	}

	for {
		moduleName := w32.UTF16PtrToString(&entry.SzModule[0])
		if moduleName == dllName {
			return true, nil
		}
		if !w32.Module32Next(handle, &entry) {
			break
		}
	}

	return false, nil
}

// inject loads a DLL into the target process
func inject(pid int, lib string) error {
	// Get absolute path
	absPath, err := filepath.Abs(lib)
	if err != nil {
		return ErrLibraryNotFound
	}

	// Open process
	handle, err := w32.OpenProcess(w32.PROCESS_ALL_ACCESS, false, uint32(pid))
	if err != nil {
		return err
	}
	defer w32.CloseHandle(handle)

	// Allocate memory for DLL path
	pathLen := len(absPath) + 1
	remoteAddr, err := w32.VirtualAllocEx(handle, 0, pathLen, w32.MEM_COMMIT, w32.PAGE_EXECUTE_READWRITE)
	if err != nil {
		return err
	}

	// Write DLL path
	err = w32.WriteProcessMemory(handle, uint32(remoteAddr), []byte(absPath), uint(pathLen))
	if err != nil {
		return err
	}

	// Get LoadLibraryA address
	kernel32 := w32.GetModuleHandleA("kernel32.dll")
	loadLibAddr, err := w32.GetProcAddress(kernel32, "LoadLibraryA")
	if err != nil {
		return err
	}

	// Create remote thread
	threadHandle, _, err := w32.CreateRemoteThread(handle, nil, 0, uint32(loadLibAddr), remoteAddr, 0)
	if err != nil {
		return err
	}
	defer w32.CloseHandle(threadHandle)

	// Wait for completion (with timeout)
	w32.WaitForSingleObject(threadHandle, 5000)

	return nil
}

// eject unloads a DLL from the target process
func eject(pid int, lib string) error {
	dllName := filepath.Base(lib)

	// Find module handle
	handle := w32.CreateToolhelp32Snapshot(w32.TH32CS_SNAPMODULE|w32.TH32CS_SNAPMODULE32, uint32(pid))
	if handle == 0 {
		return ErrModuleSnapshot
	}
	defer w32.CloseHandle(handle)

	var entry w32.MODULEENTRY32
	entry.Size = uint32(unsafe.Sizeof(entry))
	var moduleHandle uintptr

	if w32.Module32First(handle, &entry) {
		for {
			moduleName := w32.UTF16PtrToString(&entry.SzModule[0])
			if moduleName == dllName {
				moduleHandle = uintptr(unsafe.Pointer(entry.ModBaseAddr))
				break
			}
			if !w32.Module32Next(handle, &entry) {
				break
			}
		}
	}

	if moduleHandle == 0 {
		return ErrModuleNotFound
	}

	// Open process
	procHandle, err := w32.OpenProcess(w32.PROCESS_ALL_ACCESS, false, uint32(pid))
	if err != nil {
		return err
	}
	defer w32.CloseHandle(procHandle)

	// Get FreeLibrary address
	kernel32 := w32.GetModuleHandleA("kernel32.dll")
	freeLibAddr, err := w32.GetProcAddress(kernel32, "FreeLibrary")
	if err != nil {
		return err
	}

	// Create remote thread to call FreeLibrary
	threadHandle, _, err := w32.CreateRemoteThread(procHandle, nil, 0, uint32(freeLibAddr), moduleHandle, 0)
	if err != nil {
		return err
	}
	defer w32.CloseHandle(threadHandle)

	w32.WaitForSingleObject(threadHandle, 5000)

	return nil
}
