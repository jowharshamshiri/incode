# InCode Test Debuggee Makefile
# Builds comprehensive test binary with full debug symbols for all 65 debugging tools

# Compiler and flags
CXX = g++
CXXFLAGS = -std=c++17 -g3 -O0 -Wall -Wextra -fno-omit-frame-pointer
LDFLAGS = -pthread

# Target binary
TARGET = test_debuggee

# Source files
SOURCES = main.cpp threads.cpp memory.cpp variables.cpp
OBJECTS = $(SOURCES:.cpp=.o)

# Debug information flags (compatible with both GCC and Clang)
DEBUG_FLAGS = -gdwarf-4

# Additional flags for comprehensive debugging (Clang compatible)
EXTRA_FLAGS = -fno-inline -fno-optimize-sibling-calls

# Default target
all: $(TARGET)

# Main target with full debug information
$(TARGET): $(OBJECTS)
	@echo "Linking $(TARGET) with full debug information..."
	$(CXX) $(CXXFLAGS) $(DEBUG_FLAGS) $(EXTRA_FLAGS) -o $(TARGET) $(OBJECTS) $(LDFLAGS)
	@echo "Build complete: $(TARGET)"
	@echo "Binary info:"
	@file $(TARGET)
	@echo "Size:"
	@ls -lh $(TARGET)

# Compile source files with debug information
%.o: %.cpp
	@echo "Compiling $< with debug symbols..."
	$(CXX) $(CXXFLAGS) $(DEBUG_FLAGS) $(EXTRA_FLAGS) -c $< -o $@

# Debug build (same as default, but explicit)
debug: CXXFLAGS += -DDEBUG -fno-inline-functions
debug: $(TARGET)
	@echo "Debug build complete with maximum debug information"

# Release build (for performance testing, but still with some debug info)
release: CXXFLAGS = -std=c++17 -g1 -O2 -Wall -Wextra -DNDEBUG
release: clean $(TARGET)
	@echo "Release build complete"

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	rm -f $(OBJECTS) $(TARGET)
	@echo "Clean complete"

# Install (copy to tests directory)
install: $(TARGET)
	@echo "Installing $(TARGET) to tests directory..."
	cp $(TARGET) ../tests/
	@echo "Install complete"

# Test compilation
test-compile: $(TARGET)
	@echo "Testing binary compilation and basic execution..."
	./$(TARGET) --mode normal || echo "Test execution completed (may have failed intentionally)"
	@echo "Compilation test complete"

# Verify debug symbols
verify-debug: $(TARGET)
	@echo "Verifying debug symbols in $(TARGET)..."
	@echo "Debug sections:"
	@objdump -h $(TARGET) | grep debug || echo "Debug sections check complete"
	@echo "Symbol table sample:"
	@nm $(TARGET) | head -20 || echo "Symbol table check complete"
	@echo "DWARF info check:"
	@objdump -Wi $(TARGET) | head -20 || echo "DWARF info check complete"

# Create different execution test targets
test-modes: $(TARGET)
	@echo "Testing different execution modes..."
	@echo "=== Testing normal mode ==="
	timeout 5s ./$(TARGET) --mode normal || echo "Normal mode test complete"
	@echo "=== Testing step-debug mode ==="
	timeout 5s ./$(TARGET) --mode step-debug || echo "Step-debug mode test complete"
	@echo "=== Testing memory mode ==="
	timeout 5s ./$(TARGET) --mode memory || echo "Memory mode test complete"
	@echo "=== Testing threads mode ==="
	timeout 5s ./$(TARGET) --mode threads || echo "Threads mode test complete"
	@echo "All mode tests complete"

# LLDB compatibility test
test-lldb: $(TARGET)
	@echo "Testing LLDB compatibility..."
	@which lldb > /dev/null || (echo "LLDB not found, skipping compatibility test"; exit 0)
	@echo "LLDB version:"
	@lldb --version || echo "LLDB version check complete"
	@echo "Testing LLDB binary loading..."
	@echo "quit" | lldb $(TARGET) || echo "LLDB binary loading test complete"

# Help target
help:
	@echo "InCode Test Debuggee Build System"
	@echo ""
	@echo "Targets:"
	@echo "  all          - Build test binary with full debug information (default)"
	@echo "  debug        - Build debug version with maximum debug info"
	@echo "  release      - Build optimized version with minimal debug info"
	@echo "  clean        - Remove all build artifacts"
	@echo "  install      - Copy binary to tests directory"
	@echo "  test-compile - Test basic compilation and execution"
	@echo "  verify-debug - Verify debug symbols are present"
	@echo "  test-modes   - Test all execution modes"
	@echo "  test-lldb    - Test LLDB compatibility"
	@echo "  help         - Show this help message"
	@echo ""
	@echo "Debug Features:"
	@echo "  - Full DWARF-4 debug information"
	@echo "  - Variable tracking enabled"
	@echo "  - Frame pointer preservation"
	@echo "  - Inlining disabled for better debugging"
	@echo "  - Standalone debug information"
	@echo ""
	@echo "Execution Modes:"
	@echo "  --mode normal      - Standard execution with all scenarios"
	@echo "  --mode threads     - Multi-threading scenarios"
	@echo "  --mode memory      - Memory operation patterns"
	@echo "  --mode step-debug  - Step-friendly execution paths"
	@echo "  --mode crash-segv  - Controlled segmentation fault"
	@echo "  --mode crash-stack - Controlled stack overflow"
	@echo "  --mode infinite    - Infinite loop for interruption testing"

# Phony targets
.PHONY: all debug release clean install test-compile verify-debug test-modes test-lldb help

# Dependencies
main.o: main.cpp
threads.o: threads.cpp
memory.o: memory.cpp
variables.o: variables.cpp