"""
Python bindings for C2PA FFI library using ctypes

This module provides a Pythonic wrapper around the C2PA C FFI bindings,
with automatic memory management, error handling, and builder patterns.
"""

import ctypes
import json
from pathlib import Path
from typing import Optional, Dict, Any


# Load the library
lib_path_release = Path(__file__).parent.parent.parent / "target" / "release"
lib_path_debug = Path(__file__).parent.parent.parent / "target" / "debug"

lib = None
for lib_path in [lib_path_release, lib_path_debug]:
    try:
        lib = ctypes.CDLL(str(lib_path / "libcimpl_c2pa.dylib"))
        break
    except OSError:
        try:
            lib = ctypes.CDLL(str(lib_path / "libcimpl_c2pa.so"))
            break
        except OSError:
            continue

if lib is None:
    raise ImportError(
        f"Could not find c2pa library. Tried:\n"
        f"  {lib_path_release}\n"
        f"  {lib_path_debug}\n"
        f"Build the library first with: cargo build"
    )


# Error codes
class C2paErrorCode:
    """C2PA error codes"""
    INVALID_SETTINGS = 100
    SIGNER_ERROR = 101
    CONTEXT_ERROR = 102
    INVALID_FORMAT = 103
    IO_ERROR = 104
    SERIALIZATION_ERROR = 105


class C2paError(Exception):
    """Base exception for C2PA errors"""
    
    def __init__(self, code: int, message: str):
        self.code = code
        self.message = message
        super().__init__(f"C2PA Error {code}: {message}")
    
    @classmethod
    def from_last_error(cls):
        """Create exception from the last FFI error"""
        code = lib.c2pa_error_code()
        msg_ptr = lib.c2pa_last_error()
        if msg_ptr:
            msg = ctypes.string_at(msg_ptr).decode('utf-8')
            lib.cimpl_free(msg_ptr)
        else:
            msg = "Unknown error"
        return cls(code, msg)


# Function declarations
lib.c2pa_context_new.restype = ctypes.c_void_p
lib.c2pa_context_new.argtypes = []

lib.c2pa_context_with_settings.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
lib.c2pa_context_with_settings.restype = ctypes.c_int32

lib.c2pa_context_with_settings_toml.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
lib.c2pa_context_with_settings_toml.restype = ctypes.c_int32

lib.c2pa_context_with_settings_obj.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
lib.c2pa_context_with_settings_obj.restype = ctypes.c_int32

lib.c2pa_context_free.argtypes = [ctypes.c_void_p]
lib.c2pa_context_free.restype = ctypes.c_int32

lib.c2pa_settings_new.restype = ctypes.c_void_p
lib.c2pa_settings_new.argtypes = []

lib.c2pa_settings_from_json.argtypes = [ctypes.c_char_p]
lib.c2pa_settings_from_json.restype = ctypes.c_void_p

lib.c2pa_settings_from_toml.argtypes = [ctypes.c_char_p]
lib.c2pa_settings_from_toml.restype = ctypes.c_void_p

lib.c2pa_settings_to_json.argtypes = [ctypes.c_void_p]
lib.c2pa_settings_to_json.restype = ctypes.POINTER(ctypes.c_char)

lib.c2pa_settings_to_toml.argtypes = [ctypes.c_void_p]
lib.c2pa_settings_to_toml.restype = ctypes.POINTER(ctypes.c_char)

lib.c2pa_settings_free.argtypes = [ctypes.c_void_p]
lib.c2pa_settings_free.restype = ctypes.c_int32

lib.c2pa_error_code.restype = ctypes.c_int32
lib.c2pa_error_code.argtypes = []

lib.c2pa_last_error.restype = ctypes.POINTER(ctypes.c_char)
lib.c2pa_last_error.argtypes = []

lib.cimpl_free.argtypes = [ctypes.c_void_p]
lib.cimpl_free.restype = ctypes.c_int32


