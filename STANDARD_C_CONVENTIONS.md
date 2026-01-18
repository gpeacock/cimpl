# Standard C Error Convention - Final Implementation

## Summary

Updated cimple to follow **standard C error conventions** (like `errno`), where error details are only checked **after** a function indicates failure.

## Key Changes

### 1. Simplified Error Handling

**Before** (manual type mapping):
```c
void print_error_with_type() {
    int32_t code = mystring_error_code();
    const char* error_type = "Unknown";
    if (code == ERROR_NULL_PARAMETER) error_type = "NullParameter";
    // ... manual mapping for each code
    fprintf(stderr, "Error [%s] (code %d): %s\n", error_type, code, msg);
}
```

**After** (message already formatted):
```c
void print_error() {
    int32_t code = mystring_error_code();
    char* msg = mystring_last_error();
    // Message already includes type: "NullParameter: details"
    fprintf(stderr, "Error %d: %s\n", code, msg);
    mystring_string_free(msg);
}
```

### 2. Standard C Convention (errno pattern)

**Return values indicate success/failure:**
- Pointer functions: `NULL` on error, valid pointer on success
- Integer functions: `-1` on error, `0` on success  
- Some functions never error (documented)

**Error details are conditional:**
- Only check `mystring_error_code()` / `mystring_last_error()` **AFTER** failure
- Like `errno` - don't check unless the function failed

### 3. Error Message Format

Error messages are self-describing with format: `"ErrorType: details"`

Examples:
- `"NullParameter: new_value"`
- `"InvalidHandle: 1"`
- `"DatabaseError: connection timeout"` (custom errors)

### 4. Error Code Ranges

Documented in `src/error.rs`:
- **0**: Ok (no error)
- **1-999**: Reserved for cimple library errors
- **1000+**: Available for user library custom errors

Users can extend with custom errors:
```rust
pub enum MyLibError {
    Cimple(cimple::Error),           // codes 1-999
    DatabaseError(String),            // code 1000
    NetworkTimeout(String),           // code 1001
    InvalidCredentials(String),       // code 1002
}
```

### 5. Documentation

**C Header** (`include/cimple_example.h`) documents:
- Return value conventions
- When to check errors (conditional, like errno)
- Error code ranges
- Complete examples for C, C++, and Python

**Example C Usage:**
```c
// Constructor
MyStringHandle* handle = mystring_create("test");
if (handle == NULL) {
    // NOW check error
    fprintf(stderr, "Error %d: %s\n", 
            mystring_error_code(),
            mystring_last_error());
}

// Operation
if (mystring_set_value(handle, "new") != 0) {
    // NOW check error
    fprintf(stderr, "Failed: %s\n", mystring_last_error());
}
```

## Benefits

✅ **Standard C convention** - Familiar errno-like pattern
✅ **No manual mapping** - Error type is in the message string  
✅ **Extensible** - Users can add custom error codes 1000+
✅ **Self-documenting** - Message format includes error type
✅ **AI-friendly** - Clear, simple pattern to follow

## Test Results

All tests pass:
```
Error 1: NullParameter: new_value
Error 3: InvalidHandle: 1

=== All tests completed successfully! ===
```

## Files Updated

1. `example/example.c` - Simplified error printing (removed manual mapping)
2. `example/cbindgen.toml` - Enhanced documentation with C conventions
3. `src/error.rs` - Already had error code ranges documented

## For AI Binding Generators

When AI sees the header, it finds:

**Simple Pattern:**
```c
// Check return value first
if (mylib_operation() != 0) {
    // Then get error details
    int code = mylib_error_code();        // e.g., 1
    char* msg = mylib_error_message();    // e.g., "NullParameter: param"
    // Use code for exception types, msg for display
}
```

**Generate Exception Classes:**
```python
class MyLibError(Exception):
    def __init__(self):
        self.code = lib.mylib_error_code()
        msg_ptr = lib.mylib_error_message()
        self.message = ctypes.c_char_p(msg_ptr).value.decode()
        lib.mylib_error_free(msg_ptr)
    
    def __str__(self):
        return f"[{self.code}] {self.message}"
```

Clean, simple, and very C-idiomatic! 🎉
