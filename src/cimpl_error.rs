// Copyright 2026 Adobe. All rights reserved.
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

pub type Result<T> = std::result::Result<T, CimplError>;

// LAST_ERROR handling borrowed from Copyright (c) 2018 Michael Bryan
thread_local! {
    static LAST_ERROR: RefCell<Option<CimplError>> = const { RefCell::new(None) };
}

/// CimplError - holds an error code and message
///
/// This is a simple struct that can represent any error with an integer code
/// and a descriptive message. Library developers implement `From` to convert
/// their error types to this struct.
///
/// # Error Code Ranges
///
/// - **0**: No error (returned by error_code functions when no error is set)
/// - **1-99**: Reserved for cimpl infrastructure errors
/// - **100+**: Available for library-specific errors
///
/// # Example
///
/// ```rust,ignore
/// // Define your error codes
/// #[repr(i32)]
/// pub enum MyLibError {
///     ParseError = 100,
///     ValidationError = 101,
/// }
///
/// // Implement From for your error type
/// impl From<mylib::Error> for CimplError {
///     fn from(e: mylib::Error) -> Self {
///         match e {
///             mylib::Error::Parse(msg) => {
///                 CimplError::new(
///                     MyLibError::ParseError as i32,
///                     format!("ParseError: {}", msg)
///                 )
///             }
///             mylib::Error::Validation(msg) => {
///                 CimplError::new(
///                     MyLibError::ValidationError as i32,
///                     format!("ValidationError: {}", msg)
///                 )
///             }
///         }
///     }
/// }
///
/// // Macros automatically use From/Into
/// let result = ok_or_return_null!(parse_something());
/// ```
///
#[derive(Debug, Clone)]
pub struct CimplError {
    code: i32,
    message: String,
}

impl CimplError {
    /// Creates a new error with the given code and message
    pub fn new<S: Into<String>>(code: i32, message: S) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
    /// Returns the error code
    pub fn code(&self) -> i32 {
        self.code
    }

    /// Returns the error message
    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn null_parameter<S: Into<String>>(param: S) -> Self {
        Self::new(1, format!("NullParameter: {}", param.into()))
    }

    pub fn string_too_long<S: Into<String>>(param: S) -> Self {
        Self::new(2, format!("StringTooLong: {}", param.into()))
    }

    pub fn untracked_pointer(ptr: u64) -> Self {
        Self::new(3, format!("UntrackedPointer: 0x{:x}", ptr))
    }

    pub fn wrong_pointer_type(ptr: u64) -> Self {
        Self::new(4, format!("WrongPointerType: 0x{:x}", ptr))
    }

    pub fn mutex_poisoned() -> Self {
        Self::new(6, "MutexPoisoned: thread panic detected".to_string())
    }

    pub fn invalid_buffer_size(size: usize, param: &str) -> Self {
        Self::new(7, format!("InvalidBufferSize: {} for '{}'", size, param))
    }

    pub fn other<S: Into<String>>(msg: S) -> Self {
        Self::new(5, format!("Other: {}", msg.into()))
    }

    /// Peeks at the last error message without clearing it
    ///
    /// Returns None if no error is set. This does not clear the error.
    pub fn last_message() -> Option<String> {
        LAST_ERROR.with(|prev| prev.borrow().as_ref().map(|e| e.message.clone()))
    }

    /// Peeks at the last error code without clearing it
    ///
    /// Returns 0 if no error is set. This does not clear the error.
    ///
    /// # Error Code Convention
    ///
    /// - **0**: No error set
    /// - **1-99**: cimpl infrastructure errors
    /// - **100+**: Library-specific errors
    pub fn last_code() -> i32 {
        LAST_ERROR.with(|prev| prev.borrow().as_ref().map(|e| e.code).unwrap_or(0))
    }

    /// Sets this error as the last error
    pub fn set_last(self) {
        LAST_ERROR.with(|prev| *prev.borrow_mut() = Some(self));
    }

    /// Takes the last error and clears it
    ///
    /// This is rarely needed - errors naturally get overwritten by new errors.
    /// Provided for completeness and testing.
    pub fn take_last() -> Option<CimplError> {
        LAST_ERROR.with(|prev| prev.borrow_mut().take())
    }
}

