# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-02-16

### Note

This is a **new project** taking over the `cimpl` crate name. Previous versions (0.1.0, 0.1.1, 0.2.0) were a different, unrelated project and have been yanked. This version represents a complete rewrite with a different purpose and API.

### Added

**Core FFI Infrastructure:**
- Comprehensive macro system for safe C FFI bindings
- Type-safe pointer tracking with validation and leak detection
- Thread-local error storage with string-based error format (`"VariantName: details"`)
- Automatic memory management with `box_tracked!`, `arc_tracked!` macros
- Universal `cimpl_free()` function for tracked allocations

**Pointer Handling Macros:**
- `ptr_or_return!()` family - Null pointer validation
- `deref_or_return!()` family - Safe pointer dereferencing (immutable)
- `deref_mut_or_return!()` family - Safe pointer dereferencing (mutable)
- Support for custom return values (`_null`, `_int`, `_zero`, `_false` variants)

**String Conversion Macros:**
- `cstr_or_return!()` family - C string to Rust with UTF-8 validation
- `to_c_string()` - Rust String to tracked C string
- `option_to_c_string!()` - Option<String> to C string (NULL if None)

**Byte Array Macros:**
- `bytes_or_return!()` family - Validate and convert C byte arrays
- `to_c_bytes()` - Rust Vec<u8> to tracked C byte array

**Result Handling Macros:**
- `ok_or_return!()` family - Result unwrapping with automatic error conversion
- Works with any error type implementing `Into<cimpl::Error>`

**Option Handling Macros:**
- `some_or_return!()` family - Option unwrapping with custom errors
- `some_or_return_other!()` family - Option with Error::other message

**Error System:**
- `Error::new(variant, message)` - Manual error construction
- `Error::from_error(e)` - Automatic error conversion from any std::error::Error
- `Error::last_message()` - Thread-local error retrieval
- `Error::set_last()` - Set thread-local error
- String-based format enables easy parsing in any language

**Documentation:**
- Comprehensive `README.md` with examples and comparison to alternatives
- `AI_WORKFLOW.md` - Complete guide for AI-assisted FFI development
- `PHILOSOPHY.md` - Design principles and three-stage pipeline explanation
- Pre-flight checklist for common anti-patterns

**Examples:**
- `examples/reference/` - Complete ValueConverter example showing all patterns
  - Clean lib.rs/ffi.rs separation
  - Full Python bindings with ctypes
  - Demonstrates struct methods, constructors, error handling
- `examples/uuid/` - Real-world external crate wrapping
  - 15 FFI functions exposing uuid crate
  - Complete Python bindings
  - **AI-generated in 15 minutes** from documentation
  - Demonstrates direct external crate usage pattern

### Features

- ✅ Pattern-driven design enabling AI code generation
- ✅ Zero unsafe code needed in user FFI functions
- ✅ Type validation prevents pointer confusion
- ✅ Memory leak detection in tests
- ✅ Double-free protection
- ✅ Cross-language error format
- ✅ cbindgen integration for C header generation
- ✅ Production-ready examples with documentation

### Philosophy

The vision: **Write Rust once → Clean C API → AI generates bindings for all languages**

This library enables:
1. Safe, ergonomic FFI from Rust
2. Universal C ABI as stable interface
3. AI-friendly patterns for automatic binding generation

Proven with real AI generation: Complete UUID FFI wrapper created in 15 minutes with zero errors.

---

## Previous Versions (Different Project - Yanked)

**Note:** Versions 0.1.0, 0.1.1, and 0.2.0 were a different project by a previous owner and are not related to this crate. They have been yanked and should not be used.
