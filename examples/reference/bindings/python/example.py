#!/usr/bin/env python3
"""
Example usage of the ValueConverter Python bindings.

This demonstrates:
- Multiple constructor patterns
- Type conversions (with error handling)
- Context managers for automatic cleanup
- Exception handling for invalid conversions
"""

from value_converter import (
    ValueConverter,
    OutOfRangeError,
    InvalidUtf8Error,
    InvalidHexError,
    BufferTooLargeError,
)


def example_basic_conversions():
    """Basic type conversions"""
    print("=== Basic Conversions ===")
    
    # Integer to hex
    with ValueConverter.from_i32(42) as vc:
        print(f"42 as hex: {vc.to_hex()}")
        print(f"42 as bytes: {vc.to_bytes()}")
    
    # String to hex
    with ValueConverter.from_string("Hi") as vc:
        print(f"'Hi' as hex: {vc.to_hex()}")
        print(f"Length: {len(vc)} bytes")
    
    # Hex to string
    with ValueConverter.from_hex("48656c6c6f") as vc:
        print(f"Hex decoded: {vc.to_string()}")
    
    print()


def example_roundtrip_conversions():
    """Demonstrate roundtrip conversions"""
    print("=== Roundtrip Conversions ===")
    
    # i32 -> bytes -> hex -> bytes -> i32
    original = -12345
    with ValueConverter.from_i32(original) as vc:
        hex_str = vc.to_hex()
        print(f"Original i32: {original}")
        print(f"As hex: {hex_str}")
    
    with ValueConverter.from_hex(hex_str) as vc:
        result = vc.to_i32()
        print(f"Back to i32: {result}")
        assert result == original, "Roundtrip failed!"
    
    # String -> bytes -> hex -> bytes -> string
    original_str = "Test"
    with ValueConverter.from_string(original_str) as vc:
        hex_str = vc.to_hex()
        print(f"\nOriginal string: '{original_str}'")
        print(f"As hex: {hex_str}")
    
    with ValueConverter.from_hex(hex_str) as vc:
        result_str = vc.to_string()
        print(f"Back to string: '{result_str}'")
        assert result_str == original_str, "Roundtrip failed!"
    
    print()


def example_error_handling():
    """Demonstrate error handling"""
    print("=== Error Handling ===")
    
    # Wrong size for i32 (needs exactly 4 bytes)
    try:
        with ValueConverter.from_string("Hi") as vc:  # Only 2 bytes
            value = vc.to_i32()  # Needs 4 bytes
    except OutOfRangeError as e:
        print(f"✓ Caught OutOfRangeError: {e}")
    
    # Invalid UTF-8
    try:
        with ValueConverter.from_bytes(b'\xff\xfe') as vc:
            text = vc.to_string()
    except InvalidUtf8Error as e:
        print(f"✓ Caught InvalidUtf8Error: {e}")
    
    # Invalid hex
    try:
        vc = ValueConverter.from_hex("ZZZZ")  # Not valid hex
    except InvalidHexError as e:
        print(f"✓ Caught InvalidHexError: {e}")
    
    # Buffer too large
    try:
        vc = ValueConverter.from_string("123456789")  # More than 8 bytes
    except BufferTooLargeError as e:
        print(f"✓ Caught BufferTooLargeError: {e}")
    
    print()


def example_different_types():
    """Show conversions between different integer types"""
    print("=== Different Integer Types ===")
    
    # u32
    with ValueConverter.from_u32(0xDEADBEEF) as vc:
        print(f"u32 0xDEADBEEF as hex: {vc.to_hex()}")
        print(f"Back to u32: 0x{vc.to_u32():08X}")
    
    # i64
    with ValueConverter.from_i64(-9223372036854775808) as vc:
        print(f"\ni64 min value as hex: {vc.to_hex()}")
        print(f"Back to i64: {vc.to_i64()}")
    
    # u64
    with ValueConverter.from_u64(18446744073709551615) as vc:
        print(f"\nu64 max value as hex: {vc.to_hex()}")
        print(f"Back to u64: {vc.to_u64()}")
    
    print()


def example_properties():
    """Show utility properties"""
    print("=== Properties ===")
    
    print(f"Maximum buffer size: {ValueConverter.max_size()} bytes")
    
    with ValueConverter.from_string("Test") as vc:
        print(f"'Test' length: {len(vc)} bytes")
        print(f"Representation: {repr(vc)}")
    
    print()


def main():
    """Run all examples"""
    print("ValueConverter Python Bindings - Examples\n")
    
    example_basic_conversions()
    example_roundtrip_conversions()
    example_error_handling()
    example_different_types()
    example_properties()
    
    print("✓ All examples completed successfully!")


if __name__ == "__main__":
    main()
