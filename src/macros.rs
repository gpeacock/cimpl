// Copyright 2024 Adobe. All rights reserved.
// This file is licensed to you under the Apache License,
// Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
// or the MIT license (http://opensource.org/licenses/MIT),
// at your option.

// Unless required by applicable law or agreed to in writing,
// this software is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR REPRESENTATIONS OF ANY KIND, either express or
// implied. See the LICENSE-MIT and LICENSE-APACHE files for the
// specific language governing permissions and limitations under
// each license.

//! FFI Helper Macros
//!
//! This module provides a set of macros for building safe,
//! ergonomic C FFI bindings. The macros handle common FFI patterns like:
//! - Null pointer checking
//! - C string conversion
//! - Result/error handling with early returns
//! - Handle-based object management
//!
//! All macros that perform early returns include `_or_return_` in their names
//! to make control flow explicit and obvious.

// Re-export types/functions that macros need
#[doc(hidden)]
#[allow(unused_imports)] // May not be directly used but needed for macro expansion
pub use crate::utils::{get_registry, validate_pointer};

// ============================================================================
// Pointer Management Macros
// ============================================================================
//
// These macros follow a consistent naming pattern:
// - deref_or_return_*: Validate and return reference immediately
// - deref_mut_or_return_*: Same as above, but mutable
//
// All variants support the standard suffixes:
// - _null: Returns NULL on error
// - _neg: Returns -1 on error
// - _zero: Returns 0 on error
// - _false: Returns false on error
// - (base): Custom return value

// ----------------------------------------------------------------------------
// Deref Macros - Return reference immediately
// ----------------------------------------------------------------------------

/// Validate pointer and dereference immutably, returning reference
/// Returns early with custom value on error
#[macro_export]
macro_rules! deref_or_return {
    ($ptr:expr, $type:ty, $err_val:expr) => {{
        $crate::ptr_or_return!($ptr, $err_val);
        match $crate::validate_pointer::<$type>($ptr) {
            Ok(()) => unsafe { &*($ptr as *const $type) },
            Err(e) => {
                e.set_last();
                return $err_val;
            }
        }
    }};
}

/// Validate pointer and dereference immutably, returning reference
/// Returns NULL on error
#[macro_export]
macro_rules! deref_or_return_null {
    ($ptr:expr, $type:ty) => {{
        $crate::deref_or_return!($ptr, $type, std::ptr::null_mut())
    }};
}

/// Validate pointer and dereference immutably, returning reference
/// Returns -1 on error
#[macro_export]
macro_rules! deref_or_return_neg {
    ($ptr:expr, $type:ty) => {{
        $crate::deref_or_return!($ptr, $type, -1)
    }};
}

/// Validate pointer and dereference immutably, returning reference
/// Returns 0 on error
#[macro_export]
macro_rules! deref_or_return_zero {
    ($ptr:expr, $type:ty) => {{
        $crate::deref_or_return!($ptr, $type, 0)
    }};
}

/// Validate pointer and dereference immutably, returning reference
/// Returns false on error
#[macro_export]
macro_rules! deref_or_return_false {
    ($ptr:expr, $type:ty) => {{
        $crate::deref_or_return!($ptr, $type, false)
    }};
}

/// Validate pointer and dereference mutably, returning reference
/// Returns early with custom value on error
#[macro_export]
macro_rules! deref_mut_or_return {
    ($ptr:expr, $type:ty, $err_val:expr) => {{
        $crate::ptr_or_return!($ptr, $err_val);
        match $crate::validate_pointer::<$type>($ptr) {
            Ok(()) => unsafe { &mut *($ptr as *mut $type) },
            Err(e) => {
                e.set_last();
                return $err_val;
            }
        }
    }};
}

/// Validate pointer and dereference mutably, returning reference
/// Returns NULL on error
#[macro_export]
macro_rules! deref_mut_or_return_null {
    ($ptr:expr, $type:ty) => {{
        $crate::deref_mut_or_return!($ptr, $type, std::ptr::null_mut())
    }};
}

