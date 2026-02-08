# Settings API Implementation

## Overview

Added comprehensive Settings API support to c2pa-example, following the same wrapper pattern as Context for builder-style operations.

## New Types

### C2paSettings
Opaque pointer wrapping `c2pa::settings::Settings` with a stable pointer for FFI.

## API Functions

### Factory Functions

```c
// Create with defaults
C2paSettings* c2pa_settings_new();

// Create from JSON string
C2paSettings* c2pa_settings_from_json(const char* json);

// Create from TOML string
C2paSettings* c2pa_settings_from_toml(const char* toml);
```

### Serialization

```c
// Serialize to JSON (caller must free)
char* c2pa_settings_to_json(C2paSettings* settings);

// Serialize to TOML (caller must free)
char* c2pa_settings_to_toml(C2paSettings* settings);
```

### Context Integration

```c
// Apply Settings object to Context (builder-style)
int c2pa_context_with_settings_obj(C2paContext* ctx, C2paSettings* settings);
```

### Memory Management

```c
// Free Settings
int c2pa_settings_free(C2paSettings* settings);
```

## Usage Examples

### Create and Configure Settings

```c
// Start with defaults
C2paSettings* settings = c2pa_settings_new();

// Serialize to see current state
char* json = c2pa_settings_to_json(settings);
printf("Current settings: %s\n", json);
c2pa_free(json);

c2pa_settings_free(settings);
```

### Load from Configuration

```c
// Load from JSON
const char* config = "{\"verify\": {\"verify_after_sign\": true}}";
C2paSettings* settings = c2pa_settings_from_json(config);
if (!settings) {
    printf("Error: %s\n", c2pa_last_error());
    return 1;
}

// Use with Context
C2paContext* ctx = c2pa_context_new();
if (c2pa_context_with_settings_obj(ctx, settings) != 0) {
    printf("Error: %s\n", c2pa_last_error());
}

c2pa_settings_free(settings);
c2pa_context_free(ctx);
```

### Round-trip Configuration

```c
// Load from TOML
C2paSettings* settings = c2pa_settings_from_toml("[verify]\nverify_after_sign = true");

// Convert to JSON for inspection
char* json = c2pa_settings_to_json(settings);
printf("As JSON: %s\n", json);
c2pa_free(json);

c2pa_settings_free(settings);
```

## Implementation Notes

### Macro Usage (Following Anti-Pattern Guide)

All code follows the macro-first approach from `src/macros.rs`:

✅ **Correct patterns used:**
- `cstr_or_return_null!` - for C string conversion
- `ok_or_return_null!` - for Result handling
- `deref_or_return_null!` - for immutable pointer dereferencing  
- `deref_or_return_neg!` - for immutable pointer with -1 return
- `box_tracked!` - for heap allocation

❌ **No anti-patterns:**
- No manual `if ptr.is_null()` checks
- No manual `match result { Ok/Err }` patterns
- No manual `unsafe` pointer dereferencing

### Error Handling

- All functions that can fail set the last error via `Error::set_last()`
- Errors can be retrieved with `c2pa_error_code()` and `c2pa_last_error()`
- Uses `C2paInternalError` wrapper for error conversion
- TOML errors are wrapped as "Other" variant with string message

### Dependencies Added

- `toml = "0.8"` - For TOML serialization/deserialization

## Python Bindings Example

The Settings API enables clean Python bindings:

```python
class Settings:
    def __init__(self, ptr):
        self._ptr = ptr
    
    @classmethod
    def new(cls):
        ptr = lib.c2pa_settings_new()
        if not ptr:
            raise C2paError.from_last_error()
        return cls(ptr)
    
    @classmethod
    def from_json(cls, json_str):
        if isinstance(json_str, dict):
            import json
            json_str = json.dumps(json_str)
        ptr = lib.c2pa_settings_from_json(json_str.encode())
        if not ptr:
            raise C2paError.from_last_error()
        return cls(ptr)
    
    def to_json(self):
        json_ptr = lib.c2pa_settings_to_json(self._ptr)
        if not json_ptr:
            raise C2paError.from_last_error()
        json_str = ctypes.string_at(json_ptr).decode()
        lib.c2pa_free(json_ptr)
        return json_str
    
    def __del__(self):
        if self._ptr:
            lib.c2pa_settings_free(self._ptr)

# Usage
settings = Settings.from_json({"verify": {"verify_after_sign": True}})
ctx = Context.new()
ctx.with_settings_obj(settings)  # Can reuse settings for multiple contexts
```

## Testing

Build verification:
```bash
cd c2pa-example
cargo build --release
```

The generated C header at `include/c2pa.h` includes all Settings type definitions and function declarations.
