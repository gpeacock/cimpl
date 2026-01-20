# AI Binding Generation Guide

This document explains how to test `cimpl`'s AI-friendliness by generating language bindings.

## The Goal

`cimpl` aims to make the C header file (`include/cimple_uuid.h`) so clear and well-documented that AI models can generate high-quality language bindings with minimal prompting.

## Testing Approach

### 1. Create the Test Environment

```bash
cd uuid-example
mkdir -p .ai-generated/<language>
```

The `.ai-generated/` directory is gitignored for experimentation.

### 2. Provide the AI with the C Header

Give the AI model **only** the C header file:

```bash
cat include/cimple_uuid.h
```

### 3. Prompt Examples

#### Python (ctypes)
```
Generate Python bindings for this C library using ctypes.

Requirements:
- Custom exception classes based on error codes
- Pythonic API (properties, operators, context managers)
- Automatic memory management
- Type hints

Header file:
[paste cimple_uuid.h]
```

#### Node.js (Koffi)
```
Generate Node.js bindings for this C library using Koffi FFI.

Requirements:
- Custom error classes based on error codes
- Idiomatic JavaScript API (classes, getters, methods)
- Automatic memory management with finalization
- Modern ES6+ syntax

Header file:
[paste cimple_uuid.h]
```

#### Lua (LuaJIT FFI)
```
Generate LuaJIT FFI bindings for this C library.

Requirements:
- Metatables for object-oriented API
- Metamethods for operators (__tostring, __lt, __eq, etc.)
- Automatic garbage collection
- Error handling with pcall

Header file:
[paste cimple_uuid.h]
```

### 4. Compare with Reference Implementations

```bash
# Compare structure
diff -u .ai-generated/python/uuid_bindings.py bindings/python/uuid_bindings.py

# Test functionality
cd .ai-generated/python
python3 ../../bindings/python/test.py  # Use the same test suite
```

### 5. Iterate on Header Documentation

If the AI-generated binding is missing features or has bugs:

1. **Don't fix the generated code directly**
2. Instead, improve the C header documentation
3. Regenerate the binding with the updated header
4. Repeat until the AI produces correct code

This process helps improve `cimpl`'s AI-friendliness for all users.

## What to Look For

Good AI-generated bindings should have:

- ✅ **Correct FFI declarations** - All functions and types properly declared
- ✅ **Error handling** - Custom exception/error classes for each error code
- ✅ **Memory management** - Automatic cleanup, no manual `free()` calls
- ✅ **Idiomatic API** - Not just raw FFI wrappers, but language-appropriate interfaces
- ✅ **Type safety** - Where supported by the language
- ✅ **Documentation** - Docstrings or comments
- ✅ **Tests** - If possible, the AI should generate basic tests

## Evaluation Criteria

Rate the generated binding on a scale of 1-5:

| Aspect | Score | Notes |
|--------|-------|-------|
| FFI Declarations | 1-5 | Are all functions correctly declared? |
| Error Handling | 1-5 | Custom exceptions? Correct codes? |
| Memory Management | 1-5 | Automatic cleanup? No leaks? |
| API Design | 1-5 | Idiomatic? Easy to use? |
| Type Safety | 1-5 | Type hints/annotations where appropriate? |
| Documentation | 1-5 | Clear docstrings? |
| Correctness | 1-5 | Does it pass the test suite? |

**Total: ___ / 35**

- **30-35**: Excellent - Production ready
- **25-29**: Good - Minor tweaks needed
- **20-24**: Fair - Some issues to address
- **<20**: Poor - Significant problems

## Example Test Session

```bash
# 1. Generate Python binding with Claude
cd uuid-example/.ai-generated/python
# Paste the C header and prompt to your AI

# 2. Save the output
# Save to uuid_bindings.py

# 3. Test it
python3 << 'EOF'
from uuid_bindings import Uuid, v4
uuid = v4()
print(f"Generated: {uuid}")
EOF

# 4. Run full test suite
cp ../../bindings/python/test.py .
# Update import if needed
python3 test.py

# 5. Compare with reference
diff -u uuid_bindings.py ../../bindings/python/uuid_bindings.py | head -50
```

## Common Issues

### Issue: AI doesn't generate error classes

**Fix**: Enhance the C header comments around error codes:

```c
/**
 * @note Error codes can be used to create exception classes:
 * @code{.py}
 * class ParseError(Exception):
 *     def __init__(self, message):
 *         super().__init__(message)
 *         self.code = ERROR_UUID_PARSE_ERROR
 * @endcode
 */
extern const int32_t ERROR_UUID_PARSE_ERROR;
```

### Issue: AI doesn't add automatic memory management

**Fix**: Add examples in the header:

```c
/**
 * @par Example (Python):
 * @code{.py}
 * class Uuid:
 *     def __del__(self):
 *         if self._handle:
 *             cimpl_free(self._handle)
 *             self._handle = None
 * @endcode
 */
```

### Issue: API isn't idiomatic

**Fix**: Provide examples of the desired API in each function's documentation.

## Success Metrics

A successful `cimpl` implementation means:

1. AI generates functionally correct bindings from header alone
2. Generated bindings pass the reference test suite
3. Generated bindings have idiomatic APIs
4. Less than 10% of code needs manual adjustment
5. AI can generate for multiple languages with similar prompts

## Contributing

If you test AI generation and find issues:

1. Document what went wrong
2. Improve the C header documentation
3. Test that the improvement helps AI generation
4. Submit a PR with the header improvements

Don't submit the AI-generated code itself - submit the improvements to the header that enable better AI generation.
