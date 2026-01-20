# Wrapping External Crates with cimpl

This guide shows how to create C FFI bindings for **any** Rust crate, even ones you don't own or control.

## The Process

```
External Crate (uuid, serde, etc.)
         ↓
    Your FFI Wrapper (using cimpl)
         ↓
    C Header (via cbindgen)
         ↓
    Language Bindings (Python, Node.js, Lua, etc.)
```

## Step 1: Choose Your Crate

Let's use `uuid` as an example - a popular, pure-Rust crate with no existing C bindings.

Add it to your `Cargo.toml`:

```toml
[dependencies]
cimpl = "0.1"
uuid = { version = "1.11", features = ["v4", "v7"] }

[build-dependencies]
cbindgen = "0.27"
```

## Step 2: Use External Types Directly

**Key insight:** You don't need to wrap external types! Use them directly in your FFI:

```rust
use uuid::Uuid;  // External crate's type

#[no_mangle]
pub extern "C" fn uuid_new_v4() -> *mut Uuid {
    box_tracked!(Uuid::new_v4())
}

#[no_mangle]
pub extern "C" fn uuid_to_string(uuid: *mut Uuid) -> *mut c_char {
    let obj = deref_or_return_null!(uuid, Uuid);
    to_c_string(obj.to_string())
}
```

`cbindgen` will automatically generate:

```c
typedef struct Uuid Uuid;  // Opaque forward declaration
```

Perfect! The type is opaque to C, which is exactly what we want.

## Step 3: Map External Errors to C Error Codes

External crates have their own error types. Map them to C error codes:

```rust
// 1. Declare constants for cbindgen to see
#[no_mangle]
pub static ERROR_OK: i32 = 0;

#[no_mangle]
pub static ERROR_NULL_PARAMETER: i32 = 1;

#[no_mangle]
pub static ERROR_UUID_PARSE_ERROR: i32 = 100;

// 2. Create error mapping table
define_error_codes! {
    error_type: uuid::Error,
    table_name: UUID_ERROR_TABLE,
    codes: {
        // Match patterns to error codes
        _ => ("ParseError", ERROR_UUID_PARSE_ERROR),
    }
}

// 3. Use in your FFI functions
#[no_mangle]
pub extern "C" fn uuid_parse(s: *const c_char) -> *mut Uuid {
    let s_str = cstr_or_return_null!(s);
    
    // This macro automatically uses the error table!
    let uuid = ok_or_return_null_with_table!(
        Uuid::from_str(&s_str),
        UUID_ERROR_TABLE
    );
    
    box_tracked!(uuid)
}
```

If you need more granular error matching:

```rust
define_error_codes! {
    error_type: ComplexError,
    table_name: ERROR_TABLE,
    codes: {
        ComplexError::Io(_) => ("IoError", ERROR_IO),
        ComplexError::Parse(_) => ("ParseError", ERROR_PARSE),
        ComplexError::Network { code: 404, .. } => ("NotFound", ERROR_NOT_FOUND),
        ComplexError::Network { .. } => ("NetworkError", ERROR_NETWORK),
        _ => ("OtherError", ERROR_OTHER),
    }
}
```

## Step 4: Expose Core Functionality

Think about what users actually need:

### Constructors

```rust
/// Create a new random UUID (version 4)
#[no_mangle]
pub extern "C" fn uuid_new_v4() -> *mut Uuid {
    box_tracked!(Uuid::new_v4())
}

/// Create a timestamp-based UUID (version 7)
#[no_mangle]
pub extern "C" fn uuid_new_v7() -> *mut Uuid {
    box_tracked!(Uuid::now_v7())
}

/// Parse a UUID from a string
#[no_mangle]
pub extern "C" fn uuid_parse(s: *const c_char) -> *mut Uuid {
    let s_str = cstr_or_return_null!(s);
    let uuid = ok_or_return_null_with_table!(
        Uuid::from_str(&s_str),
        UUID_ERROR_TABLE
    );
    box_tracked!(uuid)
}
```

### Accessors

```rust
/// Convert UUID to string (hyphenated format)
#[no_mangle]
pub extern "C" fn uuid_to_string(uuid: *mut Uuid) -> *mut c_char {
    let obj = deref_or_return_null!(uuid, Uuid);
    to_c_string(obj.to_string())
}

/// Get UUID as 16-byte array
#[no_mangle]
pub extern "C" fn uuid_as_bytes(uuid: *mut Uuid) -> *mut u8 {
    let obj = deref_or_return_null!(uuid, Uuid);
    to_c_bytes(obj.as_bytes().to_vec()) as *mut u8
}
```

### Predicates

