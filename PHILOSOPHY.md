# The cimpl Philosophy

## Core Insight: The C ABI is Universal and Timeless

The foundation of `cimpl` is a simple but powerful observation:

**A well-designed C API is the most portable and long-lasting interface you can create.**

Languages evolve. FFI libraries come and go. Tooling breaks and gets fixed. But the C ABI has remained stable for decades and will continue to be the common denominator for FFI.

## The Three-Stage Pipeline

```
┌─────────────────────────────────────────────────────────────┐
│ Stage 1: Rust (Safe Implementation)                         │
│                                                              │
│  • Write once in safe Rust                                   │
│  • Use cimpl's macros for safety                            │
│  • Leverage Rust's type system                              │
│  • Zero unsafe code in your logic                           │
└──────────────────────┬───────────────────────────────────────┘
                       │ cimpl + cbindgen
                       ↓
┌─────────────────────────────────────────────────────────────┐
│ Stage 2: C API (Universal Interface) ← THE STABLE LAYER     │
│                                                              │
│  • Clean, documented C header                                │
│  • Standard C conventions (NULL = error, -1 = error)        │
│  • String-based error messages ("VariantName: details")     │
│  • Library-specific *_free() wrapping cimpl::cimpl_free()   │
│  • This layer NEVER needs to change                         │
└──────────────────────┬───────────────────────────────────────┘
                       │ Language-specific FFI (or AI codegen)
                       ↓
┌─────────────────────────────────────────────────────────────┐
│ Stage 3: Target Language (Idiomatic Bindings)               │
│                                                              │
│  • Python (ctypes, cffi)                                     │
│  • Lua (LuaJIT FFI)                                         │
│  • Ruby (FFI gem)                                           │
│  • Go (cgo)                                                 │
│  • C# (P/Invoke)                                            │
│  • Java (JNA, JNI)                                          │
│  • Swift (C interop)                                        │
│  • ... any language with FFI support                        │
│                                                              │
│  Note: For Node.js/WASM, use wasm-bindgen instead          │
└─────────────────────────────────────────────────────────────┘
```

## Why This Works

### 1. The C Layer is Universal

Every major language has mature, battle-tested C FFI support:
- It's not dependent on Rust-specific tools
- It works with decades of existing tooling
- It's what AI models are trained on (massive C API documentation)
- It's what bindings generators expect

### 2. Language Tooling Independence

When a language's FFI tooling has issues:
- ✅ Your C API remains unchanged
- ✅ Other languages are unaffected
- ✅ You can switch to alternative FFI libraries
- ✅ You can wait for the tooling to be fixed

