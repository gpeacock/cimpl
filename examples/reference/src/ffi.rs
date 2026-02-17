//! # C FFI Bindings for Value Converter Library
//!
//! ## Why This Example Exists
//!
//! Most Rust FFI examples are trivial toy code:
//! ```rust,ignore
//! #[no_mangle]
//! pub extern "C" fn add(a: i32, b: i32) -> i32 { a + b }
//! ```
//!
//! **This example is different.** It demonstrates **production-ready patterns** for
//! wrapping a real Rust library with C FFI.
//!
//! ## What Makes This Special
//!
//! 1. **Wraps a real Rust API** - Practical value conversion (see `lib.rs`)
//! 2. **Object-Oriented API** - Multiple constructors, conversion methods
//! 3. **Real Error Handling** - Result<T,E> for fallible conversions (overflow, invalid UTF-8, etc.)
//! 4. **Memory Safety** - Tracked allocations, type validation, leak detection
//! 5. **Modern Rust** - Uses `thiserror`, `cimpl::Error::from_error()` for automatic conversion
//!
//! ## Patterns Demonstrated
//!
//! - ✅ **Multiple constructors** - `from_i32()`, `from_string()`, `from_hex()`, etc.
//! - ✅ **Fallible conversions** - `to_i32()` returns Result (might be wrong size)
//! - ✅ **Type validation** - bytes might not be valid UTF-8 or numeric types
//! - ✅ **Byte arrays** in/out with length parameters
//! - ✅ **Error propagation** with descriptive messages
//! - ✅ **Automatic error formatting** with `thiserror` + `cimpl::Error::from_error()`
//!
//! Compare this file to `lib.rs` to see the separation between:
//! - Pure Rust API (lib.rs) - idiomatic, safe, ergonomic
//! - C FFI layer (this file) - thin wrapper using cimpl

use std::ffi::c_void;
use std::os::raw::c_char;

use cimpl::{
    box_tracked, cstr_or_return_null, ok_or_return_false, ptr_or_return,
    deref_or_return_null, ok_or_return_null, 
    option_to_c_string, to_c_bytes, to_c_string,
};

// Import our Rust API
use crate::{Error, ValueConverter, MAX_BUFFER_SIZE};

// ============================================================================
// FFI Error Conversion
// ============================================================================

impl From<Error> for cimpl::Error {
    fn from(e: Error) -> Self {
        // Automatic! Uses Debug for variant name, Display for message
        cimpl::Error::from_error(e)
    }
}

// ============================================================================
// FFI: Constructors (Multiple ways to create a ValueConverter)
// ============================================================================

/// Create from signed 32-bit integer
#[no_mangle]
pub extern "C" fn vc_from_i32(value: i32) -> *mut ValueConverter {
    let converter = ValueConverter::from_i32(value);
    box_tracked!(converter)
}

/// Create from unsigned 32-bit integer
#[no_mangle]
pub extern "C" fn vc_from_u32(value: u32) -> *mut ValueConverter {
    let converter = ValueConverter::from_u32(value);
    box_tracked!(converter)
}

/// Create from signed 64-bit integer
#[no_mangle]
pub extern "C" fn vc_from_i64(value: i64) -> *mut ValueConverter {
    let converter = ValueConverter::from_i64(value);
    box_tracked!(converter)
}

/// Create from unsigned 64-bit integer
#[no_mangle]
pub extern "C" fn vc_from_u64(value: u64) -> *mut ValueConverter {
    let converter = ValueConverter::from_u64(value);
    box_tracked!(converter)
}

/// Create from byte array (can fail if too large or empty)
#[no_mangle]
pub extern "C" fn vc_from_bytes(data: *const u8, len: usize) -> *mut ValueConverter {
    use cimpl::bytes_or_return_null;
    let bytes = bytes_or_return_null!(data, len, "data");
    let converter = ok_or_return_null!(ValueConverter::from_bytes(bytes));
    box_tracked!(converter)
}

/// Create from UTF-8 string (can fail if too large or empty)
#[no_mangle]
pub extern "C" fn vc_from_string(s: *const c_char) -> *mut ValueConverter {
    let string = cstr_or_return_null!(s);
    let converter = ok_or_return_null!(ValueConverter::from_string(&string));
    box_tracked!(converter)
}

