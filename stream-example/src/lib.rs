// Copyright 2024 Adobe. All rights reserved.
// This file is licensed to you under the Apache License,
// Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
// or the MIT license (http://opensource.org/licenses/MIT),
// at your option.

// Unless required by applicable law or agreed to in writing,
// this software is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR REPRESENTATIONS OF ANY KIND, either express or
// implied. See the LICENSE-MIT and LICENSE-APACHE files for the
// specific language governing permissions and limitations under
// each license.

//! # CimplStream - Callback-based I/O streams
//!
//! This library demonstrates how to implement callback-based streams using cimpl.
//! It allows C code to provide Read/Write/Seek implementations through callbacks,
//! which are then usable by Rust code as standard I/O traits.
//!
//! ## Key Features
//!
//! - Bridges C callback-based I/O to Rust's Read, Write, and Seek traits
//! - Safe pointer validation using cimpl macros
//! - Universal memory management with `cimpl_free()`
//! - Standard error handling with error codes and messages
//!
//! ## Building
//!
//! ```bash
//! cargo build --release
//! ```
//!
//! This generates:
//! - `target/release/libcimpl_stream.{a,so,dylib}` - The library
//! - `include/cimpl_stream.h` - C header with full documentation

use std::io::{Read, Seek, SeekFrom, Write};

use cimpl::{
    box_tracked, deref_mut_or_return_neg, ok_or_return, ptr_or_return_int, ptr_or_return_null, Error,
};

// ============================================================================
// Error Codes
// ============================================================================

/// Error codes for stream operations
///
/// This enum provides all possible error codes returned by cimpl_stream functions.
/// It includes both core cimpl errors (0-99) and stream-specific errors (100+).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CimplStreamError {
    /// No error occurred
    Ok = 0,
    
    // Core cimpl errors (1-99)
    /// A required parameter was NULL
    NullParameter = 1,
    /// String exceeds maximum allowed length
    StringTooLong = 2,
    /// Handle value is invalid or already freed
    InvalidHandle = 3,
    /// Handle type doesn't match the expected type
    WrongHandleType = 4,
    /// Other unspecified error
    Other = 5,
    
    // Stream-specific errors (100+)
    /// I/O operation failed (read, write, seek, or flush)
    IoOperation = 100,
    /// Invalid buffer pointer provided
    InvalidBuffer = 101,
}

// Map std::io::Error to cimpl Error
const ERROR_MAPPER: fn(&std::io::Error) -> (i32, &'static str) = 
    |_e| (CimplStreamError::IoOperation as i32, "IoError");

// ============================================================================
// Stream Context and Callbacks
// ============================================================================

/// Opaque context for stream callbacks.
///
/// This pointer is provided by the caller when creating a stream and is passed
/// back to each callback. It typically points to the caller's native stream object.
#[repr(C)]
#[derive(Debug)]
pub struct CimplStreamContext {
    _private: [u8; 0],
}

/// Seek modes for stream positioning.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum CimplSeekMode {
    /// Seek from the start of the stream.
    Start = 0,
    /// Seek from the current position.
    Current = 1,
    /// Seek from the end of the stream.
    End = 2,
}

/// Read callback: reads data from the stream.
///
/// # Parameters
/// - `context`: The stream context provided when creating the stream
/// - `data`: Buffer to write the read data into
/// - `len`: Number of bytes to read
///
/// # Returns
/// - Number of bytes actually read (>= 0) on success
/// - -1 on error
pub type CimplReadCallback = unsafe extern "C" fn(
    context: *mut CimplStreamContext,
    data: *mut u8,
    len: usize,
) -> isize;

/// Seek callback: changes the current position in the stream.
///
/// # Parameters
/// - `context`: The stream context provided when creating the stream
/// - `offset`: Offset to seek to (interpretation depends on mode)
/// - `mode`: How to interpret the offset (Start, Current, or End)
///
/// # Returns
/// - New position in the stream (>= 0) on success
/// - -1 on error
pub type CimplSeekCallback = unsafe extern "C" fn(
    context: *mut CimplStreamContext,
    offset: i64,
    mode: CimplSeekMode,
) -> i64;

