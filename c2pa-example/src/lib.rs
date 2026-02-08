//! # C2PA Rust SDK - C FFI Bindings
//!
//! Minimal FFI wrapper around c2pa-rs focusing on Context and Settings
//!
//! ## ⚠️  MACRO-FIRST DEVELOPMENT ⚠️
//!
//! **See `cimpl/src/macros.rs` for the anti-pattern checklist and macro guide.**
//!
//! Before writing code, verify you're not using manual patterns:
//! - `if ptr.is_null()` → Use `deref_or_return!` macro
//! - `match result { Ok/Err }` → Use `ok_or_return!` macro  
//! - `unsafe { &*ptr }` → Use `deref_or_return!` macro
//!
//! **When in doubt, check the macro documentation first!**

use std::os::raw::c_char;

use cimpl::{
    box_tracked, cimpl_free, cstr_or_return, cstr_or_return_null,
    deref_or_return_neg, deref_or_return_null, deref_mut_or_return_neg,
    ok_or_return, ok_or_return_null, option_to_c_string, to_c_string, CimplError,
};

// ============================================================================
// Error Handling
// ============================================================================

/// Error codes for C2PA operations (100+)
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum C2paError {
    // C2PA-specific errors (100+)
    InvalidSettings = 100,    // Settings JSON/TOML parse error
    SignerError = 101,        // Signer creation/config error
    ContextError = 102,       // Context creation error
    InvalidFormat = 103,      // Invalid file format
    IoError = 104,            // File I/O error
    SerializationError = 105, // JSON/TOML serialization error
}

// Internal error enum that wraps various error types (not exported to C)
#[derive(Debug)]
enum C2paInternalError {
    C2pa(c2pa::Error),
    Json(serde_json::Error),
    Other(String),
}

impl From<c2pa::Error> for C2paInternalError {
    fn from(e: c2pa::Error) -> Self {
        C2paInternalError::C2pa(e)
    }
}

impl From<serde_json::Error> for C2paInternalError {
    fn from(e: serde_json::Error) -> Self {
        C2paInternalError::Json(e)
    }
}

impl From<C2paInternalError> for CimplError {
    fn from(e: C2paInternalError) -> Self {
        let (code, name, msg) = match e {
            C2paInternalError::C2pa(e) => {
                let (c, n) = match &e {
                    c2pa::Error::InvalidAsset(_) => (C2paError::InvalidFormat, "InvalidFormat"),
                    c2pa::Error::IoError(_) => (C2paError::IoError, "IoError"),
                    c2pa::Error::BadParam(_) => (C2paError::InvalidSettings, "InvalidSettings"),
                    _ => (C2paError::ContextError, "ContextError"),
                };
                (c, n, format!("{}", e))
            }
            C2paInternalError::Json(e) => {
                (C2paError::SerializationError, "SerializationError", format!("{}", e))
            }
            C2paInternalError::Other(msg) => {
                (C2paError::SerializationError, "SerializationError", msg)
            }
        };
        CimplError::new(code as i32, format!("{}: {}", name, msg))
    }
}

// ============================================================================
// Context (stable pointer for builder pattern)
// ============================================================================

/// C2PA Context - wraps c2pa::Context with a stable pointer for FFI
/// The pointer never changes, but the inner Context can be replaced to support builder patterns
pub struct C2paContext {
    inner: c2pa::Context,
}

impl C2paContext {
    fn new() -> Self {
        Self {
            inner: c2pa::Context::new(),
        }
    }
}

// ============================================================================
// Context API
// ============================================================================

/// Create a new C2PA Context with default settings
///
/// The Context is the central configuration object for all C2PA operations.
/// Returns NULL on error.
///
/// # Example
/// ```c
/// C2paContext* ctx = c2pa_context_new();
/// if (!ctx) {
///     printf("Error: %s\n", c2pa_last_error());
///     return 1;
/// }
/// c2pa_context_free(ctx);
/// ```
#[no_mangle]
pub extern "C" fn c2pa_context_new() -> *mut C2paContext {
    let ctx = C2paContext::new();
    box_tracked!(ctx)
}

/// Configure Context with JSON settings (builder-style, mutates in place)
///
/// This modifies the Context and returns the same pointer for chaining.
/// Returns 0 on success, non-zero on error.
///
/// # Parameters
/// - `ctx`: Context to modify
/// - `settings_json`: JSON string containing settings configuration
///
/// # Example
/// ```c
/// C2paContext* ctx = c2pa_context_new();
/// if (c2pa_context_with_settings(ctx, "{\"verify\": {\"verify_after_sign\": true}}") != 0) {
///     printf("Error: %s\n", c2pa_last_error());
/// }
/// ```
#[no_mangle]
pub extern "C" fn c2pa_context_with_settings(
    ctx: *mut C2paContext,
    settings_json: *const c_char,
) -> i32 {
    let json = cstr_or_return!(settings_json, -1);
    let ctx_ref = deref_mut_or_return_neg!(ctx, C2paContext);
    
    // Create new Context with settings and replace the inner one
    ok_or_return!(
        c2pa::Context::new().with_settings(json).map_err(C2paInternalError::C2pa),
        |new_ctx| {
            ctx_ref.inner = new_ctx;
            0
        },
        -1
    )
}

/// Configure Context with TOML settings (builder-style, mutates in place)
///
/// This modifies the Context and returns 0 on success for chaining.
///
/// # Parameters
/// - `ctx`: Context to modify  
/// - `settings_toml`: TOML string containing settings configuration
#[no_mangle]
pub extern "C" fn c2pa_context_with_settings_toml(
    ctx: *mut C2paContext,
    settings_toml: *const c_char,
) -> i32 {
    let toml = cstr_or_return!(settings_toml, -1);
    let ctx_ref = deref_mut_or_return_neg!(ctx, C2paContext);
    
    ok_or_return!(
        c2pa::Context::new().with_settings(toml).map_err(C2paInternalError::C2pa),
        |new_ctx| {
            ctx_ref.inner = new_ctx;
            0
        },
        -1
    )
}

