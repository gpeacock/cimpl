# Language Bindings Guide

This guide documents best practices and gotchas for creating bindings from `cimpl`-generated C APIs to various programming languages.

## Universal Principles

All language bindings follow the same pattern:

1. **Load the shared library** (`.dylib`, `.so`, `.dll`)
2. **Declare C function signatures** using the language's FFI
3. **Create idiomatic wrapper classes** that handle memory and errors
4. **Map C error codes to language exceptions/errors**
5. **Provide automatic memory management** (destructors, finalizers, `__del__`, etc.)

## Node.js / JavaScript

### ✅ Recommended: Koffi

**Status**: Actively maintained, works with Node.js 18, 20, 22, 23+

```javascript
const koffi = require('koffi');
const lib = koffi.load('./target/release/libmylib.dylib');

// Define opaque pointer type
const ThingPtr = koffi.pointer(koffi.opaque('Thing'));

// Declare functions
const thing_new = lib.func('thing_new', ThingPtr, []);
const thing_free = lib.func('cimpl_free', 'int32', [koffi.pointer('void')]);
const thing_get_name = lib.func('thing_get_name', 'str', [ThingPtr]);
```

**Pros**:
- Modern, maintained library
- Clean API
- Works with all Node.js versions
- Good performance

**Cons**:
- Requires native module (but no compilation needed)

**Installation**:
```bash
npm install koffi
```

**Resources**:
- GitHub: https://github.com/Koromix/koffi
- Works out of the box on macOS, Linux, Windows

### ❌ Avoid: ffi-napi

**Status**: Unmaintained, broken on Node.js 18.20.8+ and 23+

**Why it fails**:
```
error: no matching function for call to 'napi_add_finalizer'
note: candidate function not viable: no known conversion from 
'napi_finalize' to 'node_api_nogc_finalize'
```

Node.js changed N-API signatures, and `ffi-napi` hasn't been updated.

**Do not use** - it won't compile on modern Node.js versions.

### Alternative: napi-rs

For compile-time bindings (not dynamic FFI):

```rust
// In Cargo.toml
[dependencies]
napi = "2"
napi-derive = "2"

// In lib.rs
use napi_derive::napi;

#[napi]
pub fn thing_new() -> Thing {
    Thing::new()
}
```

**Pros**:
- Type-safe
- No runtime FFI overhead
- Works with all Node.js versions

**Cons**:
- Requires compiling per platform
- More build complexity
- Not using the universal C API (Rust→Node directly)

**Use case**: When you want TypeScript types and don't need the universal C layer.

## Python

### ✅ Recommended: ctypes (built-in)

**Status**: Part of Python standard library, stable, widely used

```python
from ctypes import *
import platform

# Load library (platform-specific extension)
lib_name = {
    'Darwin': './target/release/libmylib.dylib',
    'Linux': './target/release/libmylib.so',
    'Windows': './target/release/mylib.dll'
}[platform.system()]

lib = CDLL(lib_name)

# Declare function signatures
lib.thing_new.restype = c_void_p
lib.thing_get_name.argtypes = [c_void_p]
lib.thing_get_name.restype = c_char_p
lib.cimpl_free.argtypes = [c_void_p]
lib.cimpl_free.restype = c_int

# Wrap in Python class
class Thing:
    def __init__(self, handle):
        self._handle = handle
        self._freed = False
    
    @classmethod
    def new(cls):
        handle = lib.thing_new()
        if not handle:
            raise RuntimeError("Failed to create Thing")
        return cls(handle)
    
    def get_name(self):
        if self._freed:
            raise RuntimeError("Thing has been freed")
        name = lib.thing_get_name(self._handle)
        return name.decode('utf-8') if name else None
    
    def __del__(self):
        if not self._freed and self._handle:
            lib.cimpl_free(self._handle)
            self._freed = True
```

**Pros**:
- No external dependencies
- Works everywhere Python works
- Well-documented
- Widely used in production

**Cons**:
- Manual type declarations
- No static type checking
- Pointer management is manual

### Alternative: cffi

```python
from cffi import FFI
ffi = FFI()

ffi.cdef("""
    typedef struct Thing Thing;
    Thing* thing_new(void);
    char* thing_get_name(Thing* thing);
    int32_t cimpl_free(void* ptr);
""")

lib = ffi.dlopen("./target/release/libmylib.dylib")
```

**Pros**:
- Cleaner than ctypes
- Better performance
- Can use C header directly

**Cons**:
- External dependency
- Slightly more setup

**Use case**: When you want better performance or cleaner code than ctypes.

## Lua / LuaJIT

### ✅ Recommended: LuaJIT FFI (built-in)

**Status**: Part of LuaJIT, fast, stable

