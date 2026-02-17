# AI Workflow Guidelines for cimpl FFI Development

This document provides specific guidance for AI assistants working with the cimpl library.

## 🔍 PRE-FLIGHT SCAN (DO THIS FIRST!)

**Before submitting ANY FFI code**, run this pattern scan:

### Pattern Detection Checklist

Scan the code for these exact patterns and replace:

```regex
PATTERN 1: if.*\.is_null\(\).*Error.*return
→ REPLACE: deref_or_return! or deref_mut_or_return!

PATTERN 2: match.*\{.*Ok\(.*\).*=>.*Err\(.*\).*=>.*Error.*set_last.*return
→ REPLACE: ok_or_return! or ok_or_return_null!

PATTERN 3: unsafe.*\{.*if.*is_null.*&(mut)?\s*\*
→ REPLACE: deref_or_return! or deref_mut_or_return!

PATTERN 4: Manual string conversion with bounds checking
→ REPLACE: cstr_or_return!
```

### Literal Examples to Search For

Search your code for these literal strings:
- `if ptr.is_null()` → Should use a macro
- `if ctx.is_null()` → Should use a macro
- `match result { Ok` → Should use `ok_or_return!`
- `unsafe { &mut *` → Should use `deref_mut_or_return!`
- `unsafe { &*` → Should use `deref_or_return!`

**If you find ANY of these, you missed a macro. Go back and fix it.**

---

## Core Principle: ALWAYS Use Macros When Available

**CRITICAL**: Before writing any manual FFI pattern, check if a cimpl macro exists for it. The macros are in `src/macros.rs` and provide consistent error handling.

## Recommended Project Structure

**Separate your pure Rust API from FFI bindings:**

```
src/
  lib.rs     - Pure Rust API (no FFI concerns)
  ffi.rs     - C FFI wrapper using cimpl
```

**Why this matters:**
- ✅ Clear separation between "your library" and "FFI glue"
- ✅ Pure Rust API can be used by other Rust crates
- ✅ FFI layer is a thin wrapper, easy to review
- ✅ Shows how to add FFI to an existing Rust library

**Example:**

`lib.rs` - Standard Rust:
```rust
pub struct ValueConverter { /* ... */ }

impl ValueConverter {
    pub fn from_i32(value: i32) -> Self { /* ... */ }
    pub fn to_string(&self) -> Result<String, Error> { /* ... */ }
}
```

`ffi.rs` - FFI wrapper:
```rust
#[no_mangle]
pub extern "C" fn vc_from_i32(value: i32) -> *mut ValueConverter {
    box_tracked!(ValueConverter::from_i32(value))
}

#[no_mangle]
pub extern "C" fn vc_to_string(ptr: *mut ValueConverter) -> *mut c_char {
    let converter = deref_or_return_null!(ptr, ValueConverter);
    let result = ok_or_return_null!(converter.to_string());
    to_c_string(result)
}
```

See `examples/reference/` for a complete implementation of this pattern.

## Common Anti-Patterns to AVOID

### ❌ DON'T: Manual null checks
```rust
// BAD
let ctx_ref = unsafe {
    if ctx.is_null() {
        cimpl::Error::null_parameter("ctx").set_last();
        return -1;
    }
    &mut *ctx
};
```

### ✅ DO: Use deref macros
```rust
// GOOD
let ctx_ref = deref_mut_or_return_int!(ctx, C2paContext);
```

### ❌ DON'T: Manual Result matching
```rust
// BAD
match some_operation() {
    Ok(value) => {
        do_something(value);
        0
    }
    Err(e) => {
        cimpl::Error::from(e).set_last();
        -1
    }
}
```

### ✅ DO: Use ok_or_return macros
```rust
// GOOD - automatic error conversion via From trait
let value = ok_or_return_int!(some_operation());
do_something(value);
0

// Or with transformation
ok_or_return!(
    some_operation(),
    |value| {
        do_something(value);
        0
    },
    -1
)
```

### ❌ DON'T: Manual byte array validation
```rust
// BAD
if data.is_null() {
    cimpl::Error::null_parameter("data").set_last();
    return std::ptr::null_mut();
}
let bytes = unsafe { std::slice::from_raw_parts(data, len) };
```

