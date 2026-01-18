// Copyright 2024 Adobe. All rights reserved.
// This file is licensed to you under the Apache License,
// Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
// or the MIT license (http://opensource.org/licenses/MIT),
// at your option.

//! # Cimple Example Library
//!
//! This example demonstrates how to use the cimple utilities to create
//! safe, ergonomic C FFI bindings. It showcases:
//!
//! - Handle-based API for managing Rust objects from C
//! - Safe string conversion and memory management
//! - Error handling with thread-local last error
//! - Allocation tracking to prevent double-frees
//!
//! ## Building
//!
//! ```bash
//! cargo build --release
//! ```
//!
//! This will generate:
//! - `target/release/libcimple_example.a` (static library)
//! - `target/release/libcimple_example.so/dylib/dll` (dynamic library)
//! - `include/cimple_example.h` (C header file)

use std::os::raw::c_char;

use cimple::{
    cstr_or_return_int, cstr_or_return_null, free_c_string, guard_handle_mut_or_return_neg,
    guard_handle_or_null, ptr_or_return_int, to_c_string, Error,
};

// ============================================================================
// Example Rust Type - This is what we're wrapping
// ============================================================================

/// A simple string wrapper that demonstrates handle-based FFI
struct MyString {
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
// C FFI - Opaque Handle Type
// ============================================================================

/// Opaque handle to a MyString object.
///
/// This handle must be created with `mystring_create()` and freed with
/// `mystring_free()`. Never attempt to dereference this pointer directly
/// from C code.
#[repr(C)]
pub struct MyStringHandle {
    _private: [u8; 0],
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
/// - Handle to the new MyString object, or NULL on error
///
/// # Memory Management
/// The returned handle must be freed with `mystring_free()` when no longer needed.
///
/// # Example
/// ```c
/// MyStringHandle* handle = mystring_create("Hello, World!");
/// if (handle != NULL) {
///     // Use the handle...
///     mystring_free(handle);
/// }
/// ```
#[no_mangle]
pub extern "C" fn mystring_create(initial: *const c_char) -> *mut MyStringHandle {
    // Convert C string to Rust String with automatic null check
    let initial_str = cstr_or_return_null!(initial);

    // Create the Rust object
    let my_string = MyString::new(initial_str);

    // Convert to handle and return (returns null on error)
    let handle = cimple::get_handles().insert(my_string);
    cimple::handle_to_ptr::<MyStringHandle>(handle)
}

/// Frees a MyString object.
///
/// # Parameters
/// - `handle`: Handle to free (can be NULL)
///
/// # Returns
/// - 0 on success
/// - -1 on error (invalid handle or already freed)
///
/// # Safety
/// After calling this function, the handle is no longer valid and must not be used.
/// Passing NULL is safe and will return 0.
/// Passing an invalid or already-freed handle will return -1 and set the last error.
///
/// # Example
/// ```c
/// MyStringHandle* handle = mystring_create("test");
/// int result = mystring_free(handle);
/// // handle is now invalid - don't use it!
/// ```
#[no_mangle]
pub extern "C" fn mystring_free(handle: *mut MyStringHandle) -> i32 {
    cimple::free_handle!(handle, MyString)
}

// ============================================================================
// C FFI - Getters
// ============================================================================

/// Gets the current value of the string.
///
/// # Parameters
/// - `handle`: Handle to MyString object
///
/// # Returns
/// - Pointer to a newly allocated C string containing the value, or NULL on error
///
/// # Memory Management
/// The returned string is allocated by Rust and must be freed with `mystring_string_free()`.
///
/// # Example
/// ```c
/// char* value = mystring_get_value(handle);
/// if (value != NULL) {
///     printf("Value: %s\n", value);
///     mystring_string_free(value);
/// }
/// ```
#[no_mangle]
pub extern "C" fn mystring_get_value(handle: *mut MyStringHandle) -> *mut c_char {
    guard_handle_or_null!(handle, MyString, obj);
    to_c_string(obj.get_value().to_string())
}

/// Gets the length of the string in bytes.
///
/// # Parameters
/// - `handle`: Handle to MyString object
///
/// # Returns
/// - Length of the string in bytes, or 0 on error
///
/// # Note
/// For UTF-8 strings, this returns the byte length, not the character count.
///
/// # Example
/// ```c
/// size_t len = mystring_len(handle);
/// printf("Length: %zu bytes\n", len);
/// ```
#[no_mangle]
pub extern "C" fn mystring_len(handle: *mut MyStringHandle) -> usize {
    cimple::guard_handle_or_default!(handle, MyString, obj, 0);
    obj.len()
}

// ============================================================================
// C FFI - Setters
// ============================================================================

/// Sets a new value for the string.
///
/// # Parameters
/// - `handle`: Handle to MyString object
/// - `new_value`: New string value (must be valid UTF-8)
///
/// # Returns
/// - 0 on success
/// - -1 on error
///
/// # Example
/// ```c
/// if (mystring_set_value(handle, "New value") == 0) {
///     printf("Value updated successfully\n");
/// }
/// ```
#[no_mangle]
pub extern "C" fn mystring_set_value(handle: *mut MyStringHandle, new_value: *const c_char) -> i32 {
    guard_handle_mut_or_return_neg!(handle, MyString, obj);
    let new_value_str = cstr_or_return_int!(new_value);
    obj.set_value(new_value_str);
    0
}

/// Appends a string to the end of the current value.
///
/// # Parameters
/// - `handle`: Handle to MyString object
/// - `suffix`: String to append (must be valid UTF-8)
///
/// # Returns
/// - 0 on success
/// - -1 on error
///
/// # Example
/// ```c
/// mystring_create("Hello");
/// mystring_append(handle, ", World!");
/// // Value is now "Hello, World!"
/// ```
#[no_mangle]
pub extern "C" fn mystring_append(handle: *mut MyStringHandle, suffix: *const c_char) -> i32 {
    guard_handle_mut_or_return_neg!(handle, MyString, obj);
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
/// - `handle`: Handle to MyString object
///
/// # Returns
/// - Pointer to a newly allocated C string containing the uppercase version, or NULL on error
///
/// # Memory Management
/// The returned string must be freed with `mystring_string_free()`.
///
/// # Example
/// ```c
/// char* upper = mystring_to_uppercase(handle);
/// if (upper != NULL) {
///     printf("Uppercase: %s\n", upper);
///     mystring_string_free(upper);
/// }
/// ```
#[no_mangle]
pub extern "C" fn mystring_to_uppercase(handle: *mut MyStringHandle) -> *mut c_char {
    guard_handle_or_null!(handle, MyString, obj);
    to_c_string(obj.to_uppercase())
}

// ============================================================================
// C FFI - Memory Management for Returned Strings
// ============================================================================

/// Frees a string returned by this library.
///
/// # Parameters
/// - `str`: String pointer to free (can be NULL)
///
/// # Returns
/// - 0 on success
/// - -1 if the string was not allocated by this library (double-free or invalid pointer)
///
/// # Safety
/// Only call this on strings returned by functions in this library
/// (e.g., `mystring_get_value()`, `mystring_to_uppercase()`).
/// Do not call this on strings you allocated yourself.
/// Passing NULL is safe and will return 0.
///
/// # Example
/// ```c
/// char* value = mystring_get_value(handle);
/// // Use value...
/// mystring_string_free(value);  // Must free!
/// ```
#[no_mangle]
pub extern "C" fn mystring_string_free(str: *mut c_char) -> i32 {
    ptr_or_return_int!(str);
    if unsafe { free_c_string(str) } {
        0
    } else {
        -1
    }
}

// ============================================================================
// C FFI - Error Handling
// ============================================================================

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
