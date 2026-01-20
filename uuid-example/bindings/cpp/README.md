# C++ Bindings for cimpl-uuid

Modern C++ wrapper for the cimpl-uuid library, demonstrating best practices for wrapping C APIs in idiomatic C++.

## Features

This C++ binding showcases:

- ✅ **RAII** - Automatic memory management (no manual `free()` calls)
- ✅ **Exception handling** - Converts C error codes to C++ exceptions
- ✅ **Smart pointers** - Uses `unique_ptr` with custom deleter
- ✅ **Move semantics** - Efficient resource transfer
- ✅ **Copy semantics** - Deep copying with value semantics
- ✅ **Operator overloading** - Natural C++ syntax (`==`, `<`, `<<`, etc.)
- ✅ **Stream output** - Works with `std::cout`
- ✅ **STL integration** - Works with `std::sort`, `std::vector`, etc.
- ✅ **C++20 spaceship operator** - Three-way comparison (`<=>`)
- ✅ **Type safety** - Strong typing with no raw pointers exposed

## Quick Start

```bash
# Build the Rust library first (from uuid-example root)
cd ../..
cargo build --release
cd bindings/cpp

# Build and run the example
make run
```

## Usage Example

```cpp
#include "uuid.hpp"
#include <iostream>
#include <vector>
#include <algorithm>

using namespace cimpl;

int main() {
    try {
        // Generate a random UUID (v4)
        auto uuid = Uuid::newV4();
        std::cout << "Generated: " << uuid << "\n";
        
        // Parse a UUID from string
        auto parsed = Uuid::parse("550e8400-e29b-41d4-a716-446655440000");
        std::cout << "Parsed: " << parsed << "\n";
        
        // Compare UUIDs
        if (uuid == parsed) {
            std::cout << "UUIDs are equal\n";
        }
        
        // Sort UUIDs
        std::vector<Uuid> uuids;
        uuids.push_back(Uuid::newV4());
        uuids.push_back(Uuid::newV4());
        uuids.push_back(Uuid::newV4());
        std::sort(uuids.begin(), uuids.end());
        
        // Print sorted UUIDs
        for (const auto& u : uuids) {
            std::cout << u << "\n";
        }
        
        // Memory is automatically freed (RAII)
        
    } catch (const UuidError& e) {
        std::cerr << "Error: " << e.what() << "\n";
        std::cerr << "Code: " << e.code() << "\n";
        return 1;
    }
    
    return 0;
}
```

## API Reference

### Factory Methods (Static)

```cpp
auto uuid = Uuid::newV4();           // Random UUID (version 4)
auto uuid = Uuid::newV7();           // Timestamp-based UUID (version 7)
auto uuid = Uuid::parse("...");      // Parse from string
auto uuid = Uuid::nil();             // All zeros
auto uuid = Uuid::max();             // All ones
```

### Conversions

```cpp
std::string str = uuid.toString();   // "550e8400-e29b-41d4-a716-446655440000"
std::string urn = uuid.toUrn();      // "urn:uuid:550e8400-..."
auto bytes = uuid.asBytes();         // std::vector<uint8_t> (16 bytes)
```

### Predicates

```cpp
bool is_nil = uuid.isNil();          // Check if all zeros
bool is_max = uuid.isMax();          // Check if all ones
```

### Comparison

```cpp
bool equal = (uuid1 == uuid2);       // Equality
bool less = (uuid1 < uuid2);         // Less than
bool greater = (uuid1 > uuid2);      // Greater than
// Also: !=, <=, >=, and <=> (C++20 spaceship)
```

### Stream Output

```cpp
std::cout << uuid << "\n";           // Print to stream
```

### Copy and Move

```cpp
Uuid copy(uuid);                     // Copy constructor (deep copy)
Uuid moved = std::move(uuid);        // Move constructor (transfer)
```

## Memory Management

The C++ wrapper uses RAII, so you never need to manually free memory:

```cpp
{
    auto uuid = Uuid::newV4();       // Allocates
    std::cout << uuid << "\n";
    // uuid automatically freed here when it goes out of scope
}
```

Compare this to the C API:

