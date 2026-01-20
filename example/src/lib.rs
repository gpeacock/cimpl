// Copyright 2024 Adobe. All rights reserved.
// This file is licensed to you under the Apache License,
// Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
// or the MIT license (http://opensource.org/licenses/MIT),
// at your option.

//! # Cimpl Example Library
//!
//! This example demonstrates how to use the cimpl utilities to create
//! safe, ergonomic C FFI bindings. It showcases:
//!
//! - Pointer-based API with type validation
//! - Safe string conversion and memory management
//! - Error handling with thread-local last error
//! - Universal `cimpl_free()` function for any tracked pointer
//!
//! ## Building
//!
//! ```bash
//! cargo build --release
//! ```
//!
//! This will generate:
//! - `target/release/libcimpl_example.a` (static library)
//! - `target/release/libcimpl_example.so/dylib/dll` (dynamic library)
//! - `include/cimpl_example.h` (C header file)

use std::os::raw::c_char;

use cimpl::{
    box_tracked, cstr_or_return_int, cstr_or_return_null, deref_mut_or_return_neg,
    deref_or_return_null, deref_or_return_zero, ptr_or_return_int, to_c_string, Error, ErrorCode,
};

// ============================================================================
// Example Rust Type - This is what we're wrapping
// ============================================================================

/// A simple string wrapper that demonstrates pointer-based FFI
///
/// This struct is exposed directly to C as an opaque type.
/// C code receives `*mut MyString` pointers but cannot access the internals.
pub struct MyString {
    value: String,
}

impl MyString {
    fn new(initial: String) -> Self {
        Self { value: initial }
    }

    fn get_value(&self) -> &str {
        &self.value
    }

    fn set_value(&mut self, new_value: String) {
        self.value = new_value;
    }

    fn to_uppercase(&self) -> String {
        self.value.to_uppercase()
    }

    fn append(&mut self, suffix: &str) {
        self.value.push_str(suffix);
    }

    fn len(&self) -> usize {
        self.value.len()
    }
}

// ============================================================================
// C FFI - Constructor/Destructor
// ============================================================================

/// Creates a new MyString object with the given initial value.
///
/// # Parameters
/// - `initial`: Initial string value (must be valid UTF-8)
///
/// # Returns
/// - Pointer to the new MyString object, or NULL on error
///
/// # Memory Management
/// The returned pointer must be freed with `cimpl_free()` when no longer needed.
///
/// # Example
/// ```c
/// MyString* str = mystring_create("Hello, World!");
/// if (str != NULL) {
///     // Use the string...
///     cimpl_free(str);
/// }
/// ```
#[no_mangle]
pub extern "C" fn mystring_create(initial: *const c_char) -> *mut MyString {
    let initial_str = cstr_or_return_null!(initial);
    box_tracked!(MyString::new(initial_str))
}

/// Frees a MyString object.
///
/// # Deprecated
/// Use `cimpl_free()` instead. This function is kept for API compatibility.
///
/// # Parameters
/// - `ptr`: Pointer to free (can be NULL)
///
/// # Returns
/// - 0 on success
/// - -1 on error (invalid pointer or already freed)
#[no_mangle]
pub extern "C" fn mystring_free(ptr: *mut MyString) -> i32 {
    cimpl::cimpl_free(ptr as *mut std::ffi::c_void)
}

// ============================================================================
// C FFI - Getters
// ============================================================================

/// Gets the current value of the string.
///
/// # Parameters
/// - `ptr`: Pointer to MyString object
///
/// # Returns
/// - Pointer to a newly allocated C string containing the value, or NULL on error
///
/// # Memory Management
/// The returned string must be freed with `cimpl_free()`.
#[no_mangle]
pub extern "C" fn mystring_get_value(ptr: *mut MyString) -> *mut c_char {
    let obj = deref_or_return_null!(ptr, MyString);
    to_c_string(obj.get_value().to_string())
}

/// Gets the length of the string in bytes.
///
/// # Parameters
/// - `ptr`: Pointer to MyString object
///
/// # Returns
/// - Length of the string in bytes, or 0 on error
#[no_mangle]
pub extern "C" fn mystring_len(ptr: *mut MyString) -> usize {
    deref_or_return_zero!(ptr, MyString).len()
}

// ============================================================================
// C FFI - Setters
// ============================================================================