/// Create from hex string (can fail if invalid hex, too large, or empty)
#[no_mangle]
pub extern "C" fn vc_from_hex(hex: *const c_char) -> *mut ValueConverter {
    let hex_str = cstr_or_return_null!(hex);
    let converter = ok_or_return_null!(ValueConverter::from_hex(&hex_str));
    box_tracked!(converter)
}

// ============================================================================
// FFI: Conversions (Fallible - these demonstrate Result<T, E>!)
// ============================================================================

/// Convert to signed 32-bit integer
/// Returns false on error (check vc_last_error for details)
/// 
/// This demonstrates Result<T, E> → C pattern:
/// - Might fail if wrong number of bytes
/// - Error details available via vc_last_error()
#[no_mangle]
pub extern "C" fn vc_to_i32(value: *mut ValueConverter, out: *mut i32) -> bool {
    use cimpl::deref_or_return;
    let converter = deref_or_return!(value, ValueConverter, false);
    ptr_or_return!(out, false);
    
    let result = ok_or_return_false!(converter.to_i32());
    unsafe { *out = result; }
    true
}

/// Convert to unsigned 32-bit integer
/// Returns false on error (check vc_last_error for details)
#[no_mangle]
pub extern "C" fn vc_to_u32(value: *mut ValueConverter, out: *mut u32) -> bool {
    use cimpl::deref_or_return;
    let converter = deref_or_return!(value, ValueConverter, false);
    ptr_or_return!(out, false);
    
    let result = ok_or_return_false!(converter.to_u32());
    unsafe { *out = result; }
    true
}

/// Convert to signed 64-bit integer
#[no_mangle]
pub extern "C" fn vc_to_i64(value: *mut ValueConverter, out: *mut i64) -> bool {
    use cimpl::deref_or_return;
    let converter = deref_or_return!(value, ValueConverter, false);
    ptr_or_return!(out, false);
    
    let result = ok_or_return_false!(converter.to_i64());
    unsafe { *out = result; }
    true
}

/// Convert to unsigned 64-bit integer
#[no_mangle]
pub extern "C" fn vc_to_u64(value: *mut ValueConverter, out: *mut u64) -> bool {
    use cimpl::deref_or_return;
    let converter = deref_or_return!(value, ValueConverter, false);
    ptr_or_return!(out, false);
    
    let result = ok_or_return_false!(converter.to_u64());
    unsafe { *out = result; }
    true
}

/// Convert to UTF-8 string (can fail if bytes aren't valid UTF-8)
/// Returns NULL on error
#[no_mangle]
pub extern "C" fn vc_to_string(value: *mut ValueConverter) -> *mut c_char {
    let converter = deref_or_return_null!(value, ValueConverter);
    let string = ok_or_return_null!(converter.to_string());
    to_c_string(string)
}

/// Convert to hex string (always succeeds)
#[no_mangle]
pub extern "C" fn vc_to_hex(value: *mut ValueConverter) -> *mut c_char {
    let converter = deref_or_return_null!(value, ValueConverter);
    to_c_string(converter.to_hex())
}

/// Get raw bytes with length
#[no_mangle]
pub extern "C" fn vc_to_bytes(value: *mut ValueConverter, out_len: *mut usize) -> *const u8 {
    let converter = deref_or_return_null!(value, ValueConverter);
    
    let bytes = converter.as_bytes();
    
    if !out_len.is_null() {
        unsafe { *out_len = bytes.len(); }
    }
    
    to_c_bytes(bytes.to_vec())
}

// ============================================================================
// FFI: Utility Functions
// ============================================================================

/// Get the size in bytes
#[no_mangle]
pub extern "C" fn vc_len(value: *mut ValueConverter) -> usize {
    use cimpl::deref_or_return;
    let converter = deref_or_return!(value, ValueConverter, 0);
    converter.len()
}

/// Get the maximum buffer size
#[no_mangle]
pub extern "C" fn vc_max_size() -> usize {
    MAX_BUFFER_SIZE
}

// ============================================================================
// FFI: Error Handling
// ============================================================================

/// Gets the error message of the last error (NULL if no error)
#[no_mangle]
pub extern "C" fn vc_last_error() -> *mut c_char {
    option_to_c_string!(cimpl::Error::last_message())
}

// ============================================================================
// FFI: Memory Management
// ============================================================================

/// Frees any memory allocated by this library
#[no_mangle]
pub extern "C" fn vc_free(ptr: *mut c_void) -> i32 {
    cimpl::cimpl_free(ptr)
}
