"""
Python bindings for the cimpl-uuid library

Auto-generated from cimpl_uuid.h
"""

import ctypes
from ctypes import c_char_p, c_int32, c_uint8, c_void_p, POINTER
from typing import Optional
import os


# Load the library
_lib_path = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../target/release/libcimpl_uuid.dylib"))
_lib = ctypes.CDLL(_lib_path)

# Opaque Uuid type
class _UuidHandle(ctypes.Structure):
    pass

UuidPtr = POINTER(_UuidHandle)


# Error codes (from header)
UUID_ERROR_OK = 0
UUID_ERROR_NULL_PARAMETER = 1
UUID_ERROR_STRING_TOO_LONG = 2
UUID_ERROR_INVALID_HANDLE = 3
UUID_ERROR_WRONG_HANDLE_TYPE = 4
UUID_ERROR_OTHER = 5
UUID_ERROR_PARSE_ERROR = 100


# Exception hierarchy based on error codes
class UuidError(Exception):
    """Base exception for UUID library errors"""
    def __init__(self, code: int, message: str):
        self.code = code
        self.message = message
        super().__init__(f"[{code}] {message}")


class NullParameterError(UuidError):
    """A required parameter was NULL"""
    pass


class InvalidHandleError(UuidError):
    """Handle is invalid or already freed"""
    pass


class WrongHandleTypeError(UuidError):
    """Handle type doesn't match expected type"""
    pass


class ParseError(UuidError):
    """UUID parsing failed"""
    pass


class OtherError(UuidError):
    """Other error occurred"""
    pass


# Map error codes to exception classes
_ERROR_EXCEPTIONS = {
    UUID_ERROR_NULL_PARAMETER: NullParameterError,
    UUID_ERROR_INVALID_HANDLE: InvalidHandleError,
    UUID_ERROR_WRONG_HANDLE_TYPE: WrongHandleTypeError,
    UUID_ERROR_PARSE_ERROR: ParseError,
    UUID_ERROR_OTHER: OtherError,
}


# Define C function signatures
_lib.uuid_new_v4.argtypes = []
_lib.uuid_new_v4.restype = UuidPtr

_lib.uuid_new_v7.argtypes = []
_lib.uuid_new_v7.restype = UuidPtr

_lib.uuid_parse.argtypes = [c_char_p]
_lib.uuid_parse.restype = UuidPtr

_lib.uuid_nil.argtypes = []
_lib.uuid_nil.restype = UuidPtr

_lib.uuid_max.argtypes = []
_lib.uuid_max.restype = UuidPtr

_lib.uuid_to_string.argtypes = [UuidPtr]
_lib.uuid_to_string.restype = c_char_p

_lib.uuid_to_urn.argtypes = [UuidPtr]
_lib.uuid_to_urn.restype = c_char_p

_lib.uuid_as_bytes.argtypes = [UuidPtr]
_lib.uuid_as_bytes.restype = POINTER(c_uint8)

_lib.uuid_is_nil.argtypes = [UuidPtr]
_lib.uuid_is_nil.restype = ctypes.c_bool

_lib.uuid_is_max.argtypes = [UuidPtr]
_lib.uuid_is_max.restype = ctypes.c_bool

_lib.uuid_compare.argtypes = [UuidPtr, UuidPtr]
_lib.uuid_compare.restype = c_int32

_lib.uuid_equals.argtypes = [UuidPtr, UuidPtr]
_lib.uuid_equals.restype = ctypes.c_bool

_lib.uuid_error_code.argtypes = []
_lib.uuid_error_code.restype = c_int32

_lib.uuid_last_error.argtypes = []
_lib.uuid_last_error.restype = c_char_p

_lib.uuid_clear_error.argtypes = []
_lib.uuid_clear_error.restype = None

_lib.uuid_free.argtypes = [c_void_p]
_lib.uuid_free.restype = c_int32


def _check_error_and_raise():
    """Check for errors and raise appropriate Python exception"""
    code = _lib.uuid_error_code()
    if code != UUID_ERROR_OK:
        msg_ptr = _lib.uuid_last_error()
        message = msg_ptr.decode('utf-8') if msg_ptr else "Unknown error"
        _lib.uuid_free(msg_ptr)
        _lib.uuid_clear_error()
        
        exception_class = _ERROR_EXCEPTIONS.get(code, UuidError)
        raise exception_class(code, message)


