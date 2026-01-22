//! # Cimpl Reference Example - Secret Message Processor
//!
//! This is a comprehensive reference implementation that exercises ALL cimpl patterns.
//! Use this as the canonical example when creating new FFI bindings.
//!
//! ## What This Tests
//!
//! - ✅ String parameters (C → Rust)
//! - ✅ String results (Rust → C)
//! - ✅ Byte arrays in/out
//! - ✅ Result<T, E> with custom error enum
//! - ✅ Option<T> for validation
//! - ✅ Struct lifecycle (create, modify, query, destroy)
//! - ✅ Numeric parameters
//! - ✅ Boolean returns
//! - ✅ Error handling (last_error, clear_error)
//! - ✅ Memory management (tracked allocations)
//!
//! ## Domain: Secret Messages
//!
//! We encode/decode secret messages with various silly algorithms:
//! - ROT13 encoding
//! - Reverse text
//! - Vowel removal
//! - Character substitution
//! - Hex encoding/decoding

use std::collections::HashMap;
use std::os::raw::c_char;

use cimpl::{
    box_tracked, cimpl_free, cstr_or_return_null,
    deref_or_return_null, error::CimplError, ok_or_return_false, ok_or_return_null, 
    option_to_c_string, to_c_bytes, to_c_string, 
    Error,
};

// ============================================================================
// Error Type Definitions (Step 1: Define error code enum)
// ============================================================================

/// Error codes for Secret Message Processor
///
/// This example defines its own error types (not from external crate).
/// We use this to demonstrate the full error mapping pattern.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretError {
    // Our custom errors (100+)
    InvalidHex = 100,      // Hex decoding failed
    InvalidFormat = 101,   // Message format invalid
    TooShort = 102,        // Message too short
    TooLong = 103,         // Message too long
}

// ============================================================================
// Error Mapping (Step 2: Implement From trait for automatic conversion)
// ============================================================================

/// Our internal error type for operations that can fail
#[derive(Debug)]
pub enum ProcessError {
    InvalidHex(String),
    InvalidFormat(String),
    TooShort(usize, usize), // got, expected
    TooLong(usize, usize),  // got, max
}

impl From<ProcessError> for Error {
    fn from(e: ProcessError) -> Self {
        let (code, name, detail) = match e {
            ProcessError::InvalidHex(ref s) => (SecretError::InvalidHex, "InvalidHex", s.clone()),
            ProcessError::InvalidFormat(ref s) => (SecretError::InvalidFormat, "InvalidFormat", s.clone()),
            ProcessError::TooShort(got, expected) => (
                SecretError::TooShort, 
                "TooShort", 
                format!("got {}, expected at least {}", got, expected)
            ),
            ProcessError::TooLong(got, max) => (
                SecretError::TooLong, 
                "TooLong", 
                format!("got {}, max {}", got, max)
            ),
        };
        Error::new(code as i32, format!("{}: {}", name, detail))
    }
}

// ============================================================================
// Data Structures
// ============================================================================

/// A secret message with metadata
pub struct SecretMessage {
    content: String,
    encoding: String,
    metadata: HashMap<String, String>,
}

/// Statistics about a message
#[repr(C)]
pub struct MessageStats {
    pub length: usize,
    pub word_count: usize,
    pub vowel_count: usize,
    pub consonant_count: usize,
}

// ============================================================================
// Core Encoding/Decoding Logic (Pure Rust)
// ============================================================================

/// ROT13 encoding - shifts letters by 13 positions
fn rot13(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'a'..='m' | 'A'..='M' => ((c as u8) + 13) as char,
            'n'..='z' | 'N'..='Z' => ((c as u8) - 13) as char,
            _ => c,
        })
        .collect()
}