**Note on Node.js**: While Node.js *can* use C FFI via libraries like [Koffi](https://github.com/Koromix/koffi), the better approach for Node.js is to use [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen) which provides better performance, native async support, and automatic TypeScript definitions.

### 3. AI-Friendly Code Generation

The C API layer is perfect for AI code generation because:
- **Consistent patterns**: NULL = error, string error messages, free functions
- **Rich documentation**: Doxygen comments in generated headers
- **Simple types**: Pointers, integers, strings
- **Standard conventions**: Decades of C API examples in training data
- **Parseable errors**: `"VariantName: details"` format easily converts to typed exceptions

An AI can read your generated C header and produce high-quality bindings for almost any language with minimal guidance.

## Design Principles

### 1. Safety First, Performance Second

```rust
// ❌ Don't do this (manual unsafe)
#[no_mangle]
pub extern "C" fn unsafe_function(ptr: *mut T) -> i32 {
    unsafe {
        if ptr.is_null() {
            return -1;
        }
        let obj = &mut *ptr;
        // ... error-prone manual handling
    }
}

// ✅ Do this (cimpl macros)
#[no_mangle]
pub extern "C" fn safe_function(ptr: *mut T) -> i32 {
    let obj = deref_mut_or_return_int!(ptr, T);
    // ... safe code
    0
}
```

The macros:
- Validate pointers automatically
- Check type IDs to prevent type confusion
- Track allocations to detect leaks
- Handle errors consistently

### 2. Standard C Conventions

Follow established C patterns that developers expect:

**Return values indicate success/failure:**
- Pointer-returning functions: `NULL` = error
- Integer-returning functions: `-1` = error, `0` = success
- Boolean-returning functions: `false` = error/no, `true` = success/yes

**Error details are retrieved conditionally:**
```c
ValueConverter* vc = vc_from_hex("invalid");
if (vc == NULL) {  // ← Check return value FIRST
    // NOW check error details
    char* msg = vc_last_error();  // "InvalidHex: invalid byte: in"
    fprintf(stderr, "Error: %s\n", msg);
    vc_free(msg);
}
```

**Never check error details without a failure indication.**

### 3. Namespace-Safe Free Functions

Each library wraps `cimpl::cimpl_free()` with its own prefix:

```c
ValueConverter* vc = vc_from_i32(42);
char* hex = vc_to_hex(vc);
uint8_t* bytes = vc_to_bytes(vc, &len);

vc_free(bytes);  // Free the byte array
vc_free(hex);    // Free the string
vc_free(vc);     // Free the object
```

Benefits:
- **Namespace safety**: No symbol conflicts when linking multiple libraries
- **Clear ownership**: `vc_free()` makes it obvious which library allocated it
- **Shared registry**: Under the hood, all cimpl libraries share the same tracking
- **Double-free protection** built-in

**Important:** `cimpl::cimpl_free()` is a Rust function (not `#[no_mangle]`). Libraries must wrap it:
```rust
#[no_mangle]
pub extern "C" fn vc_free(ptr: *mut c_void) -> i32 {
    cimpl::cimpl_free(ptr)
}
```

### 4. String-Based Errors

Provide descriptive error messages in a parseable format:

```rust
// Your library's error type with thiserror
#[derive(ThisError, Debug)]
pub enum Error {
    #[error("invalid hex: {0}")]
    InvalidHex(String),
    
    #[error("buffer too large: got {got} bytes, max {max}")]
    BufferTooLarge { got: usize, max: usize },
}

// Automatic conversion to cimpl::Error
impl From<Error> for cimpl::Error {
    fn from(e: Error) -> Self {
        cimpl::Error::from_error(e)  // Uses Debug for variant, Display for message
    }
}

// Or manual control
impl From<Error> for cimpl::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::InvalidHex(s) => cimpl::Error::new("InvalidHex", s),
            Error::BufferTooLarge { got, max } => 
                cimpl::Error::new("BufferTooLarge", format!("got {} bytes, max {}", got, max)),
        }
    }
}
```

Error format: `"VariantName: details"`

Examples:
```
"InvalidHex: invalid byte: ZZ"
"BufferTooLarge: got 9 bytes, max 8"
"OutOfRange: need exactly 4 bytes for i32, got 2"
```

This format is:
- ✅ Human-readable (developers can read it)
- ✅ Machine-parseable (split on `": "` to get variant and details)
- ✅ AI-friendly (easy to convert to typed exceptions)
- ✅ Cross-language (works in C, C++, Python, Swift, Kotlin, Go - proven in production)

### 5. Transparent Handle Design

Let users choose their own data structures:

```rust
// ✅ Direct Box (simple ownership)
#[no_mangle]
pub extern "C" fn uuid_new_v4() -> *mut Uuid {
    box_tracked!(Uuid::new_v4())
}

// ✅ Arc (shared ownership, if needed)
#[no_mangle]
pub extern "C" fn shared_resource_create() -> *mut Resource {
    arc_tracked!(Resource::new())
}

// ✅ Arc<Mutex> (thread-safe shared state, if needed)
#[no_mangle]
pub extern "C" fn thread_safe_create() -> *mut ThreadSafeState {
    arc_mutex_tracked!(ThreadSafeState::new())
}
```

`cimpl` tracks the type and provides a cleanup function, but you choose the wrapper that fits your needs.

## Common Patterns

### Wrapping External Crates

You can expose **any** Rust crate through `cimpl`, even ones you don't control:

```rust
// lib.rs - Pure Rust API
pub struct ValueConverter {
    data: Vec<u8>,
}

impl ValueConverter {
    pub fn from_i32(value: i32) -> Self { /* ... */ }
    pub fn to_string(&self) -> Result<String, Error> { /* ... */ }
}

// ffi.rs - C FFI wrapper
use crate::{ValueConverter, Error};
use cimpl::*;

impl From<Error> for cimpl::Error {
    fn from(e: Error) -> Self {
        cimpl::Error::from_error(e)
    }
}

#[no_mangle]
pub extern "C" fn vc_from_i32(value: i32) -> *mut ValueConverter {
    box_tracked!(ValueConverter::from_i32(value))
}

#[no_mangle]
pub extern "C" fn vc_to_string(vc: *mut ValueConverter) -> *mut c_char {
    let converter = deref_or_return_null!(vc, ValueConverter);
    let result = ok_or_return_null!(converter.to_string());
    to_c_string(result)
}
```

The external type is already opaque to C, so `cbindgen` will generate:
```c
typedef struct vc_ValueConverter vc_ValueConverter;  // Opaque forward declaration
```

Perfect!

### Error Conversion Pattern

Convert library errors to cimpl::Error for FFI:

**Automatic (Recommended with thiserror):**
```rust
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("invalid format: {0}")]
    InvalidFormat(String),
    
    #[error("out of range: {0}")]
    OutOfRange(String),
}

// One-line conversion!
impl From<Error> for cimpl::Error {
    fn from(e: Error) -> Self {
        cimpl::Error::from_error(e)
    }
}

// Use in FFI - automatic conversion via From trait
#[no_mangle]
pub extern "C" fn vc_from_hex(hex: *const c_char) -> *mut ValueConverter {
    let hex_str = cstr_or_return_null!(hex);
    let converter = ok_or_return_null!(ValueConverter::from_hex(&hex_str));
    box_tracked!(converter)
}
```

The macro automatically:
- Checks if the Result is Ok
- On Err, converts via `From` trait
- Sets the thread-local error message
- Returns NULL

**Manual (for custom control):**
```rust
impl From<Error> for cimpl::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::InvalidFormat(s) => cimpl::Error::new("InvalidFormat", s),
            Error::OutOfRange(s) => cimpl::Error::new("OutOfRange", s),
        }
    }
}
```

### Memory Management Patterns

```rust
// Returning owned strings (caller must free)
#[no_mangle]
pub extern "C" fn get_name() -> *mut c_char {
    to_c_string("Hello".to_string())
}

// Returning owned bytes (caller must free)
#[no_mangle]
pub extern "C" fn get_data() -> *mut u8 {
    to_c_bytes(vec![1, 2, 3, 4]) as *mut u8
}

// Returning owned objects (caller must free)
#[no_mangle]
pub extern "C" fn create_object() -> *mut MyObject {
    box_tracked!(MyObject::new())
}
```

All tracked allocations can be freed with library-specific `*_free()` functions that wrap `cimpl::cimpl_free()`.

## What Makes a Good C API?

### DO: Keep it Simple

```c
// ✅ Good: Simple, obvious functions
ValueConverter* vc_new_v4(void);
char* vc_to_string(ValueConverter* vc);
bool vc_is_empty(ValueConverter* vc);
void vc_free(void* ptr);  // Wraps cimpl::cimpl_free()
```

### DON'T: Over-engineer

```c
// ❌ Bad: Complex, non-standard patterns
int uuid_new_v4_ex(UuidHandle** handle, UuidOptions* opts, ErrorCtx* ctx);
size_t uuid_to_string_buf(UuidHandle* h, char* buf, size_t len, uint32_t flags);
```

### DO: Document Everything

Use Doxygen-style comments that `cbindgen` will include:

```rust
/// Creates a new ValueConverter from a signed 32-bit integer.
///
/// Returns NULL if allocation fails.
/// The returned pointer must be freed with `vc_free()`.
///
/// # Example
/// ```c
/// ValueConverter* vc = vc_from_i32(42);
/// if (vc == NULL) {
///     fprintf(stderr, "Failed to create converter\n");
///     return -1;
/// }
/// vc_free(vc);
/// ```
#[no_mangle]
pub extern "C" fn vc_from_i32(value: i32) -> *mut ValueConverter {
    box_tracked!(ValueConverter::from_i32(value))
}
```

### DO: Provide Error Context

```c
// ✅ Good: Descriptive error messages in parseable format
char* vc_last_error(void);  // Returns "VariantName: details"

// Example errors:
// "InvalidHex: invalid byte: ZZ"
// "BufferTooLarge: got 9 bytes, max 8"
// "OutOfRange: need exactly 4 bytes for i32, got 2"
```

### DON'T: Throw Exceptions or Panic

```rust
// ❌ Bad: Will crash the C caller
#[no_mangle]
pub extern "C" fn bad_function() {
    panic!("Oops!");  // ← Undefined behavior in FFI
}

// ✅ Good: Handle errors gracefully
#[no_mangle]
pub extern "C" fn good_function() -> i32 {
    match risky_operation() {
        Ok(_) => 0,
        Err(e) => {
            e.set_last();
            -1
        }
    }
}
```

## Language-Specific Binding Tips

### Python (ctypes)

Python's `ctypes` is built-in and works great:

```python
from ctypes import *

# Load library
lib = CDLL("./libvalue_converter.so")

# Define functions
lib.vc_from_i32.restype = c_void_p
lib.vc_from_i32.argtypes = [c_int32]

lib.vc_last_error.restype = c_char_p
lib.vc_last_error.argtypes = []

# Call
vc_ptr = lib.vc_from_i32(42)
if not vc_ptr:
    error = lib.vc_last_error().decode('utf-8')
    variant, _, details = error.partition(': ')
    print(f"Error {variant}: {details}")
```

Wrap in Python classes for idiomatic APIs.

### Node.js (Koffi)

Use Koffi (not ffi-napi, which is unmaintained):

```javascript
const koffi = require('koffi');

const lib = koffi.load('./libvalue_converter.dylib');
const VCPtr = koffi.pointer(koffi.opaque('vc_ValueConverter'));

const vc_from_i32 = lib.func('vc_from_i32', VCPtr, ['int32']);
const vc_free = lib.func('vc_free', 'int', ['void *']);
```

### Lua (LuaJIT FFI)

LuaJIT's FFI is amazing:

```lua
local ffi = require("ffi")

ffi.cdef[[
    typedef struct vc_ValueConverter vc_ValueConverter;
    vc_ValueConverter* vc_from_i32(int32_t value);
    void vc_free(void* ptr);
]]

local lib = ffi.load("value_converter")
local vc = lib.vc_from_i32(42)
```

### Ruby (FFI gem)

```ruby
require 'ffi'

module ValueConverter
  extend FFI::Library
  ffi_lib './libvalue_converter.so'
  
  attach_function :vc_from_i32, [:int32], :pointer
  attach_function :vc_free, [:pointer], :int
end
```

## When Things Go Wrong

### Language FFI Library Breaks

**Example:** `ffi-napi` stopped working with Node.js 18.20.8+

**Solution:**
1. ✅ Your C API is fine - don't change it
2. ✅ Try an alternative FFI library (like Koffi)
3. ✅ OR use compile-time bindings (like napi-rs)
4. ✅ OR wait for the FFI library to be fixed

### Other languages are completely unaffected!

### Type Confusion Bugs

If you get "WrongHandleType" errors:

```rust
// ❌ Wrong: Type mismatch
let ptr = box_tracked!(TypeA::new());
deref_or_return_null!(ptr, TypeB)  // ← Error! TypeId mismatch
```

This is **intentional** - it prevents memory corruption from type confusion.

### Memory Leaks

`cimpl` tracks allocations and can detect leaks:

```rust
// In tests or debug builds
#[test]
fn check_leaks() {
    let initial = get_allocations().len();
    
    // ... your FFI calls ...
    
    let final_count = get_allocations().len();
    assert_eq!(initial, final_count, "Memory leak detected!");
}
```

## Philosophy Summary

1. **The C ABI is timeless** - build on solid ground
2. **Safety through validation** - not through complexity
3. **Standard C conventions** - developers know what to expect
4. **Universal patterns** - one way to do things
5. **AI-friendly design** - enable code generation
6. **Language independence** - your C API outlives any specific tooling

Build once in Rust. Expose through C. Use everywhere.

That's `cimpl`.
