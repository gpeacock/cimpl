# Table-Based Error Mapping - Final Implementation

## Overview

Successfully implemented a **table-based error mapping system** that allows library authors to declare error mappings once and have them automatically applied throughout their FFI code.

## The Complete Pattern

### Step 1: Declare Error Constants (cbindgen visibility)
```rust
// Manually declare so cbindgen sees them
#[no_mangle]
pub static ERROR_UUID_PARSE_ERROR: i32 = 100;
```

### Step 2: Create Mapping Table (single declaration)
```rust
define_error_codes! {
    error_type: uuid::Error,
    table_name: UUID_ERROR_TABLE,
    codes: {
        // Pattern => (Name, Constant)
        InvalidLength(_) => ("ParseError", ERROR_UUID_PARSE_ERROR),
        InvalidCharacter(_, _) => ("InvalidChar", ERROR_UUID_INVALID_CHAR),
        _ => ("ParseError", ERROR_UUID_PARSE_ERROR),  // Catch-all
    }
}
```

### Step 3: Use Automatically in FFI (clean!)
```rust
pub extern "C" fn uuid_parse(s: *const c_char) -> *mut Uuid {
    let s_str = cstr_or_return_null!(s);
    let uuid = ok_or_return_null_with_table!(Uuid::from_str(&s_str), UUID_ERROR_TABLE);
    box_tracked!(uuid)
}
```

## What Was Implemented

### 1. Table-Generating Macro
`define_error_codes!` generates a static lookup table:
```rust
static UUID_ERROR_TABLE: &[(fn(&uuid::Error) -> bool, &str, i32)] = &[
    (|e| matches!(e, uuid::Error::InvalidLength(_)), "ParseError", 100),
    // ...
];
```

### 2. Generic Conversion Method
```rust
impl Error {
    pub fn from_table<E: std::fmt::Display>(
        e: E,
        table: &[(fn(&E) -> bool, &str, i32)],
    ) -> Self {
        for (matcher, name, code) in table {
            if matcher(&e) {
                return Error::LibraryError(*code, format!("{}: {}", name, e));
            }
        }
        Error::Other(format!("Other: {}", e))
    }
}
```

### 3. Automatic Result Handling Macros
New macros that combine Result unwrapping with table-based error conversion:
- `ok_or_return_with_table!(result, table, err_val)`
- `ok_or_return_null_with_table!(result, table)`
- `ok_or_return_neg_with_table!(result, table)`
- `ok_or_return_zero_with_table!(result, table)`
- `ok_or_return_false_with_table!(result, table)`

## Benefits

✅ **Single source of truth**: Error constants declared once  
✅ **cbindgen compatible**: Constants are visible to cbindgen  
✅ **Data-driven**: Minimal generated code (just a table)  
✅ **Generic**: One `Error::from_table()` method handles all types  
✅ **Automatic**: Macros handle error conversion transparently  
✅ **Clean code**: Compare before/after:

**Before:**
```rust
match Uuid::from_str(&s_str) {
    Ok(uuid) => box_tracked!(uuid),
    Err(e) => {
        Error::Other(format!("Invalid UUID: {}", e)).set_last();
        std::ptr::null_mut()
    }
}
```

**After:**
```rust
let uuid = ok_or_return_null_with_table!(Uuid::from_str(&s_str), UUID_ERROR_TABLE);
box_tracked!(uuid)
```

## Error Output

C code sees:
```
Error 100: ParseError: invalid character: expected an optional prefix of `urn:uuid:`...
```

Where:
- **100** = `ERROR_UUID_PARSE_ERROR` (programmatic check in C)
- **"ParseError:"** = Variant name from table
- **Rest** = Original error message from `uuid::Error`

## Architecture

```
┌─────────────────────────────────────────┐
│ Developer declares constants            │
│ #[no_mangle] pub static ERROR_* = 100; │
└────────────┬────────────────────────────┘
             │
             ├─────────────┐
             │             │
┌────────────▼──────┐  ┌──▼──────────────────────┐
│ cbindgen sees     │  │ define_error_codes!     │
│ and exports to .h │  │ generates table         │
└───────────────────┘  └──┬──────────────────────┘
                          │
            ┌─────────────▼──────────────┐
            │ UUID_ERROR_TABLE            │
            │ [(matcher, name, code), ...] │
            └─────────────┬───────────────┘
                          │
         ┌────────────────▼────────────────┐
         │ Error::from_table() uses table  │
         │ to convert errors               │
         └─────────────┬───────────────────┘
                       │
      ┌────────────────▼─────────────────────┐
      │ ok_or_return_*_with_table! macros     │
      │ automatically apply conversion        │
      └───────────────────────────────────────┘
```

## Files Changed

- `src/error.rs`: Added `Error::LibraryError` variant and `from_table()` method
- `src/macros.rs`: Added `define_error_codes!` and `ok_or_return_*_with_table!` macros
- `uuid-example/src/lib.rs`: Demonstrates the complete pattern

## Usage Guidelines

1. **Declare constants first** (for cbindgen)
2. **Create one mapping table** per error type
3. **Use `ok_or_return_*_with_table!` macros** in FFI functions
4. **Add constants to cbindgen.toml** `after_includes` section

## Result

A clean, declarative, table-driven error mapping system that:
- Requires minimal boilerplate
- Works seamlessly with cbindgen
- Provides automatic error conversion
- Maintains full error detail from source crates
- Gives C code programmatic error codes

Perfect for wrapping any Rust crate with meaningful errors!