### ✅ DO: Use bytes_or_return macro
```rust
// GOOD
let bytes = bytes_or_return_null!(data, len, "data");
```

## Complete Macro Reference

### Pointer Handling
- `ptr_or_return!(ptr, err_val)` - Check pointer not null
- `ptr_or_return_null!(ptr)` - Check pointer not null, return NULL
- `ptr_or_return_int!(ptr)` - Check pointer not null, return -1
- `deref_or_return!(ptr, Type, err_val)` - Deref immutable pointer
- `deref_or_return_null!(ptr, Type)` - Deref immutable, return NULL
- `deref_or_return_int!(ptr, Type)` - Deref immutable, return -1
- `deref_or_return_zero!(ptr, Type)` - Deref immutable, return 0
- `deref_or_return_false!(ptr, Type)` - Deref immutable, return false
- `deref_mut_or_return!(ptr, Type, err_val)` - Deref mutable pointer
- `deref_mut_or_return_null!(ptr, Type)` - Deref mutable, return NULL
- `deref_mut_or_return_int!(ptr, Type)` - Deref mutable, return -1

### String Conversion
- `cstr_or_return!(ptr, err_val)` - C string to Rust String with bounds check
- `cstr_or_return_null!(ptr)` - C string, return NULL on error
- `cstr_or_return_int!(ptr)` - C string, return -1 on error
- `to_c_string(s)` - Rust String to C string (tracked, must free)
- `option_to_c_string!(opt)` - Option<String> to C (NULL if None)

### Byte Array Handling
- `bytes_or_return!(ptr, len, name, err_val)` - Validate byte array
- `bytes_or_return_null!(ptr, len, name)` - Validate bytes, return NULL
- `bytes_or_return_int!(ptr, len, name)` - Validate bytes, return -1
- `to_c_bytes(vec)` - Rust Vec<u8> to C byte array (tracked, must free)

### Result Handling
- `ok_or_return!(result, transform, err_val)` - Handle Result with transform
- `ok_or_return_null!(result)` - Handle Result, return NULL
- `ok_or_return_int!(result)` - Handle Result, return -1
- `ok_or_return_zero!(result)` - Handle Result, return 0
- `ok_or_return_false!(result)` - Handle Result, return false

### Option Handling
- `some_or_return!(opt, error, err_val)` - Handle Option with custom error
- `some_or_return_null!(opt, error)` - Handle Option, return NULL
- `some_or_return_int!(opt, error)` - Handle Option, return -1
- `some_or_return_zero!(opt, error)` - Handle Option, return 0
- `some_or_return_false!(opt, error)` - Handle Option, return false
- `some_or_return_other_null!(opt, msg)` - Handle Option with Error::other message
- `some_or_return_other_int!(opt, msg)` - Handle Option with Error::other, return -1
- `some_or_return_other_zero!(opt, msg)` - Handle Option with Error::other, return 0
- `some_or_return_other_false!(opt, msg)` - Handle Option with Error::other, return false

### Object Creation
- `box_tracked!(value)` - Heap allocate and return tracked pointer
- `arc_tracked!(value)` - Arc allocate and return tracked pointer
- `cimpl_free!(ptr)` - Macro wrapper for cimpl::cimpl_free(ptr)

## Decision Tree for Common Patterns

### "I need to validate a pointer parameter"
1. Immutable access? → `deref_or_return_neg!(ptr, Type)`
2. Mutable access? → `deref_mut_or_return_neg!(ptr, Type)`
3. Just null check? → `ptr_or_return!(ptr, -1)`

### "I need to handle a Result"
1. Simple case (return NULL on error)? → `ok_or_return_null!(result)`
2. Return false on error? → `ok_or_return_false!(result)`
3. Return -1 on error? → `ok_or_return_int!(result)`
4. Need to transform success value? → `ok_or_return!(result, |v| transform(v), err_val)`
5. External error? → Just use the macro! The `From` trait handles conversion automatically

### "I need to handle an Option"
1. Validation error? → `some_or_return_other_null!(opt, "reason")`
2. Custom error? → `some_or_return_null!(opt, Error::specific(...))`

### "I need to convert strings"
1. C → Rust? → `cstr_or_return_null!(ptr)` or `cstr_or_return!(ptr, -1)`
2. Rust → C? → `to_c_string(s)` (returns tracked pointer, must free)
3. Optional string → C? → `option_to_c_string!(opt)` (NULL if None)