/// Free a Context
///
/// # Safety
/// The pointer must be valid and not used after this call.
#[no_mangle]
pub extern "C" fn c2pa_context_free(ctx: *mut C2paContext) -> i32 {
    cimpl_free(ctx as *mut std::ffi::c_void)
}

// ============================================================================
// Settings (stable pointer for builder pattern)
// ============================================================================

/// C2PA Settings - wraps c2pa::settings::Settings with a stable pointer for FFI
pub struct C2paSettings {
    inner: c2pa::settings::Settings,
}

impl C2paSettings {
    fn new() -> Self {
        Self {
            inner: c2pa::settings::Settings::default(),
        }
    }
}

// ============================================================================
// Settings API
// ============================================================================

/// Create a new Settings with defaults
///
/// Returns NULL on error.
///
/// # Example
/// ```c
/// C2paSettings* settings = c2pa_settings_new();
/// c2pa_settings_free(settings);
/// ```
#[no_mangle]
pub extern "C" fn c2pa_settings_new() -> *mut C2paSettings {
    let settings = C2paSettings::new();
    box_tracked!(settings)
}

/// Create Settings from JSON string
///
/// # Parameters
/// - `json`: JSON string containing settings configuration
///
/// Returns NULL on error.
#[no_mangle]
pub extern "C" fn c2pa_settings_from_json(json: *const c_char) -> *mut C2paSettings {
    let json_str = cstr_or_return_null!(json);
    let inner = ok_or_return_null!(
        serde_json::from_str(&json_str).map_err(C2paInternalError::Json)
    );
    let settings = C2paSettings { inner };
    box_tracked!(settings)
}

/// Create Settings from TOML string
///
/// # Parameters
/// - `toml`: TOML string containing settings configuration
///
/// Returns NULL on error.
#[no_mangle]
pub extern "C" fn c2pa_settings_from_toml(toml: *const c_char) -> *mut C2paSettings {
    let toml_str = cstr_or_return_null!(toml);
    let inner = ok_or_return_null!(
        toml::from_str(&toml_str).map_err(|e| C2paInternalError::Other(format!("{}", e)))
    );
    let settings = C2paSettings { inner };
    box_tracked!(settings)
}

/// Serialize Settings to JSON string
///
/// Returns NULL on error. Caller must free with c2pa_free().
///
/// # Example
/// ```c
/// C2paSettings* settings = c2pa_settings_new();
/// char* json = c2pa_settings_to_json(settings);
/// printf("%s\n", json);
/// c2pa_free(json);
/// c2pa_settings_free(settings);
/// ```
#[no_mangle]
pub extern "C" fn c2pa_settings_to_json(settings: *mut C2paSettings) -> *mut c_char {
    let settings_ref = deref_or_return_null!(settings, C2paSettings);
    let json = ok_or_return_null!(
        serde_json::to_string_pretty(&settings_ref.inner).map_err(C2paInternalError::Json)
    );
    to_c_string(json)
}

/// Serialize Settings to TOML string
///
/// Returns NULL on error. Caller must free with c2pa_free().
#[no_mangle]
pub extern "C" fn c2pa_settings_to_toml(settings: *mut C2paSettings) -> *mut c_char {
    let settings_ref = deref_or_return_null!(settings, C2paSettings);
    let toml = ok_or_return_null!(
        toml::to_string_pretty(&settings_ref.inner)
            .map_err(|e| C2paInternalError::Other(format!("{}", e)))
    );
    to_c_string(toml)
}

/// Apply Settings to a Context (builder-style, mutates Context in place)
///
/// This configures the Context with the given Settings.
/// Returns 0 on success, non-zero on error.
///
/// # Parameters
/// - `ctx`: Context to modify
/// - `settings`: Settings to apply
#[no_mangle]
pub extern "C" fn c2pa_context_with_settings_obj(
    ctx: *mut C2paContext,
    settings: *mut C2paSettings,
) -> i32 {
    let ctx_ref = deref_mut_or_return_neg!(ctx, C2paContext);
    let settings_ref = deref_or_return_neg!(settings, C2paSettings);
    
    ok_or_return!(
        c2pa::Context::new()
            .with_settings(settings_ref.inner.clone())
            .map_err(C2paInternalError::C2pa),
        |new_ctx| {
            ctx_ref.inner = new_ctx;
            0
        },
        -1
    )
}

/// Free Settings
///
/// # Safety
/// The pointer must be valid and not used after this call.
#[no_mangle]
pub extern "C" fn c2pa_settings_free(settings: *mut C2paSettings) -> i32 {
    cimpl_free(settings as *mut std::ffi::c_void)
}

// ============================================================================
// Error Handling Functions
// ============================================================================

/// Gets the error code of the last error (0 if no error)
#[no_mangle]
pub extern "C" fn c2pa_error_code() -> i32 {
    CimplError::last_code()
}

/// Gets the error message of the last error (NULL if no error)
/// Caller must free the returned string with c2pa_free()
#[no_mangle]
pub extern "C" fn c2pa_last_error() -> *mut c_char {
    option_to_c_string!(CimplError::last_message())
}

/// Free a string allocated by C2PA functions
///
/// # Safety
/// The pointer must be from a C2PA function that returns `*mut c_char`
#[no_mangle]
pub extern "C" fn c2pa_free(ptr: *mut c_char) -> i32 {
    cimpl_free(ptr as *mut std::ffi::c_void)
}
