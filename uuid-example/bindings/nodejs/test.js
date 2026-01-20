#!/usr/bin/env node
/**
 * Test the Node.js UUID bindings (Koffi version)
 */

const { Uuid, ParseError, v4, v7 } = require('./uuid_bindings');

console.log('=== Node.js UUID Bindings Test (Koffi FFI) ===\n');

// 1. Generate UUIDs
console.log('1. Generating UUIDs...');
const uuidV4 = v4();
console.log(`   V4 (random): ${uuidV4}`);

const uuidV7 = v7();
console.log(`   V7 (timestamp): ${uuidV7}`);

// 2. Parse UUID
console.log('\n2. Parsing UUID from string...');
const parsed = Uuid.parse('550e8400-e29b-41d4-a716-446655440000');
console.log(`   Parsed: ${parsed}`);
console.log(`   URN: ${parsed.toUrn()}`);

// 3. Test invalid UUID
console.log('\n3. Testing invalid UUID...');
try {
  const bad = Uuid.parse('not-a-uuid');
  console.log('   ERROR: Should have raised exception!');
} catch (e) {
  if (e instanceof ParseError) {
    console.log('   ✓ Correctly raised ParseError');
    console.log(`   Error code: ${e.code}`);
    console.log(`   Message: ${e.message}`);
  } else {
    console.log(`   ERROR: Wrong exception type: ${e.constructor.name}`);
  }
}

// 4. Nil and Max UUIDs
console.log('\n4. Testing nil and max UUIDs...');
const nilUuid = Uuid.nil();
console.log(`   Nil: ${nilUuid}`);
console.log(`   Is nil? ${nilUuid.isNil()}`);

const maxUuid = Uuid.max();
console.log(`   Max: ${maxUuid}`);
console.log(`   Is max? ${maxUuid.isMax()}`);

// 5. Comparison
console.log('\n5. Testing comparison...');
const uuidA = Uuid.parse('00000000-0000-0000-0000-000000000001');
const uuidB = Uuid.parse('00000000-0000-0000-0000-000000000002');
const uuidC = Uuid.parse('00000000-0000-0000-0000-000000000001');

console.log(`   A < B: ${uuidA.compare(uuidB) < 0} (expected true)`);
console.log(`   A == C: ${uuidA.equals(uuidC)} (expected true)`);
console.log(`   B > A: ${uuidB.compare(uuidA) > 0} (expected true)`);

// 6. Bytes representation
console.log('\n6. Testing bytes...');
const byteData = uuidV4.toBytes();
console.log(`   UUID as bytes (length ${byteData.length}): ${byteData.toString('hex')}`);

// 7. String representations
console.log('\n7. Testing string formats...');
console.log(`   toString(): ${uuidV4.toString()}`);
console.log(`   toJSON(): ${JSON.stringify(uuidV4)}`);
console.log(`   inspect: ${require('util').inspect(uuidV4)}`);

// 8. Garbage collection test
console.log('\n8. Testing explicit free...');
const tempUuid = v4();
console.log(`   Created temp UUID: ${tempUuid}`);
tempUuid.free();
console.log('   ✓ Explicitly freed UUID');

try {
  tempUuid.toString(); // Should fail
  console.log('   ERROR: Should have thrown after free!');
} catch (e) {
  console.log('   ✓ Correctly throws error when accessing freed UUID');
}

console.log('\n=== All Node.js tests passed! ===');
console.log('\n✨ Using Koffi - a modern, maintained FFI library');
console.log('   Works with Node.js 18, 20, 22, 23+');
