# Makefile for building and testing the cimple example

.PHONY: all build test clean run-c example help

# Detect OS
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
    LIB_EXT = so
    LIB_PATH_VAR = LD_LIBRARY_PATH
    LDFLAGS_STATIC = -lpthread -ldl -lm
endif
ifeq ($(UNAME_S),Darwin)
    LIB_EXT = dylib
    LIB_PATH_VAR = DYLD_LIBRARY_PATH
    LDFLAGS_STATIC = -lpthread -ldl -lm -framework Security
endif

LIB_DIR = target/release
INCLUDE_DIR = include
STATIC_LIB = $(LIB_DIR)/libcimple_example.a
DYNAMIC_LIB = $(LIB_DIR)/libcimple_example.$(LIB_EXT)
HEADER = $(INCLUDE_DIR)/cimple_example.h

# Default target
all: build

# Build the Rust library (generates both static and dynamic libraries + header)
build:
	@echo "Building Rust library..."
	cargo build --release
	@echo ""
	@echo "Build complete! Generated files:"
	@echo "  Static library:  $(STATIC_LIB)"
	@echo "  Dynamic library: $(DYNAMIC_LIB)"
	@echo "  C header:        $(HEADER)"

# Run Rust tests
test:
	@echo "Running Rust tests..."
	cargo test

# Build C example using dynamic library
example-dynamic: build
	@echo "Compiling C example (dynamic linking)..."
	gcc example.c -o example \
		-I$(INCLUDE_DIR) \
		-L$(LIB_DIR) \
		-lcimple_example \
		-Wall -Wextra

# Build C example using static library
example-static: build
	@echo "Compiling C example (static linking)..."
	gcc example.c -o example \
		-I$(INCLUDE_DIR) \
		$(STATIC_LIB) \
		$(LDFLAGS_STATIC) \
		-Wall -Wextra

# Default to static linking for simplicity
example: example-static

# Run the C example
run-c: example-static
	@echo "Running C example..."
	@echo ""
	./example

# Run the dynamic version
run-c-dynamic: example-dynamic
	@echo "Running C example (dynamic linking)..."
	@echo ""
	$(LIB_PATH_VAR)=$(LIB_DIR):$$$(LIB_PATH_VAR) ./example

# Clean build artifacts
clean:
	cargo clean
	rm -f example
	rm -rf $(INCLUDE_DIR)
	@echo "Cleaned build artifacts"

# Show help
help:
	@echo "Cimple Example - Makefile targets:"
	@echo ""
	@echo "  make build           - Build the Rust library (static + dynamic + header)"
	@echo "  make test            - Run Rust tests"
	@echo "  make example         - Build C example program (static linking)"
	@echo "  make example-static  - Build C example with static linking"
	@echo "  make example-dynamic - Build C example with dynamic linking"
	@echo "  make run-c           - Build and run C example (static)"
	@echo "  make run-c-dynamic   - Build and run C example (dynamic)"
	@echo "  make clean           - Remove all build artifacts"
	@echo "  make all             - Build everything (default)"
	@echo "  make help            - Show this help message"
	@echo ""
	@echo "Generated files:"
	@echo "  $(STATIC_LIB)"
	@echo "  $(DYNAMIC_LIB)"
	@echo "  $(HEADER)"
