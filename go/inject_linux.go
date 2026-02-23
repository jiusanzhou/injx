//go:build linux
// +build linux

package injgo

// Linux implementation using ptrace
// NOTE: Requires CAP_SYS_PTRACE capability or root

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"
)

// findProcessByName finds a process by name (Linux implementation)
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

// findProcessByPID opens a process by PID (Linux implementation)
func findProcessByPID(pid int) (*Process, error) {
	procPath := fmt.Sprintf("/proc/%d", pid)
	if _, err := os.Stat(procPath); os.IsNotExist(err) {
		return nil, ErrProcessNotFound
	}

	name := ""
	commPath := filepath.Join(procPath, "comm")
	if data, err := os.ReadFile(commPath); err == nil {
		name = strings.TrimSpace(string(data))
	}

	exePath := ""
	exeLink := filepath.Join(procPath, "exe")
	if target, err := os.Readlink(exeLink); err == nil {
		exePath = target
	}

	return &Process{
		PID:     pid,
		Name:    name,
		ExePath: exePath,
	}, nil
}

// findAllProcessesByName finds all processes matching the name
func findAllProcessesByName(name string) ([]*Process, error) {
	entries, err := os.ReadDir("/proc")
	if err != nil {
		return nil, err
	}

	var processes []*Process

	for _, entry := range entries {
		pid, err := strconv.Atoi(entry.Name())
		if err != nil {
			continue // Not a PID directory
		}

		proc, err := findProcessByPID(pid)
		if err != nil {
			continue
		}

		if name == "" || strings.Contains(proc.Name, name) || proc.Name == name {
			processes = append(processes, proc)
		}
	}

	return processes, nil
}

// isInjected checks if a .so is loaded in the process
func isInjected(pid int, lib string) (bool, error) {
	libName := filepath.Base(lib)

	mapsPath := fmt.Sprintf("/proc/%d/maps", pid)
	file, err := os.Open(mapsPath)
	if err != nil {
		return false, err
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		line := scanner.Text()
		if strings.Contains(line, libName) {
			return true, nil
		}
	}

	return false, nil
}

// inject loads a .so into the target process
// NOTE: This is a placeholder - full implementation requires ptrace
func inject(pid int, lib string) error {
	// TODO: Implement using ptrace + dlopen
	// This requires:
	// 1. PTRACE_ATTACH to target
	// 2. Save registers
	// 3. Find dlopen in target's libc
	// 4. Write library path to target memory
	// 5. Set up registers and call dlopen
	// 6. Restore registers
	// 7. PTRACE_DETACH

	return fmt.Errorf("Linux injection not yet implemented in Go - use Rust CLI or LD_PRELOAD=%s /path/to/app", lib)
}

// eject unloads a .so from the target process
func eject(pid int, lib string) error {
	return ErrUnsupported
}
