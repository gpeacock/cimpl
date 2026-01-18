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

//! FFI Utilities
//!
//! Provides utilities for safe FFI bindings, including:
//! - Handle-based API: Thread-safe handle management system
//! - Allocation tracking: Prevents double-free of raw pointers
//! - Buffer safety: Validates buffer sizes and pointer arithmetic

use std::{
    any::TypeId,
    collections::HashMap,
    os::raw::c_uchar,
    sync::{Arc, Mutex},
};

use crate::error::Error;

// ============================================================================
// Pointer Registry - Tracks pointers with their cleanup functions
// ============================================================================

type CleanupFn = Box<dyn FnMut() + Send>;

/// Registry that tracks pointers allocated from Rust and passed to C.
/// Each pointer is associated with its type and a cleanup function,
/// enabling type validation and universal freeing via `cimple_free()`.
pub struct PointerRegistry {
    tracked: Mutex<HashMap<usize, (TypeId, CleanupFn)>>,
}

impl PointerRegistry {
    fn new() -> Self {
        Self {
            tracked: Mutex::new(HashMap::new()),
        }
    }

    /// Track a pointer with its type and cleanup function
    fn track(&self, ptr: usize, type_id: TypeId, cleanup: CleanupFn) {
        if ptr != 0 {
            self.tracked
                .lock()
                .unwrap()
                .insert(ptr, (type_id, cleanup));
        }
    }

    /// Validate that a pointer is tracked and has the expected type
    pub fn validate(&self, ptr: usize, expected_type: TypeId) -> Result<(), Error> {
        if ptr == 0 {
            return Err(Error::NullParameter("pointer".to_string()));
        }

        let tracked = self.tracked.lock().unwrap();
        match tracked.get(&ptr) {
            Some((actual_type, _)) if *actual_type == expected_type => Ok(()),
            Some(_) => Err(Error::WrongHandleType(ptr as u64)),
            None => Err(Error::InvalidHandle(ptr as u64)),
        }
    }

    /// Free a tracked pointer by calling its cleanup function
    pub fn free(&self, ptr: usize) -> Result<(), Error> {
        if ptr == 0 {
            return Ok(()); // NULL is always safe
        }

        let mut cleanup = {
            let mut tracked = self.tracked.lock().unwrap();
            match tracked.remove(&ptr) {
                Some((_, cleanup)) => cleanup,
                None => return Err(Error::InvalidHandle(ptr as u64)),
            }
        }; // Release lock before cleanup

        cleanup(); // Run the cleanup function
        Ok(())
    }
}

impl Drop for PointerRegistry {
    fn drop(&mut self) {
        let tracked = self.tracked.lock().unwrap();
        if !tracked.is_empty() {
            eprintln!(
                "\n⚠️  WARNING: {} pointer(s) were not freed at shutdown!",
                tracked.len()
            );
            eprintln!("This indicates C code did not properly free all allocated pointers.");
            eprintln!("Each pointer should be freed exactly once with cimple_free().\n");
        }
    }
}

/// Get the global pointer registry
pub fn get_registry() -> &'static PointerRegistry {
    use std::sync::OnceLock;
    static REGISTRY: OnceLock<PointerRegistry> = OnceLock::new();
    REGISTRY.get_or_init(PointerRegistry::new)
}

// ============================================================================
// Tracking Functions for Different Wrapper Types
// ============================================================================

/// Track a Box-wrapped pointer
///
/// Use this when you allocate with `Box::into_raw()`.
/// The pointer will be freed with `Box::from_raw()` when `cimple_free()` is called.
pub fn track_box<T: 'static>(ptr: *mut T) {
    let ptr_val = ptr as usize; // Store as usize to make it Send
    let cleanup = move || unsafe {
        drop(Box::from_raw(ptr_val as *mut T));
    };
    get_registry().track(ptr as usize, TypeId::of::<T>(), Box::new(cleanup));
}

/// Track an Arc-wrapped pointer
///
/// Use this when you allocate with `Arc::into_raw()`.
/// The pointer will be freed with `Arc::from_raw()` when `cimple_free()` is called.
pub fn track_arc<T: 'static>(ptr: *mut T) {
    let ptr_val = ptr as usize; // Store as usize to make it Send
    let cleanup = move || unsafe {
        drop(Arc::from_raw(ptr_val as *const T));
    };
    get_registry().track(ptr as usize, TypeId::of::<T>(), Box::new(cleanup));
}

