//! # [Library Name] - C FFI Bindings
//!
//! Brief description of what this FFI wraps
//!
//! ## ⚠️  MACRO-FIRST DEVELOPMENT ⚠️
//!
//! **See `cimpl/src/macros.rs` for the complete anti-pattern checklist.**
//!
//! Before writing code, verify you're not using manual patterns:
//! - `if ptr.is_null()` → Use `deref_or_return!` macro
//! - `match result { Ok/Err }` → Use `ok_or_return!` macro  
//! - `unsafe { &*ptr }` → Use `deref_or_return!` macro
//!
//! **When in doubt, check the macro documentation first!**

use std::os::raw::c_char;

use cimpl::{
    box_tracked, cimpl_free, cstr_or_return, deref_mut_or_return_neg,
    ok_or_return, ok_or_return_null, option_to_c_string, to_c_string, Error,
};

// ============================================================================
// Error Handling
// ============================================================================

/// Error codes for [Library] operations (100+)
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MyLibError {
    SomeError = 100,
    AnotherError = 101,
}

/// Internal error wrapper (bridges external errors to cimpl::Error)
#[derive(Debug)]
enum MyLibInternalError {
    External(external_crate::Error),
}

impl From<external_crate::Error> for MyLibInternalError {
    fn from(e: external_crate::Error) -> Self {
        MyLibInternalError::External(e)
    }
}

impl From<MyLibInternalError> for Error {
    fn from(e: MyLibInternalError) -> Self {
        match e {
            MyLibInternalError::External(e) => {
                Error::new(MyLibError::SomeError as i32, format!("SomeError: {}", e))
            }
        }
    }
}

// ============================================================================
// Main Struct Wrapper (for builder pattern support)
// ============================================================================

/// Wrapper for stable pointer
pub struct MyStruct {
    inner: external_crate::RealStruct,
}

impl MyStruct {
    fn new() -> Self {
        Self {
            inner: external_crate::RealStruct::new(),
        }
    }
}

// ============================================================================
// FFI Functions - Pattern Examples
// ============================================================================

/// PATTERN: Constructor returning new object
#[no_mangle]
pub extern "C" fn mystruct_new() -> *mut MyStruct {
    let obj = MyStruct::new();
    box_tracked!(obj)
}

/// PATTERN: Method with string parameter
#[no_mangle]
pub extern "C" fn mystruct_set_name(
    obj: *mut MyStruct,
    name: *const c_char,
) -> i32 {
    let name = cstr_or_return!(name, -1);
    let obj_ref = deref_mut_or_return_neg!(obj, MyStruct);
    
    ok_or_return!(
        obj_ref.inner.set_name(name).map_err(MyLibInternalError::from),
        |_| 0,
        -1
    )
}

/// PATTERN: Method returning string
#[no_mangle]
pub extern "C" fn mystruct_get_name(obj: *mut MyStruct) -> *mut c_char {
    let obj_ref = deref_mut_or_return_neg!(obj, MyStruct);
    to_c_string(obj_ref.inner.get_name())
}

/// PATTERN: Method returning optional string
#[no_mangle]
pub extern "C" fn mystruct_get_description(obj: *mut MyStruct) -> *mut c_char {
    let obj_ref = deref_mut_or_return_neg!(obj, MyStruct);
    option_to_c_string!(obj_ref.inner.get_description())
}

/// PATTERN: Method with Result (external error)
#[no_mangle]
pub extern "C" fn mystruct_process(obj: *mut MyStruct) -> i32 {
    let obj_ref = deref_mut_or_return_neg!(obj, MyStruct);
    
    ok_or_return!(
        obj_ref.inner.process().map_err(MyLibInternalError::from),
        |_| 0,
        -1
    )
}

/// PATTERN: Free/destructor
#[no_mangle]
pub extern "C" fn mystruct_free(obj: *mut MyStruct) -> i32 {
    cimpl_free(obj as *mut std::ffi::c_void)
}

// ============================================================================
// Error Handling Functions (standard pattern)
// ============================================================================

#[no_mangle]
pub extern "C" fn mylib_error_code() -> i32 {
    Error::last_code()
}

#[no_mangle]
pub extern "C" fn mylib_last_error() -> *mut c_char {
    option_to_c_string!(Error::last_message())
}

#[no_mangle]
pub extern "C" fn mylib_free(ptr: *mut c_char) -> i32 {
    cimpl_free(ptr as *mut std::ffi::c_void)
}