/// Write callback: writes data to the stream.
///
/// # Parameters
/// - `context`: The stream context provided when creating the stream
/// - `data`: Buffer containing data to write
/// - `len`: Number of bytes to write
///
/// # Returns
/// - Number of bytes actually written (>= 0) on success
/// - -1 on error
pub type CimplWriteCallback = unsafe extern "C" fn(
    context: *mut CimplStreamContext,
    data: *const u8,
    len: usize,
) -> isize;

/// Flush callback: ensures all buffered data is written.
///
/// # Parameters
/// - `context`: The stream context provided when creating the stream
///
/// # Returns
/// - 0 on success
/// - -1 on error
pub type CimplFlushCallback = unsafe extern "C" fn(context: *mut CimplStreamContext) -> i32;

// ============================================================================
// Stream Structure
// ============================================================================

/// A stream that bridges C callbacks to Rust's Read/Write/Seek traits.
///
/// This structure holds callback function pointers and a context pointer.
/// It implements Rust's standard I/O traits, allowing it to be used anywhere
/// a Read, Write, or Seek trait is required.
pub struct CimplStream {
    context: *mut CimplStreamContext,
    reader: CimplReadCallback,
    seeker: CimplSeekCallback,
    writer: CimplWriteCallback,
    flusher: CimplFlushCallback,
}

// ============================================================================
// Stream Construction
// ============================================================================

/// Creates a new stream from C callbacks.
///
/// # Parameters
/// - `context`: Opaque pointer to caller's stream context (passed to all callbacks)
/// - `reader`: Callback function for reading data
/// - `seeker`: Callback function for seeking
/// - `writer`: Callback function for writing data
/// - `flusher`: Callback function for flushing
///
/// # Returns
/// - Pointer to the new stream on success
/// - NULL on error (check `cimpl_stream_last_error()` for details)
///
/// # Safety
/// - The context pointer must remain valid for the lifetime of the stream
/// - All callback functions must remain valid for the lifetime of the stream
/// - The returned stream must be freed with `cimpl_free()` when done
///
/// # Example
/// ```c
/// CimplStream* stream = cimpl_stream_new(
///     my_context,
///     my_read_callback,
///     my_seek_callback,
///     my_write_callback,
///     my_flush_callback
/// );
/// if (!stream) {
///     fprintf(stderr, "Failed to create stream\n");
///     return -1;
/// }
/// // Use the stream...
/// cimpl_free(stream);
/// ```
#[no_mangle]
pub extern "C" fn cimpl_stream_new(
    context: *mut CimplStreamContext,
    reader: CimplReadCallback,
    seeker: CimplSeekCallback,
    writer: CimplWriteCallback,
    flusher: CimplFlushCallback,
) -> *mut CimplStream {
    ptr_or_return_null!(context);

    let stream = CimplStream {
        context,
        reader,
        seeker,
        writer,
        flusher,
    };

    box_tracked!(stream)
}

// ============================================================================
// Stream Operations
// ============================================================================

/// Reads data from the stream.
///
/// # Parameters
/// - `stream`: The stream to read from
/// - `buffer`: Buffer to write the read data into (must not be NULL)
/// - `len`: Number of bytes to read
///
/// # Returns
/// - Number of bytes actually read (>= 0) on success
/// - -1 on error
///
/// # Example
/// ```c
/// uint8_t buffer[1024];
/// isize bytes_read = cimpl_stream_read(stream, buffer, sizeof(buffer));
/// if (bytes_read < 0) {
///     fprintf(stderr, "Read error\n");
/// }
/// ```
#[no_mangle]
pub extern "C" fn cimpl_stream_read(
    stream: *mut CimplStream,
    buffer: *mut u8,
    len: usize,
) -> isize {
    let s = deref_mut_or_return_neg!(stream, CimplStream);
    ptr_or_return_int!(buffer);

    // Create a safe slice from the raw pointer
    let buf = unsafe { std::slice::from_raw_parts_mut(buffer, len) };

    ok_or_return!(s.read(buf), |bytes_read| bytes_read as isize, -1)
}

