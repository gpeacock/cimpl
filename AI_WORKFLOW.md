# AI-Assisted FFI Binding Generation

This guide explains how to use AI to generate language bindings for Rust libraries using `cimpl`.

## When to Use cimpl

`cimpl` is ideal for:
- ✅ **Python** - ctypes or cffi bindings
- ✅ **Lua** - LuaJIT FFI
- ✅ **Ruby** - FFI gem
- ✅ **Go** - cgo
- ✅ **C#** - P/Invoke
- ✅ **Java** - JNA/JNI
- ✅ **Swift** - C interop
- ✅ **C/C++** - Direct usage

## When NOT to Use cimpl

For **Node.js** and **WASM targets**, use [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen) instead:
- Better performance (no FFI overhead)
- Native async/await support
- Automatic TypeScript definitions
- Direct JavaScript integration
- Smaller bundle sizes

> **Note**: While Node.js *can* use cimpl via [Koffi FFI](https://github.com/Koromix/koffi), WASM with `wasm-bindgen` is the better choice for Node.js projects.

## The Three-Stage Workflow

```
┌─────────────────────────────────────────────────────────┐
│ Stage 1: Write or Select a Rust Library                │
│   • Your own code, or an existing crate                │
│   • Standard Rust - no FFI yet                          │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│ Stage 2: AI Generates C FFI (with cimpl)                │
│   • AI writes the Rust FFI wrapper using cimpl macros   │
│   • cbindgen generates C header automatically           │
│   • Clean, documented C API                             │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│ Stage 3: AI Generates Target Language Bindings         │
│   • AI reads the C header                               │
│   • Generates Python, Lua, etc.                        │
│   • Idiomatic, safe, automatic memory management        │
└─────────────────────────────────────────────────────────┘
```

## Stage 1: Choose Your Rust Library

You have two options:

### Option A: Wrap an Existing Crate

Pick any crate from crates.io that you want to expose to other languages.

**Example**: Wrapping the `uuid` crate

```toml
[dependencies]
cimpl = "0.1"
uuid = { version = "1.11", features = ["v4", "v7"] }

[build-dependencies]
cbindgen = "0.27"
```

### Option B: Write Your Own Library

Create your Rust implementation first, without any FFI concerns.

**Example**: A custom parser library

```rust
pub struct Parser {
    // Your implementation
}

impl Parser {
    pub fn new() -> Self { /* ... */ }
    pub fn parse(&mut self, input: &str) -> Result<Ast, Error> { /* ... */ }
}
```

## Stage 2: AI Generates C FFI Wrapper

### Prompt for AI

```
I have a Rust library that I want to expose through a C FFI using the cimpl library.

The cimpl library provides these macros:
- box_tracked!(expr) - Allocate and track a Box pointer
- cstr_or_return_null!(ptr) - Convert C string with null check
- deref_or_return_null!(ptr, Type) - Validate and dereference pointer
- deref_or_return_false!(ptr, Type) - Same but return false on error
- deref_or_return_zero!(ptr, Type) - Same but return 0 on error
- deref_mut_or_return_neg!(ptr, Type) - Mutable deref, return -1 on error
- ok_or_return_null!(result) - Unwrap Result or return null
- to_c_string(string) - Convert Rust String to C string
- cimpl_free(ptr) - Universal free function

Error handling:
- Define error constants with #[no_mangle] pub static ERROR_*: i32 = <code>;
- Create ERROR_MAPPER const: fn(&ExternalError) -> (i32, &'static str)
- The macros use ERROR_MAPPER automatically for error conversion

Here's my Rust library:
[paste your library code or describe the API]

Please generate:
1. A complete Rust FFI wrapper using cimpl macros
2. A build.rs that runs cbindgen
3. A cbindgen.toml configuration file

Follow these patterns:
- Use external types directly (they're opaque to cbindgen)
- All FFI functions are #[no_mangle] pub extern "C"
- Use box_tracked! for constructors
- Use deref_or_return_* for all pointer access
- Add comprehensive doc comments (cbindgen will include them)
```

### What You'll Get

The AI will generate something like:

```rust
use cimpl::{box_tracked, cstr_or_return_null, deref_or_return_null, 
            ok_or_return_null, to_c_string};
use std::os::raw::c_char;
use uuid::Uuid;
use std::str::FromStr;

// Error codes
#[no_mangle]
pub static ERROR_UUID_PARSE_ERROR: i32 = 100;

// Error mapper
const ERROR_MAPPER: fn(&uuid::Error) -> (i32, &'static str) = 
    |_e| (ERROR_UUID_PARSE_ERROR, "ParseError");

/// Creates a new random UUID (version 4).
#[no_mangle]
pub extern "C" fn uuid_new_v4() -> *mut Uuid {
    box_tracked!(Uuid::new_v4())
}

/// Parses a UUID from a string.
#[no_mangle]
pub extern "C" fn uuid_parse(s: *const c_char) -> *mut Uuid {
    let s_str = cstr_or_return_null!(s);
    let uuid = ok_or_return_null!(Uuid::from_str(&s_str));
    box_tracked!(uuid)
}

/// Converts UUID to string.
#[no_mangle]
pub extern "C" fn uuid_to_string(uuid: *mut Uuid) -> *mut c_char {
    let obj = deref_or_return_null!(uuid, Uuid);
    to_c_string(obj.to_string())
}
```

Plus `build.rs`:

```rust
extern crate cbindgen;

fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let output_file = std::path::Path::new(&crate_dir)
        .join("include")
        .join("cimpl_uuid.h");
    
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(output_file);
}
```

And `cbindgen.toml`:

```toml
language = "C"
include_guard = "CIMPL_UUID_H"
documentation = true
documentation_style = "doxy"

[export]
include = ["Uuid"]
```

### Build and Test

```bash
cargo build --release
```

This generates:
- `target/release/libcimpl_uuid.{a,so,dylib}` - The library
- `include/cimpl_uuid.h` - The C header with full documentation

## Stage 3: AI Generates Language Bindings

Now you have a clean C header file. Use AI to generate bindings for any language!

### Example: Python Bindings

**Prompt:**

```
Generate Python bindings for this C library using ctypes.

Requirements:
- Custom exception classes based on error codes
- Pythonic API with classes and methods
- Automatic memory management (use __del__)
- Type hints
- Properties where appropriate

Here's the C header file:
[paste include/cimpl_uuid.h]
```

**What You'll Get:**

```python
import ctypes
from pathlib import Path
from typing import Optional

# Load library
lib = ctypes.CDLL(Path(__file__).parent / "libcimpl_uuid.so")

# Error codes
ERROR_UUID_PARSE_ERROR = 100

class ParseError(Exception):
    """UUID parse error"""
    pass

# Function declarations
lib.uuid_new_v4.restype = ctypes.c_void_p
lib.uuid_parse.argtypes = [ctypes.c_char_p]
lib.uuid_parse.restype = ctypes.c_void_p
lib.uuid_to_string.argtypes = [ctypes.c_void_p]
lib.uuid_to_string.restype = ctypes.POINTER(ctypes.c_char)
lib.uuid_error_code.restype = ctypes.c_int32
lib.uuid_last_error.restype = ctypes.POINTER(ctypes.c_char)
lib.cimpl_free.argtypes = [ctypes.c_void_p]
lib.cimpl_free.restype = ctypes.c_int

class Uuid:
    """UUID wrapper with automatic memory management"""
    
    def __init__(self, handle: int):
        self._handle = handle
    
    def __del__(self):
        if self._handle:
            lib.cimpl_free(self._handle)
            self._handle = None
    
    def __str__(self) -> str:
        result = lib.uuid_to_string(self._handle)
        if not result:
            self._raise_last_error()
        s = ctypes.string_at(result).decode('utf-8')
        lib.cimpl_free(result)
        return s
    
    @staticmethod
    def _raise_last_error():
        code = lib.uuid_error_code()
        msg_ptr = lib.uuid_last_error()
        msg = ctypes.string_at(msg_ptr).decode('utf-8') if msg_ptr else "Unknown error"
        if msg_ptr:
            lib.cimpl_free(msg_ptr)
        
        if code == ERROR_UUID_PARSE_ERROR:
            raise ParseError(msg)
        raise Exception(f"Error {code}: {msg}")
    
    @classmethod
    def v4(cls) -> 'Uuid':
        """Generate a random UUID v4"""
        handle = lib.uuid_new_v4()
        if not handle:
            cls._raise_last_error()
        return cls(handle)
    
    @classmethod
    def parse(cls, s: str) -> 'Uuid':
        """Parse UUID from string"""
        handle = lib.uuid_parse(s.encode('utf-8'))
        if not handle:
            cls._raise_last_error()
        return cls(handle)

# Usage:
# uuid = Uuid.v4()
# print(uuid)  # Automatic memory management
```

## Tips for Success

### 1. Iterate on the C Header

If the AI-generated bindings have issues:
- **Don't fix the binding code directly**
- Improve the C header documentation
- Regenerate the bindings
- The header is the source of truth

### 2. Use Consistent Patterns

The AI learns from patterns in your C header:
- Consistent naming (`library_type_action`)
- Error code documentation
- Example code in comments
- Clear ownership semantics

### 3. Test Incrementally

1. Generate the Rust FFI
2. Test from C first (write a simple C program)
3. Then generate language bindings
4. Test each language separately

### 4. Provide Examples

In your prompt, show example usage:

```
I want the Python API to look like this:

uuid = Uuid.v4()
print(uuid)  # Automatic __str__

try:
    uuid = Uuid.parse("invalid")
except ParseError as e:
    print(f"Error: {e}")
```

## Real-World Example

See [uuid-example/](./uuid-example/) for a complete working example:

1. **Rust FFI**: [uuid-example/src/lib.rs](./uuid-example/src/lib.rs)
   - Wraps the `uuid` crate with cimpl
   - 200 lines of clean, safe code

2. **Generated Header**: [uuid-example/include/cimpl_uuid.h](./uuid-example/include/cimpl_uuid.h)
   - Auto-generated by cbindgen
   - Full documentation

3. **Language Bindings**: [uuid-example/bindings/](./uuid-example/bindings/)
   - Python (ctypes)
   - Lua (LuaJIT FFI)
   - C++ (direct usage)

Language bindings were generated with AI assistance and minimal manual editing!

## Benefits of This Approach

### ✅ One Codebase
Write Rust once, generate bindings for all languages

### ✅ Type Safety
Rust's type system + cimpl's pointer validation

### ✅ Memory Safety
Automatic tracking, no manual free() calls in target languages

### ✅ Fast Iteration
Change the Rust API, regenerate everything

### ✅ AI-Friendly
AI understands C patterns, generates idiomatic bindings

### ✅ Future-Proof
C ABI is stable, bindings work across language versions

## Next Steps

1. **Start Simple**: Try wrapping a small library first
2. **Read the Example**: Study [uuid-example/](./uuid-example/)
3. **Use AI**: Let AI do the heavy lifting
4. **Iterate**: Improve the C header based on results
5. **Share**: Contribute your findings back to cimpl

## See Also

- [AI_GENERATION_GUIDE.md](./uuid-example/AI_GENERATION_GUIDE.md) - Detailed testing methodology
- [EXTERNAL_CRATE_EXAMPLE.md](./uuid-example/EXTERNAL_CRATE_EXAMPLE.md) - Technical deep dive
- [PHILOSOPHY.md](./PHILOSOPHY.md) - Why this approach works
