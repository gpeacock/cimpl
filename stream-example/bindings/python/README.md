# Python Bindings for CimplStream

Python bindings for the `cimpl_stream` library, providing callback-based I/O streams that bridge Python and Rust/C.

## Overview

These bindings allow you to wrap Python file-like objects (files, `BytesIO`, etc.) and use them with Rust code through the cimpl_stream interface. This is useful for:

- Passing Python file objects to Rust/C libraries
- Implementing custom I/O backends in Python
- Testing stream-based APIs with in-memory buffers

## Installation

1. Build the cimpl_stream library:

```bash
cd ../..  # Go to stream-example directory
cargo build --release
```

2. No additional Python packages required! The bindings use only standard library modules (`ctypes`, `io`, `os`).

## Usage

### Basic File Operations

```python
from cimpl_stream import Stream

# Write to a file
with open('output.txt', 'wb') as f:
    stream = Stream(f)
    stream.write(b"Hello, World!")
    stream.flush()

# Read from a file
with open('output.txt', 'rb') as f:
    stream = Stream(f)
    data = stream.read(100)
    print(data.decode('utf-8'))
```

### In-Memory Buffers

```python
import io
from cimpl_stream import Stream

# Use BytesIO for in-memory operations
buffer = io.BytesIO()
stream = Stream(buffer)

stream.write(b"In-memory data")
stream.seek(0)
data = stream.read()
print(data)
```

### Seek Operations

```python
with open('file.txt', 'rb') as f:
    stream = Stream(f)
    
    # Seek to position 10
    stream.seek(10)
    
    # Seek relative to current position
    stream.seek(-5, os.SEEK_CUR)
    
    # Seek relative to end
    stream.seek(-10, os.SEEK_END)
    
    # Get current position
    pos = stream.tell()
```

### Context Manager Support

```python
from cimpl_stream import Stream

# Automatically closes the stream
with open('file.txt', 'rb') as f:
    with Stream(f) as stream:
        data = stream.read()
    # Stream is automatically freed here
```

### Error Handling

```python
from cimpl_stream import Stream, CimplStreamError, IoError

try:
    with open('file.txt', 'rb') as f:
        stream = Stream(f)
        data = stream.read(100)
except IoError as e:
    print(f"I/O error: {e.code} - {e}")
except CimplStreamError as e:
    print(f"Stream error: {e}")
```

## API Reference

### `Stream` Class

Main class for wrapping Python file-like objects.

#### Constructor

```python
stream = Stream(file_obj)
```

- `file_obj`: A Python file-like object with `read`, `write`, `seek`, and `flush` methods

#### Methods

- **`read(size=-1)`**: Read up to `size` bytes (or default buffer size if -1)
  - Returns: `bytes`
  
- **`write(data)`**: Write bytes to the stream
  - `data`: `bytes` to write
  - Returns: Number of bytes written (`int`)
  
- **`seek(offset, whence=os.SEEK_SET)`**: Change stream position
  - `offset`: Offset in bytes (`int`)
  - `whence`: `os.SEEK_SET`, `os.SEEK_CUR`, or `os.SEEK_END`
  - Returns: New absolute position (`int`)
  
- **`tell()`**: Get current stream position
  - Returns: Current position (`int`)
  
- **`flush()`**: Flush the stream
  
- **`close()`**: Close the stream and free resources

### Exceptions

- **`CimplStreamError`**: Base exception for all stream errors
  - `.code`: Error code (`int`)
  
- **`NullParameterError`**: Raised when a null parameter is passed
  
- **`IoError`**: Raised when an I/O operation fails

### Convenience Functions

- **`wrap_file(file_obj)`**: Alias for `Stream(file_obj)`

## Examples

Run the example script:

```bash
python3 example.py
```

The example demonstrates:
1. File I/O operations
2. In-memory buffers (BytesIO)
3. Seek operations
4. Error handling

## How It Works

1. **Python file object** → Wrapped by `Stream` class
2. **Callbacks created** → Python functions for read/write/seek/flush
3. **C library called** → `cimpl_stream_new()` with callbacks
4. **Operations bridge** → Python ↔ Rust ↔ Python callbacks

```
┌─────────────────┐
│  Python Code    │
│  stream.read()  │
└────────┬────────┘
         │
         ↓
┌─────────────────┐
│ cimpl_stream    │  ← Rust FFI library
│  (C wrapper)    │
└────────┬────────┘
         │
         ↓
┌─────────────────┐
│  Read Callback  │  ← Python function
│  (Python code)  │
└────────┬────────┘
         │
         ↓
┌─────────────────┐
│  Python File    │  ← Original file object
│   f.read()      │
└─────────────────┘
```

## Memory Management

- Streams are automatically freed when the `Stream` object is garbage collected
- Use context managers (`with`) for explicit resource management
- Calling `close()` explicitly is safe and recommended

## Thread Safety

⚠️ **Note**: The bindings use Python's GIL for thread safety, but the underlying file objects must be thread-safe. Use appropriate locking if sharing streams across threads.

## Limitations

- Callback errors are printed to stderr but may not propagate perfectly
- Very large reads/writes may have performance implications due to ctypes overhead
- The wrapped file object must remain valid for the lifetime of the Stream

## See Also

- [CimplStream C API](../../include/cimpl_stream.h)
- [Example C code](../../example.c)
- [AI Workflow Guide](../../../AI_WORKFLOW.md)
