# Macro Simplification

## Summary

Simplified the `cimple` FFI macro API by removing redundant `guard_*` macros. All pointer validation now uses the `deref_or_return_*` family.

## What Changed

### Removed
- `guard_or_return!()` and all variants (`_null`, `_neg`, `_zero`, `_false`)
- `guard_mut_or_return!()` and all variants

### Why?
The `guard_*` macros were functionally identical to `deref_or_return_*` except for creating a named binding. Since Rust lets you bind any expression result with `let`, the guard macros were redundant:

```rust
// Old "guard" way
guard_mut_or_return_neg!(ptr, MyString, obj);
obj.set_value(value);

// New simplified way (same result)
let obj = deref_mut_or_return_neg!(ptr, MyString);
obj.set_value(value);
```

The only difference is typing `let obj = `, which is:
- More explicit about what's happening
- Standard Rust pattern
- Fewer macros to learn

## Current API

### For immutable access:
```rust
// One-liner
deref_or_return_zero!(ptr, MyString).len()

// Multi-line (bind to variable)
let obj = deref_or_return_null!(ptr, MyString);
obj.do_thing1();
obj.do_thing2();
```

### For mutable access:
```rust
// One-liner (rare)
deref_mut_or_return_neg!(ptr, MyString).clear();

// Multi-line (common)
let obj = deref_mut_or_return_neg!(ptr, MyString);
obj.set_value(new_value);
```

## Benefits

1. **Simpler API** - 8 fewer macros to learn
2. **More explicit** - `let obj =` makes binding clear
3. **Standard Rust** - Uses normal variable binding
4. **Same safety** - All validation still happens

## Complete Macro List

```
ptr_or_return_*           Check NULL only
cstr_or_return_*          Convert C string
deref_or_return_*         Validate + return &T
deref_mut_or_return_*     Validate + return &mut T
```

Each with suffixes: `_null`, `_neg`, `_zero`, `_false`, or custom value.

## Migration

If you were using `guard_*` macros:

**Before:**
```rust
guard_or_return_null!(ptr, MyType, obj);
obj.method();
```

**After:**
```rust
let obj = deref_or_return_null!(ptr, MyType);
obj.method();
```

Just add `let obj = ` - that's it!

## Testing

- ✅ All tests pass
- ✅ C example works
- ✅ No functional changes, just API simplification
