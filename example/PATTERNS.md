# Common Patterns Guide

This guide shows common patterns when using **cimple** to create C FFI bindings.

## Pattern 1: Simple Constructor/Destructor

```rust
use cimple::{handle_to_ptr, get_handles};

#[repr(C)]
pub struct MyObjectHandle {
    _private: [u8; 0],
}

#[no_mangle]
pub extern "C" fn myobject_create() -> *mut MyObjectHandle {
    let obj = MyObject::new();
    let handle = get_handles().insert(obj);
    handle_to_ptr::<MyObjectHandle>(handle)
}

#[no_mangle]
pub extern "C" fn myobject_free(handle: *mut MyObjectHandle) -> i32 {
    cimple::free_handle!(handle, MyObject)
}
```

## Pattern 2: Constructor with String Parameter

```rust
use cimple::cstr_or_return_null;

#[no_mangle]
pub extern "C" fn myobject_create_with_name(name: *const c_char) -> *mut MyObjectHandle {
    // Safe bounded C string reading (max 64KB)
    let name_str = cstr_or_return_null!(name);
    
    let obj = MyObject::new(name_str);
    let handle = get_handles().insert(obj);
    handle_to_ptr::<MyObjectHandle>(handle)
}
```

## Pattern 3: Method Returning String

```rust
use cimple::{guard_handle_or_null, to_c_string};

#[no_mangle]
pub extern "C" fn myobject_get_name(handle: *mut MyObjectHandle) -> *mut c_char {
    // Safely access the handle
    guard_handle_or_null!(handle, MyObject, obj);
    
    // Convert Rust String to tracked C string
    to_c_string(obj.get_name())
}

// Always provide a free function for returned strings
#[no_mangle]
pub extern "C" fn myobject_string_free(s: *mut c_char) -> i32 {
    cimple::ptr_or_return_int!(s);
    if unsafe { cimple::free_c_string(s) } {
        0
    } else {
        -1
    }
}
```

## Pattern 4: Method with Parameters

```rust
use cimple::{guard_handle_mut_or_return_neg, cstr_or_return_int};

#[no_mangle]
pub extern "C" fn myobject_set_name(
    handle: *mut MyObjectHandle,
    name: *const c_char
) -> i32 {
    // Get mutable access to the object
    guard_handle_mut_or_return_neg!(handle, MyObject, obj);
    
    // Convert the parameter
    let name_str = cstr_or_return_int!(name);
    
    // Call the method
    obj.set_name(name_str);
    
    0  // Success
}
```

## Pattern 5: Method with Result

```rust
use cimple::{guard_handle_or_null, ok_or_return};

#[no_mangle]
pub extern "C" fn myobject_process(handle: *mut MyObjectHandle) -> *mut c_char {
    guard_handle_or_null!(handle, MyObject, obj);
    
    // Handle Result with custom error conversion
    let result = ok_or_return!(
        obj.process(),
        |err| cimple::Error::Other(err.to_string()).set_last(),
        |value| value,
        std::ptr::null_mut()
    );
    
    to_c_string(result)
}
```

## Pattern 6: Method Returning Integer/Boolean

```rust
use cimple::guard_handle_or_default;

#[no_mangle]
pub extern "C" fn myobject_get_count(handle: *mut MyObjectHandle) -> usize {
    // Returns 0 on error (the default for usize)
    guard_handle_or_default!(handle, MyObject, obj, 0);
    obj.get_count()
}

#[no_mangle]
pub extern "C" fn myobject_is_valid(handle: *mut MyObjectHandle) -> bool {
    // Returns false on error
    guard_handle_or_default!(handle, MyObject, obj, false);
    obj.is_valid()
}
```

## Pattern 7: Returning Byte Arrays

```rust
use cimple::{guard_handle_or_null, to_c_bytes};

#[no_mangle]
pub extern "C" fn myobject_get_data(
    handle: *mut MyObjectHandle,
    out_len: *mut usize
) -> *const u8 {
    guard_handle_or_null!(handle, MyObject, obj);
    
    let data = obj.get_data();
    
    // Set the output length
    if !out_len.is_null() {
        unsafe { *out_len = data.len() };
    }
    
    // Convert to tracked C byte array
    to_c_bytes(data)
}

#[no_mangle]
pub extern "C" fn myobject_bytes_free(ptr: *const u8) -> i32 {
    if ptr.is_null() {
        return 0;
    }
    if unsafe { cimple::free_c_bytes(ptr) } {
        0
    } else {
        -1
    }
}
```

