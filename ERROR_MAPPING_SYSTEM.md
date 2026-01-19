# Error Mapping System - Implementation Summary

## Overview

Successfully implemented a library error mapping system for `cimple` that allows library authors to map their crate's errors to integer error codes for C FFI.

## What Was Implemented

### 1. Enhanced Error Type
Added `Error::LibraryError(i32, String)` variant to support custom error codes:

```rust
pub enum Error {
    // ... existing variants ...
    #[error("{1}")]
    LibraryError(i32, String),
}
```

### 2. Error Code System
- **0-99**: Core cimple infrastructure errors (consistent across all libraries)
  - 0: ERROR_OK
  - 1: ERROR_NULL_PARAMETER
  - 2: ERROR_STRING_TOO_LONG
  - 3: ERROR_INVALID_HANDLE
  - 4: ERROR_WRONG_HANDLE_TYPE
  - 5: ERROR_OTHER
- **100+**: Library-specific errors (defined by each wrapper library)

### 3. Two New Macros

#### `map_error!` - Maps library errors to codes with variant names
```rust
map_error!(e, {
    uuid::Error::InvalidLength(_) => (100, "InvalidLength"),
    uuid::Error::InvalidCharacter(..) => (101, "InvalidCharacter"),
})
```

**Features**:
- Automatically formats as "VariantName: original error message"
- Falls back to ERROR_OTHER (5) for unmatched errors
- Uses Display trait for the original error message

#### `define_error_codes!` - Generates C constants
```rust
define_error_codes! {
    prefix: "UUID",
    codes: {
        ParseError = 100,
    }
}
```

**Generates**:
```rust
#[no_mangle]
pub static ERROR_UUID_PARSE_ERROR: i32 = 100;
```

## Usage Pattern

In your FFI library (e.g., uuid-example):

```rust
// 1. Define error codes
define_error_codes! {
    prefix: "UUID",
    codes: {
        ParseError = 100,
    }
}

// 2. Map errors in FFI functions
#[no_mangle]
pub extern "C" fn uuid_parse(s: *const c_char) -> *mut Uuid {
    let s_str = cstr_or_return_null!(s);
    match Uuid::from_str(&s_str) {
        Ok(uuid) => box_tracked!(uuid),
        Err(e) => {
            map_error!(e, {
                _ => (100, "ParseError"),
            }).set_last();
            std::ptr::null_mut()
        }
    }
}

// 3. Add to cbindgen.toml so C can use the constant
after_includes = """
extern const int32_t ERROR_UUID_PARSE_ERROR;
"""
```

## Result

C code now sees:
```c
Error 100: ParseError: invalid character: expected an optional prefix of `urn:uuid:` followed by [0-9a-fA-F-], found `n` at 1
```

Where:
- **100** = ERROR_UUID_PARSE_ERROR (library-specific, can be checked programmatically)
- **"ParseError:"** = Variant name prefix (for quick identification)
- **Rest** = Original detailed error message from uuid crate

## Benefits

✅ **Library-first design**: Each library owns its error space (100+)  
✅ **Explicit control**: Developer chooses which errors get specific codes  
✅ **Preserves detail**: Original error messages are kept intact  
✅ **C-friendly**: Integer codes for programmatic handling  
✅ **Discoverable**: Constants exported to C header  
✅ **Flexible**: Unknown errors gracefully fall back to ERROR_OTHER  
✅ **No boilerplate**: Macros handle the repetitive work  

## Files Changed

- `src/error.rs`: Added `LibraryError` variant and `code_as_i32()` method
- `src/macros.rs`: Added `map_error!` and `define_error_codes!` macros
- `src/lib.rs`: Re-exported `paste` crate for macro use
- `Cargo.toml`: Added `paste` dependency
- `uuid-example/src/lib.rs`: Demonstrated usage with UUID parse errors
- `uuid-example/cbindgen.toml`: Added ERROR_UUID_PARSE_ERROR constant

## Next Steps

This pattern can now be used by any library wrapping external Rust crates:
- `regex-c`: Map regex::Error to ERROR_REGEX_*
- `serde-c`: Map serde::Error to ERROR_SERDE_*
- Any crate with meaningful error variants!

The developer just needs to:
1. Identify which errors they want C code to handle programmatically
2. Use `define_error_codes!` to assign codes
3. Use `map_error!` in FFI functions to map errors
4. Add the constants to cbindgen.toml

Simple, explicit, and powerful!
