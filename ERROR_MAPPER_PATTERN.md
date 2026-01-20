# Error Mapper Pattern

As of the latest version, `cimpl` supports a cleaner error handling pattern using error mapper functions instead of tables.

## Overview

Instead of defining error tables with the `define_error_codes!` macro, you now write a simple mapping function that converts your library's errors to `(code, name)` pairs.

## Benefits

1. **Transparent** - Just a regular Rust function, no macro magic
2. **Flexible** - Full control over error mapping logic
3. **Debuggable** - Can step through and test the mapper
4. **AI-Friendly** - Easy pattern for code generation
5. **Clean API** - No need to pass tables to every macro call

## Pattern

### Step 1: Declare Error Codes

These must be `#[no_mangle]` statics so `cbindgen` can see them:

```rust
// Core cimpl infrastructure errors (0-99) - always include these
#[no_mangle]
pub static ERROR_OK: i32 = 0;
#[no_mangle]
pub static ERROR_NULL_PARAMETER: i32 = 1;
#[no_mangle]
pub static ERROR_STRING_TOO_LONG: i32 = 2;
#[no_mangle]
pub static ERROR_INVALID_HANDLE: i32 = 3;
#[no_mangle]
pub static ERROR_WRONG_HANDLE_TYPE: i32 = 4;
#[no_mangle]
pub static ERROR_OTHER: i32 = 5;

// Your library-specific errors (100+)
#[no_mangle]
pub static ERROR_UUID_PARSE_ERROR: i32 = 100;
#[no_mangle]
pub static ERROR_UUID_INVALID_LENGTH: i32 = 101;
#[no_mangle]
pub static ERROR_UUID_INVALID_CHARACTER: i32 = 102;
```

### Step 2: Write Mapper Function

Create a function that maps your error type to `(code, name)`:

```rust
/// Maps uuid::Error to cimpl error codes
fn map_uuid_error(e: &uuid::Error) -> (i32, &'static str) {
    match e {
        uuid::Error::InvalidLength(_) => (ERROR_UUID_INVALID_LENGTH, "InvalidLength"),
        uuid::Error::InvalidCharacter(_, _) => (ERROR_UUID_INVALID_CHARACTER, "InvalidCharacter"),
        _ => (ERROR_UUID_PARSE_ERROR, "ParseError"),
    }
}
```

**Advanced: You can customize freely:**

```rust
fn map_uuid_error(e: &uuid::Error) -> (i32, &'static str) {
    match e {
        // Combine multiple errors into one
        uuid::Error::InvalidLength(_) | 
        uuid::Error::InvalidGroups(_) => (ERROR_UUID_PARSE_ERROR, "ParseError"),
        
        // Add context-specific handling
        uuid::Error::InvalidLength(0) => (ERROR_UUID_EMPTY, "EmptyUuid"),
        
        // Use helper functions
        _ if is_network_error(e) => (ERROR_UUID_NETWORK, "NetworkError"),
        
        // Fallback
        _ => (ERROR_UUID_OTHER, "UuidError"),
    }
}
```

### Step 3: Register the Mapper

Create a const that the macros will use:

```rust
const ERROR_MAPPER: fn(&uuid::Error) -> (i32, &'static str) = map_uuid_error;
```

**Note**: You can also use `register_error_mapper!(map_uuid_error)` macro, but explicit const declaration is clearer.

### Step 4: Use Clean Macros

Now `ok_or_return_*` macros automatically use `ERROR_MAPPER`:

```rust
#[no_mangle]
pub extern "C" fn uuid_parse(s: *const c_char) -> *mut Uuid {
    let s_str = cstr_or_return_null!(s);
    let uuid = ok_or_return_null!(Uuid::from_str(&s_str));  // ← Uses ERROR_MAPPER!
    box_tracked!(uuid)
}

#[no_mangle]
pub extern "C" fn uuid_from_bytes(bytes: *const u8) -> *mut Uuid {
    ptr_or_return_null!(bytes);
    let slice = unsafe { std::slice::from_raw_parts(bytes, 16) };
    let uuid = ok_or_return_null!(Uuid::from_slice(slice));  // ← Clean!
    box_tracked!(uuid)
}
```

## Complete Example

