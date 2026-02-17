# Python Bindings for ValueConverter

Python bindings for the ValueConverter C library, demonstrating how to wrap a cimpl-based FFI library.

## What This Demonstrates

This is **not a toy example**. Unlike trivial FFI demos, these bindings show production patterns:

✅ **Error handling** - C errors converted to typed Python exceptions  
✅ **Memory management** - Automatic cleanup with context managers  
✅ **Type safety** - Proper ctypes signatures for all functions  
✅ **Pythonic API** - Natural Python interface wrapping C  
✅ **String parsing** - Parsing cimpl's "VariantName: details" error format  

## Installation

First, build the Rust library:

```bash
cd ../../
cargo build
```

No pip install needed - the bindings will automatically find the compiled library.

## Usage

### Basic Example

```python
from value_converter import ValueConverter

# Create from integer, convert to hex
with ValueConverter.from_i32(42) as vc:
    print(vc.to_hex())  # "2a000000"

# Create from string, convert to hex
with ValueConverter.from_string("Hi") as vc:
    print(vc.to_hex())  # "4869"
    print(len(vc))      # 2
```

### Error Handling

```python
from value_converter import ValueConverter, OutOfRangeError

try:
    with ValueConverter.from_string("Hi") as vc:  # Only 2 bytes
        value = vc.to_i32()  # Needs exactly 4 bytes
except OutOfRangeError as e:
    print(f"Error: {e}")
```

### All Constructor Methods

```python
# From integers
vc = ValueConverter.from_i32(-42)
vc = ValueConverter.from_u32(42)
vc = ValueConverter.from_i64(-9223372036854775808)
vc = ValueConverter.from_u64(18446744073709551615)

# From data
vc = ValueConverter.from_bytes(b"Hello")
vc = ValueConverter.from_string("Hello")
vc = ValueConverter.from_hex("48656c6c6f")
```

### All Conversion Methods

```python
with ValueConverter.from_i32(42) as vc:
    # To integers (may raise OutOfRangeError if wrong size)
    i32_val = vc.to_i32()    # Needs 4 bytes
    u32_val = vc.to_u32()    # Needs 4 bytes
    i64_val = vc.to_i64()    # Needs 8 bytes
    u64_val = vc.to_u64()    # Needs 8 bytes
    
    # To data (always succeeds)
    hex_str = vc.to_hex()    # "2a000000"
    bytes_data = vc.to_bytes()  # b'*\x00\x00\x00'
    
    # To string (may raise InvalidUtf8Error)
    text = vc.to_string()  # May fail if not valid UTF-8
    
    # Properties
    size = len(vc)  # Size in bytes
```

## Exception Types

All exceptions inherit from `ValueConverterError`:

- `OutOfRangeError` - Value out of range for target type
- `InvalidUtf8Error` - Bytes aren't valid UTF-8
- `InvalidHexError` - Invalid hex string
- `BufferTooLargeError` - Buffer exceeds 8 bytes
- `EmptyValueError` - Value is empty

## Running Examples

```bash
python3 example.py
```

## How It Works

### Error Parsing

The C library returns errors in the format `"VariantName: details"`. The Python binding parses this and raises the appropriate typed exception:

```python
# C error: "OutOfRange: need exactly 4 bytes for i32, got 2"
# Python: raises OutOfRangeError("need exactly 4 bytes for i32, got 2")
```

### Memory Management

Uses Python context managers (`with` statements) to ensure automatic cleanup:

```python
with ValueConverter.from_i32(42) as vc:
    print(vc.to_hex())
# vc_free() called automatically here
```

Or manual cleanup:

```python
vc = ValueConverter.from_i32(42)
try:
    print(vc.to_hex())
finally:
    vc.__exit__(None, None, None)  # Explicit cleanup
```

### Library Loading

The binding automatically finds the compiled library in `target/debug` or `target/release`, handling platform differences (`.so` on Linux, `.dylib` on macOS, `.dll` on Windows).

## Key Implementation Details

**C Function Wrappers** (`value_converter.py`, lines 75-115):
- All `vc_*` functions wrapped with proper ctypes signatures
- Handles pointer types, out-parameters, return values

**Error Handling** (`_check_error()`, lines 134-145):
- Calls `vc_last_error()` after failures
- Parses "VariantName: details" format
- Maps to appropriate Python exception class

**Context Manager** (`__enter__`/`__exit__`, lines 239-249):
- Ensures `vc_free()` is called even on exceptions
- Prevents double-free

**Pythonic API** (lines 252-364):
- Class methods for construction (`from_i32()`, etc.)
- Instance methods for conversion (`to_hex()`, etc.)
- `__len__()` for natural size queries
- `__repr__()` for debugging
