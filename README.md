# rust-cfb
[![Build Status](https://github.com/mdsteele/rust-cfb/actions/workflows/tests.yml/badge.svg)](https://github.com/mdsteele/rust-cfb/actions/workflows/tests.yml)
[![Crates.io](https://img.shields.io/crates/v/cfb.svg)](https://crates.io/crates/cfb)
[![Documentation](https://docs.rs/cfb/badge.svg)](https://docs.rs/cfb)

A Rust library for reading/writing [Compound File Binary](
https://en.wikipedia.org/wiki/Compound_File_Binary_Format) (structured storage)
files.  See [MS-CFB](https://msdn.microsoft.com/en-us/library/dd942138.aspx)
for the format specification.

## Quick Start

```bash
# Build the library
cargo build

# Run the cfbtool
cargo run --example cfbtool -- --help

# Create and test with a sample file
cargo run --example create_test_cfb
cargo run --example cfbtool -- ls --long test.cfb:
```

## Documentation

- **📖 [Complete Project Guide](CFB_PROJECT_GUIDE.md)** - Comprehensive documentation
- **⚡ [Quick Start Guide](QUICK_START.md)** - Get up and running fast
- **🔧 [C++ Wrapper Project](https://github.com/mmhliton/cfbcpp)** - C++ tools and bindings

## Features

- ✅ Read/Write CFB files
- ✅ Navigate storage hierarchies  
- ✅ Stream data operations
- ✅ Cross-platform Linux support
- ✅ C++ FFI bindings
- ✅ Command-line tools (`cfbtool`)

## cfbtool Examples

```bash
# List file contents
cargo run --example cfbtool -- ls document.doc:

# Read a stream
cargo run --example cfbtool -- cat document.doc:WordDocument

# Create new stream
cargo run --example cfbtool -- create --file-path test.cfb --inner-path Storage --stream-name NewStream
```
## License

rust-cfb is made available under the
[MIT License](http://spdx.org/licenses/MIT.html).