class Uuid:
    """Python wrapper for UUID objects"""
    
    def __init__(self, handle: UuidPtr):
        """Internal constructor - use class methods to create instances"""
        self._handle = handle
        if not handle:
            _check_error_and_raise()
    
    @classmethod
    def v4(cls) -> 'Uuid':
        """Generate a random UUID (version 4)"""
        handle = _lib.uuid_new_v4()
        return cls(handle)
    
    @classmethod
    def v7(cls) -> 'Uuid':
        """Generate a timestamp-based UUID (version 7)"""
        handle = _lib.uuid_new_v7()
        return cls(handle)
    
    @classmethod
    def parse(cls, s: str) -> 'Uuid':
        """Parse a UUID from a string
        
        Raises:
            ParseError: If the string is not a valid UUID
        """
        handle = _lib.uuid_parse(s.encode('utf-8'))
        if not handle:
            _check_error_and_raise()
        return cls(handle)
    
    @classmethod
    def nil(cls) -> 'Uuid':
        """Create a nil UUID (all zeros)"""
        handle = _lib.uuid_nil()
        return cls(handle)
    
    @classmethod
    def max(cls) -> 'Uuid':
        """Create a max UUID (all ones)"""
        handle = _lib.uuid_max()
        return cls(handle)
    
    def __str__(self) -> str:
        """Convert UUID to string (hyphenated format)"""
        result = _lib.uuid_to_string(self._handle)
        if not result:
            _check_error_and_raise()
        s = result.decode('utf-8')
        _lib.uuid_free(result)
        return s
    
    def to_urn(self) -> str:
        """Convert UUID to URN format"""
        result = _lib.uuid_to_urn(self._handle)
        if not result:
            _check_error_and_raise()
        s = result.decode('utf-8')
        _lib.uuid_free(result)
        return s
    
    def to_bytes(self) -> bytes:
        """Get UUID as 16 bytes"""
        result = _lib.uuid_as_bytes(self._handle)
        if not result:
            _check_error_and_raise()
        # Copy the bytes before freeing
        byte_array = bytes(result[i] for i in range(16))
        _lib.uuid_free(result)
        return byte_array
    
    def is_nil(self) -> bool:
        """Check if UUID is nil (all zeros)"""
        return _lib.uuid_is_nil(self._handle)
    
    def is_max(self) -> bool:
        """Check if UUID is max (all ones)"""
        return _lib.uuid_is_max(self._handle)
    
    def __eq__(self, other) -> bool:
        """Check equality with another UUID"""
        if not isinstance(other, Uuid):
            return NotImplemented
        return _lib.uuid_equals(self._handle, other._handle)
    
    def __lt__(self, other) -> bool:
        """Compare UUIDs"""
        if not isinstance(other, Uuid):
            return NotImplemented
        return _lib.uuid_compare(self._handle, other._handle) < 0
    
    def __le__(self, other) -> bool:
        if not isinstance(other, Uuid):
            return NotImplemented
        return _lib.uuid_compare(self._handle, other._handle) <= 0
    
    def __gt__(self, other) -> bool:
        if not isinstance(other, Uuid):
            return NotImplemented
        return _lib.uuid_compare(self._handle, other._handle) > 0
    
    def __ge__(self, other) -> bool:
        if not isinstance(other, Uuid):
            return NotImplemented
        return _lib.uuid_compare(self._handle, other._handle) >= 0
    
    def __repr__(self) -> str:
        return f"Uuid('{self}')"
    
    def __del__(self):
        """Free the UUID when Python object is garbage collected"""
        if hasattr(self, '_handle') and self._handle:
            _lib.uuid_free(self._handle)
            self._handle = None


# Convenience functions
def uuid4() -> Uuid:
    """Generate a random UUID (version 4)"""
    return Uuid.v4()


def uuid7() -> Uuid:
    """Generate a timestamp-based UUID (version 7)"""
    return Uuid.v7()
