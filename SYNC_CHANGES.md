# Sync Changes from c2pa-rs

This document summarizes the changes synchronized from the c2pa-rs version of cimpl back to the standalone cimpl library.

## Branch: `sync-from-c2pa-rs`

Created from: `main`
Date: 2026-02-08
Commits: b6851e1 (initial sync), a8aa497 (compatibility fixes)

## Overview

The c2pa-rs project has been extensively testing and refining the cimpl FFI utilities. This sync brings those battle-tested improvements back to the standalone library while maintaining **drop-in compatibility** - code using c2pa-rs cimpl can switch to standalone cimpl without modifications.

## Key Changes

### 1. Simplified Error Conversion in Macros (`src/macros.rs`) ⚠️ BREAKING CHANGE

**Changed `ok_or_return!` macro behavior:**

The macro now converts errors to strings using the `Display` trait instead of requiring `From<E> for CimplError` implementations:

```rust
// Before: Required From trait implementation
impl From<uuid::Error> for CimplError {
    fn from(e: uuid::Error) -> Self { ... }
}

// After: No From trait needed - automatic string conversion
let uuid = ok_or_return_null!(Uuid::from_str(&s));
// Error message comes from format!("{}", uuid::Error)
```

**Benefits:**
- Works with ANY error type that implements `Display`
- No need to implement `From` traits for external crate errors
- Simpler and more flexible
- Consistent with c2pa-rs battle-tested approach

**Impact:** Existing `From<E> for CimplError` implementations are no longer used by the macros. They can be removed unless used elsewhere.

### 2. Enhanced Error Types (`src/cimpl_error.rs`)

**Added new error constructor methods:**
- `code()` - Returns the error code
- `message()` - Returns the error message  
- `untracked_pointer(ptr)` - Replaces `invalid_handle()`
- `wrong_pointer_type(ptr)` - Replaces `wrong_handle_type()`
- `mutex_poisoned()` - For mutex lock failures (error code 6)
- `invalid_buffer_size(size, param)` - For buffer validation (error code 7)

**Rationale:** More descriptive error names that better reflect the actual error conditions. "Untracked pointer" is more accurate than "invalid handle" when pointer tracking fails.

### 2. Automatic Memory Leak Detection (`src/utils.rs`)

**Added `Drop` implementation for `PointerRegistry`:**

When the program shuts down, the registry now automatically checks for unfreed pointers and reports them:

```
⚠️  WARNING: 3 pointer(s) were not freed at shutdown!
This indicates C code did not properly free all allocated pointers.
Each pointer should be freed exactly once with cimpl_free().
```

**Benefits:**
- Catches forgotten `cimpl_free()` calls during development
- Runs in all builds (debug, release, test)
- Helps identify resource leaks early

### 3. Test-Mode Debugging (`src/utils.rs`)

**Enhanced `cimpl_free` with test-mode error output:**

In `#[cfg(test)]` builds, `cimpl_free` now prints detailed diagnostics to stderr when it fails:

```
⚠️  ERROR: cimpl_free failed for pointer 0x12345678: pointer not tracked
This usually means:
1. The pointer was not allocated with box_tracked!/track_box
2. The pointer was already freed (double-free)
3. The pointer is invalid/corrupted
```

**Benefits:**
- Immediate feedback during testing
- Helps diagnose memory management bugs
- No stderr spam in production builds

### 4. Improved Error Handling (`src/utils.rs`)

**Updated validation methods to use mutex error handling:**
- `validate()` now returns `CimplError::mutex_poisoned()` on lock failures
- `free()` now returns `CimplError::mutex_poisoned()` on lock failures
- More specific error for buffer size validation

**Better error names:**
- `untracked_pointer` instead of `invalid_handle`
- `wrong_pointer_type` instead of `wrong_handle_type`

### 5. Enhanced Documentation (`src/lib.rs` and `src/macros.rs`)

**Added comprehensive documentation sections:**

1. **Memory Safety section** explaining leak detection
2. **Test Mode Debugging & Leak Detection** explaining the debug features
3. **Why These Errors Matter** explaining the severity of memory bugs
4. **Production Behavior** clarifying what happens in release builds

**Updated Quick Reference to include:**
- Byte array validation macros
- Clearer distinction between `_int` (returns -1) vs `_neg` (also -1, deprecated)
- Better examples showing real-world usage patterns

### 6. New Macros (`src/macros.rs`)

**Added byte array validation macros:**

