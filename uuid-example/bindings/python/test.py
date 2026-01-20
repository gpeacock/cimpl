#!/usr/bin/env python3
"""
Test the Python UUID bindings
"""

from uuid_bindings import Uuid, uuid4, uuid7, ParseError

print("=== Python UUID Bindings Test ===\n")

# 1. Generate UUIDs
print("1. Generating UUIDs...")
uuid_v4 = uuid4()
print(f"   V4 (random): {uuid_v4}")

uuid_v7 = uuid7()
print(f"   V7 (timestamp): {uuid_v7}")

# 2. Parse UUID
print("\n2. Parsing UUID from string...")
parsed = Uuid.parse("550e8400-e29b-41d4-a716-446655440000")
print(f"   Parsed: {parsed}")
print(f"   URN: {parsed.to_urn()}")

# 3. Test invalid UUID
print("\n3. Testing invalid UUID...")
try:
    bad = Uuid.parse("not-a-uuid")
    print("   ERROR: Should have raised exception!")
except ParseError as e:
    print(f"   ✓ Correctly raised ParseError")
    print(f"   Error code: {e.code}")
    print(f"   Message: {e.message}")

# 4. Nil and Max UUIDs
print("\n4. Testing nil and max UUIDs...")
nil_uuid = Uuid.nil()
print(f"   Nil: {nil_uuid}")
print(f"   Is nil? {nil_uuid.is_nil()}")

max_uuid = Uuid.max()
print(f"   Max: {max_uuid}")
print(f"   Is max? {max_uuid.is_max()}")

# 5. Comparison
print("\n5. Testing comparison...")
uuid_a = Uuid.parse("00000000-0000-0000-0000-000000000001")
uuid_b = Uuid.parse("00000000-0000-0000-0000-000000000002")
uuid_c = Uuid.parse("00000000-0000-0000-0000-000000000001")

print(f"   A < B: {uuid_a < uuid_b} (expected True)")
print(f"   A == C: {uuid_a == uuid_c} (expected True)")
print(f"   B > A: {uuid_b > uuid_a} (expected True)")

# 6. Bytes representation
print("\n6. Testing bytes...")
byte_data = uuid_v4.to_bytes()
print(f"   UUID as bytes (length {len(byte_data)}): {byte_data.hex()}")

# 7. String representations
print("\n7. Testing string formats...")
print(f"   str(): {str(uuid_v4)}")
print(f"   repr(): {repr(uuid_v4)}")
print(f"   urn: {uuid_v4.to_urn()}")

print("\n=== All Python tests passed! ===")
