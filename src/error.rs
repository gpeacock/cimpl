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

use std::cell::RefCell;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

// LAST_ERROR handling borrowed from Copyright (c) 2018 Michael Bryan
thread_local! {
    static LAST_ERROR: RefCell<Option<Error>> = const { RefCell::new(None) };
}

/// Error codes for FFI - enables language bindings to create typed exceptions
///
/// This enum provides integer error codes that can be used by C/C++/Python/etc
/// to create proper exception types. Each variant maps to an Error enum case.
///
/// # Error Code Convention
///
/// Error codes follow this convention:
/// - **0**: Ok (no error)
/// - **1-999**: Reserved for cimpl library errors
/// - **1000+**: Available for user library custom errors
///
/// # Example: User Library with Custom Errors
///
/// When building a library with cimpl, you can define custom error codes
/// starting at 1000:
///
/// ```rust,no_run
/// use cimpl::{ErrorCode, to_c_string};
/// use std::ffi::c_char;
/// use std::ptr::null_mut;
///
/// pub enum MyLibError {
///     // Wrap cimpl errors (codes 1-999)
///     Cimpl(cimpl::Error),
///     
///     // Custom errors (codes 1000+)
///     DatabaseError(String),      // code 1000
///     NetworkTimeout(String),     // code 1001
///     InvalidCredentials(String), // code 1002
/// }
///
/// impl MyLibError {
///     pub fn code(&self) -> i32 {
///         match self {
///             MyLibError::Cimpl(e) => e.code_as_i32(),
///             MyLibError::DatabaseError(_) => 1000,
///             MyLibError::NetworkTimeout(_) => 1001,
///             MyLibError::InvalidCredentials(_) => 1002,
///         }
///     }
///     
///     pub fn to_string(&self) -> String {
///         match self {
///             MyLibError::Cimpl(e) => e.to_string(),
///             MyLibError::DatabaseError(msg) => format!("DatabaseError: {}", msg),
///             MyLibError::NetworkTimeout(msg) => format!("NetworkTimeout: {}", msg),
///             MyLibError::InvalidCredentials(msg) => format!("InvalidCredentials: {}", msg),
///         }
///     }
/// }
///
/// // Expose to C with same pattern
/// fn get_last_error() -> Option<MyLibError> { None } // stub
///
/// #[no_mangle]
/// pub extern "C" fn mylib_error_code() -> i32 {
///     get_last_error().map(|e| e.code()).unwrap_or(0)
/// }
///
/// #[no_mangle]
/// pub extern "C" fn mylib_last_error() -> *mut c_char {
///     get_last_error().map(|e| to_c_string(e.to_string())).unwrap_or(null_mut())
/// }
/// ```
///
/// # C/C++ Usage
/// ```cpp
/// if (mylib_operation() != 0) {
///     int code = mylib_error_code();
///     if (code == ERROR_NULL_PARAMETER) {
///         // Handle cimpl error
///     } else if (code >= 1000) {
///         // Handle custom library error
///     }
/// }
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// No error occurred
    Ok = 0,
    /// A required parameter was NULL
    NullParameter = 1,
    /// String exceeds maximum allowed length
    StringTooLong = 2,
    /// Handle value is invalid or already freed
    InvalidHandle = 3,
    /// Handle type doesn't match the expected type
    WrongHandleType = 4,
    /// Other error occurred
    Other = 5,
    // 6-99: Reserved for future cimpl library errors
    // 100+: Available for library-specific custom errors
}

#[derive(Error, Debug)]
/// Defines all possible FFI errors
pub enum Error {
    #[error("NullParameter: {0}")]
    NullParameter(String),
    #[error("StringTooLong: {0}")]
    StringTooLong(String),
    #[error("InvalidHandle: {0}")]
    InvalidHandle(u64),
    #[error("WrongHandleType: {0}")]
    WrongHandleType(u64),
    #[error("Other: {0}")]
    Other(String),
    #[error("{1}")]
    LibraryError(i32, String),
}

impl Error {
    /// Returns the last error as String
    pub fn last_message() -> Option<String> {
        LAST_ERROR.with(|prev| prev.borrow().as_ref().map(|e| e.to_string()))
    }

    /// Returns the last error code
    ///
    /// This is useful for creating typed exceptions in language bindings.
    /// Returns 0 (ErrorCode::Ok) if no error is set.
    ///
    /// Error code ranges:
    /// - 0: No error
    /// - 1-99: Core cimpl infrastructure errors
    /// - 100+: Library-specific errors
    pub fn last_code() -> i32 {
        LAST_ERROR.with(|prev| prev.borrow().as_ref().map(|e| e.code_as_i32()).unwrap_or(0))
    }

    /// Gets the error code for this error as an i32
    ///
    /// Maps each Error variant to its corresponding error code for FFI use.
    pub fn code_as_i32(&self) -> i32 {
        match self {
            Error::NullParameter(_) => ErrorCode::NullParameter as i32,
            Error::StringTooLong(_) => ErrorCode::StringTooLong as i32,
            Error::InvalidHandle(_) => ErrorCode::InvalidHandle as i32,
            Error::WrongHandleType(_) => ErrorCode::WrongHandleType as i32,
            Error::Other(_) => ErrorCode::Other as i32,
            Error::LibraryError(code, _) => *code,
        }
    }

    /// Converts a library error using a mapping function
    ///
    /// This method provides a simple way to map external library errors to cimpl errors
    /// using a function that directly maps errors to (code, name) pairs.
    ///
    /// # Arguments
    ///
    /// * `e` - The external error to convert
    /// * `mapper` - A function that takes a reference to the error and returns `(code, name)`
    ///
    /// # Note
    ///
    /// This method is primarily used internally by the `ok_or_return!` macros via the
    /// `ERROR_MAPPER` constant pattern. You typically don't need to call it directly.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Define error codes
    /// #[no_mangle]
    /// pub static ERROR_UUID_PARSE_ERROR: i32 = 100;
    ///
    /// // Define mapper function
    /// const ERROR_MAPPER: fn(&uuid::Error) -> (i32, &'static str) =
    ///     |_e| (ERROR_UUID_PARSE_ERROR, "ParseError");
    ///
    /// // The ok_or_return! macros use ERROR_MAPPER automatically
    /// #[no_mangle]
    /// pub extern "C" fn uuid_parse(s: *const c_char) -> *mut Uuid {
    ///     let s_str = cstr_or_return_null!(s);
    ///     let uuid = ok_or_return_null!(Uuid::from_str(&s_str));
    ///     box_tracked!(uuid)
    /// }
    /// ```
    pub fn from_mapper<E: std::fmt::Display>(e: E, mapper: fn(&E) -> (i32, &'static str)) -> Self {
        let (code, name) = mapper(&e);
        Error::LibraryError(code, format!("{}: {}", name, e))
    }

    /// Sets the last error
    pub fn set_last(self) {
        LAST_ERROR.with(|prev| *prev.borrow_mut() = Some(self));
    }

    /// Takes the the last error and clears it
    pub fn take_last() -> Option<Error> {
        LAST_ERROR.with(|prev| prev.borrow_mut().take())
    }
}
