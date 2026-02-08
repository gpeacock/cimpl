# Python Bindings for C2PA FFI

Python wrapper around the C2PA C FFI bindings using `ctypes`.

## Features

- ✅ **Automatic memory management** - No manual cleanup needed
- ✅ **Pythonic API** - Classes, properties, and builder patterns
- ✅ **Type hints** - Full type annotations for IDE support
- ✅ **Error handling** - Python exceptions from C errors
- ✅ **Dictionary support** - Convert between Python dicts and JSON
- ✅ **Builder pattern** - Fluent method chaining

## Installation

No installation needed! Just make sure the Rust library is built:

```bash
cd ../..  # Go to c2pa-example root
cargo build --release
```

## Quick Start

```python
from c2pa import Context, Settings

# Simple context creation
ctx = Context.new()

# Builder pattern with dict
ctx = Context.new().with_settings_dict({
    "verify": {"verify_after_sign": True}
})

# Create settings separately
settings = Settings.from_dict({
    "verify": {"verify_after_sign": False}
})
json_str = settings.to_json()
print(json_str)
```

## Examples

### 1. Create Settings

```python
from c2pa import Settings

# From defaults
settings = Settings.new()

# From Python dictionary
settings = Settings.from_dict({
    "verify": {"verify_after_sign": True}
})

# From JSON string
settings = Settings.from_json('{"verify": {"verify_after_sign": true}}')

# From TOML string
settings = Settings.from_toml('[verify]\nverify_after_sign = true')
```

### 2. Serialize Settings

```python
settings = Settings.new()

# To JSON string
json_str = settings.to_json()
print(json_str)

# To Python dictionary
config = settings.to_dict()
print(config)

# To TOML string
toml_str = settings.to_toml()
print(toml_str)
```

### 3. Context with Builder Pattern

```python
from c2pa import Context

# Create and configure in one go
ctx = (Context.new()
       .with_settings_dict({"verify": {"verify_after_sign": True}}))

# Or step by step
ctx = Context.new()
ctx.with_settings_json('{"verify": {"verify_after_sign": false}}')
# ctx pointer stays the same!
```

### 4. Error Handling

```python
from c2pa import Context, C2paError, C2paErrorCode

try:
    settings = Settings.from_json('invalid json')
except C2paError as e:
    print(f"Error {e.code}: {e.message}")
    if e.code == C2paErrorCode.INVALID_SETTINGS:
        print("Invalid JSON/TOML settings")
```

## Running the Example

```bash
# Make sure the library is built
cd ../..
cargo build --release

# Run the example
cd bindings/python
python3 example.py
```

Expected output:
```
============================================================
C2PA Python Bindings Example
============================================================

1. Creating Settings with defaults...
   ✓ Settings created
   Settings as JSON: {"verify":{"verify_after_sign":true}...
   Settings as TOML (first 100 chars): [verify]
verify_after_sign = true...

2. Creating Settings from Python dict...
   ✓ Settings created from dict
   Verify after sign: True

3. Creating Settings from JSON string...
   ✓ Settings created from JSON

4. Creating Settings from TOML string...
   ✓ Settings created from TOML

5. Creating Context...
   ✓ Context created

6. Using builder pattern with JSON...
   ✓ Context configured with JSON

7. Using builder pattern with Python dict...
   ✓ Context configured with dict

8. Using builder pattern with Settings object...
   ✓ Context configured with Settings object

9. Using builder pattern with TOML...
   ✓ Context configured with TOML

10. Testing error handling with invalid JSON...
   ✓ Caught expected error (code 100): InvalidSettings: ...

============================================================
All examples completed successfully!
============================================================
```

## API Reference

### Classes

#### `Settings`
- `Settings.new()` - Create with defaults
- `Settings.from_json(json_str)` - Create from JSON
- `Settings.from_dict(config)` - Create from Python dict
- `Settings.from_toml(toml_str)` - Create from TOML
- `settings.to_json()` - Serialize to JSON
- `settings.to_dict()` - Convert to Python dict
- `settings.to_toml()` - Serialize to TOML

#### `Context`
- `Context.new()` - Create new context
- `ctx.with_settings_json(json_str)` - Configure with JSON (builder)
- `ctx.with_settings_dict(config)` - Configure with dict (builder)
- `ctx.with_settings_toml(toml_str)` - Configure with TOML (builder)
- `ctx.with_settings(settings)` - Configure with Settings object (builder)

#### `C2paError`
- Exception raised on C2PA errors
- `e.code` - Error code (int)
- `e.message` - Error message (str)

#### `C2paErrorCode`
- `INVALID_SETTINGS` (100) - JSON/TOML parse error
- `SIGNER_ERROR` (101) - Signer creation error
- `CONTEXT_ERROR` (102) - Context creation error
- `INVALID_FORMAT` (103) - Invalid file format
- `IO_ERROR` (104) - File I/O error
- `SERIALIZATION_ERROR` (105) - Serialization error

### Functions

- `get_last_error()` - Get last error message
- `get_error_code()` - Get last error code

## Memory Management

Memory is automatically managed:
- Objects are freed when they go out of scope (`__del__`)
- No need to call any free functions manually
- Safe to use in long-running programs

## Platform Support

- ✅ macOS (libcimpl_c2pa.dylib)
- ✅ Linux (libcimpl_c2pa.so)
- ✅ Windows (cimpl_c2pa.dll) - needs testing

## Requirements

- Python 3.6+
- Built c2pa-example Rust library

## License

Same as parent project (Apache 2.0 / MIT)
