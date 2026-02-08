# C2PA Rust SDK - C FFI Bindings

Minimal FFI wrapper around [c2pa-rs](https://github.com/contentauth/c2pa-rs) providing C bindings for Context and Settings.

## Overview

This example demonstrates creating `cimpl` bindings for the c2pa-rs library, focusing on:

- ✅ **Context** - Central configuration object for C2PA operations
- ✅ **Settings** - Configuration via JSON/TOML strings
- ✅ **Error Handling** - Simplified error mapping using From trait
- ✅ **Memory Management** - Tracked allocation/deallocation

## API

### Settings Management

```c
// Create new Settings with defaults
C2paSettings* settings = c2pa_settings_new();

// Create from JSON
const char* json = "{\"verify\": {\"verify_after_sign\": true}}";
C2paSettings* settings = c2pa_settings_from_json(json);

// Create from TOML
const char* toml = "[verify]\nverify_after_sign = true";
C2paSettings* settings = c2pa_settings_from_toml(toml);

// Serialize to JSON
char* json_str = c2pa_settings_to_json(settings);
printf("%s\n", json_str);
c2pa_free(json_str);

// Serialize to TOML
char* toml_str = c2pa_settings_to_toml(settings);
printf("%s\n", toml_str);
c2pa_free(toml_str);

// Free settings
c2pa_settings_free(settings);
```

### Context Management (Builder Pattern)

```c
// Create a new context
C2paContext* ctx = c2pa_context_new();

// Option 1: Configure with JSON string
const char* json = "{\"verify\": {\"verify_after_sign\": true}}";
if (c2pa_context_with_settings(ctx, json) != 0) {
    printf("Error: %s\n", c2pa_last_error());
}

// Option 2: Configure with TOML string
const char* toml = "[verify]\nverify_after_sign = true";
if (c2pa_context_with_settings_toml(ctx, toml) != 0) {
    printf("Error: %s\n", c2pa_last_error());
}

// Option 3: Configure with Settings object
C2paSettings* settings = c2pa_settings_from_json(json);
if (c2pa_context_with_settings_obj(ctx, settings) != 0) {
    printf("Error: %s\n", c2pa_last_error());
}
c2pa_settings_free(settings);

// Free context when done
c2pa_context_free(ctx);
```

**Key Feature:** The context pointer stays the same throughout - perfect for builder patterns in higher-level bindings!

### Settings

```c
// Get default settings as JSON
char* settings = c2pa_settings_default_json();
printf("%s\n", settings);
c2pa_free(settings);
```

### Error Handling

```c
C2paContext* ctx = c2pa_context_with_settings_json(invalid_json);
if (!ctx) {
    int code = c2pa_error_code();
    char* msg = c2pa_last_error();
    printf("Error %d: %s\n", code, msg);
    c2pa_free(msg);
}
```

## Error Codes

- **100** - InvalidSettings: JSON/TOML parse error
- **101** - SignerError: Signer creation/config error
- **102** - ContextError: Context creation error
- **103** - InvalidFormat: Invalid file format
- **104** - IoError: File I/O error
- **105** - SerializationError: JSON/TOML serialization error

## Building

```bash
cargo build --release
```

The C header is automatically generated at `include/c2pa.h`.

## Python Bindings (Builder Pattern)

The stable pointer allows idiomatic builder patterns in Python:

```python
# Python binding layer (bindings/python/c2pa.py)
class Context:
    def __init__(self, ptr):
        self._ptr = ptr
    
    @classmethod
    def new(cls):
        ptr = lib.c2pa_context_new()
        if not ptr:
            raise C2paError.from_last_error()
        return cls(ptr)
    
    def with_settings(self, settings):
        """Builder-style chaining"""
        if isinstance(settings, dict):
            import json
            settings = json.dumps(settings)
        result = lib.c2pa_context_with_settings(self._ptr, settings.encode())
        if result != 0:
            raise C2paError.from_last_error()
        return self  # Same pointer, chaining works!
    
    def __del__(self):
        if self._ptr:
            lib.c2pa_context_free(self._ptr)

# Usage - idiomatic Python builder!
ctx = Context.new().with_settings({"verify": {"verify_after_sign": True}})
```

## Implementation Notes

This example demonstrates the new simplified error handling pattern:

1. Define your error enum (`C2paError`) with codes 100+
2. Create an internal error type (`C2paInternalError`) that wraps all error sources
3. Implement `From<SourceError> for InternalError` for each error source
4. Implement `From<InternalError> for cimpl::Error` to map to your error codes
5. Use macros (`cstr_or_return_null!`, `box_tracked!`) for common patterns
6. Manually convert errors where automatic conversion doesn't work

## Next Steps

- Add Reader and Builder bindings
- Add manifest creation/verification functions
- Add Python bindings
- Add C examples

## Version

- c2pa: 0.75+
- cimpl: 0.1.0
