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

pub type Result<T> = std::result::Result<T, Error>;

// LAST_ERROR handling borrowed from Copyright (c) 2018 Michael Bryan
thread_local! {
    static LAST_ERROR: RefCell<Option<Error>> = const { RefCell::new(None) };
}

/// Error - FFI error container with variant name and message
///
/// This struct holds errors in a format designed for cross-language FFI bindings.
/// Errors are formatted as: `"VariantName: details"`
///
/// Library developers implement `From` to convert their error types to this struct.
///
/// # Format Convention
///
/// Errors follow the format: `"VariantName: message details"`
/// - **VariantName**: The error type (for parsing in language bindings)
/// - **message details**: Human-readable description
///
/// This format allows language bindings to parse the variant name and create
/// typed exceptions/errors in the target language.
///
/// # Example
///
/// ```rust,ignore
/// // Your library's error type
/// #[derive(Debug)]
/// pub enum MyError {
///     ParseError(String),
///     ValidationError,
/// }
///
/// // Implement Display for user-friendly messages
/// impl std::fmt::Display for MyError {
///     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
///         match self {
///             MyError::ParseError(s) => write!(f, "parse failed: {}", s),
///             MyError::ValidationError => write!(f, "validation failed"),
///         }
///     }
/// }
///
/// impl std::error::Error for MyError {}
///
/// // Convert to cimpl::Error for FFI
/// impl From<MyError> for cimpl::Error {
///     fn from(e: MyError) -> Self {
///         // Option 1: Automatic (uses Debug for variant, Display for message)
///         cimpl::Error::from_error(e)
///         
///         // Option 2: Manual control
///         match e {
///             MyError::ParseError(s) => cimpl::Error::new("ParseError", s),
///             MyError::ValidationError => cimpl::Error::new("ValidationError", e.to_string()),
///         }
///     }
/// }
/// ```
///
#[derive(Debug, Clone)]
pub struct Error {
    message: String,
}

impl Error {
    /// Creates a new error with variant name and message
    ///
    /// The error will be formatted as: `"variant: message"`
    ///
    /// # Example
    /// ```
    /// # use cimpl::Error;
    /// let err = Error::new("ParseError", "invalid character 'x'");
    /// assert_eq!(err.message(), "ParseError: invalid character 'x'");
    /// assert_eq!(err.variant(), Some("ParseError"));
    /// assert_eq!(err.details(), Some("invalid character 'x'"));
    /// ```
    pub fn new(variant: &str, message: impl Into<String>) -> Self {
        Self {
            message: format!("{}: {}", variant, message.into()),
        }
    }

    /// Creates an error from a std::error::Error
    ///
    /// Extracts the variant name from Debug output and uses Display for the message.
    /// Falls back to "Unknown" variant if Debug format cannot be parsed.
    ///
    /// # Example
    /// ```
    /// # use cimpl::Error;
    /// # #[derive(Debug)]
    /// # enum MyError { Parse }
    /// # impl std::fmt::Display for MyError {
    /// #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    /// #         write!(f, "parse failed")
    /// #     }
    /// # }
    /// # impl std::error::Error for MyError {}
    /// let my_err = MyError::Parse;
    /// let err = Error::from_error(my_err);
    /// assert_eq!(err.variant(), Some("Parse"));
    /// ```
    pub fn from_error<E: std::error::Error>(e: E) -> Self {
        let debug = format!("{:?}", e);
        
        // Extract variant name from Debug output
        // Works with derived Debug: "Variant", "Variant(data)", "Variant { field }"
        let variant = debug
            .split(|c| c == '(' || c == '{')
            .next()
            .and_then(|s| {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            })
            .unwrap_or("Unknown");

        Self::new(variant, e.to_string())
    }

    /// Returns the full formatted error message
    ///
    /// Format: `"VariantName: details"`
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Extracts the variant name from the error message
    ///
    /// Returns None if the message doesn't contain the expected format.
    pub fn variant(&self) -> Option<&str> {
        self.message.split_once(": ").map(|(v, _)| v)
    }

