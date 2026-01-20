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
│  • Error codes for programmatic handling                     │
│  • Universal cimpl_free() for all allocations               │
│  • This layer NEVER needs to change                         │
└──────────────────────┬───────────────────────────────────────┘
                       │ Language-specific FFI (or AI codegen)
                       ↓
┌─────────────────────────────────────────────────────────────┐
│ Stage 3: Target Language (Idiomatic Bindings)               │
│                                                              │
│  • Python (ctypes, cffi)                                     │
│  • Node.js (Koffi, napi-rs, WASM)                          │
│  • Lua (LuaJIT FFI)                                         │
│  • Ruby (FFI gem)                                           │
│  • Go (cgo)                                                 │
│  • C# (P/Invoke)                                            │
│  • Java (JNA, JNI)                                          │
│  • Swift (C interop)                                        │
│  • ... any language with FFI support                        │
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

**Real example from this project:**
- `ffi-napi` (Node.js FFI library) broke with modern Node.js
- We switched to `koffi` (a maintained alternative)
- Changed ~10 lines of JavaScript
- The Rust code and C header: **zero changes**
- Python, Lua, and C bindings: **completely unaffected**

### 3. AI-Friendly Code Generation

The C API layer is perfect for AI code generation because:
- **Consistent patterns**: NULL = error, error codes, free functions
- **Rich documentation**: Doxygen comments in generated headers
- **Simple types**: Pointers, integers, strings
- **Standard conventions**: Decades of C API examples in training data

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
    let obj = deref_mut_or_return_neg!(ptr, T);
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
Uuid* uuid = uuid_parse("invalid");
if (uuid == NULL) {  // ← Check return value FIRST
    // NOW check error details
    int code = uuid_error_code();
    char* msg = uuid_last_error();
    fprintf(stderr, "Error %d: %s\n", code, msg);
    cimpl_free(msg);
}
```

**Never check error details without a failure indication.**

### 3. Universal Free Function

One `cimpl_free()` to rule them all:

```c
Uuid* uuid = uuid_new_v4();
char* str = uuid_to_string(uuid);
uint8_t* bytes = uuid_as_bytes(uuid);

cimpl_free(bytes);  // Free the byte array
cimpl_free(str);    // Free the string
cimpl_free(uuid);   // Free the UUID object
```

Benefits:
- Simple mental model
- No per-type free functions needed
- Works with any tracked pointer
- Double-free protection built-in

### 4. Error Codes + Messages

Provide both machine-readable codes and human-readable messages:

```c
// Error codes (0-99 = cimpl infrastructure, 100+ = library-specific)
extern const int32_t ERROR_OK;
extern const int32_t ERROR_NULL_PARAMETER;
extern const int32_t ERROR_UUID_PARSE_ERROR;

// Error retrieval functions
int32_t uuid_error_code(void);    // For programmatic handling
char* uuid_last_error(void);      // For human readers
void uuid_clear_error(void);      // Reset error state
```

Error messages include the error name:
```
"ParseError: invalid character: expected [0-9a-fA-F], found 'z'"
```

This format is:
- ✅ Human-readable (developers)
- ✅ Machine-parseable (error handlers)
- ✅ AI-friendly (code generation)

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
// External crate's type (uuid::Uuid)
use uuid::Uuid;

// Use it directly in FFI - no wrapper needed!
#[no_mangle]
pub extern "C" fn uuid_new_v4() -> *mut Uuid {
    box_tracked!(Uuid::new_v4())
}

#[no_mangle]
pub extern "C" fn uuid_to_string(uuid: *mut Uuid) -> *mut c_char {
    let obj = deref_or_return_null!(uuid, Uuid);
    to_c_string(obj.to_string())
}
```

The external type is already opaque to C, so `cbindgen` will generate:
```c
typedef struct Uuid Uuid;  // Opaque forward declaration
```

Perfect!

### Error Mapping Tables

Map library errors to C error codes declaratively:

```rust
// 1. Declare error code constants (cbindgen sees these)
#[no_mangle]
pub static ERROR_UUID_PARSE_ERROR: i32 = 100;

// 2. Create error mapper function
fn map_uuid_error(_e: &uuid::Error) -> (i32, &'static str) {
    (ERROR_UUID_PARSE_ERROR, "ParseError")
}

// 3. Register the mapper
const ERROR_MAPPER: fn(&uuid::Error) -> (i32, &'static str) = map_uuid_error;

// 4. Use in FFI functions (automatic conversion!)
#[no_mangle]
pub extern "C" fn uuid_parse(s: *const c_char) -> *mut Uuid {
    let s_str = cstr_or_return_null!(s);
    let uuid = ok_or_return_null!(Uuid::from_str(&s_str));
    box_tracked!(uuid)
}
```

The macro automatically:
- Checks if the Result is Ok
- On Err, looks up the error in the table
- Sets the thread-local error with the right code and message
- Returns NULL

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

All tracked allocations can be freed with `cimpl_free()`.

## What Makes a Good C API?

### DO: Keep it Simple

```c
// ✅ Good: Simple, obvious functions
Uuid* uuid_new_v4(void);
char* uuid_to_string(Uuid* uuid);
bool uuid_is_nil(Uuid* uuid);
void uuid_free(Uuid* uuid);  // Or just use cimpl_free()
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
/// Creates a new random UUID (version 4).
///
/// Returns NULL if random number generation fails.
/// The returned UUID must be freed with `cimpl_free()`.
///
/// # Example
/// ```c
/// Uuid* uuid = uuid_new_v4();
/// if (uuid == NULL) {
///     fprintf(stderr, "Failed to generate UUID\n");
///     return -1;
/// }
/// cimpl_free(uuid);
/// ```
#[no_mangle]
pub extern "C" fn uuid_new_v4() -> *mut Uuid {
    box_tracked!(Uuid::new_v4())
}
```

### DO: Provide Error Context

```c
// ✅ Good: Rich error information
extern const int32_t ERROR_PARSE_ERROR;     // For code
int32_t uuid_error_code(void);              // Get code
char* uuid_last_error(void);                // Get message
void uuid_clear_error(void);                // Reset
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
lib = CDLL("./libmylib.so")

# Define functions
lib.uuid_new_v4.restype = c_void_p
lib.uuid_new_v4.argtypes = []

# Call
uuid_ptr = lib.uuid_new_v4()
```

Wrap in Python classes for idiomatic APIs.

### Node.js (Koffi)

Use Koffi (not ffi-napi, which is unmaintained):

```javascript
const koffi = require('koffi');

const lib = koffi.load('./libmylib.dylib');
const UuidPtr = koffi.pointer(koffi.opaque('Uuid'));

const uuid_new_v4 = lib.func('uuid_new_v4', UuidPtr, []);
```

### Lua (LuaJIT FFI)

LuaJIT's FFI is amazing:

```lua
local ffi = require("ffi")

ffi.cdef[[
    typedef struct Uuid Uuid;
    Uuid* uuid_new_v4(void);
    void cimpl_free(void* ptr);
]]

local lib = ffi.load("mylib")
local uuid = lib.uuid_new_v4()
```

### Ruby (FFI gem)

```ruby
require 'ffi'

module MyLib
  extend FFI::Library
  ffi_lib './libmylib.so'
  
  attach_function :uuid_new_v4, [], :pointer
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
