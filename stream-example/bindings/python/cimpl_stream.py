"""
Python bindings for cimpl_stream - Callback-based I/O streams

This module provides Pythonic wrappers around the cimpl_stream C library,
allowing Python file-like objects to be used as streams in Rust/C code.
"""

import ctypes
import os
import sys
from typing import BinaryIO, Optional
from ctypes import c_void_p, c_int32, c_int64, c_uint8, c_size_t, POINTER, CFUNCTYPE


# Find the library
def _find_library():
    """Locate the cimpl_stream library."""
    script_dir = os.path.dirname(os.path.abspath(__file__))
    lib_dir = os.path.join(script_dir, '..', '..', 'target', 'release')
    
    if sys.platform == 'darwin':
        lib_name = 'libcimpl_stream.dylib'
    elif sys.platform == 'win32':
        lib_name = 'cimpl_stream.dll'
    else:
        lib_name = 'libcimpl_stream.so'
    
    lib_path = os.path.join(lib_dir, lib_name)
    
    if not os.path.exists(lib_path):
        raise FileNotFoundError(
            f"cimpl_stream library not found at {lib_path}. "
            f"Build it with: cargo build --release"
        )
    
    return lib_path


# Load library
_lib = ctypes.CDLL(_find_library())

# Type aliases
intptr_t = ctypes.c_ssize_t

# Opaque types
class CimplStreamContext(ctypes.Structure):
    pass

class CimplStream(ctypes.Structure):
    pass

# Enums (matching C header with cbindgen prefixes)
class SeekMode:
    """Seek modes for stream positioning."""
    START = 0       # CIMPL_SEEK_MODE_START
    CURRENT = 1     # CIMPL_SEEK_MODE_CURRENT
    END = 2         # CIMPL_SEEK_MODE_END

class ErrorCode:
    """Error codes for stream operations."""
    OK = 0                      # CIMPL_STREAM_ERROR_OK
    NULL_PARAMETER = 1          # CIMPL_STREAM_ERROR_NULL_PARAMETER
    STRING_TOO_LONG = 2         # CIMPL_STREAM_ERROR_STRING_TOO_LONG
    INVALID_HANDLE = 3          # CIMPL_STREAM_ERROR_INVALID_HANDLE
    WRONG_HANDLE_TYPE = 4       # CIMPL_STREAM_ERROR_WRONG_HANDLE_TYPE
    OTHER = 5                   # CIMPL_STREAM_ERROR_OTHER
    IO_OPERATION = 100          # CIMPL_STREAM_ERROR_IO_OPERATION
    INVALID_BUFFER = 101        # CIMPL_STREAM_ERROR_INVALID_BUFFER

# Callback type definitions
ReadCallback = CFUNCTYPE(intptr_t, POINTER(CimplStreamContext), POINTER(c_uint8), c_size_t)
SeekCallback = CFUNCTYPE(c_int64, POINTER(CimplStreamContext), c_int64, c_int32)
WriteCallback = CFUNCTYPE(intptr_t, POINTER(CimplStreamContext), POINTER(c_uint8), c_size_t)
FlushCallback = CFUNCTYPE(c_int32, POINTER(CimplStreamContext))

# Function signatures
_lib.cimpl_stream_new.argtypes = [
    POINTER(CimplStreamContext),
    ReadCallback,
    SeekCallback,
    WriteCallback,
    FlushCallback
]
_lib.cimpl_stream_new.restype = POINTER(CimplStream)

_lib.cimpl_stream_read.argtypes = [POINTER(CimplStream), POINTER(c_uint8), c_size_t]
_lib.cimpl_stream_read.restype = intptr_t

_lib.cimpl_stream_write.argtypes = [POINTER(CimplStream), POINTER(c_uint8), c_size_t]
_lib.cimpl_stream_write.restype = intptr_t

