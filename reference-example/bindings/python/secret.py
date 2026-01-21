"""
Python bindings for cimpl reference example - Secret Message Processor

This demonstrates idiomatic Python error handling for cimpl FFI.
"""

import ctypes
import os
from pathlib import Path

# Find the library
lib_path = Path(__file__).parent.parent.parent / "target" / "release"
if os.name == 'nt':
    lib_name = "cimpl_reference.dll"
elif os.uname().sysname == 'Darwin':
    lib_name = "libcimpl_reference.dylib"
else:
    lib_name = "libcimpl_reference.so"

lib = ctypes.CDLL(str(lib_path / lib_name))

# ============================================================================
# Error Handling
# ============================================================================

class SecretError(Exception):
    """Base exception for secret message errors"""
    def __init__(self, code, message):
        self.code = code
        self.message = message
        super().__init__(f"Error {code}: {message}")

# Error codes
SECRET_ERROR_OK = 0
SECRET_ERROR_INVALID_HEX = 100
SECRET_ERROR_INVALID_FORMAT = 101
SECRET_ERROR_TOO_SHORT = 102
SECRET_ERROR_TOO_LONG = 103

# ============================================================================
# Function Signatures
# ============================================================================

# String → String functions
lib.secret_rot13.argtypes = [ctypes.c_char_p]
lib.secret_rot13.restype = ctypes.c_void_p

lib.secret_reverse.argtypes = [ctypes.c_char_p]
lib.secret_reverse.restype = ctypes.c_void_p

lib.secret_remove_vowels.argtypes = [ctypes.c_char_p]
lib.secret_remove_vowels.restype = ctypes.c_void_p

lib.secret_uppercase.argtypes = [ctypes.c_char_p]
lib.secret_uppercase.restype = ctypes.c_void_p

lib.secret_to_hex.argtypes = [ctypes.c_char_p]
lib.secret_to_hex.restype = ctypes.c_void_p

lib.secret_from_hex.argtypes = [ctypes.c_char_p]
lib.secret_from_hex.restype = ctypes.c_void_p

# Validation functions
lib.secret_validate_length.argtypes = [ctypes.c_char_p, ctypes.c_size_t, ctypes.c_size_t]
lib.secret_validate_length.restype = ctypes.c_bool

lib.secret_is_ascii.argtypes = [ctypes.c_char_p]
lib.secret_is_ascii.restype = ctypes.c_bool

lib.secret_is_valid_hex.argtypes = [ctypes.c_char_p]
lib.secret_is_valid_hex.restype = ctypes.c_bool

# Counting functions
lib.secret_count_chars.argtypes = [ctypes.c_char_p]
lib.secret_count_chars.restype = ctypes.c_size_t

lib.secret_count_vowels.argtypes = [ctypes.c_char_p]
lib.secret_count_vowels.restype = ctypes.c_size_t

lib.secret_count_consonants.argtypes = [ctypes.c_char_p]
lib.secret_count_consonants.restype = ctypes.c_size_t

lib.secret_count_words.argtypes = [ctypes.c_char_p]
lib.secret_count_words.restype = ctypes.c_size_t

# Error handling
lib.secret_error_code.argtypes = []
lib.secret_error_code.restype = ctypes.c_int32

lib.secret_last_error.argtypes = []
lib.secret_last_error.restype = ctypes.c_void_p

lib.secret_clear_error.argtypes = []
lib.secret_clear_error.restype = None

# Memory management
lib.secret_free.argtypes = [ctypes.c_void_p]
lib.secret_free.restype = ctypes.c_bool

# ============================================================================
# Helper Functions
# ============================================================================

def _get_error():
    """Get the last error from the library"""
    code = lib.secret_error_code()
    if code == SECRET_ERROR_OK:
        return None
    
    msg_ptr = lib.secret_last_error()
    if msg_ptr:
        msg = ctypes.string_at(msg_ptr).decode('utf-8')
        lib.secret_free(msg_ptr)
        return SecretError(code, msg)
    return SecretError(code, "Unknown error")

def _call_string_fn(fn, *args):
    """Call a function that returns a string, handling errors"""
    # Convert string args to bytes
    byte_args = [arg.encode('utf-8') if isinstance(arg, str) else arg for arg in args]
    
    result = fn(*byte_args)
    if result is None or result == 0:
        error = _get_error()
        if error:
            raise error
        return None
    
    # Convert result to Python string
    s = ctypes.string_at(result).decode('utf-8')
    lib.secret_free(result)
    return s

# ============================================================================
# Public API
# ============================================================================

def rot13(text: str) -> str:
    """Encode text using ROT13 cipher"""
    return _call_string_fn(lib.secret_rot13, text)

def reverse(text: str) -> str:
    """Reverse the input string"""
    return _call_string_fn(lib.secret_reverse, text)

def remove_vowels(text: str) -> str:
    """Remove all vowels from text"""
    return _call_string_fn(lib.secret_remove_vowels, text)

def uppercase(text: str) -> str:
    """Convert text to uppercase"""
    return _call_string_fn(lib.secret_uppercase, text)

def to_hex(text: str) -> str:
    """Encode string to hex"""
    return _call_string_fn(lib.secret_to_hex, text)

def from_hex(hex_str: str) -> str:
    """Decode hex string to text (raises SecretError on invalid hex)"""
    return _call_string_fn(lib.secret_from_hex, hex_str)

def validate_length(text: str, min_len: int, max_len: int) -> bool:
    """Validate that text length is within bounds (raises SecretError if not)"""
    result = lib.secret_validate_length(text.encode('utf-8'), min_len, max_len)
    if not result:
        error = _get_error()
        if error:
            raise error
    return result

def is_ascii(text: str) -> bool:
    """Check if text contains only ASCII characters"""
    return lib.secret_is_ascii(text.encode('utf-8'))

def is_valid_hex(text: str) -> bool:
    """Check if text is valid hex"""
    return lib.secret_is_valid_hex(text.encode('utf-8'))

def count_chars(text: str) -> int:
    """Count characters in string"""
    return lib.secret_count_chars(text.encode('utf-8'))

def count_vowels(text: str) -> int:
    """Count vowels in string"""
    return lib.secret_count_vowels(text.encode('utf-8'))

def count_consonants(text: str) -> int:
    """Count consonants in string"""
    return lib.secret_count_consonants(text.encode('utf-8'))

def count_words(text: str) -> int:
    """Count words in string"""
    return lib.secret_count_words(text.encode('utf-8'))