/// Sets a new value for the string.
///
/// # Parameters
/// - `ptr`: Pointer to MyString object
/// - `new_value`: New string value (must be valid UTF-8)
///
/// # Returns
/// - 0 on success
/// - -1 on error
#[no_mangle]
pub extern "C" fn mystring_set_value(ptr: *mut MyString, new_value: *const c_char) -> i32 {
    let obj = deref_mut_or_return_neg!(ptr, MyString);
    let new_value_str = cstr_or_return_int!(new_value);
    obj.set_value(new_value_str);
    0
}

/// Appends a string to the end of the current value.
///
/// # Parameters
/// - `ptr`: Pointer to MyString object
/// - `suffix`: String to append (must be valid UTF-8)
///
/// # Returns
/// - 0 on success
/// - -1 on error
#[no_mangle]
pub extern "C" fn mystring_append(ptr: *mut MyString, suffix: *const c_char) -> i32 {
    let obj = deref_mut_or_return_neg!(ptr, MyString);
    let suffix_str = cstr_or_return_int!(suffix);
    obj.append(&suffix_str);
    0
}

// ============================================================================
// C FFI - Operations
// ============================================================================

/// Converts the string to uppercase and returns a new string.
///
/// # Parameters
/// - `ptr`: Pointer to MyString object
///
/// # Returns
/// - Pointer to a newly allocated C string containing the uppercase version, or NULL on error
///
/// # Memory Management
/// The returned string must be freed with `cimpl_free()`.
#[no_mangle]
pub extern "C" fn mystring_to_uppercase(ptr: *mut MyString) -> *mut c_char {
    let obj = deref_or_return_null!(ptr, MyString);
    to_c_string(obj.to_uppercase())
}

// ============================================================================
// C FFI - Memory Management
// ============================================================================

/// Frees a string returned by this library.
///
/// # Deprecated
/// Use `cimpl_free()` instead. This function is kept for API compatibility.
///
/// # Parameters
/// - `str`: String pointer to free (can be NULL)
///
/// # Returns
/// - 0 on success
/// - -1 if the string was not allocated by this library
#[no_mangle]
pub extern "C" fn mystring_string_free(str: *mut c_char) -> i32 {
    ptr_or_return_int!(str);
    if unsafe { cimpl::free_c_string(str) } {
        0
    } else {
        -1
    }
}

// ============================================================================
// C FFI - Error Code Constants
// ============================================================================

/// Error code constant: No error
#[no_mangle]
pub static ERROR_OK: i32 = ErrorCode::Ok as i32;

/// Error code constant: NULL parameter
#[no_mangle]
pub static ERROR_NULL_PARAMETER: i32 = ErrorCode::NullParameter as i32;

/// Error code constant: String too long
#[no_mangle]
pub static ERROR_STRING_TOO_LONG: i32 = ErrorCode::StringTooLong as i32;

/// Error code constant: Invalid handle
#[no_mangle]
pub static ERROR_INVALID_HANDLE: i32 = ErrorCode::InvalidHandle as i32;

/// Error code constant: Wrong handle type
#[no_mangle]
pub static ERROR_WRONG_HANDLE_TYPE: i32 = ErrorCode::WrongHandleType as i32;

/// Error code constant: Other error
#[no_mangle]
pub static ERROR_OTHER: i32 = ErrorCode::Other as i32;

// ============================================================================
// C FFI - Error Handling
// ============================================================================

/// Gets the last error code.
///
/// Error codes enable language bindings to create typed exceptions.
/// For example, C++ can map error codes to specific exception types,
/// and Python can create custom exception classes.
///
/// # Returns
/// - 0 (ERROR_OK) if no error
/// - Error code corresponding to the error type:
///   - 1: ERROR_NULL_PARAMETER - A required parameter was NULL
///   - 2: ERROR_STRING_TOO_LONG - String exceeds maximum length
///   - 3: ERROR_INVALID_HANDLE - Handle is invalid or already freed
///   - 4: ERROR_WRONG_HANDLE_TYPE - Handle type doesn't match expected type
///   - 5: ERROR_OTHER - Other error occurred
///
/// # Example (C)
/// ```c
/// if (mystring_set_value(handle, NULL) != 0) {
///     int32_t code = mystring_error_code();
///     char* msg = mystring_error_message();
///     fprintf(stderr, "Error %d: %s\n", code, msg);
///     mystring_string_free(msg);
/// }
/// ```
///
/// # Example (C++ Exception)
/// ```cpp
/// class MyStringException : public std::exception {
///     int code_;
///     std::string message_;
/// public:
///     MyStringException() : code_(mystring_error_code()) {
///         char* msg = mystring_error_message();
///         if (msg) {
///             message_ = msg;
///             mystring_string_free(msg);
///         }
///     }
///     const char* what() const noexcept override { return message_.c_str(); }
///     int code() const { return code_; }
/// };
/// ```
#[no_mangle]
pub extern "C" fn mystring_error_code() -> i32 {
    Error::last_code() as i32
}