_lib.cimpl_stream_seek.argtypes = [POINTER(CimplStream), c_int64, c_int32]
_lib.cimpl_stream_seek.restype = c_int64

_lib.cimpl_stream_flush.argtypes = [POINTER(CimplStream)]
_lib.cimpl_stream_flush.restype = c_int32

_lib.cimpl_stream_last_error.argtypes = []
_lib.cimpl_stream_last_error.restype = ctypes.c_char_p

_lib.cimpl_stream_error_code.argtypes = []
_lib.cimpl_stream_error_code.restype = c_int32

_lib.cimpl_stream_clear_error.argtypes = []
_lib.cimpl_stream_clear_error.restype = None

_lib.cimpl_free.argtypes = [c_void_p]
_lib.cimpl_free.restype = c_int32


# Exceptions
class CimplStreamError(Exception):
    """Base exception for cimpl_stream errors."""
    def __init__(self, message: str, code: int):
        super().__init__(message)
        self.code = code


class NullParameterError(CimplStreamError):
    """Raised when a null parameter is passed."""
    pass


class InvalidHandleError(CimplStreamError):
    """Raised when a handle is invalid or already freed."""
    pass


class IoError(CimplStreamError):
    """Raised when an I/O operation fails."""
    pass


class InvalidBufferError(CimplStreamError):
    """Raised when an invalid buffer is provided."""
    pass


def _check_error(result, func_name: str):
    """Check for errors and raise appropriate exceptions."""
    if result < 0 or result is None:
        code = _lib.cimpl_stream_error_code()
        error_ptr = _lib.cimpl_stream_last_error()
        
        if error_ptr:
            message = error_ptr.decode('utf-8')
            _lib.cimpl_free(error_ptr)
        else:
            message = f"{func_name} failed"
        
        # Map error codes to specific exception types
        if code == ErrorCode.NULL_PARAMETER:
            raise NullParameterError(message, code)
        elif code == ErrorCode.INVALID_HANDLE:
            raise InvalidHandleError(message, code)
        elif code == ErrorCode.IO_OPERATION:
            raise IoError(message, code)
        elif code == ErrorCode.INVALID_BUFFER:
            raise InvalidBufferError(message, code)
        else:
            raise CimplStreamError(message, code)
    
    return result