```lua
local ffi = require("ffi")

-- Declare C interface
ffi.cdef[[
    typedef struct Thing Thing;
    Thing* thing_new(void);
    char* thing_get_name(Thing* thing);
    int32_t cimpl_free(void* ptr);
]]

-- Load library
local lib = ffi.load("./target/release/libmylib.dylib")

-- Wrap in Lua metatable
local Thing = {}
Thing.__index = Thing

function Thing.new()
    local handle = lib.thing_new()
    if handle == nil then
        error("Failed to create Thing")
    end
    return setmetatable({ _handle = handle, _freed = false }, Thing)
end

function Thing:get_name()
    if self._freed then
        error("Thing has been freed")
    end
    local name = lib.thing_get_name(self._handle)
    return name ~= nil and ffi.string(name) or nil
end

function Thing:free()
    if not self._freed then
        lib.cimpl_free(ffi.cast("void*", self._handle))
        self._freed = true
    end
end

return Thing
```

**Pros**:
- Built into LuaJIT (no dependencies)
- Extremely fast
- Clean syntax
- Well-documented

**Cons**:
- Only works with LuaJIT (not standard Lua)

**Note**: Standard Lua 5.x doesn't have built-in FFI. Use LuaJIT for FFI bindings.

## Ruby

### ✅ Recommended: ffi gem

```ruby
require 'ffi'

module MyLib
  extend FFI::Library
  ffi_lib './target/release/libmylib.dylib'
  
  attach_function :thing_new, [], :pointer
  attach_function :thing_get_name, [:pointer], :string
  attach_function :cimpl_free, [:pointer], :int
end

class Thing
  def initialize(handle)
    @handle = handle
    @freed = false
  end
  
  def self.new
    handle = MyLib.thing_new
    raise "Failed to create Thing" if handle.null?
    new(handle)
  end
  
  def get_name
    raise "Thing has been freed" if @freed
    MyLib.thing_get_name(@handle)
  end
  
  def free
    unless @freed
      MyLib.cimpl_free(@handle)
      @freed = true
    end
  end
  
  def finalize
    free
  end
end
```

**Pros**:
- Mature, stable gem
- Clean Ruby-esque API
- Widely used

**Installation**:
```bash
gem install ffi
```

## C# / .NET

### ✅ Recommended: P/Invoke (built-in)

```csharp
using System;
using System.Runtime.InteropServices;

public class Thing : IDisposable
{
    private IntPtr _handle;
    private bool _disposed = false;
    
    [DllImport("mylib", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr thing_new();
    
    [DllImport("mylib", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr thing_get_name(IntPtr thing);
    
    [DllImport("mylib", CallingConvention = CallingConvention.Cdecl)]
    private static extern int cimpl_free(IntPtr ptr);
    
    private Thing(IntPtr handle)
    {
        _handle = handle;
    }
    
    public static Thing New()
    {
        var handle = thing_new();
        if (handle == IntPtr.Zero)
            throw new Exception("Failed to create Thing");
        return new Thing(handle);
    }
    
    public string GetName()
    {
        if (_disposed)
            throw new ObjectDisposedException("Thing");
        var ptr = thing_get_name(_handle);
        return Marshal.PtrToStringAnsi(ptr);
    }
    
    public void Dispose()
    {
        if (!_disposed)
        {
            cimpl_free(_handle);
            _disposed = true;
        }
    }
}
```

**Pros**:
- Built into .NET
- No external dependencies
- Integrates with C# idioms (`IDisposable`)

**Cons**:
- More verbose than some alternatives
- Platform-specific library loading

## Go

### ✅ Recommended: cgo (built-in)

```go
package mylib

/*
#cgo LDFLAGS: -L./target/release -lmylib
#include "mylib.h"
*/
import "C"
import "unsafe"
import "errors"

type Thing struct {
    handle *C.Thing
}

func NewThing() (*Thing, error) {
    handle := C.thing_new()
    if handle == nil {
        return nil, errors.New("failed to create thing")
    }
    return &Thing{handle: handle}, nil
}

func (t *Thing) GetName() string {
    cstr := C.thing_get_name(t.handle)
    if cstr == nil {
        return ""
    }
    return C.GoString(cstr)
}

func (t *Thing) Free() {
    if t.handle != nil {
        C.cimpl_free(unsafe.Pointer(t.handle))
        t.handle = nil
    }
}
```

**Pros**:
- Built into Go
- Type-safe
- Integrates well with Go idioms

**Cons**:
- Requires CGO_ENABLED=1
- Cross-compilation can be tricky
- Slower builds

## Java

### ✅ Recommended: JNA

```java
import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;

public interface MyLibrary extends Library {
    MyLibrary INSTANCE = Native.load("mylib", MyLibrary.class);
    
    Pointer thing_new();
    String thing_get_name(Pointer thing);
    int cimpl_free(Pointer ptr);
}

public class Thing implements AutoCloseable {
    private Pointer handle;
    private boolean closed = false;
    
    private Thing(Pointer handle) {
        this.handle = handle;
    }
    
    public static Thing create() {
        Pointer handle = MyLibrary.INSTANCE.thing_new();
        if (handle == null) {
            throw new RuntimeException("Failed to create Thing");
        }
        return new Thing(handle);
    }
    
    public String getName() {
        if (closed) {
            throw new IllegalStateException("Thing has been closed");
        }
        return MyLibrary.INSTANCE.thing_get_name(handle);
    }
    
    @Override
    public void close() {
        if (!closed) {
            MyLibrary.INSTANCE.cimpl_free(handle);
            closed = true;
        }
    }
}
```