/// Seeks to a position in the stream.
///
/// # Parameters
/// - `stream`: The stream to seek in
/// - `offset`: Offset to seek to (interpretation depends on mode)
/// - `mode`: How to interpret the offset (Start, Current, or End)
///
/// # Returns
/// - New position in the stream (>= 0) on success
/// - -1 on error
///
/// # Example
/// ```c
/// // Seek to the end of the stream
/// int64_t end_pos = cimpl_stream_seek(stream, 0, CIMPL_SEEK_END);
/// 
/// // Seek to the beginning
/// int64_t start_pos = cimpl_stream_seek(stream, 0, CIMPL_SEEK_START);
/// 
/// // Seek forward 100 bytes from current position
/// int64_t new_pos = cimpl_stream_seek(stream, 100, CIMPL_SEEK_CURRENT);
/// ```
#[no_mangle]
pub extern "C" fn cimpl_stream_seek(
    stream: *mut CimplStream,
    offset: i64,
    mode: CimplSeekMode,
) -> i64 {
    let s = deref_mut_or_return_neg!(stream, CimplStream);

    let seek_from = match mode {
        CimplSeekMode::Start => SeekFrom::Start(offset as u64),
        CimplSeekMode::Current => SeekFrom::Current(offset),
        CimplSeekMode::End => SeekFrom::End(offset),
    };

    ok_or_return!(s.seek(seek_from), |pos| pos as i64, -1)
}

/// Writes data to the stream.
///
/// # Parameters
/// - `stream`: The stream to write to
/// - `data`: Buffer containing data to write (must not be NULL)
/// - `len`: Number of bytes to write
///
/// # Returns
/// - Number of bytes actually written (>= 0) on success
/// - -1 on error
///
/// # Example
/// ```c
/// const char* message = "Hello, World!";
/// isize bytes_written = cimpl_stream_write(stream, (const uint8_t*)message, strlen(message));
/// if (bytes_written < 0) {
///     fprintf(stderr, "Write error\n");
/// }
/// ```
#[no_mangle]
pub extern "C" fn cimpl_stream_write(
    stream: *mut CimplStream,
    data: *const u8,
    len: usize,
) -> isize {
    let s = deref_mut_or_return_neg!(stream, CimplStream);
    ptr_or_return_int!(data);

    let buf = unsafe { std::slice::from_raw_parts(data, len) };

    ok_or_return!(s.write(buf), |bytes_written| bytes_written as isize, -1)
}

/// Flushes the stream, ensuring all buffered data is written.
///
/// # Parameters
/// - `stream`: The stream to flush
///
/// # Returns
/// - 0 on success
/// - -1 on error
///
/// # Example
/// ```c
/// if (cimpl_stream_flush(stream) != 0) {
///     fprintf(stderr, "Flush error\n");
/// }
/// ```
#[no_mangle]
pub extern "C" fn cimpl_stream_flush(stream: *mut CimplStream) -> i32 {
    let s = deref_mut_or_return_neg!(stream, CimplStream);

    ok_or_return!(s.flush(), |_| 0, -1)
}

// ============================================================================
// Trait Implementations
// ============================================================================

impl Read for CimplStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.len() > isize::MAX as usize {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Read buffer is too large",
            ));
        }

        let bytes_read = unsafe { (self.reader)(self.context, buf.as_mut_ptr(), buf.len()) };

        if bytes_read < 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Read callback returned error",
            ));
        }

        Ok(bytes_read as usize)
    }
}