class Stream:
    """
    A stream that wraps a Python file-like object for use with cimpl_stream.
    
    This allows Rust/C code to read from and write to Python file objects
    through a callback-based interface.
    
    Example:
        with open('test.txt', 'rb') as f:
            stream = Stream(f)
            data = stream.read(100)
            stream.seek(0)
            stream.write(b"Hello!")
    """
    
    def __init__(self, file_obj: BinaryIO):
        """
        Create a stream from a Python file-like object.
        
        Args:
            file_obj: A file-like object with read, write, seek, and flush methods.
        """
        self._file = file_obj
        self._handle: Optional[POINTER(CimplStream)] = None
        
        # Create callback functions that capture self
        @ReadCallback
        def read_cb(ctx, data, length):
            try:
                bytes_data = self._file.read(length)
                if not bytes_data:
                    return 0
                bytes_read = len(bytes_data)
                ctypes.memmove(data, bytes_data, bytes_read)
                return bytes_read
            except Exception as e:
                print(f"Read callback error: {e}", file=sys.stderr)
                return -1
        
        @SeekCallback
        def seek_cb(ctx, offset, mode):
            try:
                if mode == SeekMode.START:
                    whence = os.SEEK_SET
                elif mode == SeekMode.CURRENT:
                    whence = os.SEEK_CUR
                elif mode == SeekMode.END:
                    whence = os.SEEK_END
                else:
                    return -1
                
                new_pos = self._file.seek(offset, whence)
                return new_pos
            except Exception as e:
                print(f"Seek callback error: {e}", file=sys.stderr)
                return -1
        
        @WriteCallback
        def write_cb(ctx, data, length):
            try:
                bytes_data = ctypes.string_at(data, length)
                bytes_written = self._file.write(bytes_data)
                return bytes_written if bytes_written is not None else length
            except Exception as e:
                print(f"Write callback error: {e}", file=sys.stderr)
                return -1
        
        @FlushCallback
        def flush_cb(ctx):
            try:
                self._file.flush()
                return 0
            except Exception as e:
                print(f"Flush callback error: {e}", file=sys.stderr)
                return -1
        
        # Keep references to prevent garbage collection
        self._callbacks = (read_cb, seek_cb, write_cb, flush_cb)
        
        # Create the stream (use id(self) as context pointer)
        context = ctypes.cast(id(self), POINTER(CimplStreamContext))
        self._handle = _lib.cimpl_stream_new(
            context,
            read_cb,
            seek_cb,
            write_cb,
            flush_cb
        )
        
        if not self._handle:
            _check_error(None, "cimpl_stream_new")
    
    def read(self, size: int = -1) -> bytes:
        """
        Read data from the stream.
        
        Args:
            size: Number of bytes to read. -1 reads until EOF.
            
        Returns:
            Bytes read from the stream.
        """
        if not self._handle:
            raise CimplStreamError("Stream is closed", 0)
        
        if size < 0:
            size = 8192  # Default buffer size
        
        buffer = (c_uint8 * size)()
        bytes_read = _lib.cimpl_stream_read(self._handle, buffer, size)
        _check_error(bytes_read, "cimpl_stream_read")
        
        return bytes(buffer[:bytes_read])
    
    def write(self, data: bytes) -> int:
        """
        Write data to the stream.
        
        Args:
            data: Bytes to write.
            
        Returns:
            Number of bytes written.
        """
        if not self._handle:
            raise CimplStreamError("Stream is closed", 0)
        
        buffer = (c_uint8 * len(data)).from_buffer_copy(data)
        bytes_written = _lib.cimpl_stream_write(self._handle, buffer, len(data))
        _check_error(bytes_written, "cimpl_stream_write")
        
        return bytes_written
    
    def seek(self, offset: int, whence: int = os.SEEK_SET) -> int:
        """
        Change the stream position.
        
        Args:
            offset: Offset in bytes.
            whence: os.SEEK_SET, os.SEEK_CUR, or os.SEEK_END.
            
        Returns:
            New absolute position.
        """
        if not self._handle:
            raise CimplStreamError("Stream is closed", 0)
        
        if whence == os.SEEK_SET:
            mode = SeekMode.START
        elif whence == os.SEEK_CUR:
            mode = SeekMode.CURRENT
        elif whence == os.SEEK_END:
            mode = SeekMode.END
        else:
            raise ValueError(f"Invalid whence value: {whence}")
        
        new_pos = _lib.cimpl_stream_seek(self._handle, offset, mode)
        _check_error(new_pos, "cimpl_stream_seek")
        
        return new_pos
    
    def flush(self) -> None:
        """Flush the stream."""
        if not self._handle:
            raise CimplStreamError("Stream is closed", 0)
        
        result = _lib.cimpl_stream_flush(self._handle)
        _check_error(result, "cimpl_stream_flush")
    
    def tell(self) -> int:
        """Get the current stream position."""
        return self.seek(0, os.SEEK_CUR)
    
    def close(self) -> None:
        """Close the stream and free resources."""
        if self._handle:
            _lib.cimpl_free(self._handle)
            self._handle = None
    
    def __enter__(self):
        """Context manager entry."""
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.close()
        return False
    
    def __del__(self):
        """Destructor to ensure cleanup."""
        self.close()


# Convenience function
def wrap_file(file_obj: BinaryIO) -> Stream:
    """
    Wrap a Python file-like object as a cimpl_stream.
    
    Args:
        file_obj: A file-like object with read, write, seek, and flush methods.
        
    Returns:
        A Stream object wrapping the file.
    """
    return Stream(file_obj)
