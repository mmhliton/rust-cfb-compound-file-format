# CFB (Compound File Binary) Project Guide

## Overview

This guide documents the complete setup, compilation, and usage of two related projects for working with Microsoft Compound File Binary format files:

1. **rust-cfb-compound-file-format**: A Rust library for reading/writing CFB files
2. **cfbcpp**: A C++ wrapper and tools for the Rust CFB library

Both projects have been successfully ported to Linux and are available on GitHub.

## Table of Contents

- [Project Structure](#project-structure)
- [Installation & Setup](#installation--setup)
- [Rust CFB Library](#rust-cfb-library)
- [C++ CFB Tools](#c-cfb-tools)
- [cfbtool Usage Guide](#cfbtool-usage-guide)
- [GitHub Repositories](#github-repositories)
- [Development Workflow](#development-workflow)
- [Troubleshooting](#troubleshooting)

## Project Structure

### rust-cfb-compound-file-format
```
rust-cfb-compound-file-format/
├── Cargo.toml                 # Rust package configuration
├── src/
│   ├── lib.rs                # Main library code
│   ├── ffi.rs                # Foreign Function Interface for C++
│   └── internal/             # Internal implementation modules
├── examples/
│   ├── cfbtool.rs            # Command-line tool for CFB files
│   └── create_test_cfb.rs    # Test file generator
├── tests/                    # Unit tests
└── target/                   # Compiled artifacts
```

### cfbcpp
```
cfbcpp/
├── CMakeLists.txt            # CMake build configuration
├── include/
│   ├── cfb_wrapper.h         # C++ wrapper header
│   └── cfb.h                 # CFB library header
├── src/
│   ├── cfb_wrapper.cpp       # C++ wrapper implementation
│   └── cfbtool_cpp.cpp       # C++ version of cfbtool
└── build/                    # Build artifacts
```

## Installation & Setup

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install CMake and build tools
sudo apt update
sudo apt install cmake build-essential pkg-config libssl-dev

# Install Git
sudo apt install git
```

### Clone Repositories

```bash
# Clone the Rust CFB library
git clone https://github.com/mmhliton/rust-cfb-compound-file-format.git
cd rust-cfb-compound-file-format

# Clone the C++ wrapper project
git clone https://github.com/mmhliton/cfbcpp.git
```

## Rust CFB Library

### Building the Library

```bash
cd rust-cfb-compound-file-format

# Build the library
cargo build

# Build optimized release version
cargo build --release

# Run tests to verify everything works
cargo test
```

### Library Features

- **Read CFB files**: Open and navigate compound file structures
- **Write CFB files**: Create new compound files with storages and streams
- **Stream operations**: Read/write data to streams within compound files
- **Storage management**: Create and manage hierarchical storage structures
- **FFI support**: C-compatible interface for use with other languages

### Crate Configuration

The library is configured in `Cargo.toml` to produce both Rust library and C-compatible static library:

```toml
[lib]
crate-type = ["rlib", "staticlib"]
```

## C++ CFB Tools

### Building the C++ Project

```bash
cd cfbcpp

# Create build directory
mkdir -p build
cd build

# Configure with CMake
cmake ..

# Build the project
make

# The executable will be created as 'cfbtool_cpp'
```

### C++ Project Features

- **CFB file reading**: Access compound files from C++
- **Cross-platform compatibility**: Linux-compatible implementation
- **OpenSSL integration**: Uses OpenSSL for cryptographic functions
- **File I/O operations**: Custom Linux file I/O implementation

## cfbtool Usage Guide

The `cfbtool` is a command-line utility for working with CFB files. It's available in both Rust and C++ versions.

### Running cfbtool (Rust version)

```bash
cd rust-cfb-compound-file-format
cargo run --example cfbtool -- [COMMAND] [OPTIONS]
```

### Available Commands

#### 1. Help
```bash
cargo run --example cfbtool -- --help
```

#### 2. List Contents (ls)

**Basic listing:**
```bash
cargo run --example cfbtool -- ls file.cfb:
```

**Detailed listing:**
```bash
cargo run --example cfbtool -- ls --long file.cfb:
```

**Hierarchical listing:**
```bash
cargo run --example cfbtool -- ls --all file.cfb:
```

**List specific storage:**
```bash
cargo run --example cfbtool -- ls file.cfb:StorageName
```

#### 3. Read Stream Contents (cat)

**Read root-level stream:**
```bash
cargo run --example cfbtool -- cat file.cfb:StreamName
```

**Read nested stream:**
```bash
cargo run --example cfbtool -- cat file.cfb:Storage/StreamName
```

#### 4. Create New Stream (create)

```bash
cargo run --example cfbtool -- create \
  --file-path file.cfb \
  --inner-path StorageName \
  --stream-name NewStreamName
```

This creates a stream with predefined test data:
- String: "Hello"
- Integer: 123
- Float: 45.67
- Double: 89.1011

#### 5. Change Storage CLSID (chcls)

```bash
cargo run --example cfbtool -- chcls [UUID] file.cfb:StorageName
```

### Example Workflow

#### Creating a Test File

```bash
# Generate a test CFB file
cargo run --example create_test_cfb

# List the contents
cargo run --example cfbtool -- ls --long test.cfb:
# Output:
# -00000000        43 B    1601-01-01   RootStream
# +00000000         0 B    2025-10-18   TestStorage

# List contents of TestStorage
cargo run --example cfbtool -- ls test.cfb:TestStorage
# Output:
# TestStream

# Read stream content
cargo run --example cfbtool -- cat test.cfb:RootStream
# Output: This is a root level stream with test data.

cargo run --example cfbtool -- cat test.cfb:TestStorage/TestStream
# Output: Hello, World! This is test data in a compound file stream.

# Create a new stream
cargo run --example cfbtool -- create \
  --file-path test.cfb \
  --inner-path TestStorage \
  --stream-name NewTestStream

# Verify creation
cargo run --example cfbtool -- ls test.cfb:TestStorage
# Output:
# TestStream
# NewTestStream

# View hierarchical structure
cargo run --example cfbtool -- ls --all test.cfb:
# Output:
# Root Entry
#   RootStream
#   TestStorage
#     TestStream
#     NewTestStream
```

### Recursive Traversal & Timing (`traverse_cfb` example)

The `traverse_cfb` example provides a fast, read-only recursive walk of every storage and optionally every stream in a compound file. It is useful for:

- Getting a full hierarchical overview (including nested storages)
- Previewing the first bytes of each stream (hex + text) for quick identification
- Measuring how long a traversal of a given file takes (printed at the end)
- Performing a lightweight structural scan without stream content output (for speed)

#### Features

- Depth-first traversal of all storages starting at the root
- Optional stream preview printing (size + first up to 64 bytes hex and ASCII)
- Graceful warnings on inaccessible entries
- Execution time measurement using `std::time::Instant`

#### Running the example

Usage:
```bash
cargo run --example traverse_cfb -- <file> [print-streams=true|false]
```

Examples:
```bash
# Full traversal with stream previews (default)
cargo run --example traverse_cfb -- test.cfb

# Explicitly show streams
cargo run --example traverse_cfb -- test.cfb true

# Suppress stream previews (faster, only storages shown)
cargo run --example traverse_cfb -- test.cfb false
```

#### Sample output (with printing enabled)

```
Traversing compound file: test.cfb
📁 .(root)
  📄 RootStream (len=43 bytes)
     hex: 54 68 69 73 20 69 73 20 61 20 72 6f 6f 74 20 6c
     txt: This is a root level stream with test data.
  📁 TestStorage
    📄 TestStream (len=58 bytes)
       hex: 48 65 6c 6c 6f 2c 20 57 6f 72 6c 64 21 20 54 68
       txt: Hello, World! This is test data in a compound file stream.
Traversal completed in 0.000193s
```

#### Sample output (printing disabled)
```
Traversing compound file: test.cfb
📁 .(root)
  📁 TestStorage
Traversal completed in 0.000053s
```

Timing appears in Rust's debug `Duration` format. For very large files (GB-scale) piping to a pager helps:

```bash
cargo run --example traverse_cfb -- large_test_1gb.cfb false | less
```

#### Exit codes

- `0` on successful traversal
- `1` if the file path is missing or cannot be opened

#### Future enhancements (ideas)

- Optional JSON export (`--json`) of hierarchy and per-stream metadata
- Depth limiting (`--max-depth N`) for shallow scans
- Stream dumping (`--dump <path>` or `--grep <pattern>`) for targeted extraction
- Per-storage timing metrics for performance profiling

This example complements `cfbtool` by offering a quick hierarchical scan focused on inspection, performance measurement, and now selective stream output.

### Understanding CFB File Format

CFB files use a hierarchical structure:

- **Root Entry**: The top-level container
- **Storages**: Directories that can contain other storages or streams
- **Streams**: Files that contain actual data
- **Path notation**: Use `/` to separate levels (e.g., `Storage/SubStorage/Stream`)
- **Tool notation**: Use `:` to separate file path from internal path (e.g., `file.cfb:Storage/Stream`)

### Data Format Details

When using the `create` command, data is stored in little-endian binary format:
- `[u32 string_length][string bytes][i32][f32][f64]`
- String length as 4-byte unsigned integer
- String data as UTF-8 bytes
- Integer as 4-byte signed integer
- Float as 4-byte IEEE 754 float
- Double as 8-byte IEEE 754 double

## GitHub Repositories

### Repository Information

Both projects are hosted on GitHub under the `mmhliton` account:

- **Rust CFB**: https://github.com/mmhliton/rust-cfb-compound-file-format.git
- **C++ CFB**: https://github.com/mmhliton/cfbcpp.git

### Git Configuration

Both repositories are configured with:
- `origin`: Points to the `mmhliton` repositories
- `upstream`: Points to original source repositories (for Rust project)

### Pushing Changes

```bash
# For authenticated pushing (replace with your token)
git push https://mmhliton:[YOUR_TOKEN]@github.com/mmhliton/[REPO].git master
```

## Development Workflow

### Making Changes to Rust Library

```bash
cd rust-cfb-compound-file-format

# Make your changes
# ...

# Test the changes
cargo test

# Build and test cfbtool
cargo run --example cfbtool -- --help

# Commit and push
git add .
git commit -m "Your commit message"
git push origin master
```

### Making Changes to C++ Project

```bash
cd cfbcpp

# Make your changes
# ...

# Rebuild
cd build
make

# Test the executable
./cfbtool_cpp

# Commit and push
git add .
git commit -m "Your commit message"
git push origin master
```

### Adding New Features

1. **For Rust library**: Add functions to `src/lib.rs` or relevant modules
2. **For C++ wrapper**: Update `include/cfb_wrapper.h` and `src/cfb_wrapper.cpp`
3. **For cfbtool**: Extend `examples/cfbtool.rs` with new commands
4. **Update tests**: Add appropriate test cases
5. **Update documentation**: Keep this guide current

## Troubleshooting

### Common Issues

#### 1. Compilation Errors

**Rust compilation fails:**
```bash
# Update Rust toolchain
rustup update

# Clean and rebuild
cargo clean
cargo build
```

**C++ compilation fails:**
```bash
# Clean build directory
rm -rf build
mkdir build
cd build
cmake ..
make
```

#### 2. Missing Dependencies

**Rust dependencies:**
```bash
# Update Cargo.toml if needed
cargo update
```

**System dependencies:**
```bash
sudo apt install cmake build-essential pkg-config libssl-dev
```

#### 3. Runtime Errors

**File not found:**
- Verify file paths are correct
- Ensure CFB file exists and is readable
- Check file permissions

**Invalid CFB file:**
- Verify file is a valid compound file
- Try with a known good CFB file (e.g., an old .doc file)

**Authentication errors (Git):**
- Use personal access tokens instead of passwords
- Verify repository permissions
- Check remote URLs with `git remote -v`

### Performance Considerations

- **Large files**: CFB files can be several GB; ensure sufficient memory
- **Network files**: Access over network may be slow; consider local copies
- **Concurrent access**: CFB format doesn't support concurrent writers

### Platform-Specific Notes

#### Linux
- OpenSSL is used for cryptographic functions
- File I/O uses standard POSIX calls
- Case-sensitive file systems require exact path matching

#### Windows Compatibility
- Original code was Windows-specific
- Linux port maintains compatibility with Windows-created CFB files
- Endianness is handled correctly for cross-platform compatibility

## Advanced Usage

### Programming with the Rust Library

```rust
use cfb::CompoundFile;
use std::io::Cursor;

fn example() -> std::io::Result<()> {
    // Create a new compound file
    let mut cursor = Cursor::new(Vec::new());
    let mut comp = CompoundFile::create(&mut cursor)?;
    
    // Create storage and stream
    comp.create_storage("MyStorage")?;
    let mut stream = comp.create_stream("MyStorage/MyStream")?;
    stream.write_all(b"Hello, CFB!")?;
    
    comp.flush()?;
    Ok(())
}
```

### Using the C++ Wrapper

```cpp
#include "cfb_wrapper.h"

int main() {
    // Open existing CFB file
    auto file_data = ReadFileToBytes("example.cfb");
    
    // Process the compound file
    // ... your code here
    
    return 0;
}
```

## Contributing

### Development Setup

1. Fork both repositories on GitHub
2. Clone your forks locally
3. Set up upstream remotes to track original repositories
4. Create feature branches for new work
5. Submit pull requests for review

### Code Style

- **Rust**: Follow standard Rust formatting (`cargo fmt`)
- **C++**: Follow project conventions (see existing code)
- **Documentation**: Update this guide for significant changes
- **Testing**: Add tests for new functionality

### Commit Guidelines

- Use clear, descriptive commit messages
- Include issue numbers when applicable
- Keep commits focused and atomic
- Test before committing

---

This guide covers the complete CFB project ecosystem. For questions or issues, refer to the GitHub repositories or create new issues for tracking.