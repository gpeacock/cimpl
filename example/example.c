#include <stdio.h>
#include <stdlib.h>
#include "include/cimple_example.h"

// Helper to print error details after a function fails
// Only call this when a function returns an error (NULL or -1)
void print_error() {
    int32_t code = mystring_error_code();
    char* msg = mystring_last_error();
    
    if (msg != NULL) {
        // Message already includes error type: "NullParameter: details"
        fprintf(stderr, "Error %d: %s\n", code, msg);
        mystring_string_free(msg);
    } else {
        fprintf(stderr, "Error %d: (no message)\n", code);
    }
}

int main() {
    printf("=== Cimple Example: String Manipulation Library ===\n\n");

    // Test 1: Create a string
    printf("1. Creating string with 'Hello, World!'...\n");
    MyString* str = mystring_create("Hello, World!");
    if (str == NULL) {
        fprintf(stderr, "Failed to create string!\n");
        print_error();
        return 1;
    }
    printf("   ✓ String created successfully\n\n");

    // Test 2: Get the current value
    printf("2. Getting current value...\n");
    char* value = mystring_get_value(str);
    if (value != NULL) {
        printf("   Value: '%s'\n", value);
        printf("   Length: %zu bytes\n", mystring_len(str));
        cimple_free(value);  // Universal free!
    } else {
        print_error();
    }
    printf("\n");

    // Test 3: Convert to uppercase
    printf("3. Converting to uppercase...\n");
    char* upper = mystring_to_uppercase(str);
    if (upper != NULL) {
        printf("   Uppercase: '%s'\n", upper);
        cimple_free(upper);  // Universal free!
    } else {
        print_error();
    }
    printf("\n");

    // Test 4: Append to string
    printf("4. Appending ' How are you?'...\n");
    if (mystring_append(str, " How are you?") == 0) {
        value = mystring_get_value(str);
        if (value != NULL) {
            printf("   Result: '%s'\n", value);
            cimple_free(value);
        }
    } else {
        print_error();
    }
    printf("\n");

    // Test 5: Set a new value
    printf("5. Setting new value to 'Goodbye!'...\n");
    if (mystring_set_value(str, "Goodbye!") == 0) {
        value = mystring_get_value(str);
        if (value != NULL) {
            printf("   New value: '%s'\n", value);
            cimple_free(value);
        }
    } else {
        print_error();
    }
    printf("\n");

    // Test 6: Error handling (standard C convention)
    printf("6. Testing error handling (passing NULL)...\n");
    mystring_clear_error();
    if (mystring_set_value(str, NULL) != 0) {
        // Function returned -1, so NOW we check error details
        printf("   ✓ Correctly rejected NULL parameter\n");
        printf("   ");
        print_error();
    }
    printf("\n");

    // Test 7: Clean up - use universal cimple_free!
    printf("7. Freeing the string with cimple_free()...\n");
    if (cimple_free(str) == 0) {
        printf("   ✓ String freed successfully\n");
    } else {
        print_error();
    }
    printf("\n");

    // Test 8: Double-free detection
    printf("8. Testing double-free protection...\n");
    if (cimple_free(str) != 0) {
        // Function returned -1, so NOW we check error details
        printf("   ✓ Double-free correctly detected and prevented\n");
        printf("   ");
        print_error();
    }
    printf("\n");

    printf("=== All tests completed successfully! ===\n");
    return 0;
}
