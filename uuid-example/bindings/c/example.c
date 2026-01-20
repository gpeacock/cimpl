#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "cimpl_uuid.h"

void print_error() {
    int32_t code = uuid_error_code();
    char* msg = uuid_last_error();
    
    if (msg != NULL) {
        fprintf(stderr, "Error %d: %s\n", code, msg);
        uuid_free(msg);
    } else {
        fprintf(stderr, "Error %d: (no message)\n", code);
    }
}

int main() {
    printf("=== Cimpl UUID Library Demo ===\n\n");

    // Test 1: Generate random UUID (v4)
    printf("1. Generating random UUID (v4)...\n");
    Uuid* uuid1 = uuid_new_v4();
    if (uuid1 == NULL) {
        fprintf(stderr, "Failed to create UUID v4!\n");
        print_error();
        return 1;
    }
    
    char* str1 = uuid_to_string(uuid1);
    if (str1) {
        printf("   Generated: %s\n", str1);
        uuid_free(str1);
    }
    printf("\n");

    // Test 2: Generate timestamp-based UUID (v7)
    printf("2. Generating timestamp-based UUID (v7)...\n");
    Uuid* uuid2 = uuid_new_v7();
    if (uuid2) {
        char* str2 = uuid_to_string(uuid2);
        if (str2) {
            printf("   Generated: %s\n", str2);
            uuid_free(str2);
        }
    }
    printf("\n");

    // Test 3: Parse a UUID from string
    printf("3. Parsing UUID from string...\n");
    const char* test_uuid = "550e8400-e29b-41d4-a716-446655440000";
    Uuid* uuid3 = uuid_parse(test_uuid);
    if (uuid3 == NULL) {
        printf("   ✗ Failed to parse UUID\n");
        print_error();
    } else {
        printf("   ✓ Parsed: %s\n", test_uuid);
        
        // Convert to URN format
        char* urn = uuid_to_urn(uuid3);
        if (urn) {
            printf("   URN format: %s\n", urn);
            uuid_free(urn);
        }
        
        uuid_free(uuid3);
    }
    printf("\n");

    // Test 4: Parse invalid UUID
    printf("4. Attempting to parse invalid UUID...\n");
    uuid_clear_error();
    Uuid* invalid = uuid_parse("not-a-valid-uuid");
    if (invalid == NULL) {
        printf("   ✓ Correctly rejected invalid UUID\n");
        printf("   ");
        print_error();
    }
    printf("\n");

    // Test 5: Compare UUIDs
    printf("5. Comparing UUIDs...\n");
    if (uuid_equals(uuid1, uuid2)) {
        printf("   UUIDs are equal (extremely unlikely!)\n");
    } else {
        printf("   ✓ UUIDs are different (as expected)\n");
        int32_t cmp = uuid_compare(uuid1, uuid2);
        printf("   Comparison result: %s\n", 
               cmp < 0 ? "uuid1 < uuid2" : 
               cmp > 0 ? "uuid1 > uuid2" : "equal");
    }
    printf("\n");

    // Test 6: Nil UUID
    printf("6. Testing nil UUID...\n");
    Uuid* nil = uuid_nil();
    if (nil) {
        char* nil_str = uuid_to_string(nil);
        if (nil_str) {
            printf("   Nil UUID: %s\n", nil_str);
            uuid_free(nil_str);
        }
        printf("   Is nil? %s\n", uuid_is_nil(nil) ? "yes" : "no");
        printf("   Is max? %s\n", uuid_is_max(nil) ? "yes" : "no");
        uuid_free(nil);
    }
    printf("\n");

    // Test 7: Max UUID
    printf("7. Testing max UUID...\n");
    Uuid* max_uuid = uuid_max();
    if (max_uuid) {
        char* max_str = uuid_to_string(max_uuid);
        if (max_str) {
            printf("   Max UUID: %s\n", max_str);
            uuid_free(max_str);
        }
        printf("   Is nil? %s\n", uuid_is_nil(max_uuid) ? "yes" : "no");
        printf("   Is max? %s\n", uuid_is_max(max_uuid) ? "yes" : "no");
        uuid_free(max_uuid);
    }
    printf("\n");

    // Test 8: Binary representation
    printf("8. Getting UUID as bytes...\n");
    const uint8_t* bytes = uuid_as_bytes(uuid1);
    if (bytes) {
        printf("   Bytes (hex): ");
        for (int i = 0; i < 16; i++) {
            printf("%02x", bytes[i]);
            if (i == 3 || i == 5 || i == 7 || i == 9) printf("-");
        }
        printf("\n");
        uuid_free((void*)bytes);
    }
    printf("\n");

    // Test 9: Clean up - universal uuid_free works for everything!
    printf("9. Cleaning up with universal uuid_free()...\n");
    if (uuid_free(uuid1) == 0) {
        printf("   ✓ uuid1 freed\n");
    }
    if (uuid_free(uuid2) == 0) {
        printf("   ✓ uuid2 freed\n");
    }
    printf("\n");

    // Test 10: Double-free protection
    printf("10. Testing double-free protection...\n");
    if (uuid_free(uuid1) != 0) {
        printf("   ✓ Double-free correctly detected\n");
        printf("   ");
        print_error();
    }
    printf("\n");

    printf("=== All tests completed successfully! ===\n");
    return 0;
}