```rust
/// Check if UUID is nil (all zeros)
#[no_mangle]
pub extern "C" fn uuid_is_nil(uuid: *mut Uuid) -> bool {
    deref_or_return_false!(uuid, Uuid).is_nil()
}
```

### Comparisons

```rust
/// Compare two UUIDs
/// Returns: -1 if a < b, 0 if equal, 1 if a > b
#[no_mangle]
pub extern "C" fn uuid_compare(a: *mut Uuid, b: *mut Uuid) -> i32 {
    let obj_a = deref_or_return_zero!(a, Uuid);
    let obj_b = deref_or_return_zero!(b, Uuid);
    match obj_a.cmp(obj_b) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}
```

## Step 5: Add Error Handling Functions

```rust
/// Get the error code of the last error
#[no_mangle]
pub extern "C" fn uuid_error_code() -> i32 {
    Error::last_code() as i32
}

/// Get the error message of the last error
/// Returns NULL if no error occurred
/// Caller must free the returned string with cimpl_free()
#[no_mangle]
pub extern "C" fn uuid_last_error() -> *mut c_char {
    match Error::last_message() {
        Some(msg) => to_c_string(msg),
        None => std::ptr::null_mut(),
    }
}

/// Clear the last error
#[no_mangle]
pub extern "C" fn uuid_clear_error() {
    Error::take_last();
}
```

## Step 6: Configure cbindgen

Create `cbindgen.toml`:

```toml
language = "C"
include_guard = "MY_LIBRARY_H"
autogen_warning = "/* Warning, this file is autogenerated by cbindgen. Don't modify this manually. */"
documentation = true
documentation_style = "doxy"
cpp_compat = true

after_includes = """
// Opaque type
typedef struct Uuid Uuid;

/**
 * @brief Universal free function for any pointer allocated by this library
 *
 * @param ptr Pointer to free (can be NULL)
 * @return 0 on success, -1 on error
 */
int32_t cimpl_free(void *ptr);

/* Library-specific error codes (100+) */
extern const int32_t ERROR_UUID_PARSE_ERROR;
"""

header = """
/**
 * @file my_library.h
 * @brief C FFI for the Rust uuid crate
 *
 * @section error_convention Error Convention
 *
 * This library follows standard C error conventions:
 * - Pointer-returning functions: Return NULL on error
 * - Integer-returning functions: Return -1 on error, 0 on success
 * - Boolean-returning functions: Return false on error
 *
 * Check error details ONLY after a function indicates failure:
 * @code
 * Uuid* uuid = uuid_parse("invalid");
 * if (uuid == NULL) {
 *     int code = uuid_error_code();
 *     char* msg = uuid_last_error();
 *     printf("Error %d: %s\n", code, msg);
 *     cimpl_free(msg);
 * }
 * @endcode
 *
 * @section memory Memory Management
 *
 * All pointers returned by this library must be freed with cimpl_free()
 *
 * @section thread_safety Thread Safety
 *
 * All functions are thread-safe. Error state is stored per-thread.
 */
"""
```

## Step 7: Add build.rs

```rust
use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set");
    
    let output_file = PathBuf::from(&crate_dir)
        .join("include")
        .join("my_library.h")
        .display()
        .to_string();

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");

    let config = cbindgen::Config::from_file("cbindgen.toml")
        .expect("Couldn't parse config file");

    cbindgen::generate_with_config(&crate_dir, config)
        .expect("Unable to generate bindings")
        .write_to_file(&output_file);
}
```

## Step 8: Build and Test

```bash
cargo build --release
```

Your library will be at:
- `target/release/libmy_library.dylib` (macOS)
- `target/release/libmy_library.so` (Linux)
- `target/release/my_library.dll` (Windows)

And your header at:
- `include/my_library.h`

## Complete Example

See `uuid-example/` in this repository for a complete, working example that demonstrates:

- ✅ Wrapping an external crate (`uuid`)
- ✅ Multiple constructors (v4, v7, parse, nil, max)
- ✅ Accessors (to_string, to_urn, as_bytes)
- ✅ Predicates (is_nil, is_max)
- ✅ Comparisons (compare, equals)
- ✅ Error handling with custom error codes
- ✅ Memory management with `cimpl_free()`
- ✅ Full language bindings:
  - C example program
  - Python bindings (ctypes)
  - Node.js bindings (Koffi)
  - Lua bindings (LuaJIT FFI)

The entire Rust wrapper is ~200 lines of code. The generated bindings are idiomatic in each language.

## Tips and Tricks

### Tip 1: Start Simple

Don't wrap every method. Start with core functionality:
- 2-3 constructors
- 2-3 accessors  
- 1-2 predicates
- Error handling functions

You can always add more later.

### Tip 2: Group Related Functions

Use consistent prefixes:

