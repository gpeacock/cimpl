# Pointer Registry Redesign

## Overview

The `cimple` FFI library has been redesigned to use a cleaner, more flexible pointer-based API instead of the previous handle-based approach. This provides:

1. **Mandatory type validation** for all pointers passed from Rust to C
2. **Optional overhead** - users choose their wrapper type (Box, Arc, Arc<Mutex>)
3. **Universal `cimple_free()`** - one function to free any tracked pointer

## Key Changes

### 1. Pointer Registry with Cleanup Functions

Instead of forcing `Arc<Mutex<Box<dyn Any>>>` for every handle, we now track pointers with their cleanup functions:

```rust
pub struct PointerRegistry {
    tracked: Mutex<HashMap<usize, (TypeId, CleanupFn)>>,
}
```

Each tracked pointer stores:
- **TypeId**: For mandatory type validation
- **CleanupFn**: The correct destructor (Box::from_raw, Arc::from_raw, etc.)

### 2. Tracking Functions for Different Wrapper Types

Users can now choose the appropriate wrapper for their use case:

```rust
// Simple ownership - lowest overhead
let ptr = Box::into_raw(Box::new(value));
track_box(ptr);

// Shared ownership
let ptr = Arc::into_raw(Arc::new(value)) as *mut_;
track_arc(ptr);

// Shared ownership with interior mutability
let ptr = Arc::into_raw(Arc::new(Mutex::new(value)));
track_arc_mutex(ptr);
```

Helper macros make this even easier:

```rust
// One-liner: allocate, track, and return
let ptr = box_tracked!(MyStruct::new());
let ptr = arc_tracked!(MyStruct::new());
```

### 3. Universal Free Function

A single function that frees ANY tracked pointer:

```rust
#[no_mangle]
pub extern "C" fn cimple_free(ptr: *mut c_void) -> i32 {
    match free_tracked_pointer(ptr as *mut u8) {
        Ok(()) => 0,
        Err(e) => {
            e.set_last();
            -1
        }
    }
}
```

From C, it's beautifully simple:

```c
MyString* str = mystring_create("hello");
char* value = mystring_get_value(str);

cimple_free(value);  // Free the string
cimple_free(str);    // Free the MyString - same function!
```

### 4. Direct Pointer Types

Instead of opaque "handles", we now export the actual struct type:

```rust
// Rust side - struct is pub, but fields are private
pub struct MyString {
    value: String,
}

// C side - receives real pointers to opaque type
MyString* str = mystring_create("hello");
```

The type is tracked in the registry, so validation ensures C can't pass the wrong pointer type.

### 5. New Validation Macros

Simpler macros that work with direct pointers:

```rust
// Validate and dereference immutably
let obj = validate_and_deref!(ptr, MyString);

// Validate and dereference mutably
let obj = validate_and_deref_mut_neg!(ptr, MyString);
```

## Example Usage

### Rust FFI Function

```rust
#[no_mangle]
pub extern "C" fn mystring_create(initial: *const c_char) -> *mut MyString {
    let initial_str = cstr_or_return_null!(initial);
    box_tracked!(MyString::new(initial_str))  // Allocate, track, return
}

#[no_mangle]
pub extern "C" fn mystring_set_value(ptr: *mut MyString, new_value: *const c_char) -> i32 {
    let obj = validate_and_deref_mut_neg!(ptr, MyString);  // Validate type
    let new_value_str = cstr_or_return_int!(new_value);
    obj.set_value(new_value_str);
    0
}
```

### C Usage

```c
MyString* str = mystring_create("Hello");
if (str == NULL) {
    fprintf(stderr, "Error: %s\n", mystring_last_error());
    return -1;
}

mystring_set_value(str, "Goodbye");
char* value = mystring_get_value(str);

printf("%s\n", value);

// Universal free for everything
cimple_free(value);
cimple_free(str);
```

## Benefits

1. **Cleaner API**: Direct pointers instead of opaque handles
2. **Type Safety**: Mandatory validation prevents wrong-type errors
3. **Flexibility**: Users choose the wrapper (Box, Arc, Arc<Mutex>)
4. **Simplicity**: One `cimple_free()` function for everything
5. **AI-Friendly**: Clear, simple C API that's easy to bind from other languages

## Migration from Handle-Based API

Old handle-based code:

```rust
let handle = cimple::get_handles().insert(my_value);
cimple::handle_to_ptr::<MyType>(handle)

// Later...
guard_handle_or_null!(handle, MyType, obj);
free_handle!(handle, MyType)
```

New pointer-based code:

```rust
box_tracked!(my_value)

// Later...
validate_and_deref!(ptr, MyType)
cimple_free(ptr)  // Universal!
```

## Technical Details

### Thread Safety

- The `PointerRegistry` uses `Mutex<HashMap>` for thread-safe tracking
- Cleanup functions are `Box<dyn FnMut() + Send>` for cross-thread safety
- Pointers are stored as `usize` in closures to satisfy `Send` bounds

### Double-Free Protection

Attempting to free the same pointer twice returns an error:

```c
cimple_free(ptr);  // OK
cimple_free(ptr);  // Returns -1, sets error
```

### Leak Detection

On shutdown, any pointers not freed will trigger a warning:

```
⚠️  WARNING: 2 pointer(s) were not freed at shutdown!
This indicates C code did not properly free all allocated pointers.
Each pointer should be freed exactly once with cimple_free().
```

## Conclusion

This redesign provides the best of both worlds:
- **Safety**: Type validation and double-free protection
- **Flexibility**: Users control the overhead
- **Simplicity**: Universal free function and clear API

The resulting C API is clean, conventional, and perfect for AI-generated bindings to other languages.