```rust
use cimpl::{
    box_tracked, cstr_or_return_null, ok_or_return_null, 
    to_c_bytes, to_c_string, Error,
};
use uuid::Uuid;

// ===== STEP 1: Declare Error Codes =====
#[no_mangle]
pub static ERROR_OK: i32 = 0;
#[no_mangle]
pub static ERROR_NULL_PARAMETER: i32 = 1;
#[no_mangle]
pub static ERROR_INVALID_HANDLE: i32 = 3;
#[no_mangle]
pub static ERROR_UUID_PARSE_ERROR: i32 = 100;

// ===== STEP 2: Write Mapper Function =====
fn map_uuid_error(_e: &uuid::Error) -> (i32, &'static str) {
    (ERROR_UUID_PARSE_ERROR, "ParseError")
}

// ===== STEP 3: Register Mapper =====
const ERROR_MAPPER: fn(&uuid::Error) -> (i32, &'static str) = map_uuid_error;

// ===== STEP 4: Use Clean Macros =====
#[no_mangle]
pub extern "C" fn uuid_parse(s: *const c_char) -> *mut Uuid {
    let s_str = cstr_or_return_null!(s);
    let uuid = ok_or_return_null!(Uuid::from_str(&s_str));
    box_tracked!(uuid)
}

#[no_mangle]
pub extern "C" fn uuid_to_string(uuid: *mut Uuid) -> *mut c_char {
    let obj = deref_or_return_null!(uuid, Uuid);
    to_c_string(obj.to_string())
}
```

## AI Code Generation

This pattern is perfect for AI code generation. Given an error enum from any crate, an AI can easily generate:

1. **Error code constants** - Sequential numbering starting at 100
2. **Mapper function** - Pattern matching on error variants
3. **Const declaration** - Simple `const ERROR_MAPPER = ...`

### Example Prompt for AI

```
Generate cimpl bindings for the `reqwest` crate error type.

1. Declare error codes starting at 100
2. Create a mapper function for reqwest::Error
3. Declare the ERROR_MAPPER const
```

**AI Output:**

```rust
#[no_mangle]
pub static ERROR_REQWEST_TIMEOUT: i32 = 100;
#[no_mangle]
pub static ERROR_REQWEST_REQUEST: i32 = 101;
#[no_mangle]
pub static ERROR_REQWEST_CONNECT: i32 = 102;

fn map_reqwest_error(e: &reqwest::Error) -> (i32, &'static str) {
    if e.is_timeout() {
        (ERROR_REQWEST_TIMEOUT, "Timeout")
    } else if e.is_request() {
        (ERROR_REQWEST_REQUEST, "RequestError")
    } else if e.is_connect() {
        (ERROR_REQWEST_CONNECT, "ConnectionError")
    } else {
        (199, "HttpError")
    }
}

const ERROR_MAPPER: fn(&reqwest::Error) -> (i32, &'static str) = map_reqwest_error;
```

## Migration from Table Pattern

If you're using the old `define_error_codes!` table pattern:

### Before (Table Pattern)
```rust
define_error_codes! {
    error_type: uuid::Error,
    table_name: UUID_ERROR_TABLE,
    codes: {
        _ => ("ParseError", ERROR_UUID_PARSE_ERROR),
    }
}

let uuid = ok_or_return_null_with_table!(Uuid::from_str(&s_str), UUID_ERROR_TABLE);
```

### After (Mapper Pattern)
```rust
fn map_uuid_error(_e: &uuid::Error) -> (i32, &'static str) {
    (ERROR_UUID_PARSE_ERROR, "ParseError")
}
const ERROR_MAPPER: fn(&uuid::Error) -> (i32, &'static str) = map_uuid_error;

let uuid = ok_or_return_null!(Uuid::from_str(&s_str));
```

**Note**: The table pattern still works! You can use `ok_or_return_*_with_table!` macros if you prefer.

## Best Practices

1. **Always declare infrastructure error codes** (0-99) even if you don't use them
2. **Start library errors at 100** and increment sequentially
3. **Use descriptive error names** that match the original error variant when possible
4. **Test your mapper function** - it's just a normal Rust function!
5. **Document error codes** in your C header for language binding generators

## Summary

The error mapper pattern provides:
- ✅ **Transparency** - No macro magic, just functions
- ✅ **Flexibility** - Full Rust pattern matching power
- ✅ **Simplicity** - One mapper, clean macros everywhere
- ✅ **AI-Friendly** - Easy to generate automatically

This is now the recommended approach for error handling in `cimpl` bindings!
