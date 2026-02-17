"""
Python bindings for the UUID C library using ctypes.

This module provides a Pythonic interface to UUID operations implemented in Rust.
"""

import ctypes
import os
import platform
from pathlib import Path
from typing import Optional

# Determine library name based on platform
if platform.system() == "Darwin":
    LIB_NAME = "libuuid_ffi.dylib"
elif platform.system() == "Windows":
    LIB_NAME = "uuid_ffi.dll"
else:  # Linux and others
    LIB_NAME = "libuuid_ffi.so"

# Try to load the library
def _find_library():
    """Find the UUID FFI library in common locations."""
    # Try current directory
    if os.path.exists(LIB_NAME):
        return LIB_NAME
    
    # Try target/debug and target/release
    for target_dir in ["../../target/debug", "../../target/release", "../target/debug", "../target/release"]:
        lib_path = os.path.join(target_dir, LIB_NAME)
        if os.path.exists(lib_path):
            return lib_path
    
    # Try relative to this file
    script_dir = Path(__file__).parent
    for rel_path in [".", "../..", "../../target/debug", "../../target/release"]:
        lib_path = script_dir / rel_path / LIB_NAME
        if lib_path.exists():
            return str(lib_path)
    
    raise FileNotFoundError(
        f"Could not find {LIB_NAME}. "
        f"Please build the library first with 'cargo build'"
    )

# Load the library
_lib = ctypes.CDLL(_find_library())

# Define function signatures

# UUID Creation
_lib.uuid_new_v4.restype = ctypes.c_void_p
_lib.uuid_new_v4.argtypes = []

_lib.uuid_nil.restype = ctypes.c_void_p
_lib.uuid_nil.argtypes = []

_lib.uuid_max.restype = ctypes.c_void_p
_lib.uuid_max.argtypes = []

_lib.uuid_parse.restype = ctypes.c_void_p
_lib.uuid_parse.argtypes = [ctypes.c_char_p]

_lib.uuid_from_bytes.restype = ctypes.c_void_p
_lib.uuid_from_bytes.argtypes = [ctypes.POINTER(ctypes.c_uint8)]

# UUID Conversion
_lib.uuid_to_hyphenated.restype = ctypes.c_char_p
_lib.uuid_to_hyphenated.argtypes = [ctypes.c_void_p]

_lib.uuid_to_simple.restype = ctypes.c_char_p
_lib.uuid_to_simple.argtypes = [ctypes.c_void_p]

_lib.uuid_to_urn.restype = ctypes.c_char_p
_lib.uuid_to_urn.argtypes = [ctypes.c_void_p]

_lib.uuid_as_bytes.restype = ctypes.c_bool
_lib.uuid_as_bytes.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8)]

# UUID Comparison
_lib.uuid_equals.restype = ctypes.c_bool
_lib.uuid_equals.argtypes = [ctypes.c_void_p, ctypes.c_void_p]

_lib.uuid_is_nil.restype = ctypes.c_bool
_lib.uuid_is_nil.argtypes = [ctypes.c_void_p]

_lib.uuid_is_max.restype = ctypes.c_bool
_lib.uuid_is_max.argtypes = [ctypes.c_void_p]

# Error Handling
_lib.uuid_last_error.restype = ctypes.c_char_p
_lib.uuid_last_error.argtypes = []

# Memory Management
_lib.uuid_free.restype = ctypes.c_int32
_lib.uuid_free.argtypes = [ctypes.c_void_p]


# Python Exception Types
class UuidError(Exception):
    """Base exception for UUID operations."""
    pass


class ParseError(UuidError):
    """Error parsing a UUID string."""
    pass


class InvalidFormatError(UuidError):
    """Invalid UUID format."""
    pass


def _parse_error(error_msg: str):
    """Parse error message and return appropriate exception type."""
    if not error_msg:
        return UuidError("Unknown error")
    
    # Error format is "VariantName: details"
    if ": " in error_msg:
        variant, _, details = error_msg.partition(": ")
        variant = variant.strip()
        details = details.strip()
    else:
        variant = "Unknown"
        details = error_msg
    
    # Map variant names to exception types
    exception_map = {
        "ParseError": ParseError,
        "InvalidFormat": InvalidFormatError,
    }
    
    exception_class = exception_map.get(variant, UuidError)
    return exception_class(details)