## Pattern 8: Taking Byte Arrays as Input

```rust
use cimple::{guard_handle_mut_or_return_neg, safe_slice_from_raw_parts};

#[no_mangle]
pub extern "C" fn myobject_set_data(
    handle: *mut MyObjectHandle,
    data: *const u8,
    len: usize
) -> i32 {
    guard_handle_mut_or_return_neg!(handle, MyObject, obj);
    
    // Safely create a slice with bounds checking
    let slice = match unsafe { safe_slice_from_raw_parts(data, len, "data") } {
        Ok(s) => s,
        Err(e) => {
            e.set_last();
            return -1;
        }
    };
    
    obj.set_data(slice.to_vec());
    0
}
```

## Pattern 9: Optional String Parameters

```rust
use cimple::cstr_option;

#[no_mangle]
pub extern "C" fn myobject_create_with_optional_name(
    name: *const c_char  // Can be NULL
) -> *mut MyObjectHandle {
    // Convert to Option<String>
    let name_opt = cstr_option!(name);
    
    let obj = MyObject::new_with_optional_name(name_opt);
    let handle = get_handles().insert(obj);
    handle_to_ptr::<MyObjectHandle>(handle)
}
```

## Pattern 10: Error Handling

```rust
use cimple::Error;

#[no_mangle]
pub extern "C" fn myobject_last_error() -> *mut c_char {
    match Error::last_message() {
        Some(msg) => to_c_string(msg),
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn myobject_clear_error() {
    Error::take_last();
}
```

## Pattern 10: Error Handling (Standard C Convention)

Cimple follows standard C error conventions similar to `errno`. Functions indicate
failure through return values, and error details are retrieved conditionally.

```rust
use cimple::Error;

/// Gets the last error code.
///
/// Returns an integer error code. Only call after a function indicates failure.
///
/// # Returns
/// - 0 (ERROR_OK) if no error
/// - Error code 1-999 for cimple errors
/// - Error code 1000+ for custom library errors
#[no_mangle]
pub extern "C" fn myobject_error_code() -> i32 {
    Error::last_code() as i32
}

/// Gets the last error message.
///
/// Returns a string including the error type, e.g., "NullParameter: value"
///
/// # Returns
/// - Pointer to error message string, or NULL if no error
///
/// # Memory Management
/// The returned string must be freed with myobject_error_free()
#[no_mangle]
pub extern "C" fn myobject_last_error() -> *mut c_char {
    match Error::last_message() {
        Some(msg) => to_c_string(msg),
        None => std::ptr::null_mut(),
    }
}

/// Clears the last error.
#[no_mangle]
pub extern "C" fn myobject_clear_error() {
    Error::take_last();
}
```

### C Usage Pattern

```c
// Check return value FIRST, then get error details
MyObjectHandle* handle = myobject_create("test");
if (handle == NULL) {
    // NOW check error details
    int code = myobject_error_code();
    char* msg = myobject_last_error();
    fprintf(stderr, "Error %d: %s\n", code, msg);
    myobject_error_free(msg);
}

// Same pattern for operations
if (myobject_process(handle) != 0) {
    // NOW check error details
    fprintf(stderr, "Error: %s\n", myobject_last_error());
}
```

## Pattern 11: Exception Wrapper (AI-Generated Bindings)

### C++ Exception Class

Language bindings should create an exception class that wraps the error code and message:

```cpp
#include <exception>
#include <string>
#include "mylib.h"

class MyLibException : public std::exception {
private:
    int code_;
    std::string message_;
    
public:
    MyLibException() {
        code_ = mylib_error_code();
        char* msg = mylib_error_message();
        if (msg != nullptr) {
            message_ = std::string(msg);
            mylib_error_free(msg);
        }
    }
    
    MyLibException(const std::string& msg) : code_(5), message_(msg) {}
    
    const char* what() const noexcept override {
        return message_.c_str();
    }
    
    int code() const { return code_; }
    
    bool is_null_parameter() const { return code_ == 1; }
    bool is_invalid_handle() const { return code_ == 3; }
};

// Helper macro for checking results
#define CHECK_MYLIB(expr) \
    do { \
        auto result = (expr); \
        if (!result) { \
            throw MyLibException(); \
        } \
        return result; \
    } while(0)

// Usage:
MyObjectHandle* handle = myobject_create();
if (handle == nullptr) {
    throw MyLibException();
}
```