    /// Extracts the details from the error message (without variant name)
    ///
    /// Returns None if the message doesn't contain the expected format.
    pub fn details(&self) -> Option<&str> {
        self.message.split_once(": ").map(|(_, d)| d)
    }

    // Convenience constructors for common cimpl internal errors

    /// Creates a null parameter error
    pub fn null_parameter(param: impl Into<String>) -> Self {
        Self::new("NullParameter", param.into())
    }

    /// Creates a string too long error
    pub fn string_too_long(param: impl Into<String>) -> Self {
        Self::new("StringTooLong", param.into())
    }

    /// Creates an untracked pointer error
    pub fn untracked_pointer(ptr: u64) -> Self {
        Self::new("UntrackedPointer", format!("0x{:x}", ptr))
    }

    /// Creates a wrong pointer type error
    pub fn wrong_pointer_type(ptr: u64) -> Self {
        Self::new("WrongPointerType", format!("0x{:x}", ptr))
    }

    /// Creates a mutex poisoned error
    pub fn mutex_poisoned() -> Self {
        Self::new("MutexPoisoned", "thread panic detected")
    }

    /// Creates an invalid buffer size error
    pub fn invalid_buffer_size(size: usize, param: &str) -> Self {
        Self::new("InvalidBufferSize", format!("{} for '{}'", size, param))
    }

    /// Creates a generic "other" error
    pub fn other(msg: impl Into<String>) -> Self {
        Self::new("Other", msg.into())
    }

    /// Peeks at the last error message without clearing it
    ///
    /// Returns None if no error is set. This does not clear the error.
    pub fn last_message() -> Option<String> {
        LAST_ERROR.with(|prev| prev.borrow().as_ref().map(|e| e.message.clone()))
    }

    /// Sets this error as the last error
    pub fn set_last(self) {
        LAST_ERROR.with(|prev| *prev.borrow_mut() = Some(self));
    }

    /// Takes the last error and clears it
    ///
    /// This is rarely needed - errors naturally get overwritten by new errors.
    /// Provided for completeness and testing.
    pub fn take_last() -> Option<Error> {
        LAST_ERROR.with(|prev| prev.borrow_mut().take())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::new("TestError", "test message");
        assert_eq!(err.message(), "TestError: test message");
        assert_eq!(err.variant(), Some("TestError"));
        assert_eq!(err.details(), Some("test message"));
    }

    #[test]
    fn test_null_parameter_error() {
        let err = Error::null_parameter("input_ptr");
        assert_eq!(err.variant(), Some("NullParameter"));
        assert!(err.message().contains("NullParameter"));
        assert!(err.details().unwrap().contains("input_ptr"));
    }

    #[test]
    fn test_string_too_long_error() {
        let err = Error::string_too_long("name");
        assert_eq!(err.variant(), Some("StringTooLong"));
        assert!(err.details().unwrap().contains("name"));
    }

    #[test]
    fn test_untracked_pointer_error() {
        let err = Error::untracked_pointer(0xdeadbeef);
        assert_eq!(err.variant(), Some("UntrackedPointer"));
        assert!(err.details().unwrap().contains("0xdeadbeef"));
    }

    #[test]
    fn test_wrong_pointer_type_error() {
        let err = Error::wrong_pointer_type(0x12345678);
        assert_eq!(err.variant(), Some("WrongPointerType"));
        assert!(err.details().unwrap().contains("0x12345678"));
    }

    #[test]
    fn test_mutex_poisoned_error() {
        let err = Error::mutex_poisoned();
        assert_eq!(err.variant(), Some("MutexPoisoned"));
        assert!(err.details().unwrap().contains("thread panic"));
    }

    #[test]
    fn test_invalid_buffer_size_error() {
        let err = Error::invalid_buffer_size(1000, "data");
        assert_eq!(err.variant(), Some("InvalidBufferSize"));
        assert!(err.details().unwrap().contains("1000"));
        assert!(err.details().unwrap().contains("data"));
    }