/// Remove all vowels from text
fn remove_vowels(input: &str) -> String {
    input
        .chars()
        .filter(|c| !matches!(c.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u'))
        .collect()
}

/// Substitute characters using a simple cipher
fn substitute(input: &str, from: char, to: char) -> String {
    input.replace(from, &to.to_string())
}

/// Encode string to hex
fn to_hex(input: &str) -> String {
    input.bytes().map(|b| format!("{:02x}", b)).collect()
}

/// Decode hex to string
fn from_hex(hex: &str) -> Result<String, ProcessError> {
    // Must be even length
    if hex.len() % 2 != 0 {
        return Err(ProcessError::InvalidHex("odd length".to_string()));
    }
    
    let mut bytes = Vec::new();
    for i in (0..hex.len()).step_by(2) {
        let byte_str = &hex[i..i + 2];
        let byte = u8::from_str_radix(byte_str, 16)
            .map_err(|_| ProcessError::InvalidHex(byte_str.to_string()))?;
        bytes.push(byte);
    }
    
    String::from_utf8(bytes)
        .map_err(|_| ProcessError::InvalidFormat("invalid UTF-8".to_string()))
}

/// Validate message length
fn validate_length(input: &str, min: usize, max: usize) -> Result<(), ProcessError> {
    let len = input.len();
    if len < min {
        Err(ProcessError::TooShort(len, min))
    } else if len > max {
        Err(ProcessError::TooLong(len, max))
    } else {
        Ok(())
    }
}

/// Count vowels in text
fn count_vowels(input: &str) -> usize {
    input
        .chars()
        .filter(|c| matches!(c.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u'))
        .count()
}

/// Count consonants in text
fn count_consonants(input: &str) -> usize {
    input
        .chars()
        .filter(|c| c.is_ascii_alphabetic() && !matches!(c.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u'))
        .count()
}

/// Count words (split by whitespace)
fn count_words(input: &str) -> usize {
    input.split_whitespace().count()
}

// ============================================================================
// FFI Functions: String In → String Out
// ============================================================================

/// Encodes text using ROT13 cipher
/// Tests: cstr_or_return_null!, to_c_string!
#[no_mangle]
pub extern "C" fn secret_rot13(input: *const c_char) -> *mut c_char {
    let text = cstr_or_return_null!(input);
    to_c_string(rot13(&text))
}

/// Reverses the input string
/// Tests: cstr_or_return_null!, to_c_string!
#[no_mangle]
pub extern "C" fn secret_reverse(input: *const c_char) -> *mut c_char {
    let text = cstr_or_return_null!(input);
    to_c_string(text.chars().rev().collect::<String>())
}

/// Removes all vowels from text
/// Tests: cstr_or_return_null!, to_c_string!
#[no_mangle]
pub extern "C" fn secret_remove_vowels(input: *const c_char) -> *mut c_char {
    let text = cstr_or_return_null!(input);
    to_c_string(remove_vowels(&text))
}

/// Substitutes one character for another
/// Tests: cstr_or_return_null!, to_c_string!
#[no_mangle]
pub extern "C" fn secret_substitute(
    input: *const c_char,
    from: c_char,
    to: c_char,
) -> *mut c_char {
    let text = cstr_or_return_null!(input);
    to_c_string(substitute(&text, from as u8 as char, to as u8 as char))
}

/// Converts text to uppercase
/// Tests: cstr_or_return_null!, to_c_string!
#[no_mangle]
pub extern "C" fn secret_uppercase(input: *const c_char) -> *mut c_char {
    let text = cstr_or_return_null!(input);
    to_c_string(text.to_uppercase())
}

// ============================================================================
// FFI Functions: String In → String Out (with Result/Error)
// ============================================================================

/// Encodes string to hex
/// Tests: cstr_or_return_null!, to_c_string!
#[no_mangle]
pub extern "C" fn secret_to_hex(input: *const c_char) -> *mut c_char {
    let text = cstr_or_return_null!(input);
    to_c_string(to_hex(&text))
}

/// Decodes hex string to text (can fail!)
/// Tests: cstr_or_return_null!, ok_or_return_null! with automatic From conversion
#[no_mangle]
pub extern "C" fn secret_from_hex(hex: *const c_char) -> *mut c_char {
    let hex_str = cstr_or_return_null!(hex);
    let decoded = ok_or_return_null!(from_hex(&hex_str));
    to_c_string(decoded)
}

// ============================================================================
// FFI Functions: Validation (Option → bool)
// ============================================================================

/// Validates that message length is within bounds
/// Tests: cstr_or_return! with false, ok_or_return_false! with SecretError
#[no_mangle]
pub extern "C" fn secret_validate_length(
    input: *const c_char,
    min_len: usize,
    max_len: usize,
) -> bool {
    use cimpl::cstr_or_return;
    let text = cstr_or_return!(input, false);
    ok_or_return_false!(validate_length(&text, min_len, max_len));
    true
}

/// Checks if text contains only ASCII characters
/// Tests: cstr_or_return! with false, simple validation
#[no_mangle]
pub extern "C" fn secret_is_ascii(input: *const c_char) -> bool {
    use cimpl::cstr_or_return;
    let text = cstr_or_return!(input, false);
    text.is_ascii()
}

/// Checks if text is valid hex
/// Tests: cstr_or_return! with false, validation logic
#[no_mangle]
pub extern "C" fn secret_is_valid_hex(input: *const c_char) -> bool {
    use cimpl::cstr_or_return;
    let text = cstr_or_return!(input, false);
    text.len() % 2 == 0 && text.chars().all(|c| c.is_ascii_hexdigit())
}

// ============================================================================
// FFI Functions: Counting (String In → Number Out)
// ============================================================================

/// Counts characters in string
/// Tests: cstr_or_return! with 0 on error
#[no_mangle]
pub extern "C" fn secret_count_chars(input: *const c_char) -> usize {
    use cimpl::cstr_or_return;
    let text = cstr_or_return!(input, 0);
    text.chars().count()
}

/// Counts vowels in string
/// Tests: cstr_or_return! with 0 on error
#[no_mangle]
pub extern "C" fn secret_count_vowels(input: *const c_char) -> usize {
    use cimpl::cstr_or_return;
    let text = cstr_or_return!(input, 0);
    count_vowels(&text)
}

/// Counts consonants in string
/// Tests: cstr_or_return! with 0 on error
#[no_mangle]
pub extern "C" fn secret_count_consonants(input: *const c_char) -> usize {
    use cimpl::cstr_or_return;
    let text = cstr_or_return!(input, 0);
    count_consonants(&text)
}

/// Counts words in string
/// Tests: cstr_or_return! with 0 on error
#[no_mangle]
pub extern "C" fn secret_count_words(input: *const c_char) -> usize {
    use cimpl::cstr_or_return;
    let text = cstr_or_return!(input, 0);
    count_words(&text)
}

// ============================================================================
// FFI Functions: Byte Array Operations
// ============================================================================

/// Converts string to byte array
/// Tests: cstr_or_return_null!, to_c_bytes!, returning length via out parameter
#[no_mangle]
pub extern "C" fn secret_to_bytes(input: *const c_char, out_len: *mut usize) -> *const u8 {
    let text = cstr_or_return_null!(input);
    let bytes = text.as_bytes().to_vec();
    
    // Set output length if pointer provided
    if !out_len.is_null() {
        unsafe { *out_len = bytes.len(); }
    }
    
    to_c_bytes(bytes)
}

/// Converts byte array to string (if valid UTF-8)
/// Tests: byte array handling, ok_or_return_null! with Result
#[no_mangle]
pub extern "C" fn secret_from_bytes(data: *const u8, len: usize) -> *mut c_char {
    if data.is_null() {
        Error::from(CimplError::NullParameter("data".to_string())).set_last();
        return std::ptr::null_mut();
    }
    
    let bytes = unsafe { std::slice::from_raw_parts(data, len) };
    let text = ok_or_return_null!(
        String::from_utf8(bytes.to_vec())
            .map_err(|_| ProcessError::InvalidFormat("invalid UTF-8".to_string()))
    );
    to_c_string(text)
}

// ============================================================================
// FFI Functions: Struct Operations (SecretMessage)
// ============================================================================

/// Creates a new secret message
/// Tests: cstr_or_return_null!, box_tracked!
#[no_mangle]
pub extern "C" fn message_new(content: *const c_char, encoding: *const c_char) -> *mut SecretMessage {
    let content_str = cstr_or_return_null!(content);
    let encoding_str = cstr_or_return_null!(encoding);
    
    let msg = SecretMessage {
        content: content_str,
        encoding: encoding_str,
        metadata: HashMap::new(),
    };
    
    box_tracked!(msg)
}

/// Gets the content of a message
/// Tests: deref_or_return_null!, to_c_string!
#[no_mangle]
pub extern "C" fn message_get_content(msg: *mut SecretMessage) -> *mut c_char {
    let message = deref_or_return_null!(msg, SecretMessage);
    to_c_string(message.content.clone())
}

/// Gets the encoding of a message
/// Tests: deref_or_return_null!, to_c_string!
#[no_mangle]
pub extern "C" fn message_get_encoding(msg: *mut SecretMessage) -> *mut c_char {
    let message = deref_or_return_null!(msg, SecretMessage);
    to_c_string(message.encoding.clone())
}

/// Sets metadata on a message
/// Tests: deref_mut_or_return!, cstr_or_return! with false
#[no_mangle]
pub extern "C" fn message_set_metadata(
    msg: *mut SecretMessage,
    key: *const c_char,
    value: *const c_char,
) -> bool {
    use cimpl::{cstr_or_return, deref_mut_or_return};
    
    let message = deref_mut_or_return!(msg, SecretMessage, false);
    let key_str = cstr_or_return!(key, false);
    let value_str = cstr_or_return!(value, false);
    
    message.metadata.insert(key_str, value_str);
    true
}

/// Gets metadata from a message (returns NULL if not found)
/// Tests: deref_or_return_null!, cstr_or_return_null!, option_to_c_string!
#[no_mangle]
pub extern "C" fn message_get_metadata(
    msg: *mut SecretMessage,
    key: *const c_char,
) -> *mut c_char {
    let message = deref_or_return_null!(msg, SecretMessage);
    let key_str = cstr_or_return_null!(key);
    
    option_to_c_string!(message.metadata.get(&key_str).cloned())
}

/// Gets statistics about the message
/// Tests: deref_or_return_null!, struct return by value
#[no_mangle]
pub extern "C" fn message_get_stats(msg: *mut SecretMessage) -> *mut MessageStats {
    let message = deref_or_return_null!(msg, SecretMessage);
    
    let stats = MessageStats {
        length: message.content.len(),
        word_count: count_words(&message.content),
        vowel_count: count_vowels(&message.content),
        consonant_count: count_consonants(&message.content),
    };
    
    box_tracked!(stats)
}

// ============================================================================
// FFI Functions: Error Handling
// ============================================================================

/// Gets the error code of the last error (0 if no error)
#[no_mangle]
pub extern "C" fn secret_error_code() -> i32 {
    Error::last_code()
}

/// Gets the error message of the last error (NULL if no error)
/// Caller must free the returned string with secret_free()
#[no_mangle]
pub extern "C" fn secret_last_error() -> *mut c_char {
    option_to_c_string!(Error::last_message())
}

// ============================================================================
// FFI Functions: Memory Management
// ============================================================================

/// Frees any memory allocated by this library
/// Safe to call with NULL pointer
/// Returns 0 on success, -1 on error
#[no_mangle]
pub extern "C" fn secret_free(ptr: *mut std::ffi::c_void) -> i32 {
    cimpl_free(ptr)
}
