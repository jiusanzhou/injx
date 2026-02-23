package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strconv"

	"go.zoe.im/injgo"
)

const version = "0.2.0"

const helpContent = `injgo - Cross-platform library injection tool (Go)

USAGE:
    injgo <PROCESS_NAME|PID> <LIBRARY>...

EXAMPLES:
    # Windows - Inject DLL into Notepad
    injgo notepad.exe payload.dll

    # macOS - Inject dylib (requires root)
    sudo injgo Safari payload.dylib

    # Linux - Inject .so (requires CAP_SYS_PTRACE)
    sudo injgo target_app payload.so

    # Inject by PID with multiple libraries
    injgo 1234 lib1.dll lib2.dll

NOTES:
    - Windows: Requires same architecture (x86/x64) as target
    - macOS:   Full support coming soon (use Rust CLI)
    - Linux:   Full support coming soon (use Rust CLI)
`

func main() {
	fmt.Printf("injgo v%s - Cross-platform library injection\n", version)

	if len(os.Args) < 3 {
		fmt.Print(helpContent)
		os.Exit(1)
	}

	target := os.Args[1]
	libraries := os.Args[2:]

	if len(libraries) == 0 {
		fmt.Fprintln(os.Stderr, "Error: At least one library path required")
		os.Exit(1)
	}

	// Find or open process
	var proc *injgo.Process
	var err error

	pid, pidErr := strconv.Atoi(target)
	if pidErr == nil {
		fmt.Printf("Target: PID %d\n", pid)
		proc, err = injgo.FindProcessByPID(pid)
	} else {
		fmt.Printf("Target: %s\n", target)
		proc, err = injgo.FindProcessByName(target)
	}

	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}

	fmt.Printf("Found: %s (PID: %d)\n\n", proc.Name, proc.PID)

	// Inject each library
	successCount := 0
	failedCount := 0

	for _, lib := range libraries {
		absPath, _ := filepath.Abs(lib)
		fmt.Printf("  %s ... ", lib)

		// Check if already injected
		injected, _ := proc.IsInjected(absPath)
		if injected {
			fmt.Println("⚠ already injected")
			successCount++
			continue
		}

		// Inject
		err := proc.Inject(absPath)
		if err != nil {
			fmt.Printf("✗ failed: %v\n", err)
			failedCount++
		} else {
			fmt.Println("✓ success")
			successCount++
		}
	}

	fmt.Printf("\nResults: %d success, %d failed\n", successCount, failedCount)

	if failedCount > 0 {
		os.Exit(1)
	}
}
