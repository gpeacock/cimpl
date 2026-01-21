#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "include/cimpl_stream.h"

// cimpl_free is provided by the base cimpl library (statically linked into libcimpl_stream)
extern int32_t cimpl_free(void* ptr);

// ============================================================================
// File Stream Context
// ============================================================================

typedef struct {
    FILE* file;
    const char* filename;
} FileStreamContext;

// ============================================================================
// File Stream Callbacks
// ============================================================================

intptr_t file_read_callback(CimplStreamContext* ctx, uint8_t* data, size_t len) {
    FileStreamContext* file_ctx = (FileStreamContext*)ctx;
    size_t bytes_read = fread(data, 1, len, file_ctx->file);
    if (ferror(file_ctx->file)) {
        return -1;
    }
    return (intptr_t)bytes_read;
}

int64_t file_seek_callback(CimplStreamContext* ctx, int64_t offset, CimplSeekMode mode) {
    FileStreamContext* file_ctx = (FileStreamContext*)ctx;
    
    int whence;
    switch (mode) {
        case CIMPL_SEEK_MODE_START:
            whence = SEEK_SET;
            break;
        case CIMPL_SEEK_MODE_CURRENT:
            whence = SEEK_CUR;
            break;
        case CIMPL_SEEK_MODE_END:
            whence = SEEK_END;
            break;
        default:
            return -1;
    }
    
    if (fseek(file_ctx->file, offset, whence) != 0) {
        return -1;
    }
    
    long pos = ftell(file_ctx->file);
    if (pos < 0) {
        return -1;
    }
    
    return (int64_t)pos;
}

intptr_t file_write_callback(CimplStreamContext* ctx, const uint8_t* data, size_t len) {
    FileStreamContext* file_ctx = (FileStreamContext*)ctx;
    size_t bytes_written = fwrite(data, 1, len, file_ctx->file);
    if (ferror(file_ctx->file)) {
        return -1;
    }
    return (intptr_t)bytes_written;
}

int32_t file_flush_callback(CimplStreamContext* ctx) {
    FileStreamContext* file_ctx = (FileStreamContext*)ctx;
    return fflush(file_ctx->file) == 0 ? 0 : -1;
}

// ============================================================================
// Helper Functions
// ============================================================================

typedef struct {
    CimplStream* stream;
    FileStreamContext* context;
} FileStreamHandle;

FileStreamHandle* create_file_stream(const char* filename, const char* mode) {
    FileStreamContext* ctx = malloc(sizeof(FileStreamContext));
    if (!ctx) {
        return NULL;
    }
    
    ctx->file = fopen(filename, mode);
    if (!ctx->file) {
        free(ctx);
        return NULL;
    }
    
    ctx->filename = filename;
    
    CimplStream* stream = cimpl_stream_new(
        (CimplStreamContext*)ctx,
        file_read_callback,
        file_seek_callback,
        file_write_callback,
        file_flush_callback
    );
    
    if (!stream) {
        fclose(ctx->file);
        free(ctx);
        return NULL;
    }
    
    FileStreamHandle* handle = malloc(sizeof(FileStreamHandle));
    if (!handle) {
        cimpl_free(stream);
        fclose(ctx->file);
        free(ctx);
        return NULL;
    }
    
    handle->stream = stream;
    handle->context = ctx;
    return handle;
}

void close_file_stream(FileStreamHandle* handle) {
    if (!handle) return;
    
    if (handle->stream) {
        cimpl_free(handle->stream);
    }
    if (handle->context) {
        if (handle->context->file) {
            fclose(handle->context->file);
        }
        free(handle->context);
    }
    free(handle);
}

void print_error() {
    char* error = cimpl_stream_last_error();
    if (error) {
        fprintf(stderr, "Error: %s\n", error);
        cimpl_free(error);
    }
}

// ============================================================================
// Main Demo
// ============================================================================

