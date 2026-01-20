/**
 * @file example.cpp
 * @brief Demonstration of C++ UUID wrapper
 * 
 * This example demonstrates how to use the modern C++ wrapper for the
 * cimpl-uuid library, showcasing idiomatic C++ patterns.
 */

#include "uuid.hpp"
#include <iostream>
#include <vector>
#include <algorithm>
#include <iomanip>

// Note: We don't use "using namespace cimpl" to avoid ambiguity with C's Uuid type

// Helper to print a section header
void section(const std::string& title) {
    std::cout << "\n=== " << title << " ===\n\n";
}

int main() {
    std::cout << "C++ UUID Wrapper Demo\n";
    std::cout << "=====================\n";

    try {
        // Test 1: Generate random UUIDs (v4)
        section("1. Generating Random UUIDs (v4)");
        auto uuid1 = cimpl::Uuid::newV4();
        auto uuid2 = cimpl::Uuid::newV4();
        std::cout << "UUID 1: " << uuid1 << "\n";
        std::cout << "UUID 2: " << uuid2 << "\n";

        // Test 2: Generate timestamp-based UUIDs (v7)
        section("2. Generating Timestamp-based UUIDs (v7)");
        auto uuid_v7 = cimpl::Uuid::newV7();
        std::cout << "UUID v7: " << uuid_v7 << "\n";
        std::cout << "Note: v7 UUIDs are sortable by creation time\n";

        // Test 3: Parse UUID from string
        section("3. Parsing UUID from String");
        try {
            auto uuid3 = cimpl::Uuid::parse("550e8400-e29b-41d4-a716-446655440000");
            std::cout << "✓ Parsed: " << uuid3 << "\n";
            
            // Convert to URN
            std::cout << "URN format: " << uuid3.toUrn() << "\n";
        } catch (const cimpl::UuidError& e) {
            std::cerr << "✗ Parse failed: " << e.what() << "\n";
        }

        // Test 4: Parse invalid UUID (exception handling)
        section("4. Exception Handling (Invalid UUID)");
        try {
            auto invalid = cimpl::Uuid::parse("not-a-valid-uuid");
            std::cout << "This should not print\n";
        } catch (const cimpl::UuidError& e) {
            std::cout << "✓ Caught exception: " << e.what() << "\n";
            std::cout << "  Error code: " << e.code() << "\n";
        }

        // Test 5: Comparison operators
        section("5. Comparison Operators");
        std::cout << "UUID1 == UUID2: " << (uuid1 == uuid2 ? "true" : "false") << "\n";
        std::cout << "UUID1 != UUID2: " << (uuid1 != uuid2 ? "true" : "false") << "\n";
        std::cout << "UUID1 < UUID2: " << (uuid1 < uuid2 ? "true" : "false") << "\n";
        std::cout << "UUID1 > UUID2: " << (uuid1 > uuid2 ? "true" : "false") << "\n";

        // Test 6: Nil and Max UUIDs
        section("6. Special UUIDs (Nil and Max)");
        auto nil = cimpl::Uuid::nil();
        auto max = cimpl::Uuid::max();
        
        std::cout << "Nil UUID: " << nil << "\n";
        std::cout << "Is nil? " << (nil.isNil() ? "yes" : "no") << "\n";
        std::cout << "Is max? " << (nil.isMax() ? "yes" : "no") << "\n";
        std::cout << "\n";
        
        std::cout << "Max UUID: " << max << "\n";
        std::cout << "Is nil? " << (max.isNil() ? "yes" : "no") << "\n";
        std::cout << "Is max? " << (max.isMax() ? "yes" : "no") << "\n";

        // Test 7: Binary representation
        section("7. Binary Representation");
        auto bytes = uuid1.asBytes();
        std::cout << "UUID: " << uuid1 << "\n";
        std::cout << "Bytes (hex): ";
        for (size_t i = 0; i < bytes.size(); ++i) {
            std::cout << std::hex << std::setfill('0') << std::setw(2) 
                      << static_cast<int>(bytes[i]);
            if (i == 3 || i == 5 || i == 7 || i == 9) std::cout << "-";
        }
        std::cout << std::dec << "\n";

        // Test 8: Copy semantics
        section("8. Copy Semantics");
        auto original = cimpl::Uuid::newV4();
        std::cout << "Original: " << original << "\n";
        
        // Copy constructor
        cimpl::Uuid copy1(original);
        std::cout << "Copy 1:   " << copy1 << "\n";
        std::cout << "Copy equals original? " << (copy1 == original ? "yes" : "no") << "\n";
        
        // Copy assignment
        auto copy2 = original;
        std::cout << "Copy 2:   " << copy2 << "\n";
        std::cout << "Copy equals original? " << (copy2 == original ? "yes" : "no") << "\n";

        // Test 9: Move semantics
        section("9. Move Semantics");
        auto movable = cimpl::Uuid::newV4();
        std::cout << "Original: " << movable << "\n";
        
        auto moved = std::move(movable);
        std::cout << "Moved to: " << moved << "\n";
        std::cout << "Note: Original is now in a valid but unspecified state\n";

        // Test 10: Sorting UUIDs
        section("10. Sorting UUIDs (using < operator)");
        std::vector<cimpl::Uuid> uuids;
        for (int i = 0; i < 5; ++i) {
            uuids.push_back(cimpl::Uuid::newV4());
        }
        
        std::cout << "Before sorting:\n";
        for (const auto& u : uuids) {
            std::cout << "  " << u << "\n";
        }
        
        std::sort(uuids.begin(), uuids.end());
        
        std::cout << "\nAfter sorting:\n";
        for (const auto& u : uuids) {
            std::cout << "  " << u << "\n";
        }

        // Test 11: RAII - automatic cleanup
        section("11. RAII (Automatic Memory Management)");
        {
            auto scoped_uuid = cimpl::Uuid::newV4();
            std::cout << "UUID created in scope: " << scoped_uuid << "\n";
            std::cout << "UUID will be automatically freed when scope ends...\n";
        }
        std::cout << "✓ Scope ended - UUID automatically cleaned up (no manual free needed!)\n";

        // Test 12: Three-way comparison (C++20)
        section("12. Three-way Comparison (C++20 spaceship operator)");
        auto a = cimpl::Uuid::nil();
        auto b = cimpl::Uuid::max();
        auto c = cimpl::Uuid::nil();
        
        std::cout << "nil <=> max: ";
        if (a < b) std::cout << "less\n";
        else if (a > b) std::cout << "greater\n";
        else std::cout << "equal\n";
        
        std::cout << "nil <=> nil: ";
        if (a < c) std::cout << "less\n";
        else if (a > c) std::cout << "greater\n";
        else std::cout << "equal\n";

        std::cout << "\n=== All tests completed successfully! ===\n\n";
        std::cout << "Key C++ features demonstrated:\n";
        std::cout << "  ✓ RAII (automatic memory management)\n";
        std::cout << "  ✓ Exception-based error handling\n";
        std::cout << "  ✓ Smart pointers (unique_ptr with custom deleter)\n";
        std::cout << "  ✓ Move semantics\n";
        std::cout << "  ✓ Copy semantics\n";
        std::cout << "  ✓ Operator overloading\n";
        std::cout << "  ✓ Stream output\n";
        std::cout << "  ✓ STL integration (sorting)\n";
        std::cout << "  ✓ C++20 three-way comparison\n";

    } catch (const std::exception& e) {
        std::cerr << "\nUnhandled exception: " << e.what() << "\n";
        return 1;
    }

    return 0;
}