class Uuid:
    """
    A UUID (Universally Unique Identifier).
    
    This class provides a Pythonic interface to UUID operations.
    UUIDs are automatically freed when the object is garbage collected.
    """
    
    def __init__(self, ptr: int):
        """
        Initialize from a raw pointer.
        
        Users should use the factory methods instead of this constructor:
        - Uuid.new_v4()
        - Uuid.parse()
        - Uuid.from_bytes()
        - Uuid.nil()
        - Uuid.max()
        """
        if not ptr:
            error_msg = _lib.uuid_last_error()
            if error_msg:
                error_str = error_msg.decode('utf-8')
                _lib.uuid_free(error_msg)
                raise _parse_error(error_str)
            raise UuidError("Failed to create UUID")
        
        self._ptr = ptr
        self._closed = False
    
    @classmethod
    def new_v4(cls) -> 'Uuid':
        """
        Generate a new random Version 4 UUID.
        
        Returns:
            A new random UUID.
            
        Raises:
            UuidError: If random number generation fails.
            
        Example:
            >>> uuid = Uuid.new_v4()
            >>> print(uuid)
            a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8
        """
        ptr = _lib.uuid_new_v4()
        return cls(ptr)
    
    @classmethod
    def parse(cls, uuid_str: str) -> 'Uuid':
        """
        Parse a UUID from a string.
        
        Accepts various formats:
        - Hyphenated: "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"
        - Simple: "a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8"
        - URN: "urn:uuid:A1A2A3A4-B1B2-C1C2-D1D2-D3D4D5D6D7D8"
        - Braced: "{a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8}"
        
        Args:
            uuid_str: The string to parse.
            
        Returns:
            The parsed UUID.
            
        Raises:
            ParseError: If the string cannot be parsed.
            
        Example:
            >>> uuid = Uuid.parse("550e8400-e29b-41d4-a716-446655440000")
            >>> print(uuid)
            550e8400-e29b-41d4-a716-446655440000
        """
        ptr = _lib.uuid_parse(uuid_str.encode('utf-8'))
        return cls(ptr)
    
    @classmethod
    def from_bytes(cls, bytes_data: bytes) -> 'Uuid':
        """
        Create a UUID from raw bytes.
        
        Args:
            bytes_data: Exactly 16 bytes representing the UUID.
            
        Returns:
            The UUID.
            
        Raises:
            UuidError: If bytes_data is not exactly 16 bytes.
            
        Example:
            >>> data = b'\\x00' * 16
            >>> uuid = Uuid.from_bytes(data)
            >>> uuid.is_nil()
            True
        """
        if len(bytes_data) != 16:
            raise UuidError(f"UUID requires exactly 16 bytes, got {len(bytes_data)}")
        
        byte_array = (ctypes.c_uint8 * 16)(*bytes_data)
        ptr = _lib.uuid_from_bytes(byte_array)
        return cls(ptr)
    
    @classmethod
    def nil(cls) -> 'Uuid':
        """
        Get the nil UUID (all zeros).
        
        Returns:
            The nil UUID: 00000000-0000-0000-0000-000000000000
            
        Example:
            >>> uuid = Uuid.nil()
            >>> uuid.is_nil()
            True
        """
        ptr = _lib.uuid_nil()
        return cls(ptr)
    
    @classmethod
    def max(cls) -> 'Uuid':
        """
        Get the max UUID (all ones).
        
        Returns:
            The max UUID: ffffffff-ffff-ffff-ffff-ffffffffffff
            
        Example:
            >>> uuid = Uuid.max()
            >>> uuid.is_max()
            True
        """
        ptr = _lib.uuid_max()
        return cls(ptr)
    
    def to_hyphenated(self) -> str:
        """
        Format UUID as hyphenated string.
        
        Returns:
            String in format "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"
        """
        self._check_closed()
        result = _lib.uuid_to_hyphenated(self._ptr)
        if not result:
            raise UuidError("Failed to convert UUID to string")
        string = result.decode('utf-8')
        _lib.uuid_free(result)
        return string
    
    def to_simple(self) -> str:
        """
        Format UUID as simple string (no hyphens).
        
        Returns:
            String in format "a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8"
        """
        self._check_closed()
        result = _lib.uuid_to_simple(self._ptr)
        if not result:
            raise UuidError("Failed to convert UUID to string")
        string = result.decode('utf-8')
        _lib.uuid_free(result)
        return string
    
    def to_urn(self) -> str:
        """
        Format UUID as URN.
        
        Returns:
            String in format "urn:uuid:A1A2A3A4-B1B2-C1C2-D1D2-D3D4D5D6D7D8"
        """
        self._check_closed()
        result = _lib.uuid_to_urn(self._ptr)
        if not result:
            raise UuidError("Failed to convert UUID to URN")
        string = result.decode('utf-8')
        _lib.uuid_free(result)
        return string
    
    def as_bytes(self) -> bytes:
        """
        Get the raw bytes of the UUID.
        
        Returns:
            16 bytes representing the UUID.
        """
        self._check_closed()
        byte_array = (ctypes.c_uint8 * 16)()
        success = _lib.uuid_as_bytes(self._ptr, byte_array)
        if not success:
            raise UuidError("Failed to get UUID bytes")
        return bytes(byte_array)
    
    def is_nil(self) -> bool:
        """
        Check if this UUID is nil (all zeros).
        
        Returns:
            True if nil, False otherwise.
        """
        self._check_closed()
        return _lib.uuid_is_nil(self._ptr)
    
    def is_max(self) -> bool:
        """
        Check if this UUID is max (all ones).
        
        Returns:
            True if max, False otherwise.
        """
        self._check_closed()
        return _lib.uuid_is_max(self._ptr)
    
    def __eq__(self, other) -> bool:
        """Check if two UUIDs are equal."""
        if not isinstance(other, Uuid):
            return False
        self._check_closed()
        other._check_closed()
        return _lib.uuid_equals(self._ptr, other._ptr)
    
    def __ne__(self, other) -> bool:
        """Check if two UUIDs are not equal."""
        return not self.__eq__(other)
    
    def __str__(self) -> str:
        """Return hyphenated string representation."""
        return self.to_hyphenated()
    
    def __repr__(self) -> str:
        """Return Python representation."""
        if self._closed:
            return "Uuid(closed)"
        return f"Uuid('{self.to_hyphenated()}')"
    
    def __hash__(self) -> int:
        """Make UUID hashable (for use in sets and dicts)."""
        return hash(self.as_bytes())
    
    def _check_closed(self):
        """Check if UUID has been freed."""
        if self._closed:
            raise UuidError("UUID has been closed")
    
    def close(self):
        """
        Explicitly free the UUID.
        
        After calling this method, the UUID object cannot be used.
        This is called automatically when the object is garbage collected.
        """
        if not self._closed:
            _lib.uuid_free(self._ptr)
            self._closed = True
            self._ptr = None
    
    def __del__(self):
        """Free the UUID when garbage collected."""
        self.close()
    
    def __enter__(self):
        """Context manager entry."""
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.close()
        return False
