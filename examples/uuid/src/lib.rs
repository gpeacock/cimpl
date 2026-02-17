//! UUID FFI Example - Wrapping an External Crate
//!
//! This example demonstrates how to create FFI bindings for an existing,
//! popular Rust crate (uuid) that you don't control.
//!
//! ## Key Pattern: Direct Usage
//!
//! Unlike the ValueConverter example which has its own business logic,
//! the uuid crate already provides a complete, clean API. So instead of
//! creating a wrapper, we directly use `uuid::Uuid` in the FFI layer.
//!
//! **This is the recommended approach when wrapping external crates.**
//!
//! ## What We Do Here
//!
//! 1. Re-export `uuid::Uuid` for convenience
//! 2. Define a simple `Error` type (for cases where we need our own errors)
//! 3. Convert `uuid::Error` to our `Error` for FFI compatibility
//! 4. Use `uuid::Uuid` methods directly in ffi.rs
//!
//! No unnecessary abstractions, no wrapper layer - just direct usage!

use thiserror::Error as ThisError;

/// Re-export the uuid::Uuid type
/// The FFI layer will use this directly
pub use uuid::Uuid;

/// Error type for UUID operations
///
/// In practice, uuid::Error is the main error type, but we define our own
/// to demonstrate error conversion and to have a stable error interface
/// if we ever add custom validation or operations.
#[derive(ThisError, Debug)]
pub enum Error {
    /// Error parsing a UUID string
    #[error("parse error: {0}")]
    ParseError(String),
}

/// Convert uuid::Error to our Error type
impl From<uuid::Error> for Error {
    fn from(e: uuid::Error) -> Self {
        Error::ParseError(e.to_string())
    }
}

// Conditionally include FFI module
#[cfg(feature = "ffi")]
pub mod ffi;
