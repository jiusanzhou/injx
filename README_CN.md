<div align="center">

# 🔧 injx

**跨平台动态库注入工具包**

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Go](https://img.shields.io/badge/Go-1.21+-00ADD8?style=flat-square&logo=go)](https://golang.org/)
[![License](https://img.shields.io/badge/License-Apache--2.0-blue?style=flat-square)](./LICENSE)
[![Platform](https://img.shields.io/badge/平台-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey?style=flat-square)]()

[功能特性](#功能特性) • [安装](#安装) • [使用方法](#使用方法) • [API 文档](#api-文档) • [示例](#示例) • [English](./README.md)

</div>

---

## 概述

**injx** 是一个跨平台的动态库注入工具包，支持将 Windows DLL、macOS dylib 和 Linux 共享对象（.so）注入到运行中的进程。

提供两种使用方式：
- 🖥️ **命令行工具** - 快速进行注入操作
- 📦 **开发库** - Rust 和 Go API，方便集成到你的项目

## 功能特性

- 🦀 **Rust 实现** - 安全、高效、零依赖核心
- 🐹 **Go 实现** - 纯 Go 实现，支持 Windows
- 🖥️ **跨平台** - 支持 Windows、macOS 和 Linux
- 📦 **双接口** - 命令行工具和库 API
- 🔍 **进程发现** - 通过名称或 PID 查找进程
- 💉 **注入/卸载** - 运行时加载和卸载动态库
- ✅ **状态检查** - 验证库是否已注入

## 平台支持

| 平台 | 架构 | Rust | Go | 技术方案 |
|------|-----|------|-----|---------|
| Windows | x86 | ✅ | ✅ | CreateRemoteThread + LoadLibrary |
| Windows | x64 | ✅ | ✅ | CreateRemoteThread + LoadLibrary |
| macOS | ARM64 | ✅ | 🚧 | task_for_pid + thread_create_running |
| macOS | x64 | ✅ | 🚧 | task_for_pid + thread_create_running |
| Linux | x64 | ✅ | 🚧 | ptrace + dlopen |
| Linux | ARM64 | ✅ | 🚧 | ptrace + dlopen |

**图例：** ✅ 完全支持 | 🚧 开发中

## 安装

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/jiusanzhou/injx.git
cd injx

# 构建所有组件
make build

# 或单独构建
make build-rust    # Rust CLI 和库
make build-go      # Go CLI (Windows)
```

### Rust (Cargo)

```bash
# 安装 CLI
cargo install --path rust

# 或作为依赖添加
# Cargo.toml:
# [dependencies]
# injx = { git = "https://github.com/jiusanzhou/injx" }
```

### Go

```bash
go get go.zoe.im/injx
```

## 使用方法

### 命令行

```bash
# 基本用法
injx <进程名|PID> <动态库路径>...

# Windows - 注入 DLL 到记事本
injx notepad.exe payload.dll

# macOS - 注入 dylib（需要 root 权限）
sudo injx Safari payload.dylib

# Linux - 注入 .so（需要 CAP_SYS_PTRACE 或 root）
sudo injx firefox payload.so

# 注入多个库
injx 1234 lib1.dll lib2.dll lib3.dll
```

### 常用选项

```bash
injx --help              # 显示帮助
injx --version           # 显示版本
injx -l <PID>            # 列出进程已加载的库
```

## API 文档

### Rust API

```rust
use injx::{Process, InjectorExt};

// 通过名称查找进程
let process = Process::find_by_name("target.exe")?;
println!("找到进程: {} (PID: {})", process.name, process.pid);

// 通过 PID 打开进程
let process = Process::from_pid(1234)?;

// 注入动态库
process.inject("./payload.dll")?;

// 检查是否已注入
if process.is_injected("payload.dll")? {
    println!("库已加载");
}

// 卸载动态库
process.eject("payload.dll")?;
```

### Go API

```go
package main

import (
    "fmt"
    "go.zoe.im/injx"
)

func main() {
    // 通过名称查找进程
    proc, err := injx.FindByName("target.exe")
    if err != nil {
        panic(err)
    }
    fmt.Printf("找到进程: %s (PID: %d)\n", proc.Name, proc.PID)

    // 通过 PID 打开进程
    proc, err = injx.FromPID(1234)

    // 注入动态库
    err = proc.Inject("./payload.dll")

    // 检查是否已注入
    injected, err := proc.IsInjected("payload.dll")

    // 卸载动态库
    err = proc.Eject("payload.dll")
}
```

### 统一 API 接口

无论使用 Rust 还是 Go，API 设计保持一致：

| 操作 | Rust | Go |
|------|------|-----|
| 按名称查找 | `Process::find_by_name(name)` | `injx.FindByName(name)` |
| 按 PID 打开 | `Process::from_pid(pid)` | `injx.FromPID(pid)` |
| 注入 | `process.inject(path)` | `proc.Inject(path)` |
| 卸载 | `process.eject(path)` | `proc.Eject(path)` |
| 检查状态 | `process.is_injected(name)` | `proc.IsInjected(name)` |

## 示例

### 示例项目

`examples/` 目录包含各平台的示例 payload：

```
examples/
├── windows/          # Windows DLL 示例
│   ├── messagebox/   # 弹窗示例
│   ├── keylogger/    # 键盘记录演示
│   └── procinfo/     # 进程信息收集
├── macos/            # macOS dylib 示例
│   ├── hello/        # 简单示例
│   └── hook/         # 函数 hook 示例
├── linux/            # Linux .so 示例
│   └── hello/        # 简单示例
└── payloads/         # 通用 payload 模板
```

### 构建示例

```bash
# 构建所有示例
make build-examples

# 或单独构建
cd examples/windows/messageboxld --release
```

## 技术原理

### Windows
使用 `CreateRemoteThread` + `LoadLibraryW`：
1. 以 `PROCESS_ALL_ACCESS` 权限打开目标进程
2. 在目标进程中分配内存存放 DLL 路径
3. 将 DLL 路径写入分配的内存
4. 获取 kernel32.dll 中 `LoadLibraryW` 的地址
5. 创建远程线程调用 `LoadLibraryW` 加载 DLL

### macOS
使用 Mach API：
1. 通过 `task_for_pid` 获取任务端口（需要 root）
2. 在目标进程中分配内存存放 dylib 路径
3. 设置线程状态，准备调用 `dlopen`
4. 通过 `thread_create_running` 创建线程

### Linux
使用 ptrace：
1. 通过 `PTRACE_ATTACH` 附加到目标进程
2. 保存原始寄存器状态
3. 找到目标进程中 `dlopen` 的地址
4. 设置寄存器准备 dlopen 调用
5. 继续执行，然后恢复寄存器并分离

## 开发

```bash
make fmt      # 格式化代码
make lint     # 代码检查
make test     # 运行测试
make clean    # 清理构建产物
```

## 注意事项

⚠️ **安全警告**：
- 本工具仅供安全研究和学习使用
- 注入到其他进程可能触发安全软件警报
- macOS 和 Linux 需要提升权限
- 请遵守相关法律法规

## 许可证

[Apache-2.0](./LICENSE)

## 作者

[Zoe](https://zoe.im)

---

<div align="center">

**⭐ 如果这个项目对你有帮助，请给个 Star！**

</div>
