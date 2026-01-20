#!/bin/bash
# Quick demonstration script for the cimpl example

set -e

echo "=========================================="
echo "  Cimpl Example - Quick Demo"
echo "=========================================="
echo ""

# Check if we're in the example directory
if [ ! -f "Cargo.toml" ] || ! grep -q "cimpl-example" Cargo.toml 2>/dev/null; then
    echo "Error: Please run this script from the example/ directory"
    exit 1
fi

echo "Step 1: Building Rust library..."
echo "  - This will compile the Rust code"
echo "  - Generate the C header with cbindgen"
echo "  - Create static and dynamic libraries"
echo ""
cargo build --release
echo ""

echo "Step 2: Examining generated files..."
echo ""
echo "  📦 Static library:  $(ls -lh target/release/*.a | awk '{print $9, "(" $5 ")"}')"
echo "  📦 Dynamic library: $(ls -lh target/release/*.dylib target/release/*.so 2>/dev/null | awk '{print $9, "(" $5 ")"}' || echo 'N/A')"
echo "  📄 C header:        include/cimpl_example.h"
echo ""

echo "Step 3: Showing part of the generated C header..."
echo ""
echo "  ─────────────────────────────────────────"
head -30 include/cimpl_example.h
echo "  ... (see full file at include/cimpl_example.h)"
echo "  ─────────────────────────────────────────"
echo ""

echo "Step 4: Compiling C example program..."
make example-static > /dev/null 2>&1
echo "  ✓ C program compiled successfully"
echo ""

echo "Step 5: Running C example..."
echo ""
echo "  ═════════════════════════════════════════"
./example
echo "  ═════════════════════════════════════════"
echo ""

echo "=========================================="
echo "  ✅ Demo Complete!"
echo "=========================================="
echo ""
echo "What just happened:"
echo "  1. Rust code → Compiled to C libraries"
echo "  2. Rust docs → Extracted to C header"
echo "  3. C program → Used the library safely"
echo ""
echo "Next steps:"
echo "  • Read README.md for full documentation"
echo "  • Read QUICKSTART.md for quick start guide"
echo "  • Try: cat include/cimpl_example.h"
echo "  • Try: make help"
echo ""
echo "To generate Python/Node.js/Go bindings:"
echo "  1. Copy include/cimpl_example.h"
echo "  2. Give it to an AI with this prompt:"
echo "     'Create [language] bindings for this C library: [paste header]'"
echo ""
