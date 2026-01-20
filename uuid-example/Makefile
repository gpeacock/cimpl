# Makefile for cimpl-uuid example

.PHONY: all clean run-c build-rust help

# Detect OS for library extension
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    DYLIB_EXT = dylib
    EXTRA_LIBS = -framework Security
else ifeq ($(UNAME_S),Linux)
    DYLIB_EXT = so
    EXTRA_LIBS = -ldl -lm
else
    DYLIB_EXT = dll
    EXTRA_LIBS =
endif

all: build-rust example-static

help:
	@echo "Available targets:"
	@echo "  all          - Build everything (default)"
	@echo "  build-rust   - Build Rust library"
	@echo "  example-static - Build C example with static linking"
	@echo "  run-c        - Build and run C example"
	@echo "  clean        - Remove all build artifacts"

build-rust:
	@echo "Building Rust library..."
	cargo build --release
	@echo ""
	@echo "Build complete! Generated files:"
	@echo "  Static library:  target/release/libcimpl_uuid.a"
	@echo "  Dynamic library: target/release/libcimpl_uuid.$(DYLIB_EXT)"
	@echo "  C header:        include/cimpl_uuid.h"

example-static: build-rust bindings/c/example.c
	@echo "Compiling C example (static linking)..."
	gcc bindings/c/example.c -o example \
		-Iinclude \
		target/release/libcimpl_uuid.a \
		-lpthread $(EXTRA_LIBS) \
		-Wall -Wextra

run-c: example-static
	@echo "Running C example..."
	@echo ""
	./example

clean:
	cargo clean
	rm -f example
	@echo "Clean complete!"
