#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "=== Building Jinx ==="

# Detect OS
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

echo "Platform: $OS/$ARCH"

# Build Rust
echo ""
echo "--- Building Rust implementation ---"
cd "$PROJECT_ROOT/rust"
if command -v cargo &> /dev/null; then
    cargo build --release
    echo "Rust build: OK"
else
    echo "Rust build: SKIPPED (cargo not found)"
fi

# Build Go
echo ""
echo "--- Building Go implementation ---"
cd "$PROJECT_ROOT/go"
if command -v go &> /dev/null; then
    # Update module name
    go mod edit -module go.zoe.im/jinx 2>/dev/null || true
    
    if [ "$OS" = "darwin" ]; then
        echo "Note: Go injection only supports Windows currently"
        GOOS=windows GOARCH=amd64 go build -o jinx-windows-amd64.exe ./cmd/injgo 2>/dev/null || echo "Cross-compile: Windows target may need CGO"
    else
        go build -o jinx ./cmd/injgo
    fi
    echo "Go build: OK"
else
    echo "Go build: SKIPPED (go not found)"
fi

echo ""
echo "=== Build Complete ==="