/// Validate pointer and dereference mutably, returning reference
/// Returns -1 on error
#[macro_export]
macro_rules! deref_mut_or_return_neg {
    ($ptr:expr, $type:ty) => {{
        $crate::deref_mut_or_return!($ptr, $type, -1)
    }};
}

// ----------------------------------------------------------------------------
// Deprecated aliases for backward compatibility
// ----------------------------------------------------------------------------

#[deprecated(since = "0.2.0", note = "use `deref_or_return_null!` instead")]
#[macro_export]
macro_rules! validate_and_deref {
    ($ptr:expr, $type:ty) => {{
        $crate::deref_or_return_null!($ptr, $type)
    }};
}

#[deprecated(since = "0.2.0", note = "use `deref_mut_or_return_null!` instead")]
#[macro_export]
macro_rules! validate_and_deref_mut {
    ($ptr:expr, $type:ty) => {{
        $crate::deref_mut_or_return_null!($ptr, $type)
    }};
}

#[deprecated(since = "0.2.0", note = "use `deref_or_return_neg!` instead")]
#[macro_export]
macro_rules! validate_and_deref_neg {
    ($ptr:expr, $type:ty) => {{
        $crate::deref_or_return_neg!($ptr, $type)
    }};
}

#[deprecated(since = "0.2.0", note = "use `deref_mut_or_return_neg!` instead")]
#[macro_export]
macro_rules! validate_and_deref_mut_neg {
    ($ptr:expr, $type:ty) => {{
        $crate::deref_mut_or_return_neg!($ptr, $type)
    }};
}

/// Create a Box-wrapped pointer and track it
/// Returns the raw pointer
#[macro_export]
macro_rules! box_tracked {
    ($expr:expr) => {{
        let obj = $expr;
        let ptr = Box::into_raw(Box::new(obj));
        $crate::track_box(ptr);
        ptr
    }};
}

/// Create an Arc-wrapped pointer and track it
/// Returns the raw pointer
#[macro_export]
macro_rules! arc_tracked {
    ($expr:expr) => {{
        let obj = $expr;
        let ptr = Arc::into_raw(Arc::new(obj)) as *mut _;
        $crate::track_arc(ptr);
        ptr
    }};
}

/// Maximum length for C strings when using bounded conversion (64KB)
pub const MAX_CSTRING_LEN: usize = 65536;

/// Check pointer not null or early-return with error value
#[macro_export]
macro_rules! ptr_or_return {
    ($ptr:expr, $err_val:expr) => {
        if $ptr.is_null() {
            $crate::Error::set_last($crate::Error::NullParameter(stringify!($ptr).to_string()));
            return $err_val;
        }
    };
}

/// Convert C string with bounded length check or early-return with error value
/// Uses a safe bounded approach to prevent reading unbounded memory.
/// Maximum string length is MAX_CSTRING_LEN (64KB).
#[macro_export]
macro_rules! cstr_or_return {
    ($ptr:expr, $err_val:expr) => {{
        let ptr = $ptr;
        if ptr.is_null() {
            $crate::Error::set_last($crate::Error::NullParameter(stringify!($ptr).to_string()));
            return $err_val;
        } else {
            // SAFETY: We create a bounded slice up to MAX_CSTRING_LEN.
            // Caller must ensure ptr is valid for reading and points to a
            // null-terminated string within MAX_CSTRING_LEN bytes.
            let bytes = unsafe {
                std::slice::from_raw_parts(ptr as *const u8, $crate::macros::MAX_CSTRING_LEN)
            };
            match std::ffi::CStr::from_bytes_until_nul(bytes) {
                Ok(cstr) => cstr.to_string_lossy().into_owned(),
                Err(_) => {
                    $crate::Error::set_last($crate::Error::StringTooLong(
                        stringify!($ptr).to_string(),
                    ));
                    return $err_val;
                }
            }
        }
    }};
}

