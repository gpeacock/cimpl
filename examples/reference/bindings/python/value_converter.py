"""
Python bindings for the ValueConverter C library.

This demonstrates how to wrap a C FFI library created with cimpl:
- Loading the shared library
- Error handling via exceptions
- Memory management with context managers
- Pythonic API design
"""

import ctypes
import os
import sys
from pathlib import Path
from typing import Optional


# ============================================================================
# Exception Classes
# ============================================================================

class ValueConverterError(Exception):
    """Base exception for ValueConverter errors"""
    pass


class OutOfRangeError(ValueConverterError):
    """Value is out of range for the target type"""
    pass


class InvalidUtf8Error(ValueConverterError):
    """Bytes are not valid UTF-8"""
    pass


class InvalidHexError(ValueConverterError):
    """Invalid hex string"""
    pass


class BufferTooLargeError(ValueConverterError):
    """Buffer exceeds maximum size"""
    pass


class EmptyValueError(ValueConverterError):
    """Value is empty"""
    pass


# Map error variant names to exception classes
ERROR_MAP = {
    "OutOfRange": OutOfRangeError,
    "InvalidUtf8": InvalidUtf8Error,
    "InvalidHex": InvalidHexError,
    "BufferTooLarge": BufferTooLargeError,
    "EmptyValue": EmptyValueError,
}


# ============================================================================
# Library Loading
# ============================================================================

def _find_library() -> Path:
    """Find the ValueConverter shared library"""
    # Try relative to this file first (development)
    script_dir = Path(__file__).parent
    lib_dir = script_dir.parent.parent / "target" / "debug"
    
    # Platform-specific library names
    if sys.platform == "darwin":
        lib_name = "libvalue_converter.dylib"
    elif sys.platform == "win32":
        lib_name = "value_converter.dll"
    else:
        lib_name = "libvalue_converter.so"
    
    lib_path = lib_dir / lib_name
    if lib_path.exists():
        return lib_path
    
    # Try release build
    lib_dir = script_dir.parent.parent / "target" / "release"
    lib_path = lib_dir / lib_name
    if lib_path.exists():
        return lib_path
    
    raise FileNotFoundError(
        f"Could not find {lib_name}. Please build the library first:\n"
        f"  cd {script_dir.parent.parent}\n"
        f"  cargo build"
    )


# Load the library
_lib_path = _find_library()
_lib = ctypes.CDLL(str(_lib_path))


# ============================================================================
# C Function Signatures
# ============================================================================

# Constructors
_lib.vc_from_i32.argtypes = [ctypes.c_int32]
_lib.vc_from_i32.restype = ctypes.c_void_p

_lib.vc_from_u32.argtypes = [ctypes.c_uint32]
_lib.vc_from_u32.restype = ctypes.c_void_p

_lib.vc_from_i64.argtypes = [ctypes.c_int64]
_lib.vc_from_i64.restype = ctypes.c_void_p

_lib.vc_from_u64.argtypes = [ctypes.c_uint64]
_lib.vc_from_u64.restype = ctypes.c_void_p

_lib.vc_from_bytes.argtypes = [ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t]
_lib.vc_from_bytes.restype = ctypes.c_void_p

_lib.vc_from_string.argtypes = [ctypes.c_char_p]
_lib.vc_from_string.restype = ctypes.c_void_p

_lib.vc_from_hex.argtypes = [ctypes.c_char_p]
_lib.vc_from_hex.restype = ctypes.c_void_p

# Conversions
_lib.vc_to_i32.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_int32)]
_lib.vc_to_i32.restype = ctypes.c_bool

_lib.vc_to_u32.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint32)]
_lib.vc_to_u32.restype = ctypes.c_bool

_lib.vc_to_i64.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_int64)]
_lib.vc_to_i64.restype = ctypes.c_bool

_lib.vc_to_u64.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint64)]
_lib.vc_to_u64.restype = ctypes.c_bool

_lib.vc_to_string.argtypes = [ctypes.c_void_p]
_lib.vc_to_string.restype = ctypes.c_void_p

_lib.vc_to_hex.argtypes = [ctypes.c_void_p]
_lib.vc_to_hex.restype = ctypes.c_void_p

_lib.vc_to_bytes.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_size_t)]
_lib.vc_to_bytes.restype = ctypes.POINTER(ctypes.c_uint8)

# Utilities
_lib.vc_len.argtypes = [ctypes.c_void_p]
_lib.vc_len.restype = ctypes.c_size_t

_lib.vc_max_size.argtypes = []
_lib.vc_max_size.restype = ctypes.c_size_t

# Error handling
_lib.vc_last_error.argtypes = []
_lib.vc_last_error.restype = ctypes.c_void_p

# Memory management
_lib.vc_free.argtypes = [ctypes.c_void_p]
_lib.vc_free.restype = ctypes.c_int32


# ============================================================================
# Helper Functions
# ============================================================================

def _check_error():
    """Check for last error and raise appropriate exception"""
    error_ptr = _lib.vc_last_error()
    if error_ptr:
        error_str = ctypes.c_char_p(error_ptr).value.decode('utf-8')
        _lib.vc_free(error_ptr)
        
        # Parse "VariantName: details" format
        if ": " in error_str:
            variant, details = error_str.split(": ", 1)
            error_class = ERROR_MAP.get(variant, ValueConverterError)
            raise error_class(details)
        else:
            raise ValueConverterError(error_str)


def _c_string_to_python(ptr: int) -> Optional[str]:
    """Convert C string pointer to Python string and free it"""
    if not ptr:
        return None
    result = ctypes.c_char_p(ptr).value.decode('utf-8')
    _lib.vc_free(ptr)
    return result


