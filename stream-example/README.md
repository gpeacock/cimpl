# CimplStream - Callback-based I/O Streams

A demonstration of how to implement callback-based streams using `cimpl`. This example shows how to bridge C callback-based I/O to Rust's standard `Read`, `Write`, and `Seek` traits.

## What This Example Demonstrates

1. **Callback Pattern**: How to use C function pointers to implement I/O operations
2. **Trait Bridging**: How callbacks are converted to Rust's standard I/O traits
3. **Safe Pointer Handling**: Using `cimpl` macros for validation and error handling
4. **Memory Management**: Proper allocation and cleanup with `cimpl_free()`

## Key Concepts

### Stream Context

The `CimplStreamContext` is an opaque pointer that holds the caller's native stream object (e.g., a `FILE*` in C, a file handle in Python). This context is passed to each callback.

### Callbacks

Four callbacks implement the I/O operations:

- **Read**: `CimplReadCallback` - Read data from the stream
- **Seek**: `CimplSeekCallback` - Change position in the stream
- **Write**: `CimplWriteCallback` - Write data to the stream  
- **Flush**: `CimplFlushCallback` - Ensure buffered data is written

### Rust Trait Implementation

The `CimplStream` struct implements Rust's standard traits:

```rust
impl Read for CimplStream { /* calls read callback */ }
impl Seek for CimplStream { /* calls seek callback */ }
impl Write for CimplStream { /* calls write callback */ }
```

This means any Rust code expecting a `Read + Seek + Write` can use a `CimplStream`, which internally uses C callbacks!

## Building

```bash
# Build the library and generate header
make lib

# Build the C example
make example

# Build and run
make run
```

## C Usage Example

```c
#include "include/cimpl_stream.h"

// 1. Define callbacks for your native stream
isize my_read(CimplStreamContext* ctx, uint8_t* data, size_t len) {
    FILE* file = ((MyContext*)ctx)->file;
    return fread(data, 1, len, file);
}

// ... define seek, write, flush callbacks ...

// 2. Create the stream
MyContext* ctx = /* your context */;
CimplStream* stream = cimpl_stream_new(
    (CimplStreamContext*)ctx,
    my_read,
    my_seek,
    my_write,
    my_flush
);

// 3. Use the stream
uint8_t buffer[1024];
isize bytes = cimpl_stream_read(stream, buffer, sizeof(buffer));

// 4. Clean up
cimpl_free(stream);
```

## Architecture

```
┌─────────────────┐
│  C Application  │
└────────┬────────┘
         │ Creates callbacks
         ↓
┌─────────────────┐
│  CimplStream    │ ← Rust struct
│  (FFI wrapper)  │
└────────┬────────┘
         │ Implements
         ↓
┌─────────────────┐
│ Read/Write/Seek │ ← Rust traits
│   (Std traits)  │
└────────┬────────┘
         │ Calls back to
         ↓
┌─────────────────┐
│  C Callbacks    │
└────────┬────────┘
         │
         ↓
┌─────────────────┐
│  Native I/O     │ (FILE*, Python file, etc.)
└─────────────────┘
```

## Use Cases

This pattern is useful when:

1. **Language Integration**: Wrapping native streams from other languages
2. **Custom I/O**: Implementing non-standard I/O (network, memory, encryption)
3. **Interoperability**: Allowing Rust code to use C-provided I/O without copying

## Real-World Example: C2PA

The C2PA project uses this exact pattern to allow different languages to provide stream implementations that Rust code can use for reading/writing C2PA manifests.

## API Reference

See the generated header `include/cimpl_stream.h` for complete API documentation.

## Memory Safety

- All pointers are validated before use
- Callbacks must remain valid for the stream's lifetime
- The context pointer must remain valid for the stream's lifetime
- Always free streams with `cimpl_free()` when done

## Error Handling

All functions that can fail return error indicators:
- `-1` for integer-returning functions
- `NULL` for pointer-returning functions

Get detailed error messages with:

```c
char* error = cimpl_stream_last_error();
if (error) {
    fprintf(stderr, "Error: %s\n", error);
    cimpl_free(error);
}
```

## Testing

The Rust implementation includes comprehensive tests:

```bash
make test
```

## Next Steps

After understanding this example:

1. Try implementing a custom stream (e.g., memory buffer)
2. Use AI to generate bindings for other languages (Python, Lua, etc.)
3. Explore how this pattern can wrap complex I/O operations

See `../AI_WORKFLOW.md` for guidance on using AI to generate language bindings.