/// Gets the last error message.
///
/// # Returns
/// - Pointer to a C string describing the last error, or NULL if no error
///
/// # Memory Management
/// The returned string must be freed with `mystring_string_free()`.
///
/// # Example
/// ```c
/// if (mystring_set_value(handle, NULL) != 0) {
///     char* error = mystring_last_error();
///     if (error != NULL) {
///         fprintf(stderr, "Error: %s\n", error);
///         mystring_string_free(error);
///     }
/// }
/// ```
#[no_mangle]
pub extern "C" fn mystring_last_error() -> *mut c_char {
    match Error::last_message() {
        Some(msg) => to_c_string(msg),
        None => std::ptr::null_mut(),
    }
}

/// Clears the last error.
///
/// This function can be called to clear the error state before making
/// a series of calls where you want to check for new errors.
///
/// # Example
/// ```c
/// mystring_clear_error();
/// // Now any error from last_error() will be from calls after this point
/// ```
#[no_mangle]
pub extern "C" fn mystring_clear_error() {
    Error::take_last();
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn to_c_str(s: &str) -> *const c_char {
        CString::new(s).unwrap().into_raw()
    }

    #[test]
    fn test_create_and_free() {
        let initial = to_c_str("test");
        let handle = mystring_create(initial);
        assert!(!handle.is_null());

        let result = mystring_free(handle);
        assert_eq!(result, 0);

        unsafe { free_c_string(initial as *mut c_char) };
    }

    #[test]
    fn test_get_value() {
        let initial = to_c_str("hello");
        let handle = mystring_create(initial);

        let value = mystring_get_value(handle);
        assert!(!value.is_null());

        unsafe {
            let rust_string = std::ffi::CStr::from_ptr(value).to_string_lossy();
            assert_eq!(rust_string, "hello");
            free_c_string(value);
            free_c_string(initial as *mut c_char);
        }

        mystring_free(handle);
    }

    #[test]
    fn test_set_value() {
        let initial = to_c_str("old");
        let handle = mystring_create(initial);

        let new_val = to_c_str("new");
        let result = mystring_set_value(handle, new_val);
        assert_eq!(result, 0);

        let value = mystring_get_value(handle);
        unsafe {
            let rust_string = std::ffi::CStr::from_ptr(value).to_string_lossy();
            assert_eq!(rust_string, "new");
            free_c_string(value);
            free_c_string(initial as *mut c_char);
            free_c_string(new_val as *mut c_char);
        }

        mystring_free(handle);
    }

    #[test]
    fn test_to_uppercase() {
        let initial = to_c_str("hello");
        let handle = mystring_create(initial);

        let upper = mystring_to_uppercase(handle);
        unsafe {
            let rust_string = std::ffi::CStr::from_ptr(upper).to_string_lossy();
            assert_eq!(rust_string, "HELLO");
            free_c_string(upper);
            free_c_string(initial as *mut c_char);
        }

        mystring_free(handle);
    }

    #[test]
    fn test_append() {
        let initial = to_c_str("Hello");
        let handle = mystring_create(initial);

        let suffix = to_c_str(", World!");
        let result = mystring_append(handle, suffix);
        assert_eq!(result, 0);

        let value = mystring_get_value(handle);
        unsafe {
            let rust_string = std::ffi::CStr::from_ptr(value).to_string_lossy();
            assert_eq!(rust_string, "Hello, World!");
            free_c_string(value);
            free_c_string(initial as *mut c_char);
            free_c_string(suffix as *mut c_char);
        }

        mystring_free(handle);
    }

    #[test]
    fn test_len() {
        let initial = to_c_str("hello");
        let handle = mystring_create(initial);

        let len = mystring_len(handle);
        assert_eq!(len, 5);

        unsafe { free_c_string(initial as *mut c_char) };
        mystring_free(handle);
    }

    #[test]
    fn test_error_handling() {
        // Test null parameter error
        let result = mystring_set_value(std::ptr::null_mut(), to_c_str("test"));
        assert_eq!(result, -1);

        let error = mystring_last_error();
        assert!(!error.is_null());

        unsafe {
            let error_msg = std::ffi::CStr::from_ptr(error).to_string_lossy();
            assert!(error_msg.contains("NullParameter"));
            free_c_string(error);
        }

        mystring_clear_error();
        let error2 = mystring_last_error();
        assert!(error2.is_null());
    }
}
