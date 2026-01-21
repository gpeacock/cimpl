#!/bin/bash
# Demo script for CimplStream example

set -e

echo "=== Building CimplStream Library ==="
echo ""
cargo build --release

echo ""
echo "=== Header Generated ==="
ls -lh include/cimpl_stream.h

echo ""
echo "=== Building C Example ==="
make example

echo ""
echo "=== Running Example ==="
echo "(Note: Run ./example manually if execution is hanging)"
echo ""

# Try to run with a short delay
./example &
PID=$!

# Wait up to 5 seconds
for i in {1..5}; do
    if ! ps -p $PID > /dev/null 2>&1; then
        echo "Example completed"
        wait $PID
        exit 0
    fi
    sleep 1
done

# If still running, something might be wrong
if ps -p $PID > /dev/null 2>&1; then
    echo "Warning: Example is still running after 5 seconds"
    echo "This might indicate an issue. Killing process..."
    kill $PID 2>/dev/null || true
    exit 1
fi
