#!/usr/bin/env python3
"""
Example demonstrating the UUID Python bindings.

This script shows various UUID operations using the Python wrapper
around the Rust UUID library via cimpl FFI.
"""

from uuid_ffi import Uuid, ParseError, UuidError


def main():
    print("=" * 60)
    print("UUID Python Bindings Example")
    print("=" * 60)
    print()
    
    # Generate random UUIDs
    print("1. Generating random UUIDs (Version 4)")
    print("-" * 60)
    uuid1 = Uuid.new_v4()
    uuid2 = Uuid.new_v4()
    print(f"UUID 1: {uuid1}")
    print(f"UUID 2: {uuid2}")
    print(f"Are they equal? {uuid1 == uuid2}")
    print()
    
    # Parse UUID from string
    print("2. Parsing UUID from string")
    print("-" * 60)
    uuid_str = "550e8400-e29b-41d4-a716-446655440000"
    uuid3 = Uuid.parse(uuid_str)
    print(f"Parsed: {uuid3}")
    print(f"Simple format: {uuid3.to_simple()}")
    print(f"URN format: {uuid3.to_urn()}")
    print()
    
    # Parse different formats
    print("3. Parsing different UUID formats")
    print("-" * 60)
    formats = [
        "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8",  # Hyphenated
        "a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8",      # Simple
        "urn:uuid:A1A2A3A4-B1B2-C1C2-D1D2-D3D4D5D6D7D8",  # URN
        "{a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8}",  # Braced
    ]
    for fmt in formats:
        try:
            uuid = Uuid.parse(fmt)
            print(f"✓ Parsed '{fmt[:30]}...' → {uuid}")
        except ParseError as e:
            print(f"✗ Failed to parse: {e}")
    print()
    
    # Error handling
    print("4. Error handling")
    print("-" * 60)
    try:
        invalid_uuid = Uuid.parse("not-a-uuid")
        print(f"Somehow parsed: {invalid_uuid}")
    except ParseError as e:
        print(f"✓ Caught ParseError: {e}")
    print()
    
    # Special UUIDs
    print("5. Special UUIDs")
    print("-" * 60)
    nil_uuid = Uuid.nil()
    max_uuid = Uuid.max()
    print(f"Nil UUID: {nil_uuid}")
    print(f"Is nil? {nil_uuid.is_nil()}")
    print(f"Max UUID: {max_uuid}")
    print(f"Is max? {max_uuid.is_max()}")
    print()
    
    # Byte operations
    print("6. Byte operations")
    print("-" * 60)
    uuid4 = Uuid.new_v4()
    print(f"UUID: {uuid4}")
    uuid_bytes = uuid4.as_bytes()
    print(f"Bytes: {uuid_bytes.hex()}")
    print(f"Length: {len(uuid_bytes)} bytes")
    
    # Recreate from bytes
    uuid5 = Uuid.from_bytes(uuid_bytes)
    print(f"Recreated: {uuid5}")
    print(f"Equal to original? {uuid4 == uuid5}")
    print()
    
    # Comparison
    print("7. UUID comparison")
    print("-" * 60)
    uuid6 = Uuid.parse("550e8400-e29b-41d4-a716-446655440000")
    uuid7 = Uuid.parse("550e8400-e29b-41d4-a716-446655440000")
    uuid8 = Uuid.new_v4()
    print(f"UUID A: {uuid6}")
    print(f"UUID B: {uuid7}")
    print(f"UUID C: {uuid8}")
    print(f"A == B? {uuid6 == uuid7}")
    print(f"A == C? {uuid6 == uuid8}")
    print()
    
    # Context manager
    print("8. Using context manager (automatic cleanup)")
    print("-" * 60)
    with Uuid.new_v4() as uuid9:
        print(f"UUID in context: {uuid9}")
        print(f"Simple format: {uuid9.to_simple()}")
    print("✓ UUID automatically freed when exiting context")
    print()
    
    # Hash and use in collections
    print("9. Using UUIDs in sets and dicts")
    print("-" * 60)
    uuid_set = {Uuid.new_v4() for _ in range(3)}
    print(f"Created set of {len(uuid_set)} unique UUIDs")
    for i, uuid in enumerate(uuid_set, 1):
        print(f"  {i}. {uuid}")
    
    uuid_dict = {Uuid.nil(): "nil", Uuid.max(): "max"}
    print(f"\nCreated dict with {len(uuid_dict)} entries:")
    for uuid, label in uuid_dict.items():
        print(f"  {label}: {uuid}")
    print()
    
    print("=" * 60)
    print("All tests completed successfully! ✓")
    print("=" * 60)


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        print(f"\n✗ Error: {e}")
        import traceback
        traceback.print_exc()
        exit(1)