```rust
bytes_or_return!(ptr, len, "param_name", error_value)
bytes_or_return_null!(ptr, len, "param_name")
bytes_or_return_int!(ptr, len, "param_name")
```

These macros safely validate and convert C byte arrays (`*const c_uchar` + length) to Rust slices with:
- Null pointer checking
- Buffer size validation
- Overflow prevention
- Early return on error

**Use case:** Processing binary data passed from C code.

### 7. Consistent Macro Naming

**Standardized on `_int` suffix:**

The c2pa-rs version consistently uses `_int` (returns -1) instead of the older `_neg`. This branch keeps both for compatibility but the documentation now emphasizes `_int`:

- `deref_or_return_int!` (preferred) vs `deref_or_return_neg!` (older)
- `deref_mut_or_return_int!` (preferred)
- All new macros use `_int` suffix

## Files Changed

```
 src/cimpl_error.rs |  30 +++++++++++--
 src/lib.rs         |  30 ++++++++++---
 src/macros.rs      | 120 +++++++++++++++++++++++++++++++++++++++
 src/utils.rs       | 129 ++++++++++++++++++++++++++++++++++++++++
 4 files changed, 273 insertions(+), 36 deletions(-)
```

## What Was NOT Changed

These items maintain compatibility while staying independent:

1. **Type aliases** - Uses independent `Result<T> = std::result::Result<T, CimplError>` instead of c2pa's Result type. This is expected and doesn't affect API compatibility.

2. **Module structure** - c2pa-rs uses `mod.rs` (as a submodule) while standalone uses `lib.rs` (as a crate root). The public API is identical.

3. **Extra helper macros** - We kept the `some_or_return_other_*` convenience macros which aren't in c2pa-rs but don't affect compatibility (they're optional additions).

4. **`vec_to_tracked_ptr!` macro** - Not included per user preference; use the `to_c_bytes()` function instead which is equivalent.

## Drop-In Compatibility ✅

Code written for c2pa-rs cimpl will work with standalone cimpl without changes:
- All macro names match exactly (`_int` not `_neg`)
- All function signatures match
- All error types and methods match
- Behavior is identical

## Testing

Before merging this branch:

1. **Run tests:**
   ```bash
   cargo test
   ```

2. **Check documentation:**
   ```bash
   cargo doc --open
   ```

3. **Test in examples:**
   - Update `c2pa-example` to use the new error names
   - Update `reference-example` if needed
   - Run example builds

4. **Verify macro behavior:**
   - Test byte array macros with various buffer sizes
   - Verify leak detection works
   - Confirm test-mode debugging output

## Migration Guide

If you have existing code using cimpl, here's what needs updating:

### 1. Error Conversion - From Trait No Longer Needed

```rust
// OLD: Had to implement From trait for every external error
impl From<uuid::Error> for CimplError {
    fn from(e: uuid::Error) -> Self {
        CimplError::new(100, format!("ParseError: {}", e))
    }
}

// NEW: Just use the macro - automatic string conversion
let uuid = ok_or_return_null!(Uuid::from_str(&s));
// Error automatically becomes: CimplError::other("invalid uuid...")
```

**If you have custom error codes:** You can still map errors manually if needed:

```rust
let uuid = match Uuid::from_str(&s) {
    Ok(v) => v,
    Err(e) => {
        CimplError::new(100, format!("ParseError: {}", e)).set_last();
        return std::ptr::null_mut();
    }
};
```

### 2. Error Constructor Changes

```rust
// Old (still works but deprecated)
CimplError::invalid_handle(ptr as u64)
CimplError::wrong_handle_type(ptr as u64)

// New (preferred)
CimplError::untracked_pointer(ptr as u64)
CimplError::wrong_pointer_type(ptr as u64)
```

### 3. New Byte Array Validation

```rust
// Instead of manual validation:
if data.is_null() {
    return std::ptr::null_mut();
}
let bytes = unsafe { std::slice::from_raw_parts(data, len) };

// Use the macro:
let bytes = bytes_or_return_null!(data, len, "data");
```

## Benefits Summary

1. **Better debugging** - Automatic leak detection and test-mode error output
2. **Improved safety** - Better buffer validation and clearer error messages
3. **Enhanced documentation** - More examples and clearer explanations
4. **More features** - Byte array macros for binary data handling
5. **Battle-tested** - These changes have been used extensively in c2pa-rs

## Next Steps

1. Merge this branch to main after testing
2. Update all examples to use new error names
3. Update AI_WORKFLOW.md to mention new features
4. Consider adding examples of byte array usage
5. Bump version to 0.2.0 (breaking changes to error names)