impl Seek for CimplStream {
    fn seek(&mut self, from: SeekFrom) -> std::io::Result<u64> {
        let (offset, mode) = match from {
            SeekFrom::Start(pos) => (pos as i64, CimplSeekMode::Start),
            SeekFrom::Current(pos) => (pos, CimplSeekMode::Current),
            SeekFrom::End(pos) => (pos, CimplSeekMode::End),
        };

        let new_pos = unsafe { (self.seeker)(self.context, offset, mode) };

        if new_pos < 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Seek callback returned error",
            ));
        }

        Ok(new_pos as u64)
    }
}

impl Write for CimplStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if buf.len() > isize::MAX as usize {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Write buffer is too large",
            ));
        }

        let bytes_written = unsafe { (self.writer)(self.context, buf.as_ptr(), buf.len()) };

        if bytes_written < 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Write callback returned error",
            ));
        }

        Ok(bytes_written as usize)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let result = unsafe { (self.flusher)(self.context) };

        if result != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Flush callback returned error",
            ));
        }

        Ok(())
    }
}

// ============================================================================
// Error Handling
// ============================================================================
//
// NOTE: Memory management (cimpl_free) is provided by the base cimpl library.
// No need to re-export it here.

/// Gets the last error message.
///
/// # Returns
/// - Pointer to a C string describing the last error, or NULL if no error
///
/// # Memory Management
/// The returned string must be freed with `cimpl_free()`.
///
/// # Example
/// ```c
/// if (cimpl_stream_read(stream, buffer, len) < 0) {
///     char* error = cimpl_stream_last_error();
///     if (error) {
///         fprintf(stderr, "Error: %s\n", error);
///         cimpl_free(error);
///     }
/// }
/// ```
#[no_mangle]
pub extern "C" fn cimpl_stream_last_error() -> *mut std::os::raw::c_char {
    match Error::last_message() {
        Some(msg) => cimpl::to_c_string(msg),
        None => std::ptr::null_mut(),
    }
}

/// Gets the last error code.
///
/// # Returns
/// - 0 if no error
/// - Error code corresponding to the error type
///
/// # Example
/// ```c
/// if (cimpl_stream_read(stream, buffer, len) < 0) {
///     int32_t code = cimpl_stream_error_code();
///     printf("Error code: %d\n", code);
/// }
/// ```
#[no_mangle]
pub extern "C" fn cimpl_stream_error_code() -> i32 {
    Error::last_code() as i32
}

