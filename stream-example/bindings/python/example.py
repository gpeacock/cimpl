#!/usr/bin/env python3
"""
Example demonstrating Python bindings for cimpl_stream.

This shows how to wrap Python file objects and use them with the
cimpl_stream library.
"""

import io
import os
import sys
from cimpl_stream import Stream, CimplStreamError


def example_file_stream():
    """Example using a real file."""
    print("=== File Stream Example ===\n")
    
    filename = "python_test.txt"
    
    # Write to a file
    print("1. Writing to a file...")
    with open(filename, 'wb') as f:
        stream = Stream(f)
        message = b"Hello from Python via CimplStream!\nLine 2\nLine 3\n"
        bytes_written = stream.write(message)
        print(f"   Wrote {bytes_written} bytes")
        stream.flush()
    
    # Read from the file
    print("\n2. Reading from the file...")
    with open(filename, 'rb') as f:
        stream = Stream(f)
        data = stream.read(100)
        print(f"   Read {len(data)} bytes:")
        print(f"   ---")
        print(f"   {data.decode('utf-8')}", end='')
        print(f"   ---")
    
    # Seek operations
    print("\n3. Testing seek operations...")
    with open(filename, 'rb') as f:
        stream = Stream(f)
        
        # Seek to position 11 (start of "Python")
        pos = stream.seek(11)
        print(f"   Seeked to position {pos}")
        
        data = stream.read(6)
        print(f"   Read 6 bytes: '{data.decode('utf-8')}'")
        
        # Seek backward 10 bytes from current
        pos = stream.seek(-10, os.SEEK_CUR)
        print(f"   Seeked backward to position {pos}")
        
        data = stream.read(5)
        print(f"   Read 5 bytes: '{data.decode('utf-8')}'")
        
        # Seek to end and get file size
        size = stream.seek(0, os.SEEK_END)
        print(f"   File size: {size} bytes")
    
    # Append to file
    print("\n4. Appending to the file...")
    with open(filename, 'ab') as f:
        stream = Stream(f)
        extra = b"Appended from Python!\n"
        bytes_written = stream.write(extra)
        print(f"   Appended {bytes_written} bytes")
        stream.flush()
    
    # Read entire file
    print("\n5. Reading entire file after append...")
    with open(filename, 'rb') as f:
        stream = Stream(f)
        data = stream.read(200)
        print(f"   Contents:")
        print(f"   ---")
        print(f"   {data.decode('utf-8')}", end='')
        print(f"   ---")
    
    # Clean up
    os.remove(filename)
    print("\n=== File stream example completed! ===\n")


def example_memory_stream():
    """Example using an in-memory BytesIO buffer."""
    print("=== Memory Stream Example ===\n")
    
    buffer = io.BytesIO()
    
    print("1. Writing to memory buffer...")
    stream = Stream(buffer)
    message = b"Data in memory!"
    bytes_written = stream.write(message)
    print(f"   Wrote {bytes_written} bytes")
    
    print("\n2. Reading from memory buffer...")
    pos = stream.seek(0)
    print(f"   Seeked to position {pos}")
    
    data = stream.read()
    print(f"   Read {len(data)} bytes: '{data.decode('utf-8')}'")
    
    print("\n3. Seeking and partial read...")
    stream.seek(8)
    data = stream.read(6)
    print(f"   Read 6 bytes from position 8: '{data.decode('utf-8')}'")
    
    print("\n4. Getting current position...")
    pos = stream.tell()
    print(f"   Current position: {pos}")
    
    print("\n=== Memory stream example completed! ===\n")


def example_error_handling():
    """Example demonstrating error handling."""
    print("=== Error Handling Example ===\n")
    
    print("1. Testing read from closed stream...")
    stream = Stream(io.BytesIO(b"test data"))
    stream.close()
    
    try:
        stream.read(10)
        print("   ERROR: Should have raised exception!")
    except CimplStreamError as e:
        print(f"   Caught expected error: {e}")
    
    print("\n2. Testing seek with invalid whence...")
    buffer = io.BytesIO(b"test data")
    stream = Stream(buffer)
    
    try:
        stream.seek(0, 999)  # Invalid whence value
        print("   ERROR: Should have raised exception!")
    except ValueError as e:
        print(f"   Caught expected error: {e}")
    
    stream.close()
    
    print("\n=== Error handling example completed! ===\n")


def main():
    """Run all examples."""
    print("\n" + "="*60)
    print("Python CimplStream Bindings Examples")
    print("="*60 + "\n")
    
    try:
        example_file_stream()
        example_memory_stream()
        example_error_handling()
        
        print("="*60)
        print("All examples completed successfully!")
        print("="*60 + "\n")
        
    except Exception as e:
        print(f"\n❌ Error: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == '__main__':
    main()
