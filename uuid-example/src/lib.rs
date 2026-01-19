//! # Cimple UUID Library
//!
//! This library demonstrates how to use cimple to create safe C FFI bindings
//! for the Rust `uuid` crate (an external dependency). It showcases:
//!
//! - Wrapping external crate types (no opaque wrapper needed!)
//! - Universal `cimple_free()` for all allocations
//! - Multiple constructors and methods
//! - Error handling with error codes
//! - Clean macro usage throughout

use std::os::raw::c_char;
use std::str::FromStr;

use cimple::{
    box_tracked, cstr_or_return_null, define_error_codes, deref_or_return_false,
    deref_or_return_null, deref_or_return_zero, ok_or_return_null_with_table, to_c_bytes,
    to_c_string, Error,
};

// Use uuid::Uuid directly - it's already opaque to C!
use uuid::Uuid;

// ============================================================================
// Error Code Definitions
// ============================================================================

// Core cimple infrastructure errors (0-99)
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

// UUID library-specific errors (100+) - manually declared so cbindgen sees them
#[no_mangle]
pub static ERROR_UUID_PARSE_ERROR: i32 = 100;

// Define the error mapping table
define_error_codes! {
    error_type: uuid::Error,
    table_name: UUID_ERROR_TABLE,
    codes: {
        _ => ("ParseError", ERROR_UUID_PARSE_ERROR),
    }
}

// ============================================================================
// Constructors
// ============================================================================

/// Creates a new random UUID (version 4).
#[no_mangle]
pub extern "C" fn uuid_new_v4() -> *mut Uuid {
    box_tracked!(Uuid::new_v4())
}

/// Creates a new timestamp-based UUID (version 7).
#[no_mangle]
pub extern "C" fn uuid_new_v7() -> *mut Uuid {
    box_tracked!(Uuid::now_v7())
}

/// Parses a UUID from a string.
#[no_mangle]
pub extern "C" fn uuid_parse(s: *const c_char) -> *mut Uuid {
    let s_str = cstr_or_return_null!(s);
    let uuid = ok_or_return_null_with_table!(Uuid::from_str(&s_str), UUID_ERROR_TABLE);
    box_tracked!(uuid)
}

/// Creates a nil UUID (all zeros).
#[no_mangle]
pub extern "C" fn uuid_nil() -> *mut Uuid {
    box_tracked!(Uuid::nil())
}

/// Creates a maximum UUID (all ones).
#[no_mangle]
pub extern "C" fn uuid_max() -> *mut Uuid {
    box_tracked!(Uuid::max())
}

// ============================================================================
// Accessors
// ============================================================================

/// Converts a UUID to its string representation.
#[no_mangle]
pub extern "C" fn uuid_to_string(uuid: *mut Uuid) -> *mut c_char {
    let obj = deref_or_return_null!(uuid, Uuid);
    to_c_string(obj.to_string())
}

/// Converts a UUID to URN format.
#[no_mangle]
pub extern "C" fn uuid_to_urn(uuid: *mut Uuid) -> *mut c_char {
    let obj = deref_or_return_null!(uuid, Uuid);
    to_c_string(obj.urn().to_string())
}

/// Returns the UUID as a byte array (16 bytes).
#[no_mangle]
pub extern "C" fn uuid_as_bytes(uuid: *mut Uuid) -> *const u8 {
    let obj = deref_or_return_null!(uuid, Uuid);
    to_c_bytes(obj.as_bytes().to_vec())
}

// ============================================================================
// Predicates
// ============================================================================

/// Checks if a UUID is nil (all zeros).
#[no_mangle]
pub extern "C" fn uuid_is_nil(uuid: *mut Uuid) -> bool {
    deref_or_return_false!(uuid, Uuid).is_nil()
}

/// Checks if a UUID is max (all ones).
#[no_mangle]
pub extern "C" fn uuid_is_max(uuid: *mut Uuid) -> bool {
    deref_or_return_false!(uuid, Uuid).is_max()
}

// ============================================================================
// Comparison
// ============================================================================

/// Compares two UUIDs.
#[no_mangle]
pub extern "C" fn uuid_compare(a: *mut Uuid, b: *mut Uuid) -> i32 {
    let uuid_a = deref_or_return_zero!(a, Uuid);
    let uuid_b = deref_or_return_zero!(b, Uuid);
    
    match uuid_a.cmp(uuid_b) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

/// Checks if two UUIDs are equal.
#[no_mangle]
pub extern "C" fn uuid_equals(a: *mut Uuid, b: *mut Uuid) -> bool {
    let uuid_a = deref_or_return_false!(a, Uuid);
    let uuid_b = deref_or_return_false!(b, Uuid);
    uuid_a == uuid_b
}

// ============================================================================
// Memory Management
// ============================================================================

/// Frees a UUID object.
///
/// # Deprecated
/// Use `cimple_free()` instead. This function is kept for API compatibility.
#[no_mangle]
pub extern "C" fn uuid_free(uuid: *mut Uuid) -> i32 {
    cimple::cimple_free(uuid as *mut std::ffi::c_void)
}

// ============================================================================
// Error Handling
// ============================================================================

/// Gets the error code for the last error.
#[no_mangle]
pub extern "C" fn uuid_error_code() -> i32 {
    Error::last_code() as i32
}

/// Gets the error message for the last error.
#[no_mangle]
pub extern "C" fn uuid_last_error() -> *mut c_char {
    match Error::last_message() {
        Some(msg) => to_c_string(msg),
        None => std::ptr::null_mut(),
    }
}

/// Clears the last error.
#[no_mangle]
pub extern "C" fn uuid_clear_error() {
    Error::take_last();
}
