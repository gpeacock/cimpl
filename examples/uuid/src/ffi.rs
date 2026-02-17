//! C FFI bindings for UUID operations using cimpl
//!
//! This module demonstrates wrapping an external crate (uuid) directly,
//! without creating unnecessary abstraction layers.
//!
//! ## Key Pattern: Direct Usage of External Crate
//!
//! ```rust
//! // ✅ GOOD: Call uuid::Uuid methods directly
//! #[no_mangle]
//! pub extern "C" fn uuid_new_v4() -> *mut Uuid {
//!     box_tracked!(Uuid::new_v4())  // Direct call to uuid crate!
//! }
//! ```
//!
//! No wrapper, no indirection, just clean FFI functions calling the
//! external crate's API directly.
//!
//! ## When to Create a Wrapper
//!
//! Only create a wrapper (like `UuidOps`) if you're adding functionality:
//! - Custom validation beyond what the crate provides
//! - Caching or memoization
//! - Logging or metrics
//! - Business logic specific to your use case
//!
//! For simple FFI exposure of an existing crate? Use it directly!

use crate::{Error, Uuid};
use cimpl::*;
use std::ffi::{c_char, c_void};

/// Convert our Error type to cimpl::Error for FFI
impl From<Error> for cimpl::Error {
    fn from(e: Error) -> Self {
        cimpl::Error::from_error(e)
    }
}

// ============================================================================
// UUID Creation
// ============================================================================

/// Generate a new random Version 4 UUID.
///
/// Returns NULL if random number generation fails.
/// The returned UUID must be freed with `uuid_free()`.
///
/// # Example
/// ```c
/// Uuid* uuid = uuid_new_v4();
/// if (uuid == NULL) {
///     fprintf(stderr, "Failed to generate UUID\n");
///     return -1;
/// }
/// uuid_free(uuid);
/// ```
#[no_mangle]
pub extern "C" fn uuid_new_v4() -> *mut Uuid {
    box_tracked!(Uuid::new_v4())  // Direct call to uuid crate
}

/// Get the nil UUID (all zeros).
///
/// Returns NULL on allocation failure.
/// The returned UUID must be freed with `uuid_free()`.
#[no_mangle]
pub extern "C" fn uuid_nil() -> *mut Uuid {
    box_tracked!(Uuid::nil())  // Direct call to uuid crate
}

/// Get the max UUID (all ones).
///
/// Returns NULL on allocation failure.
/// The returned UUID must be freed with `uuid_free()`.
#[no_mangle]
pub extern "C" fn uuid_max() -> *mut Uuid {
    box_tracked!(Uuid::max())  // Direct call to uuid crate
}

/// Parse a UUID from a string.
///
/// Accepts various formats:
/// - Hyphenated: "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"
/// - Simple: "a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8"
/// - URN: "urn:uuid:A1A2A3A4-B1B2-C1C2-D1D2-D3D4D5D6D7D8"
/// - Braced: "{a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8}"
///
/// Returns NULL if parsing fails. Call `uuid_last_error()` for details.
/// The returned UUID must be freed with `uuid_free()`.
///
/// # Parameters
/// - `s`: Null-terminated C string containing the UUID
#[no_mangle]
pub extern "C" fn uuid_parse(s: *const c_char) -> *mut Uuid {
    let uuid_str = cstr_or_return_null!(s);
    let uuid = ok_or_return_null!(Uuid::parse_str(&uuid_str));  // Direct call
    box_tracked!(uuid)
}

/// Create a UUID from raw bytes.
///
/// # Parameters
/// - `bytes`: Pointer to 16-byte array
///
/// Returns NULL if the pointer is invalid or allocation fails.
/// The returned UUID must be freed with `uuid_free()`.
#[no_mangle]
pub extern "C" fn uuid_from_bytes(bytes: *const u8) -> *mut Uuid {
    let byte_array = bytes_or_return_null!(bytes, 16, "bytes");
    
    // Convert slice to fixed-size array
    let mut array = [0u8; 16];
    array.copy_from_slice(byte_array);
    
    box_tracked!(Uuid::from_bytes(array))  // Direct call to uuid crate
}

// ============================================================================
// UUID Conversion and Access
// ============================================================================

/// Convert UUID to hyphenated string format.
///
/// Returns "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"
///
/// Returns NULL if the UUID pointer is invalid.
/// The returned string must be freed with `uuid_free()`.
///
/// # Parameters
/// - `uuid`: Pointer to UUID
#[no_mangle]
pub extern "C" fn uuid_to_hyphenated(uuid: *const Uuid) -> *mut c_char {
    let uuid_ref = deref_or_return_null!(uuid, Uuid);
    to_c_string(uuid_ref.hyphenated().to_string())  // Direct call
}