impl std::fmt::Display for CimplError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CimplError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = CimplError::new(100, "test error");
        assert_eq!(err.code(), 100);
        assert_eq!(err.message(), "test error");
    }

    #[test]
    fn test_null_parameter_error() {
        let err = CimplError::null_parameter("input_ptr");
        assert_eq!(err.code(), 1);
        assert!(err.message().contains("NullParameter"));
        assert!(err.message().contains("input_ptr"));
    }

    #[test]
    fn test_string_too_long_error() {
        let err = CimplError::string_too_long("name");
        assert_eq!(err.code(), 2);
        assert!(err.message().contains("StringTooLong"));
        assert!(err.message().contains("name"));
    }

    #[test]
    fn test_untracked_pointer_error() {
        let err = CimplError::untracked_pointer(0xdeadbeef);
        assert_eq!(err.code(), 3);
        assert!(err.message().contains("UntrackedPointer"));
        assert!(err.message().contains("0xdeadbeef"));
    }

    #[test]
    fn test_wrong_pointer_type_error() {
        let err = CimplError::wrong_pointer_type(0x12345678);
        assert_eq!(err.code(), 4);
        assert!(err.message().contains("WrongPointerType"));
        assert!(err.message().contains("0x12345678"));
    }

    #[test]
    fn test_mutex_poisoned_error() {
        let err = CimplError::mutex_poisoned();
        assert_eq!(err.code(), 6);
        assert!(err.message().contains("MutexPoisoned"));
    }

    #[test]
    fn test_invalid_buffer_size_error() {
        let err = CimplError::invalid_buffer_size(1000, "data");
        assert_eq!(err.code(), 7);
        assert!(err.message().contains("InvalidBufferSize"));
        assert!(err.message().contains("1000"));
        assert!(err.message().contains("data"));
    }

    #[test]
    fn test_other_error() {
        let err = CimplError::other("custom message");
        assert_eq!(err.code(), 5);
        assert!(err.message().contains("Other"));
        assert!(err.message().contains("custom message"));
    }

    #[test]
    fn test_last_error_storage() {
        // Set an error
        let err = CimplError::new(42, "test error");
        err.set_last();
        
        // Retrieve it
        assert_eq!(CimplError::last_code(), 42);
        let msg = CimplError::last_message();
        assert_eq!(msg, Some("test error".to_string()));
    }

    #[test]
    fn test_last_error_none() {
        // Clear any existing error
        CimplError::take_last();
        
        assert_eq!(CimplError::last_code(), 0);
        assert_eq!(CimplError::last_message(), None);
    }

    #[test]
    fn test_take_last_clears_error() {
        // Set an error
        CimplError::new(99, "temporary").set_last();
        assert_eq!(CimplError::last_code(), 99);
        
        // Take it (should clear)
        let err = CimplError::take_last();
        assert!(err.is_some());
        assert_eq!(err.unwrap().code(), 99);
        
        // Verify it's cleared
        assert_eq!(CimplError::last_code(), 0);
        assert_eq!(CimplError::last_message(), None);
    }

    #[test]
    fn test_display_trait() {
        let err = CimplError::new(123, "display test");
        let displayed = format!("{}", err);
        assert_eq!(displayed, "display test");
    }

    #[test]
    fn test_debug_trait() {
        let err = CimplError::new(456, "debug test");
        let debugged = format!("{:?}", err);
        assert!(debugged.contains("456"));
        assert!(debugged.contains("debug test"));
    }

    #[test]
    fn test_error_trait() {
        let err = CimplError::new(789, "error trait test");
        // Verify it implements Error trait
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_thread_local_isolation() {
        use std::thread;
        
        // Set error in main thread
        CimplError::new(1, "main thread").set_last();
        
        // Spawn a new thread and verify it has no error
        let handle = thread::spawn(|| {
            assert_eq!(CimplError::last_code(), 0);
            CimplError::new(2, "spawned thread").set_last();
            assert_eq!(CimplError::last_code(), 2);
        });
        
        handle.join().unwrap();
        
        // Main thread should still have its error
        assert_eq!(CimplError::last_code(), 1);
    }

    #[test]
    fn test_error_overwrite() {
        // Set first error
        CimplError::new(100, "first").set_last();
        assert_eq!(CimplError::last_code(), 100);
        
        // Set second error (should overwrite)
        CimplError::new(200, "second").set_last();
        assert_eq!(CimplError::last_code(), 200);
        assert_eq!(CimplError::last_message(), Some("second".to_string()));
    }
}
