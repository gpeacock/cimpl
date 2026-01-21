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
#[allow(unused_imports)]
// May not be directly used but needed for macro expansion
//pub use crate::utils::validate_pointer;

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
/// If the result is Ok, the transform function is applied to the value before returning it.
/// If the result is Err, the error is converted using the supplied error mapper function.
#[macro_export]
macro_rules! ok_or_return {
    // For results using ERROR_MAPPER (new pattern)
    ($result:expr, $transform:expr, $err_val:expr) => {
        match $result {
            Ok(value) => $transform(value),
            Err(e) => {
                $crate::Error::from_mapper(e, ERROR_MAPPER).set_last();
                return $err_val;
            }
        }
    };
    // For cimpl Error type results (no conversion needed)
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
