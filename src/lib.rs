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

//! # cimpl - Simple C implementations from Rust
//!
//! Create clean, safe C FFI bindings that AI can automatically convert to any language.
//!
//! ## The Vision
//!
//! ```text
//! Rust + cimpl → Clean C API → AI-powered bindings → All languages
//! ```
//!
//! Write your library once in safe Rust, expose it through a clean C API using cimpl's macros,
//! and let AI generate high-quality bindings for Python, JavaScript, Lua, Ruby, C#, Java, Go, and more.
//!
//! ## Why cimpl?
//! 
//! This library emerged from real-world challenges with FFI in the [c2pa-rs](https://github.com/contentauth/c2pa-rs) 
//! project at Adobe. After experimenting with various Rust language binding tools, I found they all required 
//! decorating Rust code with language-specific annotations or special interface definition languages, generating 
//! volumes of incomprehensible glue code in the process.
//! 
//! Then came the insight: **AI excels at writing bindings for well-documented C APIs**. Rust natively supports 
//! C APIs, but writing them manually is tricky and error-prone. So cimpl was born - a library of macros and 
//! utilities that makes it safe and maintainable to write C bindings from Rust.
//! 
//! The result? Given this library, AI can generate both the C FFI bindings AND the language-specific bindings 
//! automatically. The UUID example in this crate was generated entirely by AI in 15 minutes with zero compilation 
//! errors, proving the concept works in practice. Look at the code. Everything generated, from the C header files to
//! to language bindings is well documented and readable. There is no incomprehensible glue code. 
//! 
//! Look at the examples in the `examples` directory. They are real-world examples of how to use cimpl to write
//! safe and maintainable C bindings. The UUID example was generated entirely by AI in 15 minutes with zero compilation
//! errors. The ValueConverter example shows patterns and how to use them.
//! 
//! ## Why cimpl?
//!
//! This library emerged from real-world challenges with FFI in the [c2pa-rs](https://github.com/contentauth/c2pa-rs) 
//! project at Adobe. After experimenting with various Rust language binding tools, I found they all required 
//! decorating Rust code with language-specific annotations or special interface definition languages, generating 
//! **Most Rust FFI examples are way to simple to be useful. Real FFI is much harder:**
//! - How do you return complex types like strings or structs?
//! - How do you propagate `Result<T, E>` errors across the FFI boundary?
//! - How do you handle object lifecycle (constructors, methods, destructors)?
//! - How do you prevent memory leaks and double-frees?
//! - How do you make errors usable in other languages?
//!
//! **cimpl solves the hard problems:**
//! - ✅ Type-safe pointer tracking with validation
//! - ✅ Automatic error handling with descriptive, parseable messages
//! - ✅ Memory leak detection in tests
//! - ✅ Clean macros for production patterns (not toy examples)
//! - ✅ Object-oriented APIs (structs with methods, not just functions)
//! - ✅ AI-friendly C headers (auto-generated via cbindgen)
//! - ✅ One codebase → many language bindings
//!
//! ## Quick Example
//!
//! ```rust,ignore
//! use cimpl::*;
//! use std::ffi::c_void;
//! use std::os::raw::c_char;
//! use thiserror::Error as ThisError;
//!
//! // Your library's error type (using thiserror for convenience)
//! #[derive(ThisError, Debug)]
//! pub enum Error {
//!     #[error("value out of range: {0}")]
//!     OutOfRange(String),
//!     
//!     #[error("invalid UTF-8: {0}")]
//!     InvalidUtf8(String),
//! }
//!
//! // Map to cimpl::Error for FFI - one line with from_error()!
//! impl From<Error> for cimpl::Error {
//!     fn from(e: Error) -> Self {
//!         cimpl::Error::from_error(e)  // Automatic: Debug → variant, Display → message
//!     }
//! }
//!
//! // Clean, safe FFI function
//! #[no_mangle]
//! pub extern "C" fn process_value(ptr: *mut MyType) -> *mut c_char {
//!     let obj = deref_or_return_null!(ptr, MyType);
//!     let result = ok_or_return_null!(obj.to_string());  // Error → cimpl::Error automatically
//!     to_c_string(result)
//! }
//!
//! // Memory management wrapper (required for namespace safety)
//! #[no_mangle]
//! pub extern "C" fn my_free(ptr: *mut c_void) -> i32 {
//!     cimpl::cimpl_free(ptr)  // Rust-level function, wrap in your C API
//! }
//! ```
//!
//! **That's it!** From this simple code:
//! - cbindgen generates a C header with proper namespace prefix
//! - Type validation ensures safety
//! - Errors map to descriptive strings: `"VariantName: details"`
//! - Memory is tracked automatically
//! - AI can generate bindings for any language
//!
//! ## Core Features
//!
//! ### String-Based Error Handling
//!
//! Errors use a consistent `"VariantName: details"` format that works across all languages:
//!
//! ```rust
//! use cimpl::Error;
//!
//! // Create errors manually
//! let err = Error::new("OutOfRange", "value must be between 0 and 100");
//!
//! // Or convert automatically from any std::error::Error
//! let err = Error::from_error(my_error);  // Uses Debug for variant, Display for message
//! ```
//!
//! This format is:
//! - ✅ Human-readable (developers)
//! - ✅ Machine-parseable (error handlers)
//! - ✅ AI-friendly (code generation)
//! - ✅ Cross-language (works in C, Python, Ruby, Swift, Go...)
//!
//! ### Pointer Safety Macros
//!
//! - [`box_tracked!`] - Allocate and track Box
//! - [`ptr_or_return!`] - Null pointer checks with automatic error messages
//! - [`deref_or_return!`] - Pointer validation and dereferencing (immutable)
//! - [`deref_mut_or_return!`] - Pointer validation and dereferencing (mutable)
//!
//! ### String Conversion
//!
//! - [`cstr_or_return!`] - C string to Rust with UTF-8 validation and bounds checking
//! - [`to_c_string()`] - Rust String to tracked C string
//! - [`option_to_c_string!`] - Option<String> to C string (NULL if None)
//!
//! ### Byte Array Handling
//!
//! - [`bytes_or_return!`] - Validate and convert C byte arrays
//! - [`to_c_bytes()`] - Rust Vec<u8> to tracked C byte array
//!
//! ### Result Handling
//!
//! - [`ok_or_return!`] - Result unwrapping with automatic error conversion
//! - [`ok_or_return_null!`] - Unwrap Result, return NULL on error
//! - [`ok_or_return_false!`] - Unwrap Result, return false on error
//! - Works with any error type implementing `Into<cimpl::Error>`
//!
//! ### Option Handling
//!
//! - [`some_or_return!`] - Option unwrapping with custom errors
//! - [`some_or_return_other_null!`] - Option with Error::other message, return NULL
//!
//! ## Memory Management
//!
//! All pointers allocated via `box_tracked!`, `arc_tracked!`, or the tracking functions are
//! registered in a global, thread-safe registry. Each library should wrap `cimpl_free()`:
//!
//! ```rust,ignore
//! #[no_mangle]
//! pub extern "C" fn mylib_free(ptr: *mut c_void) -> i32 {
//!     cimpl::cimpl_free(ptr)
//! }
//! ```
//!
//! This provides:
//! - **Namespace safety**: No symbol conflicts when linking multiple libraries
//! - **Type validation**: Wrong type returns error instead of crashing
//! - **Double-free protection**: Registry prevents freeing the same pointer twice
//! - **Leak detection**: Unfreed pointers reported at program exit
//!
//! ## AI-Friendly Design
//!
//! cimpl is designed to enable AI code generation. See [`AI_WORKFLOW.md`] in the repository for:
//! - Pre-flight checklist for catching anti-patterns
//! - Complete macro reference with decision trees
//! - Common mistakes to avoid
//! - Step-by-step guidance for generating FFI code
//!
//! **Proven**: The UUID example was generated entirely by AI in 15 minutes with zero errors.
//!
//! ## Examples
//!
//! The crate includes two complete, production-ready examples:
//!
//! - **`examples/reference/`** - ValueConverter showing all FFI patterns
//!   - Clean lib.rs/ffi.rs separation
//!   - Full Python bindings with ctypes
//!   - Demonstrates struct methods, constructors, error handling
//!
//! - **`examples/uuid/`** - Real-world external crate wrapping
//!   - 15 FFI functions exposing uuid crate
//!   - Complete Python bindings
//!   - AI-generated in 15 minutes from documentation
//!   - Demonstrates direct external crate usage pattern
//!
//! ## Philosophy
//!
//! See [`PHILOSOPHY.md`] in the repository for the complete design rationale. Key insights:
//!
//! 1. **The C ABI is timeless** - Build on solid ground
//! 2. **Safety through validation** - Not through complexity
//! 3. **Standard C conventions** - Developers know what to expect
//! 4. **Universal patterns** - One way to do things
//! 5. **AI-friendly design** - Enable code generation
//! 6. **Language independence** - Your C API outlives any specific tooling
//!
//! Build once in Rust. Expose through C. Use everywhere.
//!
//! [`AI_WORKFLOW.md`]: https://github.com/gpeacock/cimpl/blob/main/AI_WORKFLOW.md
//! [`PHILOSOPHY.md`]: https://github.com/gpeacock/cimpl/blob/main/PHILOSOPHY.md

// Declare foundational modules first
pub mod error;
pub mod utils;

// Then macros that depend on them
#[macro_use]
pub mod macros;

// Re-export main types and functions for convenience
pub use error::{Error, Result};
// Convenience alias to avoid name conflicts
pub use error::Error as CimplError;
pub use utils::{
    cimpl_free, safe_slice_from_raw_parts, to_c_bytes, to_c_string, track_arc, track_arc_mutex,
    track_box,
};

// Re-export internal utilities (for macro use only - not part of public API)
#[doc(hidden)]
pub use utils::validate_pointer;
