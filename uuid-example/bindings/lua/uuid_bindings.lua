--[[
Lua FFI bindings for the cimple-uuid library

Auto-generated from cimple_uuid.h
Requires LuaJIT with FFI support
]]

local ffi = require("ffi")

-- C declarations from the header
ffi.cdef[[
    // Opaque UUID type
    typedef struct Uuid Uuid;
    
    // Error codes
    static const int32_t ERROR_OK = 0;
    static const int32_t ERROR_NULL_PARAMETER = 1;
    static const int32_t ERROR_STRING_TOO_LONG = 2;
    static const int32_t ERROR_INVALID_HANDLE = 3;
    static const int32_t ERROR_WRONG_HANDLE_TYPE = 4;
    static const int32_t ERROR_OTHER = 5;
    static const int32_t ERROR_UUID_PARSE_ERROR = 100;
    
    // UUID functions
    Uuid* uuid_new_v4(void);
    Uuid* uuid_new_v7(void);
    Uuid* uuid_parse(const char* s);
    Uuid* uuid_nil(void);
    Uuid* uuid_max(void);
    
    char* uuid_to_string(Uuid* uuid);
    char* uuid_to_urn(Uuid* uuid);
    const uint8_t* uuid_as_bytes(Uuid* uuid);
    
    bool uuid_is_nil(Uuid* uuid);
    bool uuid_is_max(Uuid* uuid);
    
    int32_t uuid_compare(Uuid* a, Uuid* b);
    bool uuid_equals(Uuid* a, Uuid* b);
    
    int32_t uuid_error_code(void);
    char* uuid_last_error(void);
    void uuid_clear_error(void);
    
    int32_t cimple_free(void* ptr);
    
    // Standard library functions for string handling
    void free(void* ptr);
]]

-- Load the library
local lib_path = debug.getinfo(1).source:match("@?(.*/)")
if lib_path == nil then
    lib_path = "./"
end
local lib = ffi.load(lib_path .. "../../target/release/libcimple_uuid.dylib")

-- Error handling
local ERROR_NAMES = {
    [0] = "Ok",
    [1] = "NullParameter",
    [2] = "StringTooLong",
    [3] = "InvalidHandle",
    [4] = "WrongHandleType",
    [5] = "Other",
    [100] = "ParseError",
}

local function check_error()
    local code = lib.uuid_error_code()
    if code ~= 0 then
        local msg_ptr = lib.uuid_last_error()
        local message = "Unknown error"
        if msg_ptr ~= nil then
            message = ffi.string(msg_ptr)
            lib.cimple_free(msg_ptr)
        end
        lib.uuid_clear_error()
        
        local error_name = ERROR_NAMES[code] or "Unknown"
        error(string.format("[%s:%d] %s", error_name, code, message))
    end
end

-- UUID class
local Uuid = {}
Uuid.__index = Uuid

function Uuid.new(handle)
    if handle == nil then
        check_error()
        error("Failed to create UUID")
    end
    
    local self = {
        _handle = handle,
        _freed = false
    }
    
    return setmetatable(self, Uuid)
end

-- Constructors
function Uuid.v4()
    return Uuid.new(lib.uuid_new_v4())
end

function Uuid.v7()
    return Uuid.new(lib.uuid_new_v7())
end

function Uuid.parse(s)
    local handle = lib.uuid_parse(s)
    if handle == nil then
        check_error()
    end
    return Uuid.new(handle)
end

function Uuid.nil_uuid()
    return Uuid.new(lib.uuid_nil())
end

function Uuid.max_uuid()
    return Uuid.new(lib.uuid_max())
end

-- Methods
function Uuid:to_string()
    if self._freed then
        error("UUID has been freed")
    end
    
    local result = lib.uuid_to_string(self._handle)
    if result == nil then
        check_error()
    end
    
    local str = ffi.string(result)
    lib.cimple_free(result)
    return str
end

function Uuid:to_urn()
    if self._freed then
        error("UUID has been freed")
    end
    
    local result = lib.uuid_to_urn(self._handle)
    if result == nil then
        check_error()
    end
    
    local str = ffi.string(result)
    lib.cimple_free(result)
    return str
end

function Uuid:to_bytes()
    if self._freed then
        error("UUID has been freed")
    end
    
    local result = lib.uuid_as_bytes(self._handle)
    if result == nil then
        check_error()
    end
    
    local bytes = {}
    for i = 0, 15 do
        bytes[i + 1] = result[i]
    end
    
    -- Cast to void* for cimple_free
    lib.cimple_free(ffi.cast("void*", result))
    return bytes
end

function Uuid:is_nil()
    if self._freed then
        error("UUID has been freed")
    end
    return lib.uuid_is_nil(self._handle)
end

function Uuid:is_max()
    if self._freed then
        error("UUID has been freed")
    end
    return lib.uuid_is_max(self._handle)
end

function Uuid:compare(other)
    if self._freed or other._freed then
        error("UUID has been freed")
    end
    return lib.uuid_compare(self._handle, other._handle)
end

function Uuid:equals(other)
    if self._freed or other._freed then
        error("UUID has been freed")
    end
    return lib.uuid_equals(self._handle, other._handle)
end

-- Metamethods
function Uuid:__tostring()
    return self:to_string()
end

function Uuid:__eq(other)
    return self:equals(other)
end

function Uuid:__lt(other)
    return self:compare(other) < 0
end

function Uuid:__le(other)
    return self:compare(other) <= 0
end

function Uuid:__gc()
    if not self._freed and self._handle ~= nil then
        lib.cimple_free(self._handle)
        self._freed = true
    end
end

-- Explicit free (optional, GC will handle it)
function Uuid:free()
    if not self._freed and self._handle ~= nil then
        lib.cimple_free(self._handle)
        self._freed = true
    end
end

-- Module exports
return {
    Uuid = Uuid,
    v4 = Uuid.v4,
    v7 = Uuid.v7,
    parse = Uuid.parse,
    nil_uuid = Uuid.nil_uuid,
    max_uuid = Uuid.max_uuid,
    
    -- Error codes (for reference)
    ERROR_OK = 0,
    ERROR_NULL_PARAMETER = 1,
    ERROR_STRING_TOO_LONG = 2,
    ERROR_INVALID_HANDLE = 3,
    ERROR_WRONG_HANDLE_TYPE = 4,
    ERROR_OTHER = 5,
    ERROR_UUID_PARSE_ERROR = 100,
}
