# Cimple Macro Patterns

This document explains the consistent naming patterns used in `cimple` macros for FFI bindings.

## Naming Convention

All macros follow a consistent pattern: `<action>_or_return_<suffix>`

### Actions
- **`ptr`** - Check pointer is not NULL (no validation)
- **`cstr`** - Convert C string to Rust `String`
- **`deref`** - Validate pointer and return reference
- **`deref_mut`** - Validate pointer and return mutable reference

### Suffixes
- **`_null`** - Returns `NULL` (`std::ptr::null_mut()`) on error
- **`_neg`** - Returns `-1` on error
- **`_zero`** - Returns `0` on error
- **`_false`** - Returns `false` on error
- **`_int`** - Alias for `_neg` (for C string conversion)
- **(base)** - Custom return value specified as last parameter

## 1. Pointer Null Checks

Check if a pointer is NULL, return early if so. No type validation.

```rust
/// Check pointer, custom return value
ptr_or_return!(ptr, error_value)

/// Check pointer, return NULL
ptr_or_return_null!(ptr)

/// Check pointer, return -1
ptr_or_return_int!(ptr)
```

**Example:**
```rust
#[no_mangle]
pub extern "C" fn mylib_free(ptr: *mut MyType) -> i32 {
    ptr_or_return_int!(ptr);  // Just null check
    // ... custom cleanup logic ...
    0
}
```

## 2. C String Conversion

Convert C string to Rust `String` with bounded read (max 64KB).

```rust
/// Convert C string, custom return value
cstr_or_return!(ptr, error_value)

/// Convert C string, return NULL
cstr_or_return_null!(ptr)

/// Convert C string, return -1
cstr_or_return_int!(ptr)
```

**Example:**
```rust
#[no_mangle]
pub extern "C" fn mylib_process(input: *const c_char) -> *mut c_char {
    let input_str = cstr_or_return_null!(input);
    to_c_string(process(input_str))
}
```

## 3. Deref Macros - Validate and Return Reference

Validate pointer type and return a reference. Use for both one-liners and multi-line operations.

### Immutable Reference

```rust
/// Validate and deref, custom return value
deref_or_return!(ptr, Type, error_value)

/// Validate and deref, return NULL
deref_or_return_null!(ptr, Type)

/// Validate and deref, return -1
deref_or_return_neg!(ptr, Type)

/// Validate and deref, return 0
deref_or_return_zero!(ptr, Type)

/// Validate and deref, return false
deref_or_return_false!(ptr, Type)
```

**Example:**
```rust
// One-liner
#[no_mangle]
pub extern "C" fn mystring_len(ptr: *mut MyString) -> usize {
    deref_or_return_zero!(ptr, MyString).len()
}

// Multi-line with binding
#[no_mangle]
pub extern "C" fn mystring_is_complex(ptr: *mut MyString) -> bool {
    let obj = deref_or_return_false!(ptr, MyString);
    obj.value.len() > 100 && obj.value.contains("complex")
}

// Multi-line with multiple operations
#[no_mangle]
pub extern "C" fn mystring_get_info(
    ptr: *mut MyString,
    out_len: *mut usize
) -> *mut c_char {
    let obj = deref_or_return_null!(ptr, MyString);
    
    // Set output parameter
    if !out_len.is_null() {
        unsafe { *out_len = obj.value.len() };
    }
    
    to_c_string(obj.value.clone())
}
```

### Mutable Reference

```rust
/// Validate and deref mutably, custom return value
deref_mut_or_return!(ptr, Type, error_value)

/// Validate and deref mutably, return NULL
deref_mut_or_return_null!(ptr, Type)

/// Validate and deref mutably, return -1
deref_mut_or_return_neg!(ptr, Type)
```

**Example:**
```rust
// One-liner (rare for mutable operations)
#[no_mangle]
pub extern "C" fn mystring_clear(ptr: *mut MyString) -> i32 {
    deref_mut_or_return_neg!(ptr, MyString).value.clear();
    0
}

// Multi-line with binding (common pattern)
#[no_mangle]
pub extern "C" fn mystring_set_value(
    ptr: *mut MyString,
    new_value: *const c_char
) -> i32 {
    let obj = deref_mut_or_return_neg!(ptr, MyString);
    let new_value_str = cstr_or_return_int!(new_value);
    obj.set_value(new_value_str);
    0
}
```

## Decision Tree: Which Macro to Use?

```
Need to validate a pointer?
├─ No → Use ptr_or_return_*
│
└─ Yes → Use deref_or_return_* or deref_mut_or_return_*
   ├─ One-liner operation? → Call method directly
   │  Example: deref_or_return_zero!(ptr, Type).len()
   │
   └─ Multi-line operation? → Bind to a variable
      Example: let obj = deref_mut_or_return_neg!(ptr, Type);
```

## Complete Example

```rust
use std::os::raw::c_char;
use cimple::{
    box_tracked,
    cstr_or_return_int,
    cstr_or_return_null,
    deref_or_return_null,
    deref_or_return_zero,
    guard_mut_or_return_neg,
    to_c_string,
};

pub struct MyString {
    value: String,
}

// Constructor - no validation needed
#[no_mangle]
pub extern "C" fn mystring_create(initial: *const c_char) -> *mut MyString {
    let initial_str = cstr_or_return_null!(initial);
    box_tracked!(MyString { value: initial_str })
}

// Simple getter - one-liner
#[no_mangle]
pub extern "C" fn mystring_len(ptr: *mut MyString) -> usize {
    deref_or_return_zero!(ptr, MyString).len()
}

// Getter with conversion - bind to variable
#[no_mangle]
pub extern "C" fn mystring_get_value(ptr: *mut MyString) -> *mut c_char {
    let obj = deref_or_return_null!(ptr, MyString);
    to_c_string(obj.value.clone())
}

// Setter - bind to mutable variable
#[no_mangle]
pub extern "C" fn mystring_set_value(ptr: *mut MyString, new_value: *const c_char) -> i32 {
    let obj = deref_mut_or_return_neg!(ptr, MyString);
    let new_value_str = cstr_or_return_int!(new_value);
    obj.value = new_value_str;
    0
}

// Complex operation - bind to variable for multiple uses
#[no_mangle]
pub extern "C" fn mystring_get_info(
    ptr: *mut MyString,
    out_len: *mut usize,
    out_capacity: *mut usize
) -> *mut c_char {
    let obj = deref_or_return_null!(ptr, MyString);
    
    // Set multiple output parameters
    if !out_len.is_null() {
        unsafe { *out_len = obj.value.len() };
    }
    if !out_capacity.is_null() {
        unsafe { *out_capacity = obj.value.capacity() };
    }
    
    to_c_string(obj.value.clone())
}
```

## Migration from Old Names

The old `validate_and_deref_*` macros are deprecated but still work:

| Old Name | New Name |
|----------|----------|
| `validate_and_deref!` | `deref_or_return_null!` |
| `validate_and_deref_mut!` | `deref_mut_or_return_null!` |
| `validate_and_deref_neg!` | `deref_or_return_neg!` |
| `validate_and_deref_mut_neg!` | `deref_mut_or_return_neg!` |

## Summary

The consistent naming pattern makes the API predictable:
- **Action**: What the macro does (`ptr`, `cstr`, `deref`, `deref_mut`)
- **`_or_return_`**: Makes control flow explicit
- **Suffix**: What value to return on error (`null`, `neg`, `zero`, `false`)

For multi-line operations, simply bind the result to a variable:
```rust
let obj = deref_or_return_null!(ptr, MyString);
```

This consistency helps both humans and AI tools understand and generate correct FFI code!