/// Convert C string with custom length limit or early-return with error value
/// Allows specifying a custom maximum length for the string.
#[macro_export]
macro_rules! cstr_or_return_with_limit {
    ($ptr:expr, $max_len:expr, $err_val:expr) => {{
        let ptr = $ptr;
        let max_len = $max_len;
        if ptr.is_null() {
            $crate::Error::set_last($crate::Error::NullParameter(stringify!($ptr).to_string()));
            return $err_val;
        } else {
            // SAFETY: We create a bounded slice up to max_len.
            // Caller must ensure ptr is valid for reading and points to a
            // null-terminated string within max_len bytes.
            let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, max_len) };
            match std::ffi::CStr::from_bytes_until_nul(bytes) {
                Ok(cstr) => cstr.to_string_lossy().into_owned(),
                Err(_) => {
                    $crate::Error::set_last($crate::Error::StringTooLong(
                        stringify!($ptr).to_string(),
                    ));
                    return $err_val;
                }
            }
        }
    }};
}

/// Handle Result or early-return with error value
/// This is a generic macro that can be customized by the user
/// to handle their own error types. The transform function
/// is applied to the Ok value before returning it.
#[macro_export]
macro_rules! ok_or_return {
    // For generic results with custom error handling
    ($result:expr, $error_handler:expr, $transform:expr, $err_val:expr) => {
        match $result {
            Ok(value) => $transform(value),
            Err(err) => {
                $error_handler(err);
                return $err_val;
            }
        }
    };
    // For cimple Error type results (no conversion needed)
    (@local $result:expr, $transform:expr, $err_val:expr) => {
        match $result {
            Ok(value) => $transform(value),
            Err(err) => {
                err.set_last();
                return $err_val;
            }
        }
    };
}

// ============================================================================
// Named Shortcuts (self-documenting for common error values)
// ============================================================================

/// Handle Result, early-return with -1 (negative) on error
#[macro_export]
macro_rules! ok_or_return_int {
    ($result:expr) => {
        $crate::ok_or_return!($result, |v| v, -1)
    };
}

/// Handle Result, early-return with null on error
#[macro_export]
macro_rules! ok_or_return_null {
    ($result:expr) => {
        $crate::ok_or_return!($result, |v| v, std::ptr::null_mut())
    };
}

/// Handle Result, early-return with 0 on error
#[macro_export]
macro_rules! ok_or_return_zero {
    ($result:expr) => {
        $crate::ok_or_return!($result, |v| v, 0)
    };
}

/// Handle Result, early-return with false on error
#[macro_export]
macro_rules! ok_or_return_false {
    ($result:expr) => {
        $crate::ok_or_return!($result, |v| v, false)
    };
}

/// If the expression is null, set the last error and return null.
#[macro_export]
macro_rules! ptr_or_return_null {
    ($ptr : expr) => {
        $crate::ptr_or_return!($ptr, std::ptr::null_mut())
    };
}

/// If the expression is null, set the last error and return -1.
#[macro_export]
macro_rules! ptr_or_return_int {
    ($ptr : expr) => {
        $crate::ptr_or_return!($ptr, -1)
    };
}

/// If the expression is null, set the last error and return std::ptr::null_mut().
#[macro_export]
macro_rules! cstr_or_return_null {
    ($ptr : expr) => {
        $crate::cstr_or_return!($ptr, std::ptr::null_mut())
    };
}

// Internal routine to convert a *const c_char to a rust String or return a -1 int error.
#[macro_export]
macro_rules! cstr_or_return_int {
    ($ptr : expr) => {
        $crate::cstr_or_return!($ptr, -1)
    };
}

// Internal routine to convert a *const c_char to Option<String>.
#[macro_export]
macro_rules! cstr_option {
    ($ptr : expr) => {{
        let ptr = $ptr;
        if ptr.is_null() {
            None
        } else {
            // SAFETY: We create a bounded slice up to MAX_CSTRING_LEN.
            // Caller must ensure ptr is valid for reading and points to a
            // null-terminated string within MAX_CSTRING_LEN bytes.
            let bytes = unsafe {
                std::slice::from_raw_parts(ptr as *const u8, $crate::macros::MAX_CSTRING_LEN)
            };
            match std::ffi::CStr::from_bytes_until_nul(bytes) {
                Ok(cstr) => Some(cstr.to_string_lossy().into_owned()),
                Err(_) => {
                    $crate::Error::set_last($crate::Error::StringTooLong(
                        stringify!($ptr).to_string(),
                    ));
                    None
                }
            }
        }
    }};
}