class Settings:
    """
    C2PA Settings wrapper with automatic memory management
    
    Provides Pythonic access to c2pa::settings::Settings through FFI.
    """
    
    def __init__(self, handle: int):
        """Initialize with an FFI handle (internal use)"""
        if not handle:
            raise C2paError.from_last_error()
        self._handle = handle
    
    def __del__(self):
        """Automatic cleanup"""
        if self._handle:
            lib.c2pa_settings_free(self._handle)
            self._handle = None
    
    @classmethod
    def new(cls) -> 'Settings':
        """Create new Settings with defaults"""
        handle = lib.c2pa_settings_new()
        if not handle:
            raise C2paError.from_last_error()
        return cls(handle)
    
    @classmethod
    def from_json(cls, json_str: str) -> 'Settings':
        """
        Create Settings from JSON string
        
        Args:
            json_str: JSON configuration string
        
        Example:
            settings = Settings.from_json('{"verify": {"verify_after_sign": true}}')
        """
        handle = lib.c2pa_settings_from_json(json_str.encode('utf-8'))
        if not handle:
            raise C2paError.from_last_error()
        return cls(handle)
    
    @classmethod
    def from_dict(cls, config: Dict[str, Any]) -> 'Settings':
        """
        Create Settings from Python dictionary
        
        Args:
            config: Configuration dictionary
        
        Example:
            settings = Settings.from_dict({"verify": {"verify_after_sign": True}})
        """
        json_str = json.dumps(config)
        return cls.from_json(json_str)
    
    @classmethod
    def from_toml(cls, toml_str: str) -> 'Settings':
        """
        Create Settings from TOML string
        
        Args:
            toml_str: TOML configuration string
        
        Example:
            settings = Settings.from_toml('[verify]\\nverify_after_sign = true')
        """
        handle = lib.c2pa_settings_from_toml(toml_str.encode('utf-8'))
        if not handle:
            raise C2paError.from_last_error()
        return cls(handle)
    
    def to_json(self) -> str:
        """
        Serialize Settings to JSON string
        
        Returns:
            JSON string representation of settings
        """
        result = lib.c2pa_settings_to_json(self._handle)
        if not result:
            raise C2paError.from_last_error()
        json_str = ctypes.string_at(result).decode('utf-8')
        lib.cimpl_free(result)
        return json_str
    
    def to_dict(self) -> Dict[str, Any]:
        """
        Convert Settings to Python dictionary
        
        Returns:
            Dictionary representation of settings
        """
        json_str = self.to_json()
        return json.loads(json_str)
    
    def to_toml(self) -> str:
        """
        Serialize Settings to TOML string
        
        Returns:
            TOML string representation of settings
        """
        result = lib.c2pa_settings_to_toml(self._handle)
        if not result:
            raise C2paError.from_last_error()
        toml_str = ctypes.string_at(result).decode('utf-8')
        lib.cimpl_free(result)
        return toml_str


class Context:
    """
    C2PA Context wrapper with builder pattern support
    
    The Context is the central configuration object for C2PA operations.
    Supports builder-style method chaining.
    """
    
    def __init__(self, handle: int):
        """Initialize with an FFI handle (internal use)"""
        if not handle:
            raise C2paError.from_last_error()
        self._handle = handle
    
    def __del__(self):
        """Automatic cleanup"""
        if self._handle:
            lib.c2pa_context_free(self._handle)
            self._handle = None
    
    @classmethod
    def new(cls) -> 'Context':
        """
        Create a new Context with default settings
        
        Example:
            ctx = Context.new()
        """
        handle = lib.c2pa_context_new()
        if not handle:
            raise C2paError.from_last_error()
        return cls(handle)
    
    def with_settings_json(self, json_str: str) -> 'Context':
        """
        Configure Context with JSON settings (builder-style)
        
        Args:
            json_str: JSON configuration string
        
        Returns:
            Self for method chaining
        
        Example:
            ctx = Context.new().with_settings_json('{"verify": {"verify_after_sign": true}}')
        """
        result = lib.c2pa_context_with_settings(self._handle, json_str.encode('utf-8'))
        if result != 0:
            raise C2paError.from_last_error()
        return self
    
    def with_settings_dict(self, config: Dict[str, Any]) -> 'Context':
        """
        Configure Context with Python dictionary (builder-style)
        
        Args:
            config: Configuration dictionary
        
        Returns:
            Self for method chaining
        
        Example:
            ctx = Context.new().with_settings_dict({
                "verify": {"verify_after_sign": True}
            })
        """
        json_str = json.dumps(config)
        return self.with_settings_json(json_str)
    
    def with_settings_toml(self, toml_str: str) -> 'Context':
        """
        Configure Context with TOML settings (builder-style)
        
        Args:
            toml_str: TOML configuration string
        
        Returns:
            Self for method chaining
        
        Example:
            ctx = Context.new().with_settings_toml('[verify]\\nverify_after_sign = true')
        """
        result = lib.c2pa_context_with_settings_toml(self._handle, toml_str.encode('utf-8'))
        if result != 0:
            raise C2paError.from_last_error()
        return self
    
    def with_settings(self, settings: Settings) -> 'Context':
        """
        Configure Context with Settings object (builder-style)
        
        Args:
            settings: Settings object
        
        Returns:
            Self for method chaining
        
        Example:
            settings = Settings.from_dict({"verify": {"verify_after_sign": True}})
            ctx = Context.new().with_settings(settings)
        """
        result = lib.c2pa_context_with_settings_obj(self._handle, settings._handle)
        if result != 0:
            raise C2paError.from_last_error()
        return self


# Module-level functions
def get_last_error() -> Optional[str]:
    """Get the last error message"""
    msg_ptr = lib.c2pa_last_error()
    if msg_ptr:
        msg = ctypes.string_at(msg_ptr).decode('utf-8')
        lib.cimpl_free(msg_ptr)
        return msg
    return None


def get_error_code() -> int:
    """Get the last error code"""
    return lib.c2pa_error_code()


__all__ = [
    'Context',
    'Settings',
    'C2paError',
    'C2paErrorCode',
    'get_last_error',
    'get_error_code',
]
