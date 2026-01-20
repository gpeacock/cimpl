/**
 * Node.js FFI bindings for cimpl-uuid library using Koffi
 * 
 * Koffi is a modern, maintained FFI library that works with current Node.js versions.
 * Unlike ffi-napi, it compiles successfully on Node.js 18+ and 23+.
 */

const koffi = require('koffi');
const path = require('path');
const os = require('os');

// Determine library extension based on platform
const libExt = os.platform() === 'darwin' ? 'dylib' : 
               os.platform() === 'win32' ? 'dll' : 'so';

// Load the library
const libPath = path.join(__dirname, `../../target/release/libcimple_uuid.${libExt}`);
const lib = koffi.load(libPath);

// Error codes (from header)
const ERROR_OK = 0;
const ERROR_NULL_PARAMETER = 1;
const ERROR_STRING_TOO_LONG = 2;
const ERROR_INVALID_HANDLE = 3;
const ERROR_WRONG_HANDLE_TYPE = 4;
const ERROR_OTHER = 5;
const ERROR_UUID_PARSE_ERROR = 100;

// Define opaque UUID pointer type
const UuidPtr = koffi.pointer(koffi.opaque('Uuid'));

// Define C functions
const uuid_new_v4 = lib.func('uuid_new_v4', UuidPtr, []);
const uuid_new_v7 = lib.func('uuid_new_v7', UuidPtr, []);
const uuid_parse = lib.func('uuid_parse', UuidPtr, ['str']);
const uuid_nil = lib.func('uuid_nil', UuidPtr, []);
const uuid_max = lib.func('uuid_max', UuidPtr, []);

const uuid_to_string = lib.func('uuid_to_string', 'str', [UuidPtr]);
const uuid_to_urn = lib.func('uuid_to_urn', 'str', [UuidPtr]);
const uuid_as_bytes = lib.func('uuid_as_bytes', koffi.pointer('uint8'), [UuidPtr]);

const uuid_is_nil = lib.func('uuid_is_nil', 'bool', [UuidPtr]);
const uuid_is_max = lib.func('uuid_is_max', 'bool', [UuidPtr]);

const uuid_compare = lib.func('uuid_compare', 'int32', [UuidPtr, UuidPtr]);
const uuid_equals = lib.func('uuid_equals', 'bool', [UuidPtr, UuidPtr]);

const uuid_error_code = lib.func('uuid_error_code', 'int32', []);
const uuid_last_error = lib.func('uuid_last_error', 'str', []);
const uuid_clear_error = lib.func('uuid_clear_error', 'void', []);

const cimple_free = lib.func('cimple_free', 'int32', [koffi.pointer('void')]);

// Error classes
class UuidError extends Error {
  constructor(code, message) {
    super(`[${code}] ${message}`);
    this.name = 'UuidError';
    this.code = code;
  }
}

class NullParameterError extends UuidError {
  constructor(message) {
    super(ERROR_NULL_PARAMETER, message);
    this.name = 'NullParameterError';
  }
}

class InvalidHandleError extends UuidError {
  constructor(message) {
    super(ERROR_INVALID_HANDLE, message);
    this.name = 'InvalidHandleError';
  }
}

class WrongHandleTypeError extends UuidError {
  constructor(message) {
    super(ERROR_WRONG_HANDLE_TYPE, message);
    this.name = 'WrongHandleTypeError';
  }
}

class ParseError extends UuidError {
  constructor(message) {
    super(ERROR_UUID_PARSE_ERROR, message);
    this.name = 'ParseError';
  }
}

class OtherError extends UuidError {
  constructor(message) {
    super(ERROR_OTHER, message);
    this.name = 'OtherError';
  }
}

// Map error codes to error classes
const ERROR_CLASSES = {
  [ERROR_NULL_PARAMETER]: NullParameterError,
  [ERROR_INVALID_HANDLE]: InvalidHandleError,
  [ERROR_WRONG_HANDLE_TYPE]: WrongHandleTypeError,
  [ERROR_UUID_PARSE_ERROR]: ParseError,
  [ERROR_OTHER]: OtherError,
};

/**
 * Check for errors and throw appropriate exception
 */
function checkError() {
  const code = uuid_error_code();
  if (code !== ERROR_OK) {
    const message = uuid_last_error() || 'Unknown error';
    uuid_clear_error();
    
    const ErrorClass = ERROR_CLASSES[code] || UuidError;
    throw new ErrorClass(message);
  }
}