// ============================================================================
// Core Flexible Macros (explicit "return" makes control flow clear)
// ============================================================================

// ============================================================================
// Error Mapping Macro for Library-Specific Errors
// ============================================================================

/// Defines a mapping table from library errors to error codes
///
/// This macro generates a static lookup table that maps error patterns to error codes and names.
/// The developer must first manually declare the error constants (so cbindgen can see them).
///
/// # Example
///
/// ```rust,ignore
/// // Step 1: Manually declare constants (cbindgen sees these)
/// #[no_mangle]
/// pub static ERROR_UUID_PARSE_ERROR: i32 = 100;
///
/// // Step 2: Create mapping table
/// define_error_codes! {
///     error_type: uuid::Error,
///     table_name: UUID_ERROR_TABLE,
///     codes: {
///         InvalidLength(_) => ("ParseError", ERROR_UUID_PARSE_ERROR),
///         InvalidCharacter(_, _) => ("InvalidCharacter", ERROR_UUID_INVALID_CHARACTER),
///     }
/// }
/// ```
///
/// This generates a static table that can be used with `Error::from_table()`:
/// ```rust,ignore
/// Err(e) => {
///     Error::from_table(e, UUID_ERROR_TABLE).set_last();
///     std::ptr::null_mut()
/// }
/// ```
#[macro_export]
macro_rules! define_error_codes {
    (error_type: $error_type:ty, table_name: $table_name:ident, codes: { $($pattern:pat => ($name:expr, $code:expr)),* $(,)? }) => {
        static $table_name: &[(fn(&$error_type) -> bool, &str, i32)] = &[
            $(
                (|e: &$error_type| matches!(e, $pattern), $name, $code),
            )*
        ];
    };
}

// ============================================================================
// Result Handling with Error Tables
// ============================================================================

/// Unwraps a Result or returns with error mapped via table
///
/// This macro handles Result types by unwrapping Ok values or converting Err values
/// using an error mapping table and returning early with a specified value.
///
/// # Example
///
/// ```rust,ignore
/// pub extern "C" fn uuid_parse(s: *const c_char) -> *mut Uuid {
///     let s_str = cstr_or_return_null!(s);
///     let uuid = ok_or_return_null_with_table!(Uuid::from_str(&s_str), UUID_ERROR_TABLE);
///     box_tracked!(uuid)
/// }
/// ```
#[macro_export]
macro_rules! ok_or_return_with_table {
    ($result:expr, $table:expr, $err_val:expr) => {
        match $result {
            Ok(val) => val,
            Err(e) => {
                $crate::Error::from_table(e, $table).set_last();
                return $err_val;
            }
        }
    };
}

/// Unwraps a Result or returns NULL (for pointer-returning functions)
#[macro_export]
macro_rules! ok_or_return_null_with_table {
    ($result:expr, $table:expr) => {
        $crate::ok_or_return_with_table!($result, $table, std::ptr::null_mut())
    };
}

/// Unwraps a Result or returns -1 (for integer-returning functions)
#[macro_export]
macro_rules! ok_or_return_neg_with_table {
    ($result:expr, $table:expr) => {
        $crate::ok_or_return_with_table!($result, $table, -1)
    };
}

/// Unwraps a Result or returns 0 (for functions where 0 indicates error)
#[macro_export]
macro_rules! ok_or_return_zero_with_table {
    ($result:expr, $table:expr) => {
        $crate::ok_or_return_with_table!($result, $table, 0)
    };
}

/// Unwraps a Result or returns false (for bool-returning functions)
#[macro_export]
macro_rules! ok_or_return_false_with_table {
    ($result:expr, $table:expr) => {
        $crate::ok_or_return_with_table!($result, $table, false)
    };
}
