# Cimpl Example Library

A complete example demonstrating how to use the `cimpl` utilities to create safe, ergonomic C FFI bindings from Rust.

## What This Example Shows

This example library demonstrates:

1. **Handle-based API**: Thread-safe opaque handles instead of raw pointers
2. **Safe string conversion**: Bounded C string reading with automatic null checks
3. **Allocation tracking**: Prevents double-frees and detects memory leaks
4. **Error handling**: Thread-local last error mechanism
5. **Automatic header generation**: Using `cbindgen` to generate C headers from Rust
6. **Documentation**: Doxygen-style comments in the generated header

## The Three-Stage Pipeline

```
Rust (Safe) → C API (Simple) → Target Language (AI-generated)
  [cimpl]      [cbindgen]         [AI tooling]
```

### Stage 1: Write Safe Rust FFI

The Rust code uses `cimpl` macros for safety:

```rust
#[no_mangle]
pub extern "C" fn mystring_create(initial: *const c_char) -> *mut MyStringHandle {
    let initial_str = cstr_or_return_null!(initial);  // Safe C string conversion
    let my_string = MyString::new(initial_str);
    let handle = cimpl::get_handles().insert(my_string);  // Thread-safe handle
    cimpl::handle_to_ptr::<MyStringHandle>(handle)
}
```

### Stage 2: Generate C Header

Running `cargo build` automatically generates `include/cimpl_example.h` with full documentation:

```c
/**
 * Creates a new MyString object with the given initial value.
 * 
 * The returned handle must be freed with mystring_free() when no longer needed.
 */
MyStringHandle* mystring_create(const char* initial);
```

### Stage 3: Use from Any Language

The simple C API can be easily wrapped by AI for Python, Node.js, Go, etc.

## Building

### Prerequisites

- Rust toolchain (install from https://rustup.rs)
- C compiler (for testing the C example)

### Build the Library

```bash
# From the example directory
cargo build --release
```

This generates:
- **Static library**: `target/release/libcimpl_example.a`
- **Dynamic library**: `target/release/libcimpl_example.so` (Linux), `.dylib` (macOS), or `.dll` (Windows)
- **C header**: `include/cimpl_example.h`

### Run the Tests

```bash
cargo test
```

## Using the Library from C

### Example C Program

Create `example.c`:

```c
#include <stdio.h>
#include <stdlib.h>
#include "include/cimpl_example.h"

int main() {
    // Create a new string object
    MyStringHandle* handle = mystring_create("Hello, World!");
    if (handle == NULL) {
        char* error = mystring_last_error();
        fprintf(stderr, "Error: %s\n", error);
        mystring_string_free(error);
        return 1;
    }

    // Get the current value
    char* value = mystring_get_value(handle);
    printf("Original: %s\n", value);
    printf("Length: %zu bytes\n", mystring_len(handle));
    mystring_string_free(value);

    // Convert to uppercase
    char* upper = mystring_to_uppercase(handle);
    printf("Uppercase: %s\n", upper);
    mystring_string_free(upper);

    // Append to the string
    mystring_append(handle, " How are you?");
    value = mystring_get_value(handle);
    printf("After append: %s\n", value);
    mystring_string_free(value);

    // Set a new value
    mystring_set_value(handle, "Goodbye!");
    value = mystring_get_value(handle);
    printf("New value: %s\n", value);
    mystring_string_free(value);

    // Clean up
    mystring_free(handle);

    return 0;
}
```

### Compile and Run (Linux/macOS)

```bash
# Using dynamic library
gcc example.c -o example \
    -I./example/include \
    -L./target/release \
    -lcimpl_example

# Set library path and run
export LD_LIBRARY_PATH=./target/release:$LD_LIBRARY_PATH  # Linux
export DYLD_LIBRARY_PATH=./target/release:$DYLD_LIBRARY_PATH  # macOS
./example

# Using static library
gcc example.c -o example \
    -I./example/include \
    ./target/release/libcimpl_example.a \
    -lpthread -ldl -lm
./example
```

Expected output:
```
Original: Hello, World!
Length: 13 bytes
Uppercase: HELLO, WORLD!
After append: Hello, World! How are you?
New value: Goodbye!
```

## Memory Safety Features

### Double-Free Protection

```c
char* value = mystring_get_value(handle);
mystring_string_free(value);  // OK
mystring_string_free(value);  // Returns -1, prints warning, doesn't crash
```

### Leak Detection

If you forget to free handles or strings, you'll see warnings when the program exits:

```
⚠️  WARNING: 2 raw allocation(s) were not freed at shutdown!
  - 1 string(s) (approx. 14 bytes)
This indicates C code did not properly free all allocated memory.
```

### Invalid Handle Detection

```c
MyStringHandle* fake = (MyStringHandle*)0x12345;
int result = mystring_free(fake);  // Returns -1, sets error
char* error = mystring_last_error();
// error will contain: "InvalidHandle: 74565"
```

## API Design Philosophy

This example follows these principles:

1. **Opaque handles**: Never expose Rust types directly to C
2. **Clear ownership**: Caller knows who owns what
3. **Consistent error handling**: All functions return error codes or NULL
4. **Memory safety**: Track all allocations, prevent double-frees
5. **Thread safety**: Handles are protected by mutexes
6. **AI-friendly**: Simple patterns that AI can easily learn and replicate

## Project Structure

```
example/
├── Cargo.toml           # Package configuration with cbindgen
├── build.rs             # Build script that runs cbindgen
├── cbindgen.toml        # cbindgen configuration
├── src/
│   └── lib.rs           # Example library implementation
├── include/             # Generated C headers (created by build.rs)
│   └── cimpl_example.h
└── README.md            # This file
```

## Extending This Example

To create your own library:

1. **Copy this example as a template**
2. **Replace `MyString` with your own Rust types**
3. **Use cimpl macros for safety**:
   - `cstr_or_return_null!` for string parameters
   - `ptr_or_return_int!` for pointer checks
   - `guard_handle_or_null!` for handle access
   - `return_handle!` for returning handles
4. **Run `cargo build`** to generate headers
5. **Give the header to AI** to generate language bindings

## Next Steps

### For Python Bindings (via AI)

```bash
# Give this prompt to an AI with the header file:
"Create Python bindings using ctypes for this C library: [paste cimpl_example.h]"
```

### For Node.js Bindings (via AI)

```bash
# Give this prompt:
"Create Node.js N-API bindings for this C library: [paste cimpl_example.h]"
```

### For Go Bindings (via AI)

```bash
# Give this prompt:
"Create Go cgo bindings for this C library: [paste cimpl_example.h]"
```

The AI will understand the simple C API and generate idiomatic bindings for the target language!

## License

MIT OR Apache-2.0
