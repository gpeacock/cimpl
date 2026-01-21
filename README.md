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
- ✅ Automatic error handling with error codes
- ✅ Memory leak detection
- ✅ Clean macros for common patterns
- ✅ AI-friendly C headers (auto-generated via cbindgen)
- ✅ One codebase → many language bindings

> **Note**: For Node.js and WASM targets, use [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen) instead. While Node.js can use cimpl via [Koffi FFI](https://github.com/Koromix/koffi), WASM provides better performance and integration.

## Quick Example

```rust
use cimpl::*;
use uuid::Uuid;

// Define error constants (visible to cbindgen)
#[no_mangle]
pub static ERROR_UUID_PARSE_ERROR: i32 = 100;

// Register the error mapper
const ERROR_MAPPER: fn(&uuid::Error) -> (i32, &'static str) = 
    |_e| (ERROR_UUID_PARSE_ERROR, "ParseError");

// Clean, safe FFI function
#[no_mangle]
pub extern "C" fn uuid_parse(s: *const c_char) -> *mut Uuid {
    let s_str = cstr_or_return_null!(s);
    let uuid = ok_or_return_null!(Uuid::from_str(&s_str));
    box_tracked!(uuid)
}
```

**That's it!** From this simple code:
- cbindgen generates a C header
- Type validation ensures safety
- Errors map to C error codes
- Memory is tracked automatically
- AI can generate bindings for any language

**Want to wrap an existing crate?** See [AI_WORKFLOW.md](./AI_WORKFLOW.md) for step-by-step instructions on using AI to generate FFI wrappers and language bindings.

## What You Get

### From Rust to C
```c
Uuid* uuid = uuid_parse("550e8400-e29b-41d4-a716-446655440000");
if (uuid == NULL) {
    int code = uuid_error_code();  // 100
    char* msg = uuid_last_error(); // "ParseError: ..."
    printf("Error %d: %s\n", code, msg);
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
    print(f"Error {e.code}: {e.message}")
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
- **Table-based error mapping** from Rust errors to C error codes
- **AI-friendly format**: "ErrorName: details"
- **Automatic conversion** via macros
- **Standard C conventions**: NULL/−1 indicates error

### Clean Macros
- `box_tracked!()` - Allocate and track Box
- `cstr_or_return_*!()` - C string conversion with null checks
- `deref_or_return_*!()` - Pointer validation and dereferencing
- `ok_or_return_*!()` - Result unwrapping with error mapper
- Error mapper pattern for clean, flexible error handling

## Getting Started

Add to your `Cargo.toml`:

```toml
[dependencies]
cimpl = "0.1"

[build-dependencies]
cbindgen = "0.27"
```

### Examples

1. **[example/](./example/)** - **START HERE**: Simple string library showing all core patterns
   - Purpose-built to demonstrate cimpl features
   - Clean, documented code
   - Perfect for learning the basics

2. **[stream-example/](./stream-example/)** - **Callback Pattern**: Stream I/O with callbacks
   - Shows how to implement callback-based streams
   - Bridges C callbacks to Rust's Read/Write/Seek traits
   - Real-world pattern used in libraries like C2PA
   - Demonstrates bidirectional function pointer usage

3. **[uuid-example/](./uuid-example/)** - **Advanced**: Wrapping external crates with AI
   - Shows how to wrap the `uuid` crate (external dependency)
   - Demonstrates AI-assisted binding generation for Python, Lua, C++
   - See [AI_GENERATION_GUIDE.md](./uuid-example/AI_GENERATION_GUIDE.md) for the AI workflow
   - See [EXTERNAL_CRATE_EXAMPLE.md](./uuid-example/EXTERNAL_CRATE_EXAMPLE.md) for technical details

## Documentation

### Quick Start
- **[AI_WORKFLOW.md](./AI_WORKFLOW.md)** - **Complete workflow**: How to use AI to wrap any Rust library and generate bindings
  - Stage 1: Choose or write a Rust library
  - Stage 2: AI generates C FFI wrapper (using cimpl)
  - Stage 3: AI generates target language bindings
  - Includes prompts and examples

### Core Documentation
- **[DESIGN_GUIDE.md](./DESIGN_GUIDE.md)** - Design philosophy, best practices, and lessons learned
- **[LANGUAGE_BINDINGS.md](./LANGUAGE_BINDINGS.md)** - Complete guide for creating bindings in Python, JavaScript, Lua, Ruby, C#, Java, Go, Swift

### Technical References
- **[POINTER_REDESIGN.md](./POINTER_REDESIGN.md)** - Pointer registry architecture
- **[TABLE_BASED_ERROR_MAPPING.md](./TABLE_BASED_ERROR_MAPPING.md)** - Error handling system
- **[MACRO_PATTERNS.md](./MACRO_PATTERNS.md)** - Complete macro reference
- **[STANDARD_C_CONVENTIONS.md](./STANDARD_C_CONVENTIONS.md)** - Error conventions
- **[ENUM_ERROR_PATTERN.md](./ENUM_ERROR_PATTERN.md)** - Best practices for using C enums for error codes
- **[NAMESPACE_COLLISION_FIX.md](./NAMESPACE_COLLISION_FIX.md)** - Preventing enum namespace collisions with cbindgen

### Examples with Full Bindings
- **[example](./example/)** - **START HERE**: String manipulation demonstrating all cimpl patterns
  - Purpose-built to teach core concepts
  - Clean, focused code
- **[uuid-example](./uuid-example/)** - **Advanced**: Wrapping external crates with AI
  - Shows how to expose existing Rust crates through C
  - AI-generated bindings for C, Python, Lua, and C++
  - See [AI_GENERATION_GUIDE.md](./uuid-example/AI_GENERATION_GUIDE.md)

## Real-World Use

A variation of this pattern used in production at Adobe for the [C2PA project](https://github.com/contentauth/c2pa-rs), providing C, Python, and other language bindings from a single Rust codebase.

## Contributing

Contributions welcome! This project is about making FFI simple and accessible.

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
