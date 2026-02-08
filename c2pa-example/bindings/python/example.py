#!/usr/bin/env python3
"""
Example demonstrating the C2PA Python bindings

This script shows how to use the Context and Settings objects
with builder patterns and automatic memory management.
"""

import sys
from pathlib import Path

# Add the bindings directory to Python path
sys.path.insert(0, str(Path(__file__).parent))

from c2pa import Context, Settings, C2paError
import c2pa  # Import module for lib access


def main():
    print("=" * 60)
    print("C2PA Python Bindings Example")
    print("=" * 60)
    print()
    
    # Example 1: Basic Settings Creation
    print("1. Creating Settings with defaults...")
    try:
        settings = Settings.new()
        print("   ✓ Settings created")
        
        # Convert to JSON
        json_str = settings.to_json()
        print(f"   Settings as JSON: {json_str[:100]}...")
        
        # Convert to TOML
        toml_str = settings.to_toml()
        print(f"   Settings as TOML (first 100 chars): {toml_str[:100]}...")
        print()
    except C2paError as e:
        print(f"   ✗ Error: {e}")
        return 1
    
    # Example 2: Working with Settings
    print("2. Working with Settings...")
    try:
        settings1 = Settings.new()
        settings2 = Settings.new()
        print("   ✓ Created multiple Settings objects")
        print()
    except C2paError as e:
        print(f"   ✗ Error: {e}")
        return 1
    
    # Example 3: Settings to/from JSON
    print("3. Settings JSON round-trip...")
    try:
        settings = Settings.new()
        json_str = settings.to_json()
        settings2 = Settings.from_json(json_str)
        print("   ✓ Settings JSON round-trip successful")
        print(f"   JSON length: {len(json_str)} bytes")
        print()
    except C2paError as e:
        print(f"   ✗ Error: {e}")
        return 1
    
    # Example 4: Settings to/from TOML
    print("4. Settings TOML round-trip...")
    try:
        settings = Settings.new()
        toml_str = settings.to_toml()
        settings2 = Settings.from_toml(toml_str)
        print("   ✓ Settings TOML round-trip successful")
        print(f"   TOML length: {len(toml_str)} bytes")
        print()
    except C2paError as e:
        print(f"   ✗ Error: {e}")
        return 1
    
    # Example 5: Basic Context Creation
    print("5. Creating Context...")
    try:
        ctx = Context.new()
        print("   ✓ Context created")
        print()
    except C2paError as e:
        print(f"   ✗ Error: {e}")
        return 1
    
    # Example 6: Context with Settings Object
    print("6. Context with Settings object...")
    try:
        settings = Settings.new()
        ctx = Context.new()
        result = c2pa.lib.c2pa_context_with_settings_obj(ctx._handle, settings._handle)
        if result == 0:
            print("   ✓ Context configured with Settings object")
        else:
            print("   ✗ Configuration failed")
        print()
    except C2paError as e:
        print(f"   ✗ Error: {e}")
        return 1
    
    # Example 7: Multiple Contexts
    print("7. Creating multiple Contexts...")
    try:
        ctx1 = Context.new()
        ctx2 = Context.new()
        ctx3 = Context.new()
        print("   ✓ Created 3 Context objects")
        print()
    except C2paError as e:
        print(f"   ✗ Error: {e}")
        return 1
    
    # Example 8: Error Handling
    print("8. Testing error handling with invalid JSON...")
    try:
        invalid_json = '{"invalid json'
        settings = Settings.from_json(invalid_json)
        print("   ✗ Should have raised an error!")
    except C2paError as e:
        print(f"   ✓ Caught expected error (code {e.code}): {e.message[:80]}...")
        print()
    
    print("=" * 60)
    print("All examples completed successfully!")
    print("=" * 60)
    print()
    print("Note: Memory is automatically managed - no manual cleanup needed!")
    
    return 0


if __name__ == '__main__':
    sys.exit(main())
