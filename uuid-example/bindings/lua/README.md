# Lua Bindings

This directory contains LuaJIT FFI bindings for the UUID library.

## Files

- `uuid_bindings.lua` - Lua wrapper using LuaJIT FFI
- `test.lua` - Test suite demonstrating usage

## Purpose

This is a **reference implementation** demonstrating what an AI or code generator should produce from the C header file. It shows:

- Lua-idiomatic API (metatables, metamethods)
- Custom error handling based on error codes
- Automatic garbage collection integration
- FFI declarations directly from C header

## Requirements

- **LuaJIT** with FFI support (not standard Lua 5.x)

## Usage

```lua
local uuid = require("uuid_bindings")

-- Generate UUIDs
local u = uuid.v4()
print(tostring(u)) -- e.g., "550e8400-e29b-41d4-a716-446655440000"

-- Parse UUIDs
local ok, result = pcall(uuid.parse, "550e8400-e29b-41d4-a716-446655440000")
if not ok then
    print("Parse error: " .. result)
end

-- Comparison
assert(uuid.nil():is_nil())
assert(uuid.nil() < uuid.max())
```

## Running Tests

From this directory:

```bash
luajit test.lua
```

## Status

**Reference Implementation** - This binding was manually created to demonstrate the expected output for AI-generated Lua bindings. It is maintained as part of the test suite.

## AI Generation

To generate this binding with AI, provide:
1. The C header file (`include/cimple_uuid.h`)
2. A prompt like: "Generate LuaJIT FFI bindings with metatables, error handling, and garbage collection"

The header includes comprehensive documentation that AI models can use to generate similar code.

## Note on LuaJIT FFI

LuaJIT's FFI is particularly elegant for C bindings because:
- ✅ Zero-overhead FFI calls
- ✅ Direct C structure manipulation
- ✅ Easy integration with existing Lua code
- ✅ No compilation step needed
