/**
 * @file uuid.cpp
 * @brief Implementation of C++ wrapper for cimpl-uuid library
 */

#include "uuid.hpp"
#include <cstring>

namespace cimpl {

// ============================================================================
// Private helper functions
// ============================================================================

void Uuid::checkError() {
    int32_t code = ::uuid_error_code();
    if (code != 0) {
        char* msg = ::uuid_last_error();
        std::string error_msg = msg ? msg : "Unknown error";
        if (msg) {
            ::uuid_free(msg);
        }
        ::uuid_clear_error();
        throw UuidError(code, error_msg);
    }
}

Uuid::Uuid(::Uuid* ptr) : ptr_(ptr) {
    if (!ptr_) {
        checkError();
        throw UuidError(-1, "UUID pointer is null");
    }
}

// ============================================================================
// Static factory methods
// ============================================================================

Uuid Uuid::newV4() {
    ::Uuid* ptr = ::uuid_new_v4();
    return Uuid(ptr);
}

Uuid Uuid::newV7() {
    ::Uuid* ptr = ::uuid_new_v7();
    return Uuid(ptr);
}

Uuid Uuid::parse(const std::string& str) {
    ::Uuid* ptr = ::uuid_parse(str.c_str());
    if (!ptr) {
        checkError();
    }
    return Uuid(ptr);
}

Uuid Uuid::nil() {
    ::Uuid* ptr = ::uuid_nil();
    return Uuid(ptr);
}

Uuid Uuid::max() {
    ::Uuid* ptr = ::uuid_max();
    return Uuid(ptr);
}

Uuid Uuid::fromBytes(const std::vector<uint8_t>& bytes) {
    if (bytes.size() != 16) {
        throw std::invalid_argument("UUID requires exactly 16 bytes, got " + 
                                    std::to_string(bytes.size()));
    }
    
    // Create a nil UUID and then replace its bytes
    // This is a workaround since the C API doesn't have a from_bytes function
    auto uuid = nil();
    
    // Get the bytes and replace them
    // Note: This is a simplification. In a real implementation,
    // you might add a uuid_from_bytes function to the C API
    throw std::runtime_error("fromBytes not yet implemented - add uuid_from_bytes to C API");
}

// ============================================================================
// Copy operations
// ============================================================================

Uuid::Uuid(const Uuid& other) {
    // Create a new UUID by converting to string and parsing back
    // This demonstrates that copying works through the C API
    auto str = other.toString();
    ::Uuid* ptr = ::uuid_parse(str.c_str());
    if (!ptr) {
        checkError();
    }
    ptr_ = std::unique_ptr<::Uuid, UuidDeleter>(ptr);
}

Uuid& Uuid::operator=(const Uuid& other) {
    if (this != &other) {
        auto str = other.toString();
        ::Uuid* ptr = ::uuid_parse(str.c_str());
        if (!ptr) {
            checkError();
        }
        ptr_ = std::unique_ptr<::Uuid, UuidDeleter>(ptr);
    }
    return *this;
}

// ============================================================================
// Conversions
// ============================================================================

std::string Uuid::toString() const {
    char* str = ::uuid_to_string(get());
    if (!str) {
        checkError();
        return "";
    }
    std::string result(str);
    ::uuid_free(str);
    return result;
}

std::string Uuid::toUrn() const {
    char* str = ::uuid_to_urn(get());
    if (!str) {
        checkError();
        return "";
    }
    std::string result(str);
    ::uuid_free(str);
    return result;
}

std::vector<uint8_t> Uuid::asBytes() const {
    const uint8_t* bytes = ::uuid_as_bytes(get());
    if (!bytes) {
        checkError();
        return {};
    }
    
    std::vector<uint8_t> result(bytes, bytes + 16);
    ::uuid_free(const_cast<uint8_t*>(bytes));
    return result;
}

// ============================================================================
// Predicates
// ============================================================================

bool Uuid::isNil() const {
    return ::uuid_is_nil(get());
}

bool Uuid::isMax() const {
    return ::uuid_is_max(get());
}

// ============================================================================
// Comparison operators
// ============================================================================

auto Uuid::operator<=>(const Uuid& other) const -> std::strong_ordering {
    int32_t cmp = ::uuid_compare(get(), other.get());
    if (cmp < 0) return std::strong_ordering::less;
    if (cmp > 0) return std::strong_ordering::greater;
    return std::strong_ordering::equal;
}

bool Uuid::operator==(const Uuid& other) const {
    return ::uuid_equals(get(), other.get());
}

bool Uuid::operator<(const Uuid& other) const {
    return ::uuid_compare(get(), other.get()) < 0;
}

// ============================================================================
// Stream output
// ============================================================================

std::ostream& operator<<(std::ostream& os, const Uuid& uuid) {
    os << uuid.toString();
    return os;
}

} // namespace cimpl