/// Clears the last error.
///
/// This function can be called to clear the error state before making
/// a series of calls where you want to check for new errors.
#[no_mangle]
pub extern "C" fn cimpl_stream_clear_error() {
    Error::take_last();
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Simple memory buffer for testing
    struct MemoryBuffer {
        data: Mutex<Vec<u8>>,
        position: Mutex<usize>,
    }

    impl MemoryBuffer {
        fn new() -> Self {
            Self {
                data: Mutex::new(Vec::new()),
                position: Mutex::new(0),
            }
        }

        fn with_data(data: Vec<u8>) -> Self {
            Self {
                data: Mutex::new(data),
                position: Mutex::new(0),
            }
        }

        unsafe extern "C" fn read_callback(
            ctx: *mut CimplStreamContext,
            data: *mut u8,
            len: usize,
        ) -> isize {
            let buf = &*(ctx as *const MemoryBuffer);
            let mut pos = buf.position.lock().unwrap();
            let buffer = buf.data.lock().unwrap();
            
            let available = buffer.len().saturating_sub(*pos);
            let to_read = available.min(len);
            
            if to_read > 0 {
                let slice = std::slice::from_raw_parts_mut(data, to_read);
                slice.copy_from_slice(&buffer[*pos..*pos + to_read]);
                *pos += to_read;
            }
            
            to_read as isize
        }

        unsafe extern "C" fn seek_callback(
            ctx: *mut CimplStreamContext,
            offset: i64,
            mode: CimplSeekMode,
        ) -> i64 {
            let buf = &*(ctx as *const MemoryBuffer);
            let mut pos = buf.position.lock().unwrap();
            let buffer = buf.data.lock().unwrap();
            
            let new_pos = match mode {
                CimplSeekMode::Start => offset.max(0) as usize,
                CimplSeekMode::Current => (*pos as i64 + offset).max(0) as usize,
                CimplSeekMode::End => (buffer.len() as i64 + offset).max(0) as usize,
            };
            
            *pos = new_pos;
            new_pos as i64
        }

        unsafe extern "C" fn write_callback(
            ctx: *mut CimplStreamContext,
            data: *const u8,
            len: usize,
        ) -> isize {
            let buf = &*(ctx as *const MemoryBuffer);
            let mut pos = buf.position.lock().unwrap();
            let mut buffer = buf.data.lock().unwrap();
            
            let slice = std::slice::from_raw_parts(data, len);
            
            // Extend buffer if needed
            if *pos + len > buffer.len() {
                buffer.resize(*pos + len, 0);
            }
            
            buffer[*pos..*pos + len].copy_from_slice(slice);
            *pos += len;
            
            len as isize
        }

        unsafe extern "C" fn flush_callback(_ctx: *mut CimplStreamContext) -> i32 {
            0 // Nothing to flush for memory buffer
        }
    }

    #[test]
    fn test_stream_creation() {
        let buffer = Box::new(MemoryBuffer::new());
        let ctx = Box::into_raw(buffer) as *mut CimplStreamContext;
        
        let stream = cimpl_stream_new(
            ctx,
            MemoryBuffer::read_callback,
            MemoryBuffer::seek_callback,
            MemoryBuffer::write_callback,
            MemoryBuffer::flush_callback,
        );
        
        assert!(!stream.is_null());
        
        unsafe {
            cimpl::cimpl_free(stream as *mut std::ffi::c_void);
            let _ = Box::from_raw(ctx as *mut MemoryBuffer);
        }
    }

    #[test]
    fn test_stream_write_and_read() {
        let buffer = Box::new(MemoryBuffer::new());
        let ctx = Box::into_raw(buffer) as *mut CimplStreamContext;
        
        let stream = cimpl_stream_new(
            ctx,
            MemoryBuffer::read_callback,
            MemoryBuffer::seek_callback,
            MemoryBuffer::write_callback,
            MemoryBuffer::flush_callback,
        );
        
        // Write some data
        let write_data = b"Hello, World!";
        let bytes_written = cimpl_stream_write(stream, write_data.as_ptr(), write_data.len());
        assert_eq!(bytes_written, write_data.len() as isize);
        
        // Seek back to start
        let pos = cimpl_stream_seek(stream, 0, CimplSeekMode::Start);
        assert_eq!(pos, 0);
        
        // Read it back
        let mut read_buf = [0u8; 20];
        let bytes_read = cimpl_stream_read(stream, read_buf.as_mut_ptr(), read_buf.len());
        assert_eq!(bytes_read, write_data.len() as isize);
        assert_eq!(&read_buf[..bytes_read as usize], write_data);
        
        unsafe {
            cimpl::cimpl_free(stream as *mut std::ffi::c_void);
            let _ = Box::from_raw(ctx as *mut MemoryBuffer);
        }
    }

    #[test]
    fn test_stream_seek_operations() {
        let buffer = Box::new(MemoryBuffer::with_data(b"0123456789".to_vec()));
        let ctx = Box::into_raw(buffer) as *mut CimplStreamContext;
        
        let stream = cimpl_stream_new(
            ctx,
            MemoryBuffer::read_callback,
            MemoryBuffer::seek_callback,
            MemoryBuffer::write_callback,
            MemoryBuffer::flush_callback,
        );
        
        // Seek to position 5
        let pos = cimpl_stream_seek(stream, 5, CimplSeekMode::Start);
        assert_eq!(pos, 5);
        
        // Read from position 5
        let mut buf = [0u8; 3];
        let bytes_read = cimpl_stream_read(stream, buf.as_mut_ptr(), buf.len());
        assert_eq!(bytes_read, 3);
        assert_eq!(&buf, b"567");
        
        // Seek backward 5 bytes from current (should be at position 3)
        let pos = cimpl_stream_seek(stream, -5, CimplSeekMode::Current);
        assert_eq!(pos, 3);
        
        // Read from position 3
        let bytes_read = cimpl_stream_read(stream, buf.as_mut_ptr(), buf.len());
        assert_eq!(bytes_read, 3);
        assert_eq!(&buf, b"345");
        
        // Seek to 2 bytes before end
        let pos = cimpl_stream_seek(stream, -2, CimplSeekMode::End);
        assert_eq!(pos, 8);
        
        // Read last 2 bytes
        let bytes_read = cimpl_stream_read(stream, buf.as_mut_ptr(), 2);
        assert_eq!(bytes_read, 2);
        assert_eq!(&buf[..2], b"89");
        
        unsafe {
            cimpl::cimpl_free(stream as *mut std::ffi::c_void);
            let _ = Box::from_raw(ctx as *mut MemoryBuffer);
        }
    }

    #[test]
    fn test_stream_flush() {
        let buffer = Box::new(MemoryBuffer::new());
        let ctx = Box::into_raw(buffer) as *mut CimplStreamContext;
        
        let stream = cimpl_stream_new(
            ctx,
            MemoryBuffer::read_callback,
            MemoryBuffer::seek_callback,
            MemoryBuffer::write_callback,
            MemoryBuffer::flush_callback,
        );
        
        let write_data = b"test data";
        cimpl_stream_write(stream, write_data.as_ptr(), write_data.len());
        
        // Flush should succeed
        let result = cimpl_stream_flush(stream);
        assert_eq!(result, 0);
        
        unsafe {
            cimpl::cimpl_free(stream as *mut std::ffi::c_void);
            let _ = Box::from_raw(ctx as *mut MemoryBuffer);
        }
    }

    #[test]
    fn test_null_stream_handling() {
        let null_stream: *mut CimplStream = std::ptr::null_mut();
        
        // All operations should return error (-1) for null stream
        let mut buf = [0u8; 10];
        assert_eq!(cimpl_stream_read(null_stream, buf.as_mut_ptr(), 10), -1);
        assert_eq!(cimpl_stream_write(null_stream, buf.as_ptr(), 10), -1);
        assert_eq!(cimpl_stream_seek(null_stream, 0, CimplSeekMode::Start), -1);
        assert_eq!(cimpl_stream_flush(null_stream), -1);
    }

    #[test]
    fn test_null_buffer_handling() {
        let buffer = Box::new(MemoryBuffer::new());
        let ctx = Box::into_raw(buffer) as *mut CimplStreamContext;
        
        let stream = cimpl_stream_new(
            ctx,
            MemoryBuffer::read_callback,
            MemoryBuffer::seek_callback,
            MemoryBuffer::write_callback,
            MemoryBuffer::flush_callback,
        );
        
        // Null buffer should return error
        assert_eq!(cimpl_stream_read(stream, std::ptr::null_mut(), 10), -1);
        assert_eq!(cimpl_stream_write(stream, std::ptr::null(), 10), -1);
        
        unsafe {
            cimpl::cimpl_free(stream as *mut std::ffi::c_void);
            let _ = Box::from_raw(ctx as *mut MemoryBuffer);
        }
    }

    #[test]
    fn test_error_messages() {
        // Clear any previous error
        cimpl_stream_clear_error();
        
        // Try to read from null stream (should set error)
        let null_stream: *mut CimplStream = std::ptr::null_mut();
        let mut buf = [0u8; 10];
        let result = cimpl_stream_read(null_stream, buf.as_mut_ptr(), 10);
        assert_eq!(result, -1);
        
        // Should have an error message
        let error_msg = cimpl_stream_last_error();
        assert!(!error_msg.is_null());
        
        cimpl::cimpl_free(error_msg as *mut std::ffi::c_void);
        
        // Error code should be non-zero
        assert_ne!(cimpl_stream_error_code(), 0);
        
        // Clear and verify
        cimpl_stream_clear_error();
        assert_eq!(cimpl_stream_error_code(), 0);
    }
}

