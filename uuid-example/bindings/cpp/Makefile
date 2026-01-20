# Makefile for C++ UUID wrapper

CXX = g++
CXXFLAGS = -std=c++20 -Wall -Wextra -I../../include
LDFLAGS = -L../../target/release -lcimpl_uuid -lpthread

# Detect OS for additional flags
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    LDFLAGS += -framework Security
else ifeq ($(UNAME_S),Linux)
    LDFLAGS += -ldl -lm
endif

.PHONY: all clean run

all: example

# Build the C++ example (dynamic linking)
example: example.cpp uuid.cpp uuid.hpp
	@echo "Building C++ example..."
	$(CXX) $(CXXFLAGS) -o example example.cpp uuid.cpp $(LDFLAGS)
	@echo "Build complete!"

# Build with static linking
example-static: example.cpp uuid.cpp uuid.hpp
	@echo "Building C++ example (static linking)..."
	$(CXX) $(CXXFLAGS) -o example example.cpp uuid.cpp \
		../../target/release/libcimpl_uuid.a -lpthread
ifeq ($(UNAME_S),Darwin)
	$(CXX) $(CXXFLAGS) -o example example.cpp uuid.cpp \
		../../target/release/libcimpl_uuid.a -lpthread -framework Security
else
	$(CXX) $(CXXFLAGS) -o example example.cpp uuid.cpp \
		../../target/release/libcimpl_uuid.a -lpthread -ldl -lm
endif
	@echo "Build complete!"

run: example
	@echo "Running C++ example..."
	@echo ""
	./example

clean:
	rm -f example
	@echo "Clean complete!"
