# Cimpl Reference Example - Secret Message Processor

This is a **comprehensive reference implementation** that exercises ALL cimpl patterns. Use this as the canonical example when creating new FFI bindings.

## Purpose

This example is designed to systematically test every cimpl pattern without relying on external crates. It's self-contained and demonstrates:

- ✅ String parameters (C → Rust)
- ✅ String results (Rust → C)
- ✅ Byte arrays in/out
- ✅ Result<T, E> with custom error enum
- ✅ Option<T> for validation
- ✅ Struct lifecycle (create, modify, query, destroy)
- ✅ Numeric parameters and returns
- ✅ Boolean returns
- ✅ Error handling (last_error, clear_error)
- ✅ Memory management (tracked allocations)

## Domain: Secret Messages

We encode/decode "secret messages" with various silly transformations:
- **ROT13** - Classic letter rotation cipher
- **Reverse** - Reverse the string
- **Vowel removal** - Remove all vowels
- **Character substitution** - Replace characters
- **Hex encoding/decoding** - Convert to/from hex
- **Statistics** - Count characters, words, vowels, etc.

## API Reference

### Pattern Testing Matrix

Each function is designed to test specific cimpl patterns:

| Function | Tests | Return Type | Can Fail? |
|----------|-------|-------------|-----------|
| `secret_rot13` | `cstr_or_return_null!`, `to_c_string!` | String | No |
| `secret_reverse` | `cstr_or_return_null!`, `to_c_string!` | String | No |
| `secret_remove_vowels` | `cstr_or_return_null!`, `to_c_string!` | String | No |
| `secret_substitute` | `cstr_or_return_null!`, char params | String | No |
| `secret_uppercase` | `cstr_or_return_null!`, `to_c_string!` | String | No |
| `secret_to_hex` | `cstr_or_return_null!`, `to_c_string!` | String | No |
| `secret_from_hex` | `ok_or_return_null!` with `SecretError` | String | **Yes** - InvalidHex |
| `secret_validate_length` | `ok_or_return_false!` with `SecretError` | bool | **Yes** - TooShort/TooLong |
| `secret_is_ascii` | `cstr_or_return_false!`, validation | bool | No |
| `secret_is_valid_hex` | `cstr_or_return_false!`, validation | bool | No |
| `secret_count_chars` | `cstr_or_return_zero!` | usize | No |
| `secret_count_vowels` | `cstr_or_return_zero!` | usize | No |
| `secret_count_consonants` | `cstr_or_return_zero!` | usize | No |
| `secret_count_words` | `cstr_or_return_zero!` | usize | No |
| `secret_to_bytes` | `to_c_bytes!`, byte array out | bytes | No |
| `secret_from_bytes` | `ok_or_return_null!`, byte array in | String | **Yes** - InvalidFormat |
| `message_new` | `box_tracked!`, struct creation | Struct* | No |
| `message_get_content` | `deref_or_return_null!` | String | No |
| `message_get_encoding` | `deref_or_return_null!` | String | No |
| `message_set_metadata` | `deref_or_return_false!`, mutation | bool | No |
| `message_get_metadata` | `option_to_c_string!` | String? | Returns NULL if not found |
| `message_get_stats` | `deref_or_return_null!`, struct return | Struct* | No |

### Error Codes

```c
typedef enum SecretError {
    SECRET_ERROR_OK = 0,                    // No error
    SECRET_ERROR_NULL_PARAMETER = 1,        // NULL pointer passed
    SECRET_ERROR_STRING_TOO_LONG = 2,       // String exceeds max
    SECRET_ERROR_INVALID_HANDLE = 3,        // Invalid object pointer
    SECRET_ERROR_WRONG_HANDLE_TYPE = 4,     // Wrong type
    SECRET_ERROR_OTHER = 5,                 // Generic error
    
    // Library-specific errors (100+)
    SECRET_ERROR_INVALID_HEX = 100,         // Invalid hex string
    SECRET_ERROR_INVALID_FORMAT = 101,      // Format error
    SECRET_ERROR_TOO_SHORT = 102,           // Message too short
    SECRET_ERROR_TOO_LONG = 103,            // Message too long
} SecretError;
```

## Building

```bash
cargo build --release
```

The C header will be generated at `include/secret.h`.

## Example Usage (C)

```c
#include "include/secret.h"
#include <stdio.h>
#include <assert.h>

int main() {
    // Simple encoding (can't fail)
    char* encoded = secret_rot13("Hello World");
    printf("ROT13: %s\n", encoded);
    secret_free(encoded);
    
    // Hex encoding (can't fail)
    char* hex = secret_to_hex("secret");
    printf("Hex: %s\n", hex);
    
    // Hex decoding (can fail!)
    char* decoded = secret_from_hex(hex);
    if (decoded == NULL) {
        int code = secret_error_code();
        char* msg = secret_last_error();
        printf("Error %d: %s\n", code, msg);
        secret_free(msg);
        return 1;
    }
    printf("Decoded: %s\n", decoded);
    secret_free(hex);
    secret_free(decoded);
    
    // Validation
    bool valid = secret_validate_length("test", 1, 10);
    printf("Valid: %s\n", valid ? "yes" : "no");
    
    // Counting
    size_t vowels = secret_count_vowels("hello");
    printf("Vowels: %zu\n", vowels);
    
    // Struct operations
    SecretMessage* msg = message_new("Hello", "ROT13");
    assert(msg != NULL);
    
    message_set_metadata(msg, "author", "alice");
    char* author = message_get_metadata(msg, "author");
    printf("Author: %s\n", author);
    secret_free(author);
    
    MessageStats* stats = message_get_stats(msg);
    printf("Length: %zu, Words: %zu\n", stats->length, stats->word_count);
    secret_free(stats);
    secret_free(msg);
    
    return 0;
}
```

## What This Tests

### 1. String Input Validation

Every function that takes a C string uses `cstr_or_return_*!` macros to safely convert and validate input.

### 2. Error Propagation

Functions like `secret_from_hex()` demonstrate:
- Result<T, E> handling with `ok_or_return_null!`
- Custom error enum (`SecretError`)
- Error trait implementation (`CimplError`)
- Error conversion (`From<ProcessError>`)

### 3. Option Handling

`message_get_metadata()` demonstrates:
- Returning NULL for missing values
- Using `option_to_c_string!` macro

### 4. Byte Arrays

`secret_to_bytes()` and `secret_from_bytes()` demonstrate:
- Converting strings to byte arrays
- Safe byte array handling
- Using `to_c_bytes!` macro

### 5. Struct Lifecycle

The `SecretMessage` type demonstrates:
- Opaque pointer pattern
- Constructor (`message_new`)
- Accessors (`message_get_*`)
- Mutators (`message_set_*`)
- Destructor (via `secret_free`)

### 6. Multiple Return Types

- Pointer returns: NULL on error
- Boolean returns: false on error
- Numeric returns: 0 on error

## Language Bindings

See the `bindings/` directory for examples in:
- C
- Python
- C++

Each binding demonstrates idiomatic error handling for that language.
