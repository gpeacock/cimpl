# Python Bindings

This directory contains Python FFI bindings for the UUID library using `ctypes`.

## Files

- `uuid_bindings.py` - Python wrapper using ctypes
- `test.py` - Test suite demonstrating usage

## Purpose

This is a **reference implementation** demonstrating what an AI or code generator should produce from the C header file. It shows:

- Pythonic API design (context managers, properties, operators)
- Custom exception classes based on error codes
- Automatic memory management
- Type hints and documentation

## Usage

```python
from uuid_bindings import Uuid, v4, v7, ParseError

# Generate UUIDs
uuid = v4()
print(uuid)  # e.g., "550e8400-e29b-41d4-a716-446655440000"

# Parse UUIDs
try:
    uuid = Uuid("550e8400-e29b-41d4-a716-446655440000")
except ParseError as e:
    print(f"Invalid UUID: {e}")

# Comparison
assert v4().nil() == False
assert Uuid.nil() < Uuid.max()
```

## Running Tests

From this directory:

```bash
python3 test.py
```

## Status

**Reference Implementation** - This binding was manually created to demonstrate the expected output for AI-generated Python bindings. It is maintained as part of the test suite.

## AI Generation

To generate this binding with AI, provide:
1. The C header file (`include/cimpl_uuid.h`)
2. A prompt like: "Generate Python ctypes bindings with custom exceptions, memory management, and Pythonic API"

The header includes comprehensive documentation that AI models can use to generate similar code.
