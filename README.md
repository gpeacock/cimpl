# Cimple - C IMPLementation Utilities

[![Crates.io](https://img.shields.io/crates/v/cimple.svg)](https://crates.io/crates/cimple)
[![Documentation](https://docs.rs/cimple/badge.svg)](https://docs.rs/cimple)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

A Rust library providing utilities and macros for creating safe, ergonomic C FFI bindings. Extracted from techniques developed for the [c2pa-rs](https://github.com/contentauth/c2pa-rs) project.

## Why Cimple?

Instead of trying to automatically translate Rust into C bindings, **cimple** helps you craft a clean C API manually using safe, ergonomic utilities. Once you have a solid C API, use `cbindgen` to generate headers, and let AI tools create idiomatic bindings for any target language.

### The Three-Stage Pipeline

```
Stage 1: Rust (Safe)  →  Stage 2: C API (Simple)  →  Stage 3: Target Language (AI)
         [cimple]              [cbindgen]                    [AI tooling]
```

This approach gives you:
- **Safety**: Rust's guarantees + cimple's runtime checks
- **Simplicity**: Clean C API that any tool can understand  
- **Flexibility**: AI can generate idiomatic bindings for any language

## Features

### 🛡️ Pointer Registry with Type Validation
Thread-safe pointer tracking with mandatory type validation for passing Rust objects to C.

```rust
// Choose your wrapper: Box (simple), Arc (shared), Arc<Mutex> (shared+mutable)
let ptr = box_tracked!(my_object);  // Tracked automatically
// Later: cimple_free(ptr) works for any tracked type
```

### 🎯 Universal Free Function
One function to free ANY tracked pointer - objects, strings, or buffers.

```c
MyString* obj = mystring_create("hello");
char* str = mystring_get_value(obj);

cimple_free(str);  // Free the string
cimple_free(obj);  // Free the object - same function!
```

### 🔍 Allocation Tracking
Prevents double-frees and detects memory leaks automatically.

```rust
let c_string = to_c_string("Hello".to_string());  // Tracked!
// If not freed, you'll get a warning at shutdown
```

### 🪄 Ergonomic Macros
Clean, expressive macros that make control flow obvious with `_or_return_` naming.

```rust
#[no_mangle]
pub extern "C" fn process(input: *const c_char) -> *mut c_char {
    let rust_string = cstr_or_return_null!(input);
    let result = process_string(rust_string);
    to_c_string(result)
}
```

## Quick Start

### Add to your `Cargo.toml`

```toml
[dependencies]
cimple = "0.1"

[build-dependencies]
cbindgen = "0.27"
```

### Example: String Processing Library

```rust
use std::os::raw::c_char;
use cimple::{box_tracked, cstr_or_return_null, validate_and_deref, to_c_string};

// Export struct directly - opaque to C but tracked by cimple
pub struct MyString {
    value: String,
}

#[no_mangle]
pub extern "C" fn mystring_create(initial: *const c_char) -> *mut MyString {
    let initial_str = cstr_or_return_null!(initial);
    box_tracked!(MyString { value: initial_str })  // Allocate, track, return
}

#[no_mangle]
pub extern "C" fn mystring_to_uppercase(ptr: *mut MyString) -> *mut c_char {
    let obj = validate_and_deref!(ptr, MyString);  // Validate type
    to_c_string(obj.value.to_uppercase())
}

// No need for type-specific free - use universal cimple_free()
```

### Generate C Header with `cbindgen`

Create `build.rs`:

```rust
fn main() {
    cbindgen::Builder::new()
        .with_crate(".")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("include/mylib.h");
}
```

Run `cargo build` and you get a clean C header:

```c
typedef struct MyString MyString;

MyString* mystring_create(const char* initial);
char* mystring_to_uppercase(MyString* ptr);
int32_t cimple_free(void* ptr);  // Universal free!
```

### Let AI Generate Language Bindings

Give the header to an AI:

> "Create Python ctypes bindings for this C library: [paste header]"

The AI generates idiomatic code:

```python
class MyString:
    def __init__(self, initial: str):
        self._ptr = lib.mystring_create(initial.encode())
    
    def to_uppercase(self) -> str:
        result = lib.mystring_to_uppercase(self._ptr)
        value = result.decode()
        lib.cimple_free(result)  # Universal free!
        return value
    
    def __del__(self):
        lib.cimple_free(self._ptr)  # Universal free!
```

## Complete Example

See the [`example/`](example/) directory for a fully working example that demonstrates:

- ✅ Pointer-based API with type validation
- ✅ Universal `cimple_free()` for all allocations
- ✅ String conversion and memory management
- ✅ Error handling with error codes and messages
- ✅ Standard C conventions (NULL/-1 on error)
- ✅ C header generation with `cbindgen`
- ✅ C program using the library
- ✅ Makefile for easy building
- ✅ Comprehensive documentation

```bash
cd example
make run-c
```

## API Overview

### Macros

#### Pointer Validation
- `ptr_or_return!(ptr, error_value)` - Check pointer not null or return
- `ptr_or_return_null!(ptr)` - Return null on null pointer
- `ptr_or_return_int!(ptr)` - Return -1 on null pointer
- `validate_and_deref!(ptr, Type)` - Validate type and dereference immutably
- `validate_and_deref_mut_neg!(ptr, Type)` - Validate type and dereference mutably

#### String Conversion
- `cstr_or_return!(ptr, error_value)` - Convert C string with bounded read
- `cstr_or_return_null!(ptr)` - Convert or return null
- `cstr_or_return_int!(ptr)` - Convert or return -1
- `cstr_option!(ptr)` - Convert to `Option<String>`

#### Result Handling
- `ok_or_return!(result, handler, transform, error_value)` - Handle Result with custom logic
- `ok_or_return_null!(result)` - Return null on error
- `ok_or_return_int!(result)` - Return -1 on error
- `ok_or_return_zero!(result)` - Return 0 on error
- `ok_or_return_false!(result)` - Return false on error

#### Pointer Allocation
- `box_tracked!(expr)` - Create Box-wrapped tracked pointer
- `arc_tracked!(expr)` - Create Arc-wrapped tracked pointer

### Functions

#### String Management
- `to_c_string(String) -> *mut c_char` - Convert to tracked C string
- `free_c_string(*mut c_char) -> bool` - Free C string safely

#### Byte Array Management  
- `to_c_bytes(Vec<u8>) -> *const c_uchar` - Convert to tracked byte array
- `free_c_bytes(*const c_uchar) -> bool` - Free byte array safely

#### Pointer Management
- `track_box<T>(*mut T)` - Track Box-wrapped pointer
- `track_arc<T>(*mut T)` - Track Arc-wrapped pointer
- `track_arc_mutex<T>(*mut Mutex<T>)` - Track Arc<Mutex>-wrapped pointer
- `validate_pointer<T>(*mut T) -> Result<()>` - Validate pointer type
- `free_tracked_pointer(*mut u8) -> Result<()>` - Free any tracked pointer
- `cimple_free(*mut c_void) -> i32` - C-compatible universal free (FFI)

#### Buffer Safety
- `safe_slice_from_raw_parts(ptr, len, name) -> Result<&[u8]>` - Create validated slice
- `is_safe_buffer_size(size, ptr) -> bool` - Validate buffer size

#### Allocation Tracking
- `track_string_allocation(ptr, len)` - Track C string allocation
- `track_bytes_allocation(ptr, len)` - Track byte array allocation
- `untrack_allocation(ptr) -> bool` - Untrack allocation

### Error Handling
- `Error::last_message() -> Option<String>` - Get last error message
- `Error::set_last(Error)` - Set last error
- `Error::take_last() -> Option<Error>` - Take and clear last error

## Safety Features

### 1. Double-Free Prevention

```c
MyString* str = mystring_create("hello");
cimple_free(str);  // OK
cimple_free(str);  // Returns -1, sets error, doesn't crash
```

### 2. Memory Leak Detection

Forgot to free something? You'll know at shutdown:

```
⚠️  WARNING: 2 pointer(s) were not freed at shutdown!
This indicates C code did not properly free all allocated pointers.
```

### 3. Type Validation

```rust
let obj = validate_and_deref!(ptr, WrongType);  
// Returns error if wrong type or invalid pointer
```

### 4. Thread Safety

The pointer registry is protected by a mutex and works safely across threads.

### 5. Flexible Overhead

Choose the right wrapper for your use case:

```rust
box_tracked!(obj)         // Simple ownership
arc_tracked!(obj)         // Shared ownership
track_arc_mutex(ptr)      // Shared + interior mutability
```

## Design Philosophy

### 1. Explicit Control Flow
Macros that perform early returns include `_or_return_` in their names.

### 2. Direct Pointers with Type Validation
Expose real pointers to C, but track them in Rust for validation.

### 3. Universal Free Function
One function (`cimple_free`) frees any tracked pointer - objects, strings, buffers.

### 4. Flexible Overhead
Users choose their wrapper type (Box, Arc, Arc<Mutex>) based on needs.

### 3. Clear Memory Ownership
Every allocation has a clear owner and free function.

### 4. Thread Safety by Default
All shared state is protected automatically.

### 5. AI-Friendly C API
Simple, predictable patterns that AI tools can easily learn and replicate.

## Real-World Usage

This library extracts patterns proven in production with the [c2pa-rs](https://github.com/contentauth/c2pa-rs) project, which provides C bindings for Content Authenticity standard implementations.

## Contributing

Contributions are welcome! This library is designed to be:
- **Simple**: Easy to understand and use
- **Safe**: Catches common FFI mistakes
- **Flexible**: Works with your error handling approach
- **Well-documented**: Every macro and function is documented

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

Developed by Gavin Peacock at Adobe, extracted from the c2pa-rs project.

LAST_ERROR handling pattern borrowed from Michael Bryan's FFI work.