```rust
// ✅ Good: Consistent naming
uuid_new_v4()
uuid_new_v7()
uuid_parse()
uuid_to_string()
uuid_free()

// ❌ Bad: Inconsistent
create_uuid_v4()
newUuidV7()
parse()
ToString()
destroy_uuid()
```

### Tip 3: Document Everything

Use doc comments that `cbindgen` will include in the header:

```rust
/// Creates a new random UUID (version 4).
///
/// Uses a cryptographically secure random number generator.
///
/// # Returns
/// A pointer to a new UUID, or NULL if RNG initialization fails.
/// The returned pointer must be freed with `cimpl_free()`.
///
/// # Example
/// ```c
/// Uuid* uuid = uuid_new_v4();
/// if (uuid == NULL) {
///     fprintf(stderr, "Failed to generate UUID\n");
///     return -1;
/// }
/// char* str = uuid_to_string(uuid);
/// printf("Generated: %s\n", str);
/// cimpl_free(str);
/// cimpl_free(uuid);
/// ```
#[no_mangle]
pub extern "C" fn uuid_new_v4() -> *mut Uuid {
    box_tracked!(Uuid::new_v4())
}
```

### Tip 4: Error Code Ranges

Use a consistent numbering scheme:

```rust
// 0 = Success
pub static ERROR_OK: i32 = 0;

// 1-99 = cimpl infrastructure errors
pub static ERROR_NULL_PARAMETER: i32 = 1;
pub static ERROR_INVALID_HANDLE: i32 = 3;

// 100-199 = Parsing errors
pub static ERROR_PARSE_ERROR: i32 = 100;
pub static ERROR_INVALID_FORMAT: i32 = 101;

// 200-299 = I/O errors
pub static ERROR_FILE_NOT_FOUND: i32 = 200;
pub static ERROR_PERMISSION_DENIED: i32 = 201;

// 300-399 = Network errors
pub static ERROR_CONNECTION_FAILED: i32 = 300;
pub static ERROR_TIMEOUT: i32 = 301;
```

### Tip 5: Consider Builder Patterns

For complex initialization:

```rust
/// Create a new configuration with defaults
#[no_mangle]
pub extern "C" fn config_new() -> *mut Config {
    box_tracked!(Config::default())
}

/// Set a configuration option
#[no_mangle]
pub extern "C" fn config_set_timeout(config: *mut Config, timeout_ms: u32) -> i32 {
    let cfg = deref_mut_or_return_neg!(config, Config);
    cfg.timeout = Duration::from_millis(timeout_ms as u64);
    0
}

/// Create client with configuration
#[no_mangle]
pub extern "C" fn client_new_with_config(config: *mut Config) -> *mut Client {
    let cfg = deref_or_return_null!(config, Config);
    box_tracked!(Client::with_config(cfg.clone()))
}
```

## Common Patterns

### Iterators

Convert Rust iterators to C-compatible index-based access:

```rust
/// Get the number of items
#[no_mangle]
pub extern "C" fn collection_len(coll: *mut Collection) -> usize {
    deref_or_return_zero!(coll, Collection).len()
}

/// Get item at index (returns NULL if out of bounds)
#[no_mangle]
pub extern "C" fn collection_get(coll: *mut Collection, index: usize) -> *mut Item {
    let c = deref_or_return_null!(coll, Collection);
    match c.get(index) {
        Some(item) => box_tracked!(item.clone()),
        None => std::ptr::null_mut(),
    }
}
```

### Callbacks

For simple callbacks, use function pointers:

```rust
pub type Callback = extern "C" fn(user_data: *mut c_void, value: i32);

#[no_mangle]
pub extern "C" fn process_with_callback(
    items: *mut Items,
    callback: Callback,
    user_data: *mut c_void,
) -> i32 {
    let items = deref_or_return_neg!(items, Items);
    for value in items.iter() {
        callback(user_data, *value);
    }
    0
}
```

### Async Functions

Convert async Rust to blocking C:

```rust
#[no_mangle]
pub extern "C" fn fetch_data_blocking(url: *const c_char) -> *mut Data {
    let url_str = cstr_or_return_null!(url);
    
    // Block on async function
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let result = runtime.block_on(async {
        fetch_data(&url_str).await
    });
    
    match result {
        Ok(data) => box_tracked!(data),
        Err(e) => {
            Error::Other(e.to_string()).set_last();
            std::ptr::null_mut()
        }
    }
}
```

## Summary

Wrapping external crates with `cimpl` is straightforward:

1. Add the crate as a dependency
2. Use its types directly in your FFI functions
3. Map its errors to C error codes
4. Expose core functionality with `cimpl` macros
5. Configure `cbindgen` for nice headers
6. Build and enjoy universal bindings!

The result: Safe Rust code exposed through a clean C API that works with every language.
