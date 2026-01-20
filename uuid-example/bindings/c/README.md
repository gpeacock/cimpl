# C Example

This directory contains the reference C example for using the UUID library.

## Files

- `example.c` - Comprehensive C example demonstrating all UUID operations

## Purpose

This is a **reference implementation** showing how to use the C API directly. It demonstrates:

- UUID creation (v4, v7)
- UUID parsing
- String formatting
- Comparison operations
- Error handling
- Memory management with `cimpl_free()`

## Building and Running

From the `uuid-example` root directory:

```bash
make run-c
```

Or manually:

```bash
# Build the library
cargo build --release

# Compile the example
gcc example.c -o example \
    -Iinclude \
    target/release/libcimple_uuid.a \
    -lpthread -framework Security \
    -Wall -Wextra

# Run it
./example
```

## Status

**Maintained** - This example is part of the core `cimpl` test suite and is kept up to date with API changes.
