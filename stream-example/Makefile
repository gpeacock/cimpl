# Makefile for CimplStream Example

# Detect OS for library naming
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    LIB_EXT = dylib
else ifeq ($(UNAME_S),Linux)
    LIB_EXT = so
else
    LIB_EXT = so
endif

# Directories
TARGET_DIR = target/release
LIB_NAME = libcimpl_stream.$(LIB_EXT)
LIB_PATH = $(TARGET_DIR)/$(LIB_NAME)
HEADER = include/cimpl_stream.h

# Compiler settings
CC = gcc
CFLAGS = -Wall -Wextra -I.
LDFLAGS = -L$(TARGET_DIR) -lcimpl_stream

# macOS specific: add rpath for dylib lookup
ifeq ($(UNAME_S),Darwin)
    LDFLAGS += -Wl,-rpath,@executable_path/$(TARGET_DIR)
else
    LDFLAGS += -Wl,-rpath,$$ORIGIN/$(TARGET_DIR)
endif

.PHONY: all clean run build lib example help test

all: build example

help:
	@echo "CimplStream Example - Makefile targets:"
	@echo ""
	@echo "  make all      - Build library and example (default)"
	@echo "  make lib      - Build the Rust library only"
	@echo "  make example  - Build the C example"
	@echo "  make run      - Build and run the example"
	@echo "  make test     - Run Rust tests"
	@echo "  make clean    - Remove all build artifacts"
	@echo ""

# Build everything
build: lib $(HEADER)

# Build the Rust library
lib:
	cargo build --release

# The header is generated during the build
$(HEADER): lib

# Build the C example
example: $(HEADER) $(LIB_PATH)
	$(CC) $(CFLAGS) example.c -o example $(LDFLAGS)

# Run the example
run: example
	@echo "Running CimplStream example..."
	@echo ""
	./example

# Run Rust tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean
	rm -f example
	rm -f test_output.txt
	rm -f $(HEADER)

# Development helpers
.PHONY: watch check fmt clippy doc

# Watch for changes and rebuild
watch:
	cargo watch -x 'build --release'

# Check code without building
check:
	cargo check

# Format code
fmt:
	cargo fmt

# Run clippy linter
clippy:
	cargo clippy -- -D warnings

# Generate and open documentation
doc:
	cargo doc --open
