//! # Value Converter Library
//!
//! A Rust library for converting between different value representations.
//! This is a **standard Rust API** - no FFI concerns here!
//!
//! The FFI bindings are in `ffi.rs` - that's where cimpl comes in.

use thiserror::Error as ThisError;

// ============================================================================
// Error Type
// ============================================================================

/// Errors that can occur during conversion
#[derive(ThisError, Debug)]
pub enum Error {
    #[error("value out of range: {0}")]
    OutOfRange(String),
    
    #[error("invalid UTF-8: {0}")]
    InvalidUtf8(String),
    
    #[error("invalid hex: {0}")]
    InvalidHex(String),
    
    #[error("buffer too large: got {got} bytes, max {max}")]
    BufferTooLarge { got: usize, max: usize },
    
    #[error("empty value")]
    EmptyValue,
}

pub type Result<T> = std::result::Result<T, Error>;

// ============================================================================
// Value Converter API
// ============================================================================

/// Maximum buffer size (keeps values in reasonable numeric range)
pub const MAX_BUFFER_SIZE: usize = 8;

/// A value that can be represented in multiple formats
/// 
/// Internally stores bytes, can convert to/from:
/// - Signed integers (i32, i64)
/// - Unsigned integers (u32, u64)
/// - Byte arrays
/// - UTF-8 strings
/// - Hex strings
pub struct ValueConverter {
    bytes: Vec<u8>,
}

impl ValueConverter {
    /// Create from signed 32-bit integer (little-endian)
    pub fn from_i32(value: i32) -> Self {
        Self {
            bytes: value.to_le_bytes().to_vec(),
        }
    }
    
    /// Create from unsigned 32-bit integer (little-endian)
    pub fn from_u32(value: u32) -> Self {
        Self {
            bytes: value.to_le_bytes().to_vec(),
        }
    }
    
    /// Create from signed 64-bit integer (little-endian)
    pub fn from_i64(value: i64) -> Self {
        Self {
            bytes: value.to_le_bytes().to_vec(),
        }
    }
    
    /// Create from unsigned 64-bit integer (little-endian)
    pub fn from_u64(value: u64) -> Self {
        Self {
            bytes: value.to_le_bytes().to_vec(),
        }
    }
    
    /// Create from byte array
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.is_empty() {
            return Err(Error::EmptyValue);
        }
        if bytes.len() > MAX_BUFFER_SIZE {
            return Err(Error::BufferTooLarge {
                got: bytes.len(),
                max: MAX_BUFFER_SIZE,
            });
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }
    
    /// Create from UTF-8 string
    pub fn from_string(s: &str) -> Result<Self> {
        let bytes = s.as_bytes();
        if bytes.is_empty() {
            return Err(Error::EmptyValue);
        }
        if bytes.len() > MAX_BUFFER_SIZE {
            return Err(Error::BufferTooLarge {
                got: bytes.len(),
                max: MAX_BUFFER_SIZE,
            });
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }
    
    /// Create from hex string
    pub fn from_hex(hex: &str) -> Result<Self> {
        if hex.is_empty() {
            return Err(Error::EmptyValue);
        }
        if hex.len() % 2 != 0 {
            return Err(Error::InvalidHex("odd length".to_string()));
        }
        
        let mut bytes = Vec::new();
        for i in (0..hex.len()).step_by(2) {
            let byte_str = &hex[i..i + 2];
            let byte = u8::from_str_radix(byte_str, 16)
                .map_err(|_| Error::InvalidHex(format!("invalid byte: {}", byte_str)))?;
            bytes.push(byte);
        }
        
        if bytes.len() > MAX_BUFFER_SIZE {
            return Err(Error::BufferTooLarge {
                got: bytes.len(),
                max: MAX_BUFFER_SIZE,
            });
        }
        
        Ok(Self { bytes })
    }
    
    /// Convert to signed 32-bit integer (little-endian)
    pub fn to_i32(&self) -> Result<i32> {
        if self.bytes.len() != 4 {
            return Err(Error::OutOfRange(format!(
                "need exactly 4 bytes for i32, got {}",
                self.bytes.len()
            )));
        }
        let arr: [u8; 4] = self.bytes[..4].try_into().unwrap();
        Ok(i32::from_le_bytes(arr))
    }
    
    /// Convert to unsigned 32-bit integer (little-endian)
    pub fn to_u32(&self) -> Result<u32> {
        if self.bytes.len() != 4 {
            return Err(Error::OutOfRange(format!(
                "need exactly 4 bytes for u32, got {}",
                self.bytes.len()
            )));
        }
        let arr: [u8; 4] = self.bytes[..4].try_into().unwrap();
        Ok(u32::from_le_bytes(arr))
    }
    
    /// Convert to signed 64-bit integer (little-endian)
    pub fn to_i64(&self) -> Result<i64> {
        if self.bytes.len() != 8 {
            return Err(Error::OutOfRange(format!(
                "need exactly 8 bytes for i64, got {}",
                self.bytes.len()
            )));
        }
        let arr: [u8; 8] = self.bytes[..8].try_into().unwrap();
        Ok(i64::from_le_bytes(arr))
    }
    
    /// Convert to unsigned 64-bit integer (little-endian)
    pub fn to_u64(&self) -> Result<u64> {
        if self.bytes.len() != 8 {
            return Err(Error::OutOfRange(format!(
                "need exactly 8 bytes for u64, got {}",
                self.bytes.len()
            )));
        }
        let arr: [u8; 8] = self.bytes[..8].try_into().unwrap();
        Ok(u64::from_le_bytes(arr))
    }
    
    /// Get raw bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    
    /// Convert to UTF-8 string
    pub fn to_string(&self) -> Result<String> {
        String::from_utf8(self.bytes.clone())
            .map_err(|e| Error::InvalidUtf8(e.to_string()))
    }
    
    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        self.bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
    
    /// Get the size in bytes
    pub fn len(&self) -> usize {
        self.bytes.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

// ============================================================================
// FFI Module
// ============================================================================

#[cfg(feature = "ffi")]
pub mod ffi;
