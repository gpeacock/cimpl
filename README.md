# cimpl

**Simple C implementations from Rust**

Create clean, safe C FFI bindings that AI can automatically convert to any language.

[![Crates.io](https://img.shields.io/crates/v/cimpl.svg)](https://crates.io/crates/cimpl)
[![Documentation](https://docs.rs/cimpl/badge.svg)](https://docs.rs/cimpl)
[![License](https://img.shields.io/crates/l/cimpl.svg)](LICENSE-MIT)

> **Note:** This is a new project (v0.3.0+) taking over the `cimpl` crate name. Previous versions (0.1.x, 0.2.0) were a different, unrelated project and have been yanked.

## The Vision

```
Rust + cimpl → Clean C API → AI-powered bindings → All languages
```

Write your library once in safe Rust, expose it through a clean C API, and let AI generate high-quality bindings for Python, JavaScript, Lua, Ruby, C#, Java, Go, and more.

## Why cimpl?

Most Rust FFI examples show trivial toy code:
```rust
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 { a + b }
```

**Real FFI is much harder:**
- How do you return complex types like strings or structs?
- How do you propagate Result<T, E> errors across the FFI boundary?
- How do you handle object lifecycle (constructors, methods, destructors)?
- How do you prevent memory leaks and double-frees?
- How do you make errors usable in other languages?

**cimpl solves the hard problems:**
- ✅ Type-safe pointer tracking with validation
- ✅ Automatic error handling with descriptive, parseable messages
- ✅ Memory leak detection in tests
- ✅ Clean macros for production patterns (not toy examples)
- ✅ Object-oriented APIs (structs with methods, not just functions)
- ✅ AI-friendly C headers (auto-generated via cbindgen)
- ✅ One codebase → many language bindings

> **Note**: For Node.js and WASM targets, use [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen) instead. While Node.js can use cimpl via [Koffi FFI](https://github.com/Koromix/koffi), WASM provides better performance and integration.

## Quick Example

```rust
use cimpl::*;
use std::ffi::c_void;
use std::os::raw::c_char;
use thiserror::Error as ThisError;

// Your library's error type (using thiserror for convenience)
#[derive(ThisError, Debug)]
pub enum Error {
    #[error("value out of range: {0}")]
    OutOfRange(String),
    
    #[error("invalid UTF-8: {0}")]
    InvalidUtf8(String),
}

// Map to cimpl::Error for FFI - one line with from_error()!
impl From<Error> for cimpl::Error {
    fn from(e: Error) -> Self {
        cimpl::Error::from_error(e)  // Automatic: Debug → variant, Display → message
    }
}

// Clean, safe FFI function
#[no_mangle]
pub extern "C" fn vc_to_string(value: *mut ValueConverter) -> *mut c_char {
    let converter = deref_or_return_null!(value, ValueConverter);
    let result = ok_or_return_null!(converter.to_string());  // Error → cimpl::Error automatically
    to_c_string(result)
}

// Memory management wrapper (required for namespace safety)
#[no_mangle]
pub extern "C" fn vc_free(ptr: *mut c_void) -> i32 {
    cimpl::cimpl_free(ptr)  // Rust-level function, wrap in your C API
}
```

**That's it!** From this simple code:
- cbindgen generates a C header with proper namespace prefix
- Type validation ensures safety
- Errors map to descriptive strings: `"VariantName: details"`
- Memory is tracked automatically
- AI can generate bindings for any language

**AI Users:** See [AI_WORKFLOW.md](./AI_WORKFLOW.md) for a complete guide to generating FFI wrappers and language bindings with proper macro usage patterns.

## What You Get

### From Rust to C
```c
#include "value_converter.h"

// Create from integer
ValueConverter* vc = vc_from_i32(42);
if (vc == NULL) {
    char* msg = vc_last_error();  // "OutOfRange: need exactly 4 bytes..."
    printf("Error: %s\n", msg);
    vc_free(msg);
    return -1;
}

// Convert to hex
char* hex = vc_to_hex(vc);
printf("Hex: %s\n", hex);  // "2a000000"

// Cleanup (namespace-safe free function)
vc_free(hex);
vc_free(vc);
```

### From C to Python (auto-generated!)
```python
from value_converter import ValueConverter, OutOfRangeError

try:
    with ValueConverter.from_i32(42) as vc:
        print(vc.to_hex())  # "2a000000"
except OutOfRangeError as e:
    print(f"Error: {e}")
```

## Features

### Pointer Safety
- **Tracked pointers** with type validation
- **Rust-level `cimpl::cimpl_free()`** function (wrap with `{crate}_free` in your C API)
- **Shared registry** across all cimpl-based libraries
- **Double-free protection**
- **Type mismatch detection**

### Error Handling
- **String-based error messages** with consistent `"VariantName: details"` format
- **AI-friendly format**: Easy to parse and convert to typed exceptions
- **Automatic conversion** via `cimpl::Error::from_error()` or manual with `cimpl::Error::new()`
- **Works with thiserror**: Use `#[derive(ThisError)]` for ergonomic error definitions
- **Standard C conventions**: NULL/false/-1 indicates error, call `*_last_error()` for details

### Clean Macros
- `box_tracked!()` - Allocate and track Box
- `ptr_or_return!()` - Null pointer checks with automatic error messages
- `bytes_or_return_*!()` - Byte array validation with bounds checking
- `cstr_or_return_*!()` - C string conversion with null checks
- `deref_or_return_*!()` - Pointer validation and dereferencing
- `ok_or_return_*!()` - Result unwrapping with automatic error conversion
- `option_to_c_string!()` - Option to C string (NULL if None)

## Getting Started

Add to your `Cargo.toml`:

```toml
[dependencies]
cimpl = "0.1"
thiserror = "2.0"  # Recommended for ergonomic errors

[build-dependencies]
cbindgen = "0.27"
```

### For AI Users

**Read [AI_WORKFLOW.md](./AI_WORKFLOW.md) first!** It contains:
- Pre-flight checklist for catching anti-patterns
- Complete macro reference with decision trees
- Common mistakes to avoid
- Step-by-step guidance for generating FFI code

### Example

**[examples/reference/](./examples/reference/)** - Production-ready reference implementation

**This is not a toy example.** Unlike the typical `add(a, b)` FFI tutorials, this demonstrates the hard parts:

- ✅ **Real-world utility**: Value converter for type conversion (i32, u32, i64, u64, bytes, strings, hex)
- ✅ **Multiple constructors**: `from_i32()`, `from_string()`, `from_hex()`, etc.
- ✅ **Fallible conversions**: `to_i32()` might fail (wrong size), `to_string()` might fail (invalid UTF-8)
- ✅ **Proper validation**: Size limits, UTF-8 checks, overflow detection
- ✅ **Memory safety**: Tracked allocations, type validation, leak detection
- ✅ **Clear separation**: `lib.rs` (pure Rust API) vs `ffi.rs` (C FFI wrapper)

**Two-file structure:**
- `src/lib.rs` - Standard Rust library (no FFI concerns)
- `src/ffi.rs` - C FFI wrapper using cimpl

This shows how to add FFI to an existing Rust library, not how to write FFI from scratch.

See [examples/reference/README.md](./examples/reference/README.md) for detailed API reference.

## Real-World Use

A variation of this pattern used in production at Adobe for the [C2PA project](https://github.com/contentauth/c2pa-rs), providing C, Python, and other language bindings from a single Rust codebase.

## Contributing

Contributions welcome! This project is about making FFI simple and accessible.

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
