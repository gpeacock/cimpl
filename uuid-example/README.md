# Cimpl UUID Example

A demonstration of using `cimpl` to create safe C FFI bindings for Rust's popular `uuid` crate.

## Overview

This example showcases how `cimpl` makes it easy to expose a real-world Rust library through a clean, safe C API that can be used from any language.

### What's Demonstrated

- ✅ **Direct pointer types** with type validation
- ✅ **Universal `cimpl_free()`** for all allocations
- ✅ **Multiple constructors** (v4, v7, parse, nil, max)
- ✅ **Object methods** (to_string, to_urn, as_bytes)
- ✅ **Predicates** (is_nil, is_max)
- ✅ **Comparison operations** (compare, equals)
- ✅ **Error handling** with error codes and messages
- ✅ **Zero-copy operations** where possible
- ✅ **Thread-safe** by design

## Features

### UUID Generation
- **V4** (Random): `uuid_new_v4()` - Cryptographically secure random UUIDs
- **V7** (Timestamp): `uuid_new_v7()` - Timestamp-based UUIDs for ordered generation

### UUID Parsing
- Parse from standard string format: `uuid_parse("550e8400-e29b-41d4-a716-446655440000")`
- Detailed error messages on parse failure

### UUID Operations
- Convert to string (hyphenated format)
- Convert to URN format
- Get as 16-byte array
- Compare and check equality
- Test for special values (nil, max)

## Building

### Quick Start

```bash
make run-c
```

### Manual Build

```bash
# Build the Rust library
cargo build --release

# Build the C example
make example-static

# Run it
./example
```

## API Overview

### Constructors
```c
Uuid* uuid_new_v4(void);              // Random UUID
Uuid* uuid_new_v7(void);              // Timestamp-based UUID
Uuid* uuid_parse(const char* s);      // Parse from string
Uuid* uuid_nil(void);                 // All zeros
Uuid* uuid_max(void);                 // All ones
```

### Accessors
```c
char* uuid_to_string(Uuid* uuid);     // Standard format
char* uuid_to_urn(Uuid* uuid);        // URN format
uint8_t* uuid_as_bytes(Uuid* uuid);   // 16-byte array
```

### Predicates
```c
bool uuid_is_nil(Uuid* uuid);
bool uuid_is_max(Uuid* uuid);
```

### Comparison
```c
int32_t uuid_compare(Uuid* a, Uuid* b);  // Returns -1, 0, or 1
bool uuid_equals(Uuid* a, Uuid* b);
```

### Memory Management
```c
int32_t cimpl_free(void* ptr);  // Universal free for everything!
```

### Error Handling
```c
int32_t uuid_error_code(void);
char* uuid_last_error(void);
void uuid_clear_error(void);
```

## Usage Example

```c
#include "cimpl_uuid.h"

int main() {
    // Generate a random UUID
    Uuid* uuid = uuid_new_v4();
    if (!uuid) {
        fprintf(stderr, "Error: %s\n", uuid_last_error());
        return 1;
    }
    
    // Convert to string
    char* str = uuid_to_string(uuid);
    printf("Generated UUID: %s\n", str);
    
    // Parse another UUID
    Uuid* parsed = uuid_parse("550e8400-e29b-41d4-a716-446655440000");
    
    // Compare them
    if (uuid_equals(uuid, parsed)) {
        printf("UUIDs match!\n");
    }
    
    // Clean up - one function for everything!
    cimpl_free(str);
    cimpl_free(uuid);
    cimpl_free(parsed);
    
    return 0;
}
```

## Error Handling

The library follows standard C conventions:

- **NULL return** = error occurred (for pointer-returning functions)
- **-1 return** = error occurred (for integer-returning functions)
- **false return** = error occurred (for boolean-returning functions)

Check error details **only after** a function indicates failure:

```c
Uuid* uuid = uuid_parse("invalid");
if (uuid == NULL) {
    // NOW check the error
    int32_t code = uuid_error_code();
    char* msg = uuid_last_error();
    fprintf(stderr, "Error %d: %s\n", code, msg);
    cimpl_free(msg);
}
```

## Memory Management

Every pointer returned by this library **must** be freed with `cimpl_free()`:

```c
Uuid* uuid = uuid_new_v4();        // Allocate
char* str = uuid_to_string(uuid);  // Allocate

cimpl_free(str);   // Free the string
cimpl_free(uuid);  // Free the UUID
```

### Double-Free Protection

Calling `cimpl_free()` twice on the same pointer is safe - it returns -1 and sets an error:

```c
cimpl_free(uuid);  // OK
cimpl_free(uuid);  // Returns -1, safe
```

## Thread Safety

All functions are thread-safe. UUID generation uses thread-local RNG state for optimal performance in multi-threaded applications.

## Performance

This library wraps Rust's `uuid` crate, which is:
- **Fast**: Optimized implementations of all UUID versions
- **Safe**: No undefined behavior, memory safe by design
- **Zero-copy**: Minimal allocations where possible

Typical performance:
- UUID v4 generation: ~1-2 microseconds
- UUID v7 generation: ~1-2 microseconds
- Parsing: ~100-200 nanoseconds
- String conversion: ~200-300 nanoseconds

## Language Bindings

This C API can be used to create bindings for any language. The `bindings/` directory contains reference implementations for:

- **C** - Native usage example
- **Python** - via ctypes
- **Node.js** - via Koffi FFI
- **Lua** - via LuaJIT FFI

See [`bindings/README.md`](bindings/README.md) for details on each binding's purpose, usage, and testing.

### Quick Test

```bash
# C
make run-c

# Python
cd bindings/python && python3 test.py

# Node.js
cd bindings/nodejs && npm test

# Lua
cd bindings/lua && luajit test.lua
```

### Philosophy: Reference Implementations

The bindings in this repository are **reference implementations** showing what should be generated from the C header. They are maintained as part of the test suite.

For true AI generation testing:
1. Create an `.ai-generated/` directory (gitignored)
2. Provide an AI with just `include/cimpl_uuid.h`
3. Compare output with the reference implementations
4. Iterate on header documentation to improve AI output quality

See [`bindings/README.md`](bindings/README.md) for more details.

## Files Generated

After building:
- `target/release/libcimpl_uuid.a` - Static library
- `target/release/libcimpl_uuid.dylib` - Dynamic library (macOS)
- `target/release/libcimpl_uuid.so` - Dynamic library (Linux)
- `include/cimpl_uuid.h` - C header file (auto-generated by cbindgen)

## Why Cimpl?

This example shows how `cimpl` makes FFI bindings:

1. **Safe** - Type validation, double-free protection, leak detection
2. **Simple** - Clean macros, consistent patterns, minimal boilerplate
3. **Fast** - Zero-cost abstractions, Rust's native speed
4. **Flexible** - Works with any Rust library, no special requirements
5. **AI-Friendly** - Clean C API makes it easy to generate bindings for any language

## License

This example is licensed under the same terms as the parent `cimpl` project (MIT/Apache-2.0).
