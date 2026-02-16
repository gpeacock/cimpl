# cimpl

**Simple C implementations from Rust**

Create clean, safe C FFI bindings that AI can automatically convert to any language.

[![Crates.io](https://img.shields.io/crates/v/cimpl.svg)](https://crates.io/crates/cimpl)
[![Documentation](https://docs.rs/cimpl/badge.svg)](https://docs.rs/cimpl)
[![License](https://img.shields.io/crates/l/cimpl.svg)](LICENSE)

## The Vision

```
Rust + cimpl → Clean C API → AI-powered bindings → All languages
```

Write your library once in safe Rust, expose it through a clean C API, and let AI generate high-quality bindings for Python, JavaScript, Lua, Ruby, C#, Java, Go, and more.

## Why cimpl?

**Traditional FFI is hard:**
- Manual memory management
- Unsafe pointer handling
- Complex error propagation
- Language-specific bindings for each target

**cimpl makes it simple:**
- ✅ Type-safe pointer tracking with validation
- ✅ Automatic error handling with descriptive messages
- ✅ Memory leak detection
- ✅ Clean macros for common patterns
- ✅ AI-friendly C headers (auto-generated via cbindgen)
- ✅ One codebase → many language bindings

> **Note**: For Node.js and WASM targets, use [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen) instead. While Node.js can use cimpl via [Koffi FFI](https://github.com/Koromix/koffi), WASM provides better performance and integration.

## Quick Example

```rust
use cimpl::*;
use std::str::FromStr;

// Your library's error type
pub enum Error {
    ParseError(String),
}

// Map to CimplError for FFI
impl From<Error> for CimplError {
    fn from(e: Error) -> Self {
        match e {
            Error::ParseError(s) => CimplError::new(format!("ParseError {}", s)),
        }
    }
}

// Clean, safe FFI function
#[no_mangle]
pub extern "C" fn uuid_parse(s: *const c_char) -> *mut uuid::Uuid {
    let s_str = cstr_or_return_null!(s);
    
    let uuid = ok_or_return_null!(
        uuid::Uuid::from_str(&s_str)
            .map_err(|e| Error::ParseError(e.to_string()))
    );
    
    box_tracked!(uuid)
}
```

**That's it!** From this simple code:
- cbindgen generates a C header
- Type validation ensures safety
- Errors map to descriptive strings ("ErrorName details")
- Memory is tracked automatically
- AI can generate bindings for any language

**Want to wrap an existing crate?** See [AI_WORKFLOW.md](./AI_WORKFLOW.md) for step-by-step instructions on using AI to generate FFI wrappers and language bindings.

## What You Get

### From Rust to C
```c
Uuid* uuid = uuid_parse("550e8400-e29b-41d4-a716-446655440000");
if (uuid == NULL) {
    char* msg = uuid_last_error(); // "ParseError: invalid character..."
    printf("Error: %s\n", msg);
    uuid_free(msg);
    return -1;
}
uuid_free(uuid);
```

### From C to Python (auto-generated!)
```python
from uuid_py import Uuid, ParseError

try:
    uuid = Uuid.parse("550e8400-e29b-41d4-a716-446655440000")
    print(uuid)
except ParseError as e:
    print(f"Error: {e.message}")
```

### From C to Lua (auto-generated!)
```lua
local uuid = require("uuid")
local ok, result = pcall(function()
    return uuid.parse("550e8400-e29b-41d4-a716-446655440000")
end)
if ok then
    print(result)
else
    print("Error: " .. result)
end
```

## Features

### Pointer Safety
- **Tracked pointers** with type validation
- **Universal `cimpl_free()`** works on any tracked pointer
- **Double-free protection**
- **Type mismatch detection**

### Error Handling
- **String-based error messages** with consistent "ErrorName details" format
- **AI-friendly format**: Easy to parse and convert to typed exceptions
- **Automatic conversion** via `From` trait
- **Standard C conventions**: NULL/-1 indicates error, call `last_error()` for details

### Clean Macros
- `box_tracked!()` - Allocate and track Box
- `cstr_or_return_*!()` - C string conversion with null checks
- `deref_or_return_*!()` - Pointer validation and dereferencing
- `ok_or_return_*!()` - Result unwrapping with automatic error conversion
- String-based error handling for clean, flexible error messages

## Getting Started

Add to your `Cargo.toml`:

```toml
[dependencies]
cimpl = "0.1"

[build-dependencies]
cbindgen = "0.27"
```

### Example

**[examples/reference/](./examples/reference/)** - Comprehensive reference implementation demonstrating all cimpl patterns:
- String parameters and returns (C ↔ Rust)
- Byte arrays with safe handling
- Result<T, E> with custom error enums
- Option<T> for nullable values
- Struct lifecycle (create, modify, query, destroy)
- Error handling and propagation
- Memory management with tracked allocations
- Includes Python bindings showing idiomatic error handling

The example implements a "secret message processor" with various encoding/decoding operations (ROT13, hex, validation, statistics) to exercise all FFI patterns without external dependencies.

See [examples/reference/README.md](./examples/reference/README.md) for detailed API reference and usage patterns.

## Real-World Use

A variation of this pattern used in production at Adobe for the [C2PA project](https://github.com/contentauth/c2pa-rs), providing C, Python, and other language bindings from a single Rust codebase.

## Contributing

Contributions welcome! This project is about making FFI simple and accessible.

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
