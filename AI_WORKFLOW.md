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

## Common Anti-Patterns to AVOID

### ❌ DON'T: Manual null checks
```rust
// BAD
let ctx_ref = unsafe {
    if ctx.is_null() {
        Error::from(CimplError::NullParameter("ctx".to_string())).set_last();
        return -1;
    }
    &mut *ctx
};
```

### ✅ DO: Use deref macros
```rust
// GOOD
let ctx_ref = deref_mut_or_return_neg!(ctx, C2paContext);
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
        Error::from(MyInternalError::from(e)).set_last();
        -1
    }
}
```

### ✅ DO: Use ok_or_return macros
```rust
// GOOD
ok_or_return!(
    some_operation().map_err(MyInternalError::from),
    |value| {
        do_something(value);
        0
    },
    -1
)
```

## Complete Macro Reference

### Pointer Handling
- `ptr_or_return!(ptr, err_val)` - Check pointer not null
- `deref_or_return!(ptr, Type, err_val)` - Deref immutable pointer
- `deref_or_return_neg!(ptr, Type)` - Deref immutable, return -1
- `deref_mut_or_return!(ptr, Type, err_val)` - Deref mutable pointer
- `deref_mut_or_return_neg!(ptr, Type)` - Deref mutable, return -1

### String Conversion
- `cstr_or_return!(ptr, err_val)` - C string to Rust String
- `cstr_or_return_null!(ptr)` - C string, return NULL on error
- `to_c_string(s)` - Rust String to C string
- `option_to_c_string!(opt)` - Option<String> to C (NULL if None)

### Result Handling
- `ok_or_return!(result, transform, err_val)` - Handle Result with transform
- `ok_or_return_null!(result)` - Handle Result, return NULL
- `ok_or_return_int!(result)` - Handle Result, return -1
- `ok_or_return_zero!(result)` - Handle Result, return 0
- `ok_or_return_false!(result)` - Handle Result, return false

### Option Handling
- `some_or_return!(opt, error, err_val)` - Handle Option with custom error
- `some_or_return_null!(opt, error)` - Handle Option, return NULL
- `some_or_return_other_null!(opt, msg)` - Handle Option with Error::other message

### Object Creation
- `box_tracked!(value)` - Heap allocate and return tracked pointer
- `cimpl_free(ptr)` - Free tracked pointer

## Decision Tree for Common Patterns

### "I need to validate a pointer parameter"
1. Immutable access? → `deref_or_return_neg!(ptr, Type)`
2. Mutable access? → `deref_mut_or_return_neg!(ptr, Type)`
3. Just null check? → `ptr_or_return!(ptr, -1)`

### "I need to handle a Result"
1. Simple case (return NULL on error)? → `ok_or_return_null!(result)`
2. Need to transform success value? → `ok_or_return!(result, |v| transform(v), err_val)`
3. External error needs conversion? → Use `.map_err()` first, then macro

### "I need to handle an Option"
1. Validation error? → `some_or_return_other_null!(opt, "reason")`
2. Custom error? → `some_or_return_null!(opt, Error::specific(...))`

### "I need to convert strings"
1. C → Rust? → `cstr_or_return!(ptr, -1)`
2. Rust → C? → `to_c_string(s)`
3. Optional string → C? → `option_to_c_string!(opt)`

## Error Conversion Pattern

When working with external crate errors that don't directly implement `From<ExtErr> for cimpl::Error`:

```rust
// Define internal error wrapper
enum MyInternalError {
    ExternalError(external::Error),
}

// Implement From for the wrapper
impl From<external::Error> for MyInternalError {
    fn from(e: external::Error) -> Self {
        MyInternalError::ExternalError(e)
    }
}

// Implement From to cimpl::Error
impl From<MyInternalError> for cimpl::Error {
    fn from(e: MyInternalError) -> Self {
        // Map to error codes
    }
}

// Use with .map_err() in macros
ok_or_return_null!(
    external_operation().map_err(MyInternalError::from)
)
```

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
- [ ] External errors use `.map_err(InternalError::from)`
- [ ] Not manually calling `Error::from()` in match arms

### 4. Return Value Check
- [ ] Returning `-1`? → Use `_neg` or `_int` suffix
- [ ] Returning `NULL`? → Use `_null` suffix
- [ ] Returning `0`? → Use `_zero` suffix
- [ ] Returning `false`? → Use `_false` suffix

---

## Quick Reference: Read This First

Before writing any FFI function:
1. Read the banner at the top of the file you're editing
2. Run the pre-flight scan above
3. Check `src/macros.rs` lines 1-150 (the documentation)
4. Search for similar patterns in existing examples

The time spent understanding macros is recovered many times over in consistency, safety, and maintainability.
