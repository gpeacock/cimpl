#!/bin/bash
# Test all language bindings for the uuid-example

set -e  # Exit on first error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "======================================"
echo "Testing cimpl UUID Bindings"
echo "======================================"
echo ""

# Ensure library is built
echo "1. Building Rust library..."
cargo build --release
echo "✅ Rust library built"
echo ""

# Test C
echo "2. Testing C bindings..."
make run-c > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✅ C bindings work"
else
    echo "❌ C bindings failed"
    exit 1
fi
echo ""

# Test Python
echo "3. Testing Python bindings..."
cd bindings/python
python3 test.py > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✅ Python bindings work"
else
    echo "❌ Python bindings failed"
    exit 1
fi
cd ../..
echo ""

# Test Lua
echo "5. Testing Lua bindings..."
cd bindings/lua
if command -v luajit &> /dev/null; then
    luajit test.lua > /dev/null 2>&1
    if [ $? -eq 0 ]; then
        echo "✅ Lua bindings work"
    else
        echo "❌ Lua bindings failed"
        exit 1
    fi
else
    echo "⚠️  LuaJIT not installed, skipping Lua tests"
fi
cd ../..
echo ""

echo "======================================"
echo "✅ All bindings tested successfully!"
echo "======================================"
