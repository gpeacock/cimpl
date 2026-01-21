//! # Cimpl UUID Library
//!
//! This library demonstrates how to use cimpl to create safe C FFI bindings
//! for the Rust `uuid` crate (an external dependency). It showcases:
//!
//! - Wrapping external crate types (no opaque wrapper needed!)
//! - Universal `cimpl_free()` for all allocations
//! - Multiple constructors and methods
//! - Error handling with error codes
//! - Clean macro usage throughout

use std::os::raw::c_char;
use std::str::FromStr;

use cimpl::{
    box_tracked, cstr_or_return_null, deref_or_return_false, deref_or_return_null,
    deref_or_return_zero, ok_or_return_null, option_to_c_string, to_c_bytes, to_c_string, Error,
};

// Use uuid::Uuid directly - it's already opaque to C!
use uuid::Uuid;

// ============================================================================
// Error Code Definitions
// ============================================================================

// Core cimpl infrastructure errors (0-99)
// Note: Prefixed with UUID_ to avoid clashing with other C libraries
#[no_mangle]
pub static UUID_ERROR_OK: i32 = 0;
#[no_mangle]
pub static UUID_ERROR_NULL_PARAMETER: i32 = 1;
#[no_mangle]
pub static UUID_ERROR_STRING_TOO_LONG: i32 = 2;
#[no_mangle]
pub static UUID_ERROR_INVALID_HANDLE: i32 = 3;
#[no_mangle]
pub static UUID_ERROR_WRONG_HANDLE_TYPE: i32 = 4;
#[no_mangle]
pub static UUID_ERROR_OTHER: i32 = 5;

// UUID library-specific errors (100+)
#[no_mangle]
pub static UUID_ERROR_PARSE_ERROR: i32 = 100;

// ============================================================================
// Error Mapper
// ============================================================================

/// Maps uuid::Error to cimpl error codes
///
/// This function translates uuid crate errors into our C API error codes.
/// It's passed explicitly to the ok_or_return_* macros when parsing UUIDs.
fn map_uuid_error(_e: &uuid::Error) -> (i32, &'static str) {
    // For uuid crate, all errors are parse errors
    // Could be extended to match on specific error types if needed
    (UUID_ERROR_PARSE_ERROR, "ParseError")
}

const UUID_ERROR_MAPPER: fn(&uuid::Error) -> (i32, &'static str) = map_uuid_error;

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
    let uuid = ok_or_return_null!(Uuid::from_str(&s_str), UUID_ERROR_MAPPER);
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
/// Frees any pointer allocated by this library.
///
/// This is a convenience wrapper around `cimpl::cimpl_free()` that provides
/// a library-specific API. It can free:
/// - Uuid objects (returned by uuid_new_*, uuid_parse, etc.)
/// - Strings (returned by uuid_to_string, uuid_last_error, etc.)
/// - Byte arrays (returned by uuid_as_bytes)
///
/// # Safety
/// The pointer must have been allocated by this library, or be NULL.
#[no_mangle]
pub extern "C" fn uuid_free(ptr: *mut std::ffi::c_void) -> i32 {
    cimpl::cimpl_free(ptr)
}

// ============================================================================
// Error Handling
// ============================================================================

/// Gets the error code for the last error.
#[no_mangle]
pub extern "C" fn uuid_error_code() -> i32 {
    Error::last_code()
}

/// Gets the error message for the last error.
#[no_mangle]
pub extern "C" fn uuid_last_error() -> *mut c_char {
    option_to_c_string!(Error::last_message())
}

/// Clears the last error.
#[no_mangle]
pub extern "C" fn uuid_clear_error() {
    Error::take_last();
}
