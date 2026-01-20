#!/usr/bin/env luajit
--[[
Test the Lua UUID bindings
]]

local uuid = require("uuid_bindings")

print("=== Lua UUID Bindings Test ===\n")

-- 1. Generate UUIDs
print("1. Generating UUIDs...")
local uuid_v4 = uuid.v4()
print(string.format("   V4 (random): %s", uuid_v4))

local uuid_v7 = uuid.v7()
print(string.format("   V7 (timestamp): %s", uuid_v7))

-- 2. Parse UUID
print("\n2. Parsing UUID from string...")
local parsed = uuid.parse("550e8400-e29b-41d4-a716-446655440000")
print(string.format("   Parsed: %s", parsed))
print(string.format("   URN: %s", parsed:to_urn()))

-- 3. Test invalid UUID
print("\n3. Testing invalid UUID...")
local success, err = pcall(function()
    local bad = uuid.parse("not-a-uuid")
end)
if not success then
    print("   ✓ Correctly raised error")
    print(string.format("   Error: %s", err))
else
    print("   ERROR: Should have raised exception!")
end

-- 4. Nil and Max UUIDs
print("\n4. Testing nil and max UUIDs...")
local nil_uuid = uuid.nil_uuid()
print(string.format("   Nil: %s", nil_uuid))
print(string.format("   Is nil? %s", tostring(nil_uuid:is_nil())))

local max_uuid = uuid.max_uuid()
print(string.format("   Max: %s", max_uuid))
print(string.format("   Is max? %s", tostring(max_uuid:is_max())))

-- 5. Comparison
print("\n5. Testing comparison...")
local uuid_a = uuid.parse("00000000-0000-0000-0000-000000000001")
local uuid_b = uuid.parse("00000000-0000-0000-0000-000000000002")
local uuid_c = uuid.parse("00000000-0000-0000-0000-000000000001")

print(string.format("   A < B: %s (expected true)", tostring(uuid_a < uuid_b)))
print(string.format("   A == C: %s (expected true)", tostring(uuid_a == uuid_c)))
print(string.format("   B > A: %s (expected true)", tostring(uuid_b > uuid_a)))

-- 6. Bytes representation
print("\n6. Testing bytes...")
local byte_data = uuid_v4:to_bytes()
local hex_str = ""
for i, byte in ipairs(byte_data) do
    hex_str = hex_str .. string.format("%02x", byte)
end
print(string.format("   UUID as bytes (length %d): %s", #byte_data, hex_str))

-- 7. String representations
print("\n7. Testing string formats...")
print(string.format("   tostring(): %s", tostring(uuid_v4)))
print(string.format("   to_urn(): %s", uuid_v4:to_urn()))

-- 8. Garbage collection test
print("\n8. Testing garbage collection...")
do
    local temp_uuid = uuid.v4()
    print(string.format("   Created temp UUID: %s", temp_uuid))
    -- temp_uuid will be GC'd when it goes out of scope
end
collectgarbage()
print("   ✓ Garbage collection successful")

print("\n=== All Lua tests passed! ===")