### Python Error Class

```python
import ctypes

class MyLibError(Exception):
    """Exception raised for errors in MyLib."""
    
    # Error code constants
    ERROR_OK = 0
    ERROR_NULL_PARAMETER = 1
    ERROR_STRING_TOO_LONG = 2
    ERROR_INVALID_HANDLE = 3
    ERROR_WRONG_HANDLE_TYPE = 4
    ERROR_OTHER = 5
    
    def __init__(self):
        self.code = lib.mylib_error_code()
        msg_ptr = lib.mylib_error_message()
        if msg_ptr:
            self.message = ctypes.c_char_p(msg_ptr).value.decode()
            lib.mylib_error_free(msg_ptr)
        else:
            self.message = "Unknown error"
    
    def __str__(self):
        error_names = {
            self.ERROR_NULL_PARAMETER: "NullParameter",
            self.ERROR_STRING_TOO_LONG: "StringTooLong",
            self.ERROR_INVALID_HANDLE: "InvalidHandle",
            self.ERROR_WRONG_HANDLE_TYPE: "WrongHandleType",
            self.ERROR_OTHER: "Other",
        }
        error_name = error_names.get(self.code, "Unknown")
        return f"MyLib.{error_name} (code {self.code}): {self.message}"
    
    @property
    def is_null_parameter(self):
        return self.code == self.ERROR_NULL_PARAMETER
    
    @property
    def is_invalid_handle(self):
        return self.code == self.ERROR_INVALID_HANDLE

# Helper function for checking results
def check_result(result, allow_null=False):
    if not allow_null and not result:
        raise MyLibError()
    return result

# Usage:
handle = lib.myobject_create()
if not handle:
    raise MyLibError()

# Or with helper:
handle = check_result(lib.myobject_create())
```

### Go Error Type

```go
package mylib

/*
#include "mylib.h"
#include <stdlib.h>
*/
import "C"
import (
    "fmt"
    "unsafe"
)

// Error codes
const (
    ErrorOk = 0
    ErrorNullParameter = 1
    ErrorStringTooLong = 2
    ErrorInvalidHandle = 3
    ErrorWrongHandleType = 4
    ErrorOther = 5
)

type MyLibError struct {
    Code    int
    Message string
}

func (e *MyLibError) Error() string {
    errorNames := map[int]string{
        ErrorNullParameter: "NullParameter",
        ErrorStringTooLong: "StringTooLong",
        ErrorInvalidHandle: "InvalidHandle",
        ErrorWrongHandleType: "WrongHandleType",
        ErrorOther: "Other",
    }
    
    errorName := errorNames[e.Code]
    if errorName == "" {
        errorName = "Unknown"
    }
    
    return fmt.Sprintf("MyLib.%s (code %d): %s", errorName, e.Code, e.Message)
}

func getLastError() *MyLibError {
    code := int(C.mylib_error_code())
    if code == ErrorOk {
        return nil
    }
    
    msgPtr := C.mylib_error_message()
    if msgPtr == nil {
        return &MyLibError{Code: code, Message: "Unknown error"}
    }
    
    message := C.GoString(msgPtr)
    C.mylib_error_free(msgPtr)
    
    return &MyLibError{Code: code, Message: message}
}

// Usage:
handle := C.myobject_create()
if handle == nil {
    return getLastError()
}
```

## Pattern 12: String with Custom Length Limit

```rust
use cimple::cstr_or_return_with_limit;

#[no_mangle]
pub extern "C" fn myobject_set_short_name(
    handle: *mut MyObjectHandle,
    name: *const c_char
) -> i32 {
    guard_handle_mut_or_return_neg!(handle, MyObject, obj);
    
    // Only allow strings up to 256 bytes
    let name_str = cstr_or_return_with_limit!(name, 256, -1);
    
    obj.set_name(name_str);
    0
}
```

## Pattern 13: Iterator Pattern