**Installation**:
```xml
<dependency>
    <groupId>net.java.dev.jna</groupId>
    <artifactId>jna</artifactId>
    <version>5.13.0</version>
</dependency>
```

## Swift

### ✅ Recommended: C Interop (built-in)

Swift can import C headers directly:

```swift
// Import the generated C header
import mylib

class Thing {
    private var handle: OpaquePointer?
    
    init?() {
        handle = thing_new()
        guard handle != nil else {
            return nil
        }
    }
    
    var name: String? {
        guard let handle = handle else { return nil }
        guard let cstr = thing_get_name(handle) else { return nil }
        return String(cString: cstr)
    }
    
    deinit {
        if let handle = handle {
            cimpl_free(handle)
        }
    }
}
```

**Pros**:
- Native Swift integration
- No external tools needed
- Type-safe

**Use case**: iOS/macOS applications

## Common Patterns Across Languages

### Error Handling

All languages should map C error codes to native exceptions:

```python
# Python
ERROR_CODES = {
    0: None,  # OK
    1: NullParameterError,
    3: InvalidHandleError,
    100: ParseError,
}

def check_error():
    code = lib.error_code()
    if code != 0:
        msg = lib.last_error().decode('utf-8')
        lib.clear_error()
        raise ERROR_CODES.get(code, RuntimeError)(msg)
```

### Memory Management

All languages should:
1. Store the handle/pointer in the wrapper object
2. Track whether it's been freed
3. Free in destructor/finalizer
4. Provide explicit `free()` method
5. Throw if used after free

### String Handling

Most FFI libraries auto-handle C strings, but remember:
- Strings returned from Rust must be freed with `cimpl_free()`
- Strings passed to Rust are borrowed (Rust doesn't free them)

## Platform-Specific Notes

### Library Loading

**macOS**:
```
./target/release/libmylib.dylib
```

**Linux**:
```
./target/release/libmylib.so
```

**Windows**:
```
./target/release/mylib.dll
```

Most FFI libraries handle this automatically, but you may need:

```python
import platform
ext = {'Darwin': '.dylib', 'Linux': '.so', 'Windows': '.dll'}[platform.system()]
lib = CDLL(f"./target/release/libmylib{ext}")
```

### Path Issues

If the library isn't found:
- Use absolute paths
- Set `LD_LIBRARY_PATH` (Linux) or `DYLD_LIBRARY_PATH` (macOS)
- Copy library to system library directory
- On macOS, use `install_name_tool` to fix library paths

## Testing Strategy

For each language binding:

1. **Basic functionality**: Create, use, destroy objects
2. **Error handling**: Trigger errors, verify exceptions
3. **Memory management**: Verify no leaks, test explicit free
4. **Edge cases**: NULL handling, empty strings, large data
5. **Concurrent access**: If applicable, test thread safety

## Performance Considerations

FFI calls have overhead:
- **Typical overhead**: 5-50 nanoseconds per call
- **String conversion**: 10-100 nanoseconds
- **Object allocation**: 50-500 nanoseconds

For hot paths (millions of calls/sec), consider:
- Batching operations
- Using napi-rs (Node.js) or pyo3 (Python) for zero-copy bindings
- Keeping data in Rust longer before crossing the FFI boundary

## AI Generation

The consistent `cimpl` C API makes it easy for AI to generate bindings:

**Prompt template**:
```
Generate Python bindings for this C header using ctypes.
The library follows these conventions:
- Functions return NULL or -1 on error
- Call error_code() to get error type
- Call last_error() to get error message
- Free all returned pointers with cimpl_free()
- Create Python exception classes for each error code
```

Point the AI to the generated header file and it can create idiomatic bindings.

## Troubleshooting

### "Symbol not found"
- Check library loaded correctly
- Verify function name matches (use `nm` on macOS/Linux, `dumpbin` on Windows)
- Check for name mangling (should have `#[no_mangle]`)

### Segmentation Fault
- Null pointer passed to Rust (use `ptr_or_return_*!` macros)
- Double-free (use `cimpl_free` only once per pointer)
- Type mismatch (pointer validated as wrong type)

### Memory Leak
- Not calling `cimpl_free()` on returned pointers
- Check with: `lib.get_allocations()` (if exposed)

### "Invalid pointer"
- Passing stack-allocated data to Rust
- Pointer already freed
- Wrong library instance (multiple loads)

## Conclusion

The `cimpl` approach makes language bindings straightforward:
1. The C API is consistent and well-documented
2. Each language has mature FFI libraries
3. Patterns are similar across languages
4. AI can generate bindings from the C header

**Key insight**: When one language's FFI tool has issues (like `ffi-napi`), just switch to a better tool. The C API is universal and stable.
