#include <stdio.h>
#include <stdlib.h>
#include "include/cimple_example.h"

void print_error() {
    char* error = mystring_last_error();
    if (error != NULL) {
        fprintf(stderr, "Error: %s\n", error);
        mystring_string_free(error);
    }
}

int main() {
    printf("=== Cimple Example: String Manipulation Library ===\n\n");

    // Test 1: Create a string
    printf("1. Creating string with 'Hello, World!'...\n");
    MyStringHandle* handle = mystring_create("Hello, World!");
    if (handle == NULL) {
        fprintf(stderr, "Failed to create string!\n");
        print_error();
        return 1;
    }
    printf("   ✓ String created successfully\n\n");

    // Test 2: Get the current value
    printf("2. Getting current value...\n");
    char* value = mystring_get_value(handle);
    if (value != NULL) {
        printf("   Value: '%s'\n", value);
        printf("   Length: %zu bytes\n", mystring_len(handle));
        mystring_string_free(value);
    } else {
        print_error();
    }
    printf("\n");

    // Test 3: Convert to uppercase
    printf("3. Converting to uppercase...\n");
    char* upper = mystring_to_uppercase(handle);
    if (upper != NULL) {
        printf("   Uppercase: '%s'\n", upper);
        mystring_string_free(upper);
    } else {
        print_error();
    }
    printf("\n");

    // Test 4: Append to string
    printf("4. Appending ' How are you?'...\n");
    if (mystring_append(handle, " How are you?") == 0) {
        value = mystring_get_value(handle);
        if (value != NULL) {
            printf("   Result: '%s'\n", value);
            mystring_string_free(value);
        }
    } else {
        print_error();
    }
    printf("\n");

    // Test 5: Set a new value
    printf("5. Setting new value to 'Goodbye!'...\n");
    if (mystring_set_value(handle, "Goodbye!") == 0) {
        value = mystring_get_value(handle);
        if (value != NULL) {
            printf("   New value: '%s'\n", value);
            mystring_string_free(value);
        }
    } else {
        print_error();
    }
    printf("\n");

    // Test 6: Error handling - passing NULL
    printf("6. Testing error handling (passing NULL)...\n");
    mystring_clear_error();
    if (mystring_set_value(handle, NULL) != 0) {
        printf("   ✓ Correctly rejected NULL parameter\n");
        print_error();
    }
    printf("\n");

    // Test 7: Clean up
    printf("7. Freeing the string handle...\n");
    if (mystring_free(handle) == 0) {
        printf("   ✓ Handle freed successfully\n");
    } else {
        print_error();
    }
    printf("\n");

    // Test 8: Double-free detection
    printf("8. Testing double-free protection...\n");
    if (mystring_free(handle) != 0) {
        printf("   ✓ Double-free correctly detected and prevented\n");
        print_error();
    }
    printf("\n");

    printf("=== All tests completed successfully! ===\n");
    return 0;
}
