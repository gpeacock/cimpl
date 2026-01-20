# Node.js Bindings

This directory contains Node.js FFI bindings for the UUID library using **Koffi**.

## Files

- `uuid_bindings.js` - Node.js wrapper using Koffi FFI
- `test.js` - Test suite demonstrating usage
- `package.json` - NPM dependencies
- `uuid_ffi-napi.js.deprecated` - Old ffi-napi version (unmaintained, kept for reference)
- `test_ffi-napi.js.deprecated` - Old ffi-napi tests

## Purpose

This is a **reference implementation** demonstrating what an AI or code generator should produce from the C header file. It shows:

- Idiomatic JavaScript API (classes, getters, methods)
- Custom error classes based on error codes
- Automatic memory management with finalization
- Modern ES6+ syntax

## Technology: Koffi

We use **Koffi** instead of the older `ffi-napi` because:
- ✅ Actively maintained (as of 2024-2026)
- ✅ Works with Node.js 18, 20, 22, 23+
- ✅ Better performance and ergonomics
- ✅ Native support for modern Node.js N-API

The deprecated `ffi-napi` files are kept for reference, showing how FFI library choice doesn't affect the C API stability.

## Usage

```javascript
const { Uuid, v4, v7, ParseError } = require('./uuid_bindings');

// Generate UUIDs
const uuid = v4();
console.log(uuid.toString()); // e.g., "550e8400-e29b-41d4-a716-446655440000"

// Parse UUIDs
try {
  const uuid = new Uuid("550e8400-e29b-41d4-a716-446655440000");
} catch (e) {
  if (e instanceof ParseError) {
    console.error(`Invalid UUID: ${e.message}`);
  }
}

// Comparison
console.log(Uuid.nil().isNil()); // true
console.log(Uuid.nil().compare(Uuid.max())); // -1
```

## Installation

```bash
npm install
```

## Running Tests

From this directory:

```bash
npm test
```

## Status

**Reference Implementation** - This binding was manually created to demonstrate the expected output for AI-generated Node.js bindings. It is maintained as part of the test suite.

## AI Generation

To generate this binding with AI, provide:
1. The C header file (`include/cimple_uuid.h`)
2. A prompt like: "Generate Node.js Koffi bindings with custom error classes, memory management, and idiomatic JavaScript API"

The header includes comprehensive documentation that AI models can use to generate similar code.
