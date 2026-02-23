# injx - Cross-language DLL/dylib injection toolkit
# https://github.com/jiusanzhou/injx

.PHONY: all build build-rust build-go clean test help
.DEFAULT_GOAL := help

# Detect platform
UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

# Output directories
OUT_DIR := build
RUST_OUT := $(OUT_DIR)/rust
GO_OUT := $(OUT_DIR)/go

# Binary names
ifeq ($(UNAME_S),Windows_NT)
    EXT := .exe
else
    EXT :=
endif

## Build

all: build ## Build all implementations

build: build-rust build-go ## Build Rust and Go implementations

build-rust: ## Build Rust implementation
	@echo "==> Building Rust implementation..."
	@mkdir -p $(RUST_OUT)
	@cd rust && cargo build --release
	@cp rust/target/release/injx$(EXT) $(RUST_OUT)/ 2>/dev/null || true
	@echo "    Done: $(RUST_OUT)/injx$(EXT)"

build-go: ## Build Go implementation
	@echo "==> Building Go implementation..."
	@mkdir -p $(GO_OUT)
ifeq ($(UNAME_S),Darwin)
	@echo "    Note: Go injection only supports Windows, cross-compiling..."
	@cd go && GOOS=windows GOARCH=amd64 CGO_ENABLED=0 go build -o ../$(GO_OUT)/injx-windows-amd64.exe ./cmd/injgo 2>/dev/null || echo "    Cross-compile skipped (may need CGO)"
	@cd go && GOOS=windows GOARCH=386 CGO_ENABLED=0 go build -o ../$(GO_OUT)/injx-windows-386.exe ./cmd/injgo 2>/dev/null || true
else ifeq ($(UNAME_S),Linux)
	@cd go && GOOS=windows GOARCH=amd64 CGO_ENABLED=0 go build -o ../$(GO_OUT)/injx-windows-amd64.exe ./cmd/injgo
	@cd go && GOOS=windows GOARCH=386 CGO_ENABLED=0 go build -o ../$(GO_OUT)/injx-windows-386.exe ./cmd/injgo
else
	@cd go && go build -o ../$(GO_OUT)/injx$(EXT) ./cmd/injgo
endif
	@echo "    Done: $(GO_OUT)/"

## Cross-compile

build-windows: ## Cross-compile for Windows (x86 + x64)
	@echo "==> Cross-compiling for Windows..."
	@mkdir -p $(OUT_DIR)/windows
	@cd rust && cargo build --release --target x86_64-pc-windows-gnu 2>/dev/null && \
		cp target/x86_64-pc-windows-gnu/release/injx.exe ../$(OUT_DIR)/windows/injx-x64.exe || \
		echo "    Rust Windows x64: skipped (target not installed)"
	@cd go && GOOS=windows GOARCH=amd64 CGO_ENABLED=0 go build -o ../$(OUT_DIR)/windows/injx-go-x64.exe ./cmd/injgo
	@cd go && GOOS=windows GOARCH=386 CGO_ENABLED=0 go build -o ../$(OUT_DIR)/windows/injx-go-x86.exe ./cmd/injgo
	@echo "    Done: $(OUT_DIR)/windows/"

## Examples

build-examples: ## Build example DLLs (Windows only)
	@echo "==> Building example DLLs..."
	@cd rust/examples && cargo build --release 2>/dev/null || echo "    Skipped (not on Windows or target not set)"

## Test

test: test-rust test-go ## Run all tests

test-rust: ## Run Rust tests
	@echo "==> Testing Rust..."
	@cd rust && cargo test

test-go: ## Run Go tests
	@echo "==> Testing Go..."
	@cd go &st ./...

## Clean

clean: ## Clean build artifacts
	@echo "==> Cleaning..."
	@rm -rf $(OUT_DIR)
	@cd rust && cargo clean
	@cd go && go clean
	@echo "    Done"

## Development

fmt: ## Format code
	@echo "==> Formatting..."
	@cd rust && cargo fmt
	@cd go && go fmt ./...

lint: ## Lint code
	@echo "==> Linting..."
	@cd rust && cargo clippy
	@cd go && golangci-lint run 2>/dev/null || go vet ./...

## Help

help: ## Show this help
	@echo "injx - Cross-language DLL/dylib injection toolkit"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}'
