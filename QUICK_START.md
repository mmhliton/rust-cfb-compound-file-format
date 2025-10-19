# CFB Projects Quick Start

This repository contains Rust implementation of Microsoft Compound File Binary (CFB) format reader/writer, along with C++ wrapper tools.

## Quick Setup

```bash
# Clone and build Rust library
git clone https://github.com/mmhliton/rust-cfb-compound-file-format.git
cd rust-cfb-compound-file-format
cargo build

# Test with cfbtool
cargo run --example cfbtool -- --help
```

## Quick Usage

```bash
# Create test file
cargo run --example create_test_cfb

# List contents (ensure a test.cfb exists; if using the host crate run `cargo run --example create_test_cfb` first)
cargo run --example cfbtool -- ls --long test.cfb:

# Read stream
cargo run --example cfbtool -- cat test.cfb:RootStream

# Create new stream
cargo run --example cfbtool -- create --file-path test.cfb --inner-path TestStorage --stream-name NewStream
```

## Related Projects

- **C++ Wrapper / Host crate**: https://github.com/mmhliton/cfbcpp.git (contains a slim Rust host `cfbtool_host` depending on upstream crate for examples)
- **Complete Guide**: See [CFB_PROJECT_GUIDE.md](CFB_PROJECT_GUIDE.md)

## CFB File Format

CFB (Compound File Binary) is Microsoft's structured storage format used by:
- Legacy Office documents (.doc, .xls, .ppt)
- MSI installer packages
- Various Windows applications

## Features

- ✅ Read/Write CFB files
- ✅ Navigate storage hierarchies
- ✅ Stream data operations
- ✅ Cross-platform Linux support
- ✅ C++ FFI bindings
- ✅ Command-line tools