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
/// - **1-999**: Reserved for cimple library errors
/// - **1000+**: Available for user library custom errors
///
/// # Example: User Library with Custom Errors
///
/// When building a library with cimple, you can define custom error codes
/// starting at 1000:
///
/// ```rust,no_run
/// use cimple::{ErrorCode, to_c_string};
/// use std::ffi::c_char;
/// use std::ptr::null_mut;
///
/// pub enum MyLibError {
///     // Wrap cimple errors (codes 1-999)
///     Cimple(cimple::Error),
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
///             MyLibError::Cimple(e) => e.code() as i32,
///             MyLibError::DatabaseError(_) => 1000,
///             MyLibError::NetworkTimeout(_) => 1001,
///             MyLibError::InvalidCredentials(_) => 1002,
///         }
///     }
///     
///     pub fn to_string(&self) -> String {
///         match self {
///             MyLibError::Cimple(e) => e.to_string(),
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
///         // Handle cimple error
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
    // 6-999: Reserved for future cimple library errors
    // 1000+: Available for user library custom errors
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
}

impl Error {
    /// Returns the last error as String
    pub fn last_message() -> Option<String> {
        LAST_ERROR.with(|prev| prev.borrow().as_ref().map(|e| e.to_string()))
    }

    /// Returns the last error code
    ///
    /// This is useful for creating typed exceptions in language bindings.
    /// Returns `ErrorCode::Ok` if no error is set.
    pub fn last_code() -> ErrorCode {
        LAST_ERROR.with(|prev| {
            prev.borrow()
                .as_ref()
                .map(|e| e.code())
                .unwrap_or(ErrorCode::Ok)
        })
    }

    /// Gets the error code for this error
    ///
    /// Maps each Error variant to its corresponding ErrorCode for FFI use.
    pub fn code(&self) -> ErrorCode {
        match self {
            Error::NullParameter(_) => ErrorCode::NullParameter,
            Error::StringTooLong(_) => ErrorCode::StringTooLong,
            Error::InvalidHandle(_) => ErrorCode::InvalidHandle,
            Error::WrongHandleType(_) => ErrorCode::WrongHandleType,
            Error::Other(_) => ErrorCode::Other,
        }
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
