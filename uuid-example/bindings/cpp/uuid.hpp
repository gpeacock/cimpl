/**
 * @file uuid.hpp
 * @brief Modern C++ wrapper for cimpl-uuid library
 *
 * This header provides an idiomatic C++ interface to the cimpl-uuid C library,
 * demonstrating best practices for wrapping C APIs in modern C++:
 * 
 * - RAII (Resource Acquisition Is Initialization)
 * - Exception-based error handling
 * - Smart pointers with custom deleters
 * - Move semantics
 * - Operator overloading
 * - Value semantics with explicit copying
 */

#pragma once

#include <cstdint>
#include <memory>
#include <stdexcept>
#include <string>
#include <vector>
#include <compare>

extern "C" {
#include "cimpl_uuid.h"
}

namespace cimpl {

/**
 * Exception thrown when UUID operations fail
 */
class UuidError : public std::runtime_error {
public:
    explicit UuidError(int32_t code, const std::string& message)
        : std::runtime_error(message), error_code_(code) {}
    
    int32_t code() const noexcept { return error_code_; }

private:
    int32_t error_code_;
};

/**
 * Modern C++ wrapper for UUID
 * 
 * This class provides RAII-based memory management and idiomatic C++ operations
 * for UUIDs. It wraps the C API in a safe, easy-to-use interface.
 * 
 * Example:
 * @code
 * auto uuid = Uuid::newV4();
 * std::cout << uuid.toString() << std::endl;
 * 
 * auto parsed = Uuid::parse("550e8400-e29b-41d4-a716-446655440000");
 * if (uuid == parsed) {
 *     std::cout << "Same UUID!" << std::endl;
 * }
 * @endcode
 */
class Uuid {
public:
    // ========================================================================
    // Constructors (static factory methods)
    // ========================================================================

    /**
     * Creates a new random UUID (version 4)
     * @return A new UUID with random data
     * @throws UuidError if creation fails
     */
    static Uuid newV4();

    /**
     * Creates a new timestamp-based UUID (version 7)
     * @return A new UUID with timestamp and random data
     * @throws UuidError if creation fails
     */
    static Uuid newV7();

    /**
     * Parses a UUID from a string
     * @param str String in UUID format (e.g., "550e8400-e29b-41d4-a716-446655440000")
     * @return Parsed UUID
     * @throws UuidError if parsing fails
     */
    static Uuid parse(const std::string& str);

    /**
     * Creates a nil UUID (all zeros)
     * @return Nil UUID
     */
    static Uuid nil();

    /**
     * Creates a maximum UUID (all ones)
     * @return Maximum UUID
     */
    static Uuid max();

    /**
     * Creates a UUID from a byte array
     * @param bytes Vector of exactly 16 bytes
     * @return UUID created from bytes
     * @throws std::invalid_argument if bytes.size() != 16
     * @throws UuidError if creation fails
     */
    static Uuid fromBytes(const std::vector<uint8_t>& bytes);

    // ========================================================================
    // Rule of Five (with move-only semantics)
    // ========================================================================

    /**
     * Destructor - automatically frees the underlying UUID
     */
    ~Uuid() = default;

    /**
     * Move constructor
     */
    Uuid(Uuid&&) noexcept = default;

    /**
     * Move assignment operator
     */
    Uuid& operator=(Uuid&&) noexcept = default;

    /**
     * Copy constructor - creates a deep copy
     */
    Uuid(const Uuid& other);

    /**
     * Copy assignment operator - creates a deep copy
     */
    Uuid& operator=(const Uuid& other);

    // ========================================================================
    // Conversions
    // ========================================================================

    /**
     * Converts UUID to string representation
     * @return String in format "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
     */
    std::string toString() const;

    /**
     * Converts UUID to URN format
     * @return String in format "urn:uuid:xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
     */
    std::string toUrn() const;

    /**
     * Gets UUID as a byte array (16 bytes)
     * @return Vector containing the 16 UUID bytes
     */
    std::vector<uint8_t> asBytes() const;

    // ========================================================================
    // Predicates
    // ========================================================================

    /**
     * Checks if this UUID is nil (all zeros)
     * @return true if nil, false otherwise
     */
    bool isNil() const;

    /**
     * Checks if this UUID is max (all ones)
     * @return true if max, false otherwise
     */
    bool isMax() const;

    // ========================================================================
    // Comparison operators
    // ========================================================================

    /**
     * Three-way comparison operator (C++20)
     */
    auto operator<=>(const Uuid& other) const -> std::strong_ordering;

    /**
     * Equality comparison
     */
    bool operator==(const Uuid& other) const;

    /**
     * Inequality comparison
     */
    bool operator!=(const Uuid& other) const { return !(*this == other); }

    /**
     * Less-than comparison
     */
    bool operator<(const Uuid& other) const;

    /**
     * Less-than-or-equal comparison
     */
    bool operator<=(const Uuid& other) const { return !(other < *this); }

    /**
     * Greater-than comparison
     */
    bool operator>(const Uuid& other) const { return other < *this; }

    /**
     * Greater-than-or-equal comparison
     */
    bool operator>=(const Uuid& other) const { return !(*this < other); }

    // ========================================================================
    // Stream output
    // ========================================================================

    /**
     * Output stream operator
     */
    friend std::ostream& operator<<(std::ostream& os, const Uuid& uuid);

private:
    // Custom deleter for unique_ptr
    struct UuidDeleter {
        void operator()(::Uuid* ptr) const noexcept {
            if (ptr) {
                ::uuid_free(ptr);
            }
        }
    };

    // The underlying C pointer, managed by unique_ptr
    std::unique_ptr<::Uuid, UuidDeleter> ptr_;

    // Private constructor - use static factory methods instead
    explicit Uuid(::Uuid* ptr);

    // Helper to check for errors and throw if needed
    static void checkError();

    // Get raw pointer (for internal use)
    ::Uuid* get() const noexcept { return ptr_.get(); }
};

} // namespace cimpl

// Include stream support
#include <iostream>