```c
// C API requires manual memory management
Uuid* uuid = uuid_new_v4();
char* str = uuid_to_string(uuid);
printf("%s\n", str);
uuid_free(str);                      // Must remember to free!
uuid_free(uuid);                     // Must remember to free!
```

## Error Handling

The C++ wrapper converts C error codes to exceptions:

```cpp
try {
    auto uuid = Uuid::parse("invalid-uuid");
} catch (const UuidError& e) {
    std::cerr << "Error: " << e.what() << "\n";  // Error message
    std::cerr << "Code: " << e.code() << "\n";   // Error code
}
```

Compare this to the C API:

```c
// C API uses error codes
Uuid* uuid = uuid_parse("invalid-uuid");
if (uuid == NULL) {
    int32_t code = uuid_error_code();
    char* msg = uuid_last_error();
    fprintf(stderr, "Error %d: %s\n", code, msg);
    uuid_free(msg);
}
```

## Design Patterns

### RAII (Resource Acquisition Is Initialization)

The `Uuid` class uses `std::unique_ptr` with a custom deleter to automatically free the C UUID when the C++ object is destroyed:

```cpp
struct UuidDeleter {
    void operator()(::Uuid* ptr) const noexcept {
        if (ptr) ::uuid_free(ptr);
    }
};

std::unique_ptr<::Uuid, UuidDeleter> ptr_;
```

### Exception Safety

All operations that can fail throw exceptions rather than returning error codes:

```cpp
// Strong exception guarantee - if this throws, no resources are leaked
auto uuid = Uuid::parse("...");
```

### Value Semantics

UUIDs can be copied and moved like regular values:

```cpp
std::vector<Uuid> uuids;
uuids.push_back(Uuid::newV4());      // Move
Uuid copy = uuids[0];                // Copy
```

## Building Your Project

### Using the Wrapper

Include the header and link against the C library:

```cpp
// your_project.cpp
#include "uuid.hpp"

int main() {
    auto uuid = cimpl::Uuid::newV4();
    std::cout << uuid << "\n";
}
```

Build:

```bash
g++ -std=c++20 -I/path/to/cimpl-uuid/include \
    your_project.cpp uuid.cpp \
    -L/path/to/cimpl-uuid/target/release -lcimpl_uuid
```

### CMake Example

```cmake
cmake_minimum_required(VERSION 3.15)
project(my_project)

set(CMAKE_CXX_STANDARD 20)

# Find the cimpl-uuid library
find_library(CIMPL_UUID cimpl_uuid 
    HINTS ${CMAKE_SOURCE_DIR}/../target/release)

add_executable(my_app 
    main.cpp
    uuid.cpp)

target_include_directories(my_app PRIVATE 
    ${CMAKE_SOURCE_DIR}/../include)

target_link_libraries(my_app ${CIMPL_UUID})
```

## Requirements

- **C++20 compiler** (for spaceship operator `<=>`)
  - GCC 10+ or Clang 10+ or MSVC 2019+
  - Can be adapted to C++17 by removing `operator<=>`
- **Rust toolchain** (to build the underlying C library)
- **cimpl-uuid C library** (built with `cargo build --release`)

## Why This Binding Exists

This binding demonstrates how to wrap a cimpl-generated C API in modern, idiomatic C++. It's a reference implementation showing:

1. How to make C APIs feel natural in C++
2. How to provide memory safety through RAII
3. How to use exceptions instead of error codes
4. How to integrate with the C++ Standard Library

This is **the right way** to use a C library from C++ - unlike JavaScript where WASM is better, C++ developers commonly wrap C APIs, and this is the standard approach.

## Comparison with C API

| Feature | C API | C++ Wrapper |
|---------|-------|-------------|
| Memory management | Manual (`uuid_free`) | Automatic (RAII) |
| Error handling | Error codes | Exceptions |
| Type safety | Opaque pointers | Strong types |
| Comparison | Functions | Operators |
| Copying | Manual | Copy constructor |
| Sorting | Custom comparator | `std::sort` |

## License

Same as cimpl-uuid (MIT)

## See Also

- [C API Reference](../../include/cimpl_uuid.h)
- [cimpl Framework](../../../README.md)
- [C Example](../c/example.c)