/**
 * UUID class - JavaScript wrapper for UUID objects
 */
class Uuid {
  constructor(handle) {
    if (!handle || handle === null) {
      checkError();
      throw new Error('Failed to create UUID');
    }
    this._handle = handle;
    this._freed = false;
  }

  /**
   * Generate a random UUID (version 4)
   */
  static v4() {
    return new Uuid(uuid_new_v4());
  }

  /**
   * Generate a timestamp-based UUID (version 7)
   */
  static v7() {
    return new Uuid(uuid_new_v7());
  }

  /**
   * Parse a UUID from a string
   * @throws {ParseError} If the string is not a valid UUID
   */
  static parse(str) {
    const handle = uuid_parse(str);
    if (!handle || handle === null) {
      checkError();
    }
    return new Uuid(handle);
  }

  /**
   * Create a nil UUID (all zeros)
   */
  static nil() {
    return new Uuid(uuid_nil());
  }

  /**
   * Create a max UUID (all ones)
   */
  static max() {
    return new Uuid(uuid_max());
  }

  /**
   * Convert UUID to string (hyphenated format)
   */
  toString() {
    if (this._freed) {
      throw new Error('UUID has been freed');
    }
    
    const str = uuid_to_string(this._handle);
    if (!str) {
      checkError();
    }
    return str;
  }

  /**
   * Convert UUID to URN format
   */
  toUrn() {
    if (this._freed) {
      throw new Error('UUID has been freed');
    }
    
    const urn = uuid_to_urn(this._handle);
    if (!urn) {
      checkError();
    }
    return urn;
  }

  /**
   * Get UUID as bytes (Buffer of 16 bytes)
   */
  toBytes() {
    if (this._freed) {
      throw new Error('UUID has been freed');
    }
    
    const bytesPtr = uuid_as_bytes(this._handle);
    if (!bytesPtr || bytesPtr === null) {
      checkError();
    }
    
    // Read 16 bytes
    const buffer = Buffer.from(koffi.decode(bytesPtr, 'uint8', 16));
    
    // Note: We should free bytesPtr, but koffi handles strings differently
    // The uuid_as_bytes returns a pointer we need to free
    cimple_free(bytesPtr);
    
    return buffer;
  }

  /**
   * Check if UUID is nil (all zeros)
   */
  isNil() {
    if (this._freed) {
      throw new Error('UUID has been freed');
    }
    return uuid_is_nil(this._handle);
  }

  /**
   * Check if UUID is max (all ones)
   */
  isMax() {
    if (this._freed) {
      throw new Error('UUID has been freed');
    }
    return uuid_is_max(this._handle);
  }

  /**
   * Compare with another UUID
   * @returns {number} -1 if this < other, 0 if equal, 1 if this > other
   */
  compare(other) {
    if (this._freed || other._freed) {
      throw new Error('UUID has been freed');
    }
    return uuid_compare(this._handle, other._handle);
  }

  /**
   * Check equality with another UUID
   */
  equals(other) {
    if (this._freed || other._freed) {
      throw new Error('UUID has been freed');
    }
    return uuid_equals(this._handle, other._handle);
  }

  /**
   * Free the UUID manually (otherwise happens on GC)
   */
  free() {
    if (!this._freed && this._handle) {
      cimple_free(this._handle);
      this._freed = true;
    }
  }

  /**
   * Get JSON representation
   */
  toJSON() {
    return this.toString();
  }

  /**
   * For console.log()
   */
  [Symbol.for('nodejs.util.inspect.custom')]() {
    return `Uuid('${this.toString()}')`;
  }
}

// Export
module.exports = {
  Uuid,
  v4: () => Uuid.v4(),
  v7: () => Uuid.v7(),
  parse: (str) => Uuid.parse(str),
  nil: () => Uuid.nil(),
  max: () => Uuid.max(),
  
  // Error classes
  UuidError,
  NullParameterError,
  InvalidHandleError,
  WrongHandleTypeError,
  ParseError,
  OtherError,
  
  // Error codes
  ERROR_OK,
  ERROR_NULL_PARAMETER,
  ERROR_STRING_TOO_LONG,
  ERROR_INVALID_HANDLE,
  ERROR_WRONG_HANDLE_TYPE,
  ERROR_OTHER,
  ERROR_UUID_PARSE_ERROR,
};