/// Convert UUID to simple string format (no hyphens).
///
/// Returns "a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8"
///
/// Returns NULL if the UUID pointer is invalid.
/// The returned string must be freed with `uuid_free()`.
///
/// # Parameters
/// - `uuid`: Pointer to UUID
#[no_mangle]
pub extern "C" fn uuid_to_simple(uuid: *const Uuid) -> *mut c_char {
    let uuid_ref = deref_or_return_null!(uuid, Uuid);
    to_c_string(uuid_ref.simple().to_string())  // Direct call
}

/// Convert UUID to URN format.
///
/// Returns "urn:uuid:A1A2A3A4-B1B2-C1C2-D1D2-D3D4D5D6D7D8"
///
/// Returns NULL if the UUID pointer is invalid.
/// The returned string must be freed with `uuid_free()`.
///
/// # Parameters
/// - `uuid`: Pointer to UUID
#[no_mangle]
pub extern "C" fn uuid_to_urn(uuid: *const Uuid) -> *mut c_char {
    let uuid_ref = deref_or_return_null!(uuid, Uuid);
    to_c_string(uuid_ref.urn().to_string())  // Direct call
}

/// Get the raw bytes of a UUID.
///
/// Copies the 16 bytes of the UUID into the provided buffer.
///
/// Returns false if either pointer is invalid.
///
/// # Parameters
/// - `uuid`: Pointer to UUID
/// - `out_bytes`: Pointer to buffer of at least 16 bytes
#[no_mangle]
pub extern "C" fn uuid_as_bytes(uuid: *const Uuid, out_bytes: *mut u8) -> bool {
    let uuid_ref = deref_or_return_false!(uuid, Uuid);
    ptr_or_return!(out_bytes, false);
    
    let bytes = uuid_ref.as_bytes();  // Direct call
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, 16);
    }
    true
}

// ============================================================================
// UUID Comparison
// ============================================================================

/// Check if two UUIDs are equal.
///
/// Returns false if either pointer is invalid.
///
/// # Parameters
/// - `a`: Pointer to first UUID
/// - `b`: Pointer to second UUID
#[no_mangle]
pub extern "C" fn uuid_equals(a: *const Uuid, b: *const Uuid) -> bool {
    let uuid_a = deref_or_return_false!(a, Uuid);
    let uuid_b = deref_or_return_false!(b, Uuid);
    uuid_a == uuid_b
}

/// Check if UUID is nil (all zeros).
///
/// Returns false if the pointer is invalid.
///
/// # Parameters
/// - `uuid`: Pointer to UUID
#[no_mangle]
pub extern "C" fn uuid_is_nil(uuid: *const Uuid) -> bool {
    let uuid_ref = deref_or_return_false!(uuid, Uuid);
    uuid_ref.is_nil()
}

/// Check if UUID is max (all ones).
///
/// Returns false if the pointer is invalid.
///
/// # Parameters
/// - `uuid`: Pointer to UUID
#[no_mangle]
pub extern "C" fn uuid_is_max(uuid: *const Uuid) -> bool {
    let uuid_ref = deref_or_return_false!(uuid, Uuid);
    uuid_ref.is_max()
}

// ============================================================================
// Error Handling
// ============================================================================

/// Get the last error message from a failed UUID operation.
///
/// Returns a C string describing the error, or NULL if no error occurred.
/// The returned string must be freed with `uuid_free()`.
///
/// # Example
/// ```c
/// Uuid* uuid = uuid_parse("invalid");
/// if (uuid == NULL) {
///     char* msg = uuid_last_error();
///     fprintf(stderr, "Error: %s\n", msg);
///     uuid_free(msg);
/// }
/// ```
#[no_mangle]
pub extern "C" fn uuid_last_error() -> *mut c_char {
    option_to_c_string!(cimpl::Error::last_message())
}

// ============================================================================
// Memory Management
// ============================================================================

/// Free any pointer allocated by the UUID library.
///
/// This includes UUID objects, strings, and byte arrays.
/// Safe to call multiple times on the same pointer (no double-free).
///
/// Returns 0 on success, -1 if the pointer was not found in the registry.
///
/// # Parameters
/// - `ptr`: Pointer to free
#[no_mangle]
pub extern "C" fn uuid_free(ptr: *mut c_void) -> i32 {
    cimpl::cimpl_free(ptr)
}
