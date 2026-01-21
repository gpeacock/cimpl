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
//! - Option handling for validation
//! - Handle-based object management
//!
//! All macros that perform early returns include `_or_return_` in their names
//! to make control flow explicit and obvious.
//!
//! # Quick Reference: Which Macro to Use?
//!
//! ## Input Validation (from C)
//! - **Pointer from C**: `deref_or_return_null!(ptr, Type)` → validates & dereferences to `&Type`
//! - **String from C**: `cstr_or_return_null!(c_str)` → converts C string to Rust `String`
//! - **Check not null**: `ptr_or_return_null!(ptr)` → just null check, no deref
//!
//! ## Output Creation (to C)
//! - **Box a value**: `box_tracked!(value)` → heap allocate and return pointer
//! - **Return string**: `to_c_string(rust_string)` → convert to C string
//! - **Optional string**: `option_to_c_string!(opt)` → `None` becomes `NULL`
//!
//! ## Error Handling
//! - **External crate Result**: `ok_or_return_null!(result, MY_MAPPER)` → needs mapper for error conversion
//! - **cimpl::Error Result**: `ok_or_return_null!(result)` → no mapper needed
//! - **Option<T> (validation)**: `some_or_return_other_null!(option, "message")` → most common case
//! - **Option<T> (custom error)**: `some_or_return_null!(option, Error::InvalidHandle(id))` → specific error type
//!
//! ## Naming Pattern
//! All macros follow: `action_or_return_<what>`
//! - `_null`: Returns `NULL` pointer
//! - `_int`: Returns `-1`
//! - `_zero`: Returns `0`  
//! - `_false`: Returns `false`
//!
//! # Type Mapping Guide
//!
//! | Rust Type              | C receives      | Macro to use                      | Example |
//! |------------------------|-----------------|-----------------------------------|---------|
//! | `*mut T` (from C)      | -               | `deref_or_return_null!(ptr, T)`   | Getting object from C |
//! | `*const c_char` (from C)| -              | `cstr_or_return_null!(s)`         | Getting string from C |
//! | `Result<T, ExtErr>`    | pointer/int     | `ok_or_return_null!(r, MAPPER)`   | External crate errors |
//! | `Result<T, cimpl::Err>`| pointer/int     | `ok_or_return_null!(r)`           | Internal validation |
//! | `Option<T>` validate   | pointer/int     | `some_or_return_other_null!(o, msg)` | Validation failures |
//! | `Option<T>` custom     | pointer/int     | `some_or_return_null!(o, err)`    | Specific error needed |
//! | `T` (owned)            | `*mut T`        | `box_tracked!(value)`             | Returning new object |
//! | `String`               | `*mut c_char`   | `to_c_string(s)`                  | Returning string |
//! | `Option<String>`       | `*mut c_char`   | `option_to_c_string!(opt)`        | Optional string |
//!
//! # Common FFI Function Patterns
//!
//! ## Pattern 1: Constructor (returns new object)
//! ```rust,ignore
//! #[no_mangle]
//! pub extern "C" fn thing_new(value: i32) -> *mut Thing {
//!     let thing = some_or_return_other_null!(
//!         Thing::try_new(value),
//!         "Invalid value"
//!     );
//!     box_tracked!(thing)
//! }
//! ```
//!
//! ## Pattern 2: Parser (external crate Result)
//! ```rust,ignore
//! #[no_mangle]
//! pub extern "C" fn uuid_parse(s: *const c_char) -> *mut Uuid {
//!     let s_str = cstr_or_return_null!(s);
//!     let uuid = ok_or_return_null!(Uuid::from_str(&s_str), UUID_MAPPER);
//!     box_tracked!(uuid)
//! }
//! ```
//!
//! ## Pattern 3: Method (operates on object)
//! ```rust,ignore
//! #[no_mangle]
//! pub extern "C" fn thing_add(thing: *mut Thing, value: i32) -> i32 {
//!     let obj = deref_or_return_int!(thing, Thing);
//!     obj.add(value)
//! }
//! ```
//!
//! ## Pattern 4: Method with validation (Option)
//! ```rust,ignore
//! #[no_mangle]
//! pub extern "C" fn date_add_days(date: *mut Date, days: i64) -> *mut Date {
//!     let obj = deref_or_return_null!(date, Date);
//!     let new_date = some_or_return_other_null!(
//!         obj.checked_add_days(days),
//!         "Date overflow"
//!     );
//!     box_tracked!(new_date)
//! }
//! ```

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
///
/// This macro handles Result types with smart error conversion:
/// - If a mapper is provided, uses it to convert external errors to cimpl::Error
/// - If no mapper is provided, assumes the error is already cimpl::Error
///
/// # Examples
///
/// ```rust,ignore
/// // With external error type (needs mapper)
/// let uuid = ok_or_return!(Uuid::from_str(&s), |v| v, std::ptr::null_mut(), UUID_MAPPER);
///
/// // With cimpl::Error (no mapper needed)
/// let data = ok_or_return!(some_operation(), |v| v, std::ptr::null_mut());
/// ```
#[macro_export]
macro_rules! ok_or_return {
    // With explicit mapper for external error types
    ($result:expr, $transform:expr, $err_val:expr, $mapper:expr) => {
        match $result {
            Ok(value) => $transform(value),
            Err(e) => {
                $crate::Error::from_mapper(e, $mapper).set_last();
                return $err_val;
            }
        }
    };
    // Without mapper - assumes Result<T, cimpl::Error>
    ($result:expr, $transform:expr, $err_val:expr) => {
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
///
/// Supports both external errors (with mapper) and cimpl::Error (without mapper).
#[macro_export]
macro_rules! ok_or_return_int {
    ($result:expr, $mapper:expr) => {
        $crate::ok_or_return!($result, |v| v, -1, $mapper)
    };
    ($result:expr) => {
        $crate::ok_or_return!($result, |v| v, -1)
    };
}

/// Handle Result, early-return with null on error
///
/// Supports both external errors (with mapper) and cimpl::Error (without mapper).
///
/// # Examples
///
/// ```rust,ignore
/// // With external error (e.g., uuid::Error)
/// let uuid = ok_or_return_null!(Uuid::from_str(&s), PARSE_ERROR_MAPPER);
///
/// // With cimpl::Error
/// let data = ok_or_return_null!(validate_something());
/// ```
#[macro_export]
macro_rules! ok_or_return_null {
    ($result:expr, $mapper:expr) => {
        $crate::ok_or_return!($result, |v| v, std::ptr::null_mut(), $mapper)
    };
    ($result:expr) => {
        $crate::ok_or_return!($result, |v| v, std::ptr::null_mut())
    };
}

/// Handle Result, early-return with 0 on error
///
/// Supports both external errors (with mapper) and cimpl::Error (without mapper).
#[macro_export]
macro_rules! ok_or_return_zero {
    ($result:expr, $mapper:expr) => {
        $crate::ok_or_return!($result, |v| v, 0, $mapper)
    };
    ($result:expr) => {
        $crate::ok_or_return!($result, |v| v, 0)
    };
}

/// Handle Result, early-return with false on error
///
/// Supports both external errors (with mapper) and cimpl::Error (without mapper).
#[macro_export]
macro_rules! ok_or_return_false {
    ($result:expr, $mapper:expr) => {
        $crate::ok_or_return!($result, |v| v, false, $mapper)
    };
    ($result:expr) => {
        $crate::ok_or_return!($result, |v| v, false)
    };
}

// ============================================================================
// Option Handling Macros
// ============================================================================
//
// These macros convert Option<T> to FFI-friendly error returns.
// Useful for Rust APIs that return Option instead of Result.

/// Handle Option, early-return with custom value if None
///
/// Takes a cimpl::Error to set when the option is None.
///
/// # Examples
///
/// ```rust,ignore
/// // With Error::Other
/// let date = some_or_return!(
///     NaiveDate::from_ymd_opt(2024, 1, 20),
///     Error::Other("Invalid date".to_string()),
///     std::ptr::null_mut()
/// );
///
/// // With different error type
/// let handle = some_or_return!(
///     get_handle(id),
///     Error::InvalidHandle(id),
///     -1
/// );
/// ```
#[macro_export]
macro_rules! some_or_return {
    ($option:expr, $error:expr, $err_val:expr) => {
        match $option {
            Some(value) => value,
            None => {
                $error.set_last();
                return $err_val;
            }
        }
    };
}

/// Handle Option, early-return with NULL if None
///
/// Takes a cimpl::Error to set when the option is None.
///
/// # Examples
///
/// ```rust,ignore
/// let date = some_or_return_null!(
///     NaiveDate::from_ymd_opt(2024, 1, 20),
///     Error::Other("Invalid date".to_string())
/// );
/// ```
#[macro_export]
macro_rules! some_or_return_null {
    ($option:expr, $error:expr) => {
        $crate::some_or_return!($option, $error, std::ptr::null_mut())
    };
}

/// Handle Option, early-return with -1 if None
///
/// Takes a cimpl::Error to set when the option is None.
#[macro_export]
macro_rules! some_or_return_int {
    ($option:expr, $error:expr) => {
        $crate::some_or_return!($option, $error, -1)
    };
}

/// Handle Option, early-return with 0 if None
///
/// Takes a cimpl::Error to set when the option is None.
#[macro_export]
macro_rules! some_or_return_zero {
    ($option:expr, $error:expr) => {
        $crate::some_or_return!($option, $error, 0)
    };
}

/// Handle Option, early-return with false if None
///
/// Takes a cimpl::Error to set when the option is None.
#[macro_export]
macro_rules! some_or_return_false {
    ($option:expr, $error:expr) => {
        $crate::some_or_return!($option, $error, false)
    };
}

/// Convenience macro: Handle Option with Error::Other message
///
/// Automatically wraps the message string in Error::Other().
/// This is the most common case for Option handling in FFI.
///
/// # Examples
///
/// ```rust,ignore
/// let date = some_or_return_other_null!(
///     NaiveDate::from_ymd_opt(2024, 1, 20),
///     "Invalid date"
/// );
/// ```
#[macro_export]
macro_rules! some_or_return_other_null {
    ($option:expr, $msg:expr) => {
        $crate::some_or_return_null!($option, $crate::Error::Other($msg.to_string()))
    };
}

/// Convenience macro: Handle Option with Error::Other message, return -1
#[macro_export]
macro_rules! some_or_return_other_int {
    ($option:expr, $msg:expr) => {
        $crate::some_or_return_int!($option, $crate::Error::Other($msg.to_string()))
    };
}

/// Convenience macro: Handle Option with Error::Other message, return 0
#[macro_export]
macro_rules! some_or_return_other_zero {
    ($option:expr, $msg:expr) => {
        $crate::some_or_return_zero!($option, $crate::Error::Other($msg.to_string()))
    };
}

/// Convenience macro: Handle Option with Error::Other message, return false
#[macro_export]
macro_rules! some_or_return_other_false {
    ($option:expr, $msg:expr) => {
        $crate::some_or_return_false!($option, $crate::Error::Other($msg.to_string()))
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

/// Converts an `Option<String>` to a C string pointer.
/// Returns `null_mut()` if the Option is None.
///
/// This is commonly used for FFI functions that return optional strings,
/// such as error messages that may or may not be present.
///
/// # Example
/// ```rust,ignore
/// #[no_mangle]
/// pub extern "C" fn get_error_message() -> *mut c_char {
///     option_to_c_string!(Error::last_message())
/// }
/// ```
#[macro_export]
macro_rules! option_to_c_string {
    ($opt:expr) => {
        match $opt {
            Some(msg) => $crate::to_c_string(msg),
            None => std::ptr::null_mut(),
        }
    };
}
