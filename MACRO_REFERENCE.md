# Cimple FFI Macro Reference

Quick reference for all `cimple` FFI macros. All macros follow the pattern: `<action>_or_return_<suffix>`

## Macro Categories

### 1. Pointer Null Check (`ptr_or_return_*`)
Just checks if pointer is NULL, no type validation.

```rust
ptr_or_return!(ptr, error_value)
ptr_or_return_null!(ptr)         // Returns NULL
ptr_or_return_int!(ptr)          // Returns -1
```

### 2. C String Conversion (`cstr_or_return_*`)
Converts C string to Rust `String` with bounded read (max 64KB).

```rust
cstr_or_return!(ptr, error_value)
cstr_or_return_null!(ptr)        // Returns NULL
cstr_or_return_int!(ptr)         // Returns -1
```

### 3. Immutable Dereference (`deref_or_return_*`)
Validates pointer type and returns `&T`.

```rust
deref_or_return!(ptr, Type, error_value)
deref_or_return_null!(ptr, Type)     // Returns NULL
deref_or_return_neg!(ptr, Type)      // Returns -1
deref_or_return_zero!(ptr, Type)     // Returns 0
deref_or_return_false!(ptr, Type)    // Returns false
```

### 4. Mutable Dereference (`deref_mut_or_return_*`)
Validates pointer type and returns `&mut T`.

```rust
deref_mut_or_return!(ptr, Type, error_value)
deref_mut_or_return_null!(ptr, Type)  // Returns NULL
deref_mut_or_return_neg!(ptr, Type)   // Returns -1
```

## Usage Patterns

### One-Liner Operations
Call methods directly on the dereferenced pointer:

```rust
#[no_mangle]
pub extern "C" fn mystring_len(ptr: *mut MyString) -> usize {
    deref_or_return_zero!(ptr, MyString).len()
}
```

### Multi-Line Operations
Bind to a variable using `let`:

```rust
#[no_mangle]
pub extern "C" fn mystring_get_info(
    ptr: *mut MyString,
    out_len: *mut usize
) -> *mut c_char {
    let obj = deref_or_return_null!(ptr, MyString);
    
    if !out_len.is_null() {
        unsafe { *out_len = obj.value.len() };
    }
    
    to_c_string(obj.value.clone())
}
```

### Mutable Operations
Use `deref_mut_or_return_*` for mutations:

```rust
#[no_mangle]
pub extern "C" fn mystring_set_value(
    ptr: *mut MyString,
    new_value: *const c_char
) -> i32 {
    let obj = deref_mut_or_return_neg!(ptr, MyString);
    let value_str = cstr_or_return_int!(new_value);
    obj.value = value_str;
    0
}
```

## Suffix Guide

| Suffix | Return Value | Use For |
|--------|--------------|---------|
| `_null` | `NULL` | Pointer-returning functions |
| `_neg` | `-1` | Integer functions (error code) |
| `_int` | `-1` | Alias for `_neg` (C string ops) |
| `_zero` | `0` | Integer functions (count/length) |
| `_false` | `false` | Boolean functions |
| (base) | Custom | Any custom error value |

## Decision Tree

```
What do you need?
│
├─ Just NULL check? → ptr_or_return_*
│
├─ Convert C string? → cstr_or_return_*
│
└─ Validated pointer?
   ├─ Read-only? → deref_or_return_*
   └─ Mutable?  → deref_mut_or_return_*
   
One-liner or multi-line?
├─ One-liner  → Call method directly
└─ Multi-line → Bind with `let obj = ...`
```

## Complete Example

```rust
use cimple::{
    box_tracked,
    cstr_or_return_int,
    cstr_or_return_null,
    deref_or_return_null,
    deref_or_return_zero,
    deref_mut_or_return_neg,
    to_c_string,
};

pub struct MyString {
    value: String,
}

// Constructor
#[no_mangle]
pub extern "C" fn mystring_create(initial: *const c_char) -> *mut MyString {
    let initial_str = cstr_or_return_null!(initial);
    box_tracked!(MyString { value: initial_str })
}

// Simple getter (one-liner)
#[no_mangle]
pub extern "C" fn mystring_len(ptr: *mut MyString) -> usize {
    deref_or_return_zero!(ptr, MyString).len()
}

// Getter with conversion (bind to variable)
#[no_mangle]
pub extern "C" fn mystring_get_value(ptr: *mut MyString) -> *mut c_char {
    let obj = deref_or_return_null!(ptr, MyString);
    to_c_string(obj.value.clone())
}

// Setter (mutable, bind to variable)
#[no_mangle]
pub extern "C" fn mystring_set_value(
    ptr: *mut MyString,
    new_value: *const c_char
) -> i32 {
    let obj = deref_mut_or_return_neg!(ptr, MyString);
    let value_str = cstr_or_return_int!(new_value);
    obj.value = value_str;
    0
}

// Destructor (use universal cimple_free)
#[no_mangle]
pub extern "C" fn mystring_free(ptr: *mut MyString) -> i32 {
    cimple::cimple_free(ptr as *mut std::ffi::c_void)
}
```

## Key Principles

1. **Consistent naming** - Pattern is always `<action>_or_return_<suffix>`
2. **Explicit control flow** - `_or_return_` makes early returns obvious
3. **Standard Rust** - Use `let` bindings for multi-line operations
4. **Type safety** - All derefs validate pointer type automatically
5. **Error reporting** - All macros set `last_error` on failure

## See Also

- `MACRO_PATTERNS.md` - Detailed documentation with more examples
- `MACRO_SIMPLIFICATION.md` - History of API changes
- `POINTER_REDESIGN.md` - Pointer registry architecture