/// Track an Arc<Mutex<T>>-wrapped pointer
///
/// Use this when you allocate with `Arc::into_raw(Arc::new(Mutex::new(value)))`.
/// The pointer will be freed with `Arc::from_raw()` when `cimple_free()` is called.
pub fn track_arc_mutex<T: 'static>(ptr: *mut Mutex<T>) {
    let ptr_val = ptr as usize; // Store as usize to make it Send
    let cleanup = move || unsafe {
        drop(Arc::from_raw(ptr_val as *const Mutex<T>));
    };
    get_registry().track(ptr as usize, TypeId::of::<Mutex<T>>(), Box::new(cleanup));
}

/// Validate that a pointer is tracked and has the expected type
pub fn validate_pointer<T: 'static>(ptr: *mut T) -> Result<(), Error> {
    get_registry().validate(ptr as usize, TypeId::of::<T>())
}

/// Universal free function for any tracked pointer
///
/// Frees any pointer that was allocated and tracked through cimple
/// (Box, Arc, Arc<Mutex>, etc.). The correct destructor is automatically called
/// based on how the pointer was tracked.
///
/// # Returns
/// - `Ok(())` if the pointer was successfully freed
/// - `Err(Error)` if the pointer was not tracked (invalid or double-free)
///
/// # Safety
/// - Safe to call with NULL (returns Ok)
/// - Safe to call with any pointer tracked via `track_box()`, `track_arc()`, etc.
/// - DO NOT call on untracked pointers or pointers you allocated yourself
///
/// # Example
/// ```rust,no_run
/// use cimple::{track_box, free_tracked_pointer};
/// 
/// let ptr = Box::into_raw(Box::new(42));
/// track_box(ptr);
/// // ... use ptr ...
/// free_tracked_pointer(ptr as *mut u8).unwrap();
/// ```
pub fn free_tracked_pointer(ptr: *mut u8) -> Result<(), Error> {
    get_registry().free(ptr as usize)
}

/// C-compatible wrapper for `free_tracked_pointer`
///
/// This is the universal free function exposed to C. It works for ANY pointer
/// that was allocated and tracked through cimple, regardless of the wrapper type
/// (Box, Arc, etc.) or the underlying Rust type.
///
/// # Returns
/// - 0 on success
/// - -1 if pointer was not tracked (invalid or double-free)
///
/// # Safety
/// Safe to call with NULL (returns 0).
/// Safe to call with any tracked pointer.
/// DO NOT call on untracked pointers.
///
/// # Example (C)
/// ```c
/// MyString* str = mystring_create("hello");
/// char* value = mystring_get_value(str);
/// 
/// cimple_free(value);  // Free the returned string
/// cimple_free(str);    // Free the MyString - same function!
/// ```
#[no_mangle]
pub extern "C" fn cimple_free(ptr: *mut std::ffi::c_void) -> i32 {
    match free_tracked_pointer(ptr as *mut u8) {
        Ok(()) => 0,
        Err(e) => {
            e.set_last();
            -1
        }
    }
}

// ============================================================================
// Raw Pointer Allocation Tracking (for C strings and buffers)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AllocationType {
    String,
    ByteArray,
}

struct AllocationInfo {
    allocation_type: AllocationType,
    size: usize,
}

pub struct AllocationTracker {
    allocations: Mutex<HashMap<usize, AllocationInfo>>,
}

impl AllocationTracker {
    fn new() -> Self {
        Self {
            allocations: Mutex::new(HashMap::new()),
        }
    }

    /// Track a new allocation
    fn track(&self, ptr: *const u8, size: usize, allocation_type: AllocationType) {
        if !ptr.is_null() {
            let mut allocations = self.allocations.lock().unwrap();
            allocations.insert(
                ptr as usize,
                AllocationInfo {
                    allocation_type,
                    size,
                },
            );
        }
    }

    /// Untrack an allocation, returning true if it was tracked
    fn untrack(&self, ptr: *const u8) -> bool {
        if ptr.is_null() {
            return true; // NULL is always safe to "free"
        }
        let mut allocations = self.allocations.lock().unwrap();
        allocations.remove(&(ptr as usize)).is_some()
    }
}