    #[test]
    fn test_other_error() {
        let err = Error::other("custom message");
        assert_eq!(err.variant(), Some("Other"));
        assert_eq!(err.details(), Some("custom message"));
    }

    #[test]
    fn test_from_error() {
        #[derive(Debug)]
        enum TestError {
            Parse(String),
            Validate,
        }

        impl std::fmt::Display for TestError {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    TestError::Parse(s) => write!(f, "parse failed: {}", s),
                    TestError::Validate => write!(f, "validation failed"),
                }
            }
        }

        impl std::error::Error for TestError {}

        // Test Parse variant
        let test_err = TestError::Parse("bad input".to_string());
        let err = Error::from_error(test_err);
        
        assert_eq!(err.variant(), Some("Parse"));
        assert!(err.details().unwrap().contains("parse failed"));
        
        // Test Validate variant
        let test_err2 = TestError::Validate;
        let err2 = Error::from_error(test_err2);
        
        assert_eq!(err2.variant(), Some("Validate"));
        assert!(err2.details().unwrap().contains("validation failed"));
    }

    #[test]
    fn test_last_error_storage() {
        // Set an error
        let err = Error::new("TestError", "test message");
        err.set_last();

        // Retrieve it
        let msg = Error::last_message();
        assert_eq!(msg, Some("TestError: test message".to_string()));
    }

    #[test]
    fn test_last_error_none() {
        // Clear any existing error
        Error::take_last();

        assert_eq!(Error::last_message(), None);
    }

    #[test]
    fn test_take_last_clears_error() {
        // Set an error
        Error::new("Temporary", "temp").set_last();

        // Take it (should clear)
        let err = Error::take_last();
        assert!(err.is_some());
        assert_eq!(err.unwrap().variant(), Some("Temporary"));

        // Verify it's cleared
        assert_eq!(Error::last_message(), None);
    }

    #[test]
    fn test_display_trait() {
        let err = Error::new("DisplayTest", "test message");
        let displayed = format!("{}", err);
        assert_eq!(displayed, "DisplayTest: test message");
    }

    #[test]
    fn test_debug_trait() {
        let err = Error::new("DebugTest", "test message");
        let debugged = format!("{:?}", err);
        assert!(debugged.contains("DebugTest: test message"));
    }

    #[test]
    fn test_error_trait() {
        let err = Error::new("TraitTest", "test");
        // Verify it implements Error trait
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_thread_local_isolation() {
        use std::thread;

        // Set error in main thread
        Error::new("MainThread", "main").set_last();

        // Spawn a new thread and verify it has no error
        let handle = thread::spawn(|| {
            assert_eq!(Error::last_message(), None);
            Error::new("SpawnedThread", "spawned").set_last();
            assert!(Error::last_message().is_some());
        });

        handle.join().unwrap();

        // Main thread should still have its error
        assert!(Error::last_message().unwrap().contains("MainThread"));
    }

    #[test]
    fn test_error_overwrite() {
        // Set first error
        Error::new("First", "first message").set_last();

        // Set second error (should overwrite)
        Error::new("Second", "second message").set_last();
        
        let msg = Error::last_message().unwrap();
        assert!(msg.contains("Second"));
        assert!(msg.contains("second message"));
    }

    #[test]
    fn test_variant_and_details_extraction() {
        let err = Error::new("MyError", "something went wrong");
        
        assert_eq!(err.variant(), Some("MyError"));
        assert_eq!(err.details(), Some("something went wrong"));
        assert_eq!(err.message(), "MyError: something went wrong");
    }

    #[test]
    fn test_variant_with_colon_in_details() {
        let err = Error::new("IoError", "file not found: /path/to/file");
        
        assert_eq!(err.variant(), Some("IoError"));
        assert_eq!(err.details(), Some("file not found: /path/to/file"));
    }
}