# ============================================================================
# Python API
# ============================================================================

class ValueConverter:
    """
    A value that can be represented in multiple formats.
    
    Supports conversion between:
    - Signed integers (i32, i64)
    - Unsigned integers (u32, u64)
    - Byte arrays
    - UTF-8 strings
    - Hex strings
    
    Example:
        # Create from integer
        with ValueConverter.from_i32(42) as vc:
            print(vc.to_hex())  # "2a000000"
        
        # Create from string
        with ValueConverter.from_string("Hi") as vc:
            print(vc.to_hex())  # "4869"
    """
    
    def __init__(self, ptr: int):
        """Internal constructor. Use from_* class methods instead."""
        if not ptr:
            _check_error()
            raise ValueConverterError("Failed to create ValueConverter")
        self._ptr = ptr
        self._freed = False
    
    def __del__(self):
        """Cleanup on destruction"""
        if not self._freed and self._ptr:
            _lib.vc_free(self._ptr)
            self._freed = True
    
    def __enter__(self):
        """Context manager entry"""
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit"""
        if not self._freed and self._ptr:
            _lib.vc_free(self._ptr)
            self._freed = True
        return False
    
    # Constructors
    
    @classmethod
    def from_i32(cls, value: int) -> 'ValueConverter':
        """Create from signed 32-bit integer"""
        ptr = _lib.vc_from_i32(ctypes.c_int32(value))
        return cls(ptr)
    
    @classmethod
    def from_u32(cls, value: int) -> 'ValueConverter':
        """Create from unsigned 32-bit integer"""
        ptr = _lib.vc_from_u32(ctypes.c_uint32(value))
        return cls(ptr)
    
    @classmethod
    def from_i64(cls, value: int) -> 'ValueConverter':
        """Create from signed 64-bit integer"""
        ptr = _lib.vc_from_i64(ctypes.c_int64(value))
        return cls(ptr)
    
    @classmethod
    def from_u64(cls, value: int) -> 'ValueConverter':
        """Create from unsigned 64-bit integer"""
        ptr = _lib.vc_from_u64(ctypes.c_uint64(value))
        return cls(ptr)
    
    @classmethod
    def from_bytes(cls, data: bytes) -> 'ValueConverter':
        """Create from byte array (max 8 bytes)"""
        arr = (ctypes.c_uint8 * len(data))(*data)
        ptr = _lib.vc_from_bytes(arr, len(data))
        if not ptr:
            _check_error()
        return cls(ptr)
    
    @classmethod
    def from_string(cls, s: str) -> 'ValueConverter':
        """Create from UTF-8 string (max 8 bytes)"""
        ptr = _lib.vc_from_string(s.encode('utf-8'))
        if not ptr:
            _check_error()
        return cls(ptr)
    
    @classmethod
    def from_hex(cls, hex_str: str) -> 'ValueConverter':
        """Create from hex string (max 16 hex digits = 8 bytes)"""
        ptr = _lib.vc_from_hex(hex_str.encode('utf-8'))
        if not ptr:
            _check_error()
        return cls(ptr)
    
    # Conversions
    
    def to_i32(self) -> int:
        """Convert to signed 32-bit integer (must be exactly 4 bytes)"""
        result = ctypes.c_int32()
        if not _lib.vc_to_i32(self._ptr, ctypes.byref(result)):
            _check_error()
        return result.value
    
    def to_u32(self) -> int:
        """Convert to unsigned 32-bit integer (must be exactly 4 bytes)"""
        result = ctypes.c_uint32()
        if not _lib.vc_to_u32(self._ptr, ctypes.byref(result)):
            _check_error()
        return result.value
    
    def to_i64(self) -> int:
        """Convert to signed 64-bit integer (must be exactly 8 bytes)"""
        result = ctypes.c_int64()
        if not _lib.vc_to_i64(self._ptr, ctypes.byref(result)):
            _check_error()
        return result.value
    
    def to_u64(self) -> int:
        """Convert to unsigned 64-bit integer (must be exactly 8 bytes)"""
        result = ctypes.c_uint64()
        if not _lib.vc_to_u64(self._ptr, ctypes.byref(result)):
            _check_error()
        return result.value
    
    def to_string(self) -> str:
        """Convert to UTF-8 string (must be valid UTF-8)"""
        ptr = _lib.vc_to_string(self._ptr)
        if not ptr:
            _check_error()
        return _c_string_to_python(ptr)
    
    def to_hex(self) -> str:
        """Convert to hex string (always succeeds)"""
        ptr = _lib.vc_to_hex(self._ptr)
        return _c_string_to_python(ptr)
    
    def to_bytes(self) -> bytes:
        """Get raw bytes"""
        length = ctypes.c_size_t()
        ptr = _lib.vc_to_bytes(self._ptr, ctypes.byref(length))
        if not ptr:
            return b""
        # Copy the bytes before freeing
        result = bytes(ptr[:length.value])
        _lib.vc_free(ptr)
        return result
    
    def __len__(self) -> int:
        """Get the size in bytes"""
        return _lib.vc_len(self._ptr)
    
    @staticmethod
    def max_size() -> int:
        """Get the maximum buffer size"""
        return _lib.vc_max_size()
    
    def __repr__(self) -> str:
        """String representation"""
        try:
            return f"ValueConverter({len(self)} bytes, hex={self.to_hex()})"
        except:
            return f"ValueConverter({len(self)} bytes)"