int main() {
    printf("=== CimplStream Example ===\n\n");
    
    // Test 1: Write to a file using streams
    printf("1. Writing to a file...\n");
    FileStreamHandle* write_handle = create_file_stream("test_output.txt", "wb");
    if (!write_handle) {
        fprintf(stderr, "   Failed to create write stream\n");
        print_error();
        return 1;
    }
    
    const char* message = "Hello from CimplStream!\nThis is line 2.\nAnd line 3.\n";
    intptr_t bytes_written = cimpl_stream_write(
        write_handle->stream,
        (const uint8_t*)message,
        strlen(message)
    );
    
    if (bytes_written < 0) {
        fprintf(stderr, "   Write failed\n");
        print_error();
        close_file_stream(write_handle);
        return 1;
    }
    
    printf("   Wrote %zd bytes\n", bytes_written);
    
    cimpl_stream_flush(write_handle->stream);
    close_file_stream(write_handle);
    
    // Test 2: Read from the file
    printf("\n2. Reading from the file...\n");
    FileStreamHandle* read_handle = create_file_stream("test_output.txt", "rb");
    if (!read_handle) {
        fprintf(stderr, "   Failed to create read stream\n");
        print_error();
        return 1;
    }
    
    uint8_t buffer[256];
    intptr_t bytes_read = cimpl_stream_read(read_handle->stream, buffer, sizeof(buffer) - 1);
    
    if (bytes_read < 0) {
        fprintf(stderr, "   Read failed\n");
        print_error();
        close_file_stream(read_handle);
        return 1;
    }
    
    buffer[bytes_read] = '\0';
    printf("   Read %zd bytes:\n", bytes_read);
    printf("   ---\n%s   ---\n", buffer);
    
    // Test 3: Seek operations
    printf("\n3. Testing seek operations...\n");
    
    // Seek to position 6 (start of "from")
    int64_t pos = cimpl_stream_seek(read_handle->stream, 6, CIMPL_SEEK_MODE_START);
    printf("   Seeked to position %lld\n", pos);
    
    bytes_read = cimpl_stream_read(read_handle->stream, buffer, 4);
    buffer[bytes_read] = '\0';
    printf("   Read 4 bytes: '%s'\n", (char*)buffer);
    
    // Seek 10 bytes back from current position
    pos = cimpl_stream_seek(read_handle->stream, -10, CIMPL_SEEK_MODE_CURRENT);
    printf("   Seeked backward to position %lld\n", pos);
    
    bytes_read = cimpl_stream_read(read_handle->stream, buffer, 5);
    buffer[bytes_read] = '\0';
    printf("   Read 5 bytes: '%s'\n", (char*)buffer);
    
    // Seek to end and check file size
    pos = cimpl_stream_seek(read_handle->stream, 0, CIMPL_SEEK_MODE_END);
    printf("   File size: %lld bytes\n", pos);
    
    close_file_stream(read_handle);
    
    // Test 4: Append to file
    printf("\n4. Appending to the file...\n");
    FileStreamHandle* append_handle = create_file_stream("test_output.txt", "ab");
    if (!append_handle) {
        fprintf(stderr, "   Failed to create append stream\n");
        print_error();
        return 1;
    }
    
    const char* extra = "Appended line!\n";
    bytes_written = cimpl_stream_write(
        append_handle->stream,
        (const uint8_t*)extra,
        strlen(extra)
    );
    
    printf("   Appended %zd bytes\n", bytes_written);
    
    cimpl_stream_flush(append_handle->stream);
    close_file_stream(append_handle);
    
    // Test 5: Read entire file again
    printf("\n5. Reading entire file after append...\n");
    read_handle = create_file_stream("test_output.txt", "rb");
    if (!read_handle) {
        fprintf(stderr, "   Failed to create read stream\n");
        print_error();
        return 1;
    }
    
    bytes_read = cimpl_stream_read(read_handle->stream, buffer, sizeof(buffer) - 1);
    buffer[bytes_read] = '\0';
    printf("   Contents:\n   ---\n%s   ---\n", buffer);
    
    close_file_stream(read_handle);
    
    printf("\n=== All tests completed successfully! ===\n");
    printf("\nNote: Output file 'test_output.txt' has been created.\n");
    
    return 0;
}