### "I need to handle byte arrays"
1. Validate input array? → `bytes_or_return_null!(ptr, len, "param_name")`
2. Return byte array to C? → `to_c_bytes(vec)` (returns tracked pointer, must free)

## Error Handling Pattern (String-Based)

cimpl uses **string-based errors** with the format `"VariantName: details"` for cross-language compatibility.

### Converting Your Errors to cimpl::Error

**Option 1: Automatic (Recommended)**
```rust
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum MyError {
    #[error("parse failed: {0}")]
    ParseError(String),
    
    #[error("validation failed")]
    ValidationError,
}

// Automatic conversion using from_error()
impl From<MyError> for cimpl::Error {
    fn from(e: MyError) -> Self {
        // Uses Debug for variant name, Display for message
        cimpl::Error::from_error(e)
    }
}

// Use with macros - automatic conversion!
ok_or_return_null!(my_operation())  // MyError → cimpl::Error via From
```

**Option 2: Manual Control**
```rust
impl From<MyError> for cimpl::Error {
    fn from(e: MyError) -> Self {
        match e {
            MyError::ParseError(s) => cimpl::Error::new("ParseError", s),
            MyError::ValidationError => cimpl::Error::new("ValidationError", "validation failed"),
        }
    }
}
```

### Error String Format

All errors follow: `"VariantName: message details"`

Examples:
- `"OutOfRange: need exactly 4 bytes for i32, got 2"`
- `"InvalidUtf8: invalid utf-8 sequence of 1 bytes from index 0"`
- `"BufferTooLarge: got 9 bytes, max 8"`

This format allows language bindings to parse the variant and create typed exceptions.

## Checklist Before Submitting Code

- [ ] All pointer dereferences use `deref_or_return!` or `deref_mut_or_return!`
- [ ] All C string conversions use `cstr_or_return!`
- [ ] All Result handling uses `ok_or_return!` variants
- [ ] All Option handling uses `some_or_return!` variants
- [ ] No manual `if ptr.is_null()` checks
- [ ] No manual `unsafe { &*ptr }` or `unsafe { &mut *ptr }`
- [ ] No manual `match result { Ok/Err }` patterns
- [ ] Error conversions use `.map_err()` when needed

## When Manual Code is Acceptable

It's okay to write manual code when:
1. The operation is unique and no macro fits
2. You're implementing new macro functionality
3. The macro would obscure rather than clarify the code

**But always ask first**: "Is there a macro for this?"

## Final Submission Checklist

Run this checklist on every FFI function before submitting:

### 1. Visual Scan
Look at your code. Do you see any of these?
- [ ] `if ptr.is_null()`
- [ ] `if ctx.is_null()`  
- [ ] `match result { Ok`
- [ ] `unsafe { &*`
- [ ] `unsafe { &mut *`
- [ ] Manual string length checks

If YES to any → **STOP. Use a macro.**

### 2. Import Check
- [ ] Did you import the macros you're using?
- [ ] Check your `use cimpl::{...}` statement

### 3. Error Conversion Check
- [ ] Implemented `From<MyError> for cimpl::Error` (use `cimpl::Error::from_error(e)` for automatic)
- [ ] Not manually calling `cimpl::Error::from()` in match arms (let the macros do it)

### 4. Return Value Check
- [ ] Returning `-1`? → Use `_neg` or `_int` suffix
- [ ] Returning `NULL`? → Use `_null` suffix
- [ ] Returning `0`? → Use `_zero` suffix
- [ ] Returning `false`? → Use `_false` suffix

---

## Quick Reference: Read This First

Before writing any FFI function:
1. **Check the reference example**: `examples/reference/src/ffi.rs` shows all patterns
2. Run the pre-flight scan above (lines 5-37)
3. Review the macro reference (lines 92+)
4. Check `src/macros.rs` lines 1-150 for detailed documentation

**Key files to study:**
- `examples/reference/src/lib.rs` - Clean Rust API
- `examples/reference/src/ffi.rs` - FFI wrapper showing ALL patterns
- `examples/reference/bindings/python/` - Language binding example

The time spent understanding macros is recovered many times over in consistency, safety, and maintainability.