impl Drop for AllocationTracker {
    fn drop(&mut self) {
        let allocations = self.allocations.lock().unwrap();
        if !allocations.is_empty() {
            let mut string_count = 0;
            let mut string_bytes = 0;
            let mut array_count = 0;
            let mut array_bytes = 0;

            for info in allocations.values() {
                match info.allocation_type {
                    AllocationType::String => {
                        string_count += 1;
                        string_bytes += info.size;
                    }
                    AllocationType::ByteArray => {
                        array_count += 1;
                        array_bytes += info.size;
                    }
                }
            }

            eprintln!(
                "\n⚠️  WARNING: {} raw allocation(s) were not freed at shutdown!",
                allocations.len()
            );
            if string_count > 0 {
                eprintln!(
                    "  - {} string(s) (approx. {} bytes)",
                    string_count, string_bytes
                );
            }
            if array_count > 0 {
                eprintln!(
                    "  - {} byte array(s) (approx. {} bytes)",
                    array_count, array_bytes
                );
            }
            eprintln!("This indicates C code did not properly free all allocated memory.\n");
        }
    }
}

// Single global allocation tracker
pub fn get_allocations() -> &'static AllocationTracker {
    use std::sync::OnceLock;
    static ALLOCATIONS: OnceLock<AllocationTracker> = OnceLock::new();
    ALLOCATIONS.get_or_init(AllocationTracker::new)
}

// Public API for tracking allocations
pub fn track_string_allocation(ptr: *const i8, len: usize) {
    get_allocations().track(ptr as *const u8, len, AllocationType::String);
}

pub fn track_bytes_allocation(ptr: *const u8, len: usize) {
    get_allocations().track(ptr, len, AllocationType::ByteArray);
}

pub fn untrack_allocation(ptr: *const u8) -> bool {
    get_allocations().untrack(ptr)
}

// ============================================================================
// Buffer Safety Utilities
// ============================================================================

/// Validates that a buffer size is within safe bounds and doesn't cause integer overflow
/// when used with pointer arithmetic.
///
/// # Arguments
/// * `size` - Size to validate
/// * `ptr` - Pointer to validate against (for address space checks)
///
/// # Returns
/// * `true` if the size is safe to use
/// * `false` if the size would cause integer overflow
///
/// # Safety
/// Caller must ensure that `ptr` points to valid memory if not null.
/// This function performs pointer arithmetic with `ptr.add(size)` which requires
/// that the pointer and size are valid for the memory region being checked.
pub unsafe fn is_safe_buffer_size(size: usize, ptr: *const c_uchar) -> bool {
    // Combined checks for early return - improves branch prediction
    if size == 0 || size > isize::MAX as usize {
        return false;
    }

    // Check if the buffer would extend beyond address space to fail fast
    if !ptr.is_null() {
        let end_ptr = ptr.add(size);
        if end_ptr < ptr {
            return false; // Wrapped around
        }
    }

    true
}

/// Creates a safe slice from raw parts with bounds validation
///
/// # Arguments
/// * `ptr` - Pointer to the data
/// * `len` - Length of the data
/// * `param_name` - Name of the parameter for error reporting
///
/// # Returns
/// * `Ok(slice)` if the slice is safe to create
/// * `Err(Error)` if bounds validation fails
///
/// # Safety
/// Caller must ensure that:
/// - `ptr` points to valid, initialized memory for at least `len` bytes
/// - The memory remains valid for the lifetime of the returned slice
/// - The memory is not mutated while the slice exists
/// - `len` does not exceed the actual size of the allocated memory
pub unsafe fn safe_slice_from_raw_parts(
    ptr: *const c_uchar,
    len: usize,
    param_name: &str,
) -> Result<&[u8], Error> {
    if ptr.is_null() {
        return Err(Error::NullParameter(param_name.to_string()));
    }

    if !is_safe_buffer_size(len, ptr) {
        return Err(Error::Other(format!(
            "Buffer size {len} is invalid for parameter '{param_name}'",
        )));
    }

    Ok(std::slice::from_raw_parts(ptr, len))
}

