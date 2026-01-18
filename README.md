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

### 🛡️ Handle-Based API
Thread-safe handle management system for passing Rust objects to C without exposing raw pointers.

```rust
let handle = cimple::get_handles().insert(my_object);
let ptr = cimple::handle_to_ptr::<MyType>(handle);
```

### 🔍 Allocation Tracking
Prevents double-frees and detects memory leaks automatically.

```rust
let c_string = to_c_string("Hello".to_string());  // Tracked!
// If not freed, you'll get a warning at shutdown
```

### 🎯 Buffer Safety
Validates buffer sizes and prevents integer overflow in pointer arithmetic.

```rust
let slice = unsafe { 
    safe_slice_from_raw_parts(ptr, len, "buffer_name")?
};
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
use cimple::{cstr_or_return_null, guard_handle_or_null, to_c_string};

struct MyString {
    value: String,
}

// Opaque handle type for C
#[repr(C)]
pub struct MyStringHandle {
    _private: [u8; 0],
}

#[no_mangle]
pub extern "C" fn mystring_create(initial: *const c_char) -> *mut MyStringHandle {
    let initial_str = cstr_or_return_null!(initial);
    let my_string = MyString { value: initial_str };
    let handle = cimple::get_handles().insert(my_string);
    cimple::handle_to_ptr::<MyStringHandle>(handle)
}

#[no_mangle]
pub extern "C" fn mystring_to_uppercase(handle: *mut MyStringHandle) -> *mut c_char {
    guard_handle_or_null!(handle, MyString, obj);
    to_c_string(obj.value.to_uppercase())
}

#[no_mangle]
pub extern "C" fn mystring_free(handle: *mut MyStringHandle) -> i32 {
    cimple::free_handle!(handle, MyString)
}
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
typedef struct MyStringHandle MyStringHandle;

MyStringHandle* mystring_create(const char* initial);
char* mystring_to_uppercase(MyStringHandle* handle);
int32_t mystring_free(MyStringHandle* handle);
```

### Let AI Generate Language Bindings

Give the header to an AI:

> "Create Python ctypes bindings for this C library: [paste header]"

The AI generates idiomatic code:

```python
class MyString:
    def __init__(self, initial: str):
        self._handle = lib.mystring_create(initial.encode())
    
    def to_uppercase(self) -> str:
        result = lib.mystring_to_uppercase(self._handle)
        value = result.decode()
        lib.mystring_string_free(result)
        return value
```

## Complete Example

See the [`example/`](example/) directory for a fully working example that demonstrates:

- ✅ Handle-based API
- ✅ String conversion and memory management
- ✅ Error handling with thread-local errors
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

#### Handle Management
- `return_handle!(result, error_handler, Type)` - Create and return handle
- `free_handle!(ptr, Type)` - Free a handle safely
- `guard_handle_or_null!(ptr, Type, name)` - Access handle immutably
- `guard_handle_mut_or_return_neg!(ptr, Type, name)` - Access handle mutably
- `guard_handle_or_default!(ptr, Type, name, default)` - With custom default

### Functions

#### String Management
- `to_c_string(String) -> *mut c_char` - Convert to tracked C string
- `free_c_string(*mut c_char) -> bool` - Free C string safely

#### Byte Array Management  
- `to_c_bytes(Vec<u8>) -> *const c_uchar` - Convert to tracked byte array
- `free_c_bytes(*const c_uchar) -> bool` - Free byte array safely

#### Handle Management
- `get_handles() -> &'static HandleMap` - Get global handle map
- `handle_to_ptr<T>(Handle) -> *mut T` - Convert handle to pointer
- `ptr_to_handle<T>(*mut T) -> Handle` - Convert pointer to handle

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
char* str = mylib_get_string();
mylib_free_string(str);  // OK
mylib_free_string(str);  // Returns -1, prints warning, doesn't crash
```

### 2. Memory Leak Detection

Forgot to free something? You'll know at shutdown:

```
⚠️  WARNING: 2 handle(s) were not freed at shutdown!
This indicates C code did not properly free all allocated handles.
```

### 3. Bounded String Reading

No more reading unbounded memory:

```rust
cstr_or_return_null!(ptr);  // Max 64KB, returns null if no terminator
```

### 4. Thread-Safe Handles

All handles are protected by mutexes and work safely across threads.

### 5. Type-Safe Handle Casting

```rust
guard_handle_or_null!(handle, WrongType, obj);  
// Returns null and sets error if wrong type
```

## Design Philosophy

### 1. Explicit Control Flow
Macros that perform early returns include `_or_return_` in their names.

### 2. Opaque Handles > Raw Pointers
Never expose Rust types directly. Use opaque handles with clear ownership.

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