```rust
#[repr(C)]
pub struct MyIteratorHandle {
    _private: [u8; 0],
}

#[no_mangle]
pub extern "C" fn myobject_iterator(handle: *mut MyObjectHandle) -> *mut MyIteratorHandle {
    guard_handle_or_null!(handle, MyObject, obj);
    
    let iter = obj.iter();
    let handle = get_handles().insert(iter);
    handle_to_ptr::<MyIteratorHandle>(handle)
}

#[no_mangle]
pub extern "C" fn iterator_next(handle: *mut MyIteratorHandle) -> *mut c_char {
    guard_handle_mut_or_return_null!(handle, MyObjectIterator, iter);
    
    match iter.next() {
        Some(value) => to_c_string(value),
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn iterator_free(handle: *mut MyIteratorHandle) -> i32 {
    cimple::free_handle!(handle, MyObjectIterator)
}
```

## Best Practices

### 1. Always Use Opaque Handles

```rust
// ✅ Good - Opaque handle
#[repr(C)]
pub struct MyObjectHandle {
    _private: [u8; 0],
}

// ❌ Bad - Exposes Rust internals
pub type MyObjectHandle = *mut MyObject;
```

### 2. Consistent Error Returns

```rust
// ✅ Good - Consistent pattern
// - Constructors return NULL on error
// - Destructors return 0 on success, -1 on error
// - Methods return -1 on error for int, NULL for pointers

#[no_mangle]
pub extern "C" fn create() -> *mut Handle { ... }  // NULL on error

#[no_mangle]
pub extern "C" fn free(h: *mut Handle) -> i32 { ... }  // 0 ok, -1 error

#[no_mangle]
pub extern "C" fn process(h: *mut Handle) -> i32 { ... }  // -1 on error
```

### 3. Always Track Allocations

```rust
// ✅ Good - Using cimple's tracked allocations
to_c_string(s)  // Automatically tracked

// ❌ Bad - Manual CString without tracking
CString::new(s).unwrap().into_raw()  // Not tracked!
```

### 4. Document Ownership

```rust
/// Creates a new object.
/// 
/// # Memory Management
/// The returned handle must be freed with `myobject_free()`.
#[no_mangle]
pub extern "C" fn myobject_create() -> *mut MyObjectHandle { ... }

/// Gets the name as a new string.
/// 
/// # Memory Management
/// The returned string must be freed with `myobject_string_free()`.
#[no_mangle]
pub extern "C" fn myobject_get_name(h: *mut MyObjectHandle) -> *mut c_char { ... }
```

### 5. Provide Last Error Function

```rust
// Every library should have these
#[no_mangle]
pub extern "C" fn mylib_last_error() -> *mut c_char { ... }

#[no_mangle]
pub extern "C" fn mylib_clear_error() { ... }
```

## cbindgen Configuration

Add to your `cbindgen.toml`:

```toml
language = "C"
include_guard = "MY_LIB_H"
documentation = true
documentation_style = "doxy"
cpp_compat = true

[export]
# Only export what's marked with #[no_mangle]
include = ["MyObjectHandle", "MyIteratorHandle"]
```

## Common Mistakes to Avoid

### ❌ Mistake 1: Not checking NULL

```rust
// Bad
pub extern "C" fn process(input: *const c_char) {
    let s = unsafe { CStr::from_ptr(input) };  // Crashes on NULL!
    ...
}

// Good
pub extern "C" fn process(input: *const c_char) -> i32 {
    let s = cstr_or_return_int!(input);  // Safe!
    ...
}
```

### ❌ Mistake 2: Returning untracked allocations

```rust
// Bad
pub extern "C" fn get_string() -> *mut c_char {
    CString::new("hello").unwrap().into_raw()  // Not tracked!
}

// Good
pub extern "C" fn get_string() -> *mut c_char {
    to_c_string("hello".to_string())  // Tracked!
}
```

### ❌ Mistake 3: Not handling poisoned mutexes

```rust
// Bad
let guard = arc.lock().unwrap();  // Panics if poisoned!

// Good - cimple macros handle this
guard_handle_or_null!(handle, MyType, obj);  // Recovers from poison
```

## Summary

The key patterns are:
1. **Opaque handles** for all Rust objects
2. **Consistent error handling** (-1 or NULL)
3. **Tracked allocations** for all returned pointers
4. **Clear ownership** documented in comments
5. **Bounded reads** for all input strings
6. **Safety macros** for all FFI boundaries

Follow these patterns and your C API will be safe, ergonomic, and AI-friendly!