/// Converts a Rust String to a C string (*mut c_char)
///
/// The returned pointer is tracked for allocation safety and MUST be freed
/// by calling the appropriate free function (e.g., `c2pa_string_free`).
///
/// # Arguments
/// * `s` - The Rust String to convert
///
/// # Returns
/// * `*mut c_char` - Pointer to the C string, or null on error
///
/// # Safety
/// The returned pointer must be freed exactly once by C code
pub fn to_c_string(s: String) -> *mut std::os::raw::c_char {
    use std::ffi::CString;
    let len = s.len();
    match CString::new(s) {
        Ok(c_str) => {
            let ptr = c_str.into_raw();
            track_string_allocation(ptr, len + 1); // +1 for null terminator
            ptr
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Converts a `Vec <u8>` to a tracked C byte array pointer
///
/// The returned pointer is tracked for allocation safety and MUST be freed
/// by calling `free_c_bytes`.
///
/// # Arguments
/// * `bytes` - The byte vector to convert
///
/// # Returns
/// * `*const c_uchar` - Pointer to the byte array
///
/// # Safety
/// The returned pointer must be freed exactly once by calling `free_c_bytes`
pub fn to_c_bytes(bytes: Vec<u8>) -> *const c_uchar {
    let len = bytes.len();
    let ptr = Box::into_raw(bytes.into_boxed_slice()) as *const c_uchar;
    track_bytes_allocation(ptr, len);
    ptr
}

/// Safely frees a tracked C string
///
/// Validates that the pointer was allocated and tracked by Rust before freeing.
/// NULL pointers are safely ignored. Attempts to free untracked or already-freed
/// pointers are detected and logged.
///
/// # Arguments
/// * `ptr` - Pointer to the C string to free
///
/// # Returns
/// * `true` if the string was tracked and freed successfully, or if ptr was NULL
/// * `false` if the string was not tracked (double-free or invalid pointer)
///
/// # Safety
/// This function is safe to call with NULL or invalid pointers - it will not panic
pub unsafe fn free_c_string(ptr: *mut std::os::raw::c_char) -> bool {
    use std::ffi::CString;

    if ptr.is_null() {
        return true; // NULL is always safe
    }

    if untrack_allocation(ptr as *const u8) {
        drop(CString::from_raw(ptr));
        true
    } else {
        eprintln!(
            "WARNING: Attempt to free untracked or already-freed string pointer: {:p}",
            ptr
        );
        false
    }
}

/// Safely frees a tracked C byte array
///
/// Validates that the pointer was allocated and tracked by Rust before freeing.
/// NULL pointers are safely ignored. Attempts to free untracked or already-freed
/// pointers are detected and logged.
///
/// # Arguments
/// * `ptr` - Pointer to the byte array to free
///
/// # Returns
/// * `true` if the array was tracked and freed successfully, or if ptr was NULL
/// * `false` if the array was not tracked (double-free or invalid pointer)
///
/// # Safety
/// This function is safe to call with NULL or invalid pointers - it will not panic
pub unsafe fn free_c_bytes(ptr: *const c_uchar) -> bool {
    if ptr.is_null() {
        return true; // NULL is always safe
    }

    if untrack_allocation(ptr) {
        drop(Box::from_raw(ptr as *mut c_uchar));
        true
    } else {
        eprintln!(
            "WARNING: Attempt to free untracked or already-freed byte array pointer: {:p}",
            ptr
        );
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocation_tracking_double_free_string() {
        use std::ffi::CString;

        // Test that double-freeing a string is detected
        let test_string = CString::new("test allocation tracking").unwrap();
        let c_string = to_c_string(test_string.to_str().unwrap().to_string());
        assert!(!c_string.is_null());

        // First free should succeed
        let result1 = unsafe { free_c_string(c_string) };
        assert!(result1);

        // Second free should be detected and logged (not panic)
        let result2 = unsafe { free_c_string(c_string) };
        assert!(!result2);
    }

    #[test]
    fn test_allocation_tracking_null_free() {
        // Test that freeing NULL is safe
        let result1 = unsafe { free_c_string(std::ptr::null_mut()) };
        assert!(result1);

        let result2 = unsafe { free_c_bytes(std::ptr::null()) };
        assert!(result2);
    }

    #[test]
    fn test_allocation_tracking_double_free_bytes() {
        // Test that double-freeing byte arrays is detected
        let test_bytes = vec![1u8, 2, 3, 4, 5];
        let ptr = to_c_bytes(test_bytes);

        // First free should succeed
        let result1 = unsafe { free_c_bytes(ptr) };
        assert!(result1);

        // Second free should be detected and logged (not panic)
        let result2 = unsafe { free_c_bytes(ptr) };
        assert!(!result2);
    }

    #[test]
    fn test_to_c_string_basic() {
        // Test basic string conversion
        let rust_string = "Hello, C!".to_string();
        let c_string = to_c_string(rust_string);
        assert!(!c_string.is_null());

        // Clean up
        unsafe { free_c_string(c_string) };
    }

    #[test]
    fn test_to_c_bytes_basic() {
        // Test basic byte array conversion
        let bytes = vec![1, 2, 3, 4, 5];
        let ptr = to_c_bytes(bytes);
        assert!(!ptr.is_null());

        // Clean up
        unsafe { free_c_bytes(ptr) };
    }

    #[test]
    fn test_to_c_string_with_null_byte() {
        // Test that strings with embedded nulls return null
        let bad_string = "Hello\0World".to_string();
        let c_string = to_c_string(bad_string);
        assert!(c_string.is_null());
        // No need to free since it's null
    }
}
