// InCode Test Debuggee - Memory Scenarios
// Memory patterns and operations for Memory Inspection tools (F0028-F0034)

#include <iostream>
#include <vector>
#include <memory>
#include <cstring>
#include <cstdlib>
#include <algorithm>

// Global memory regions for testing
static char global_buffer[1024];
static int global_array[256];
static const char* const_string = "Constant String for Memory Testing";

// Structure for memory layout testing
struct MemoryTestStruct {
    int magic_number;       // 0x12345678
    char identifier[16];    // "MEMTEST_STRUCT"
    double value;          // 123.456789
    void* pointer;         // Points to another memory location
    int array[4];          // {10, 20, 30, 40}
    
    MemoryTestStruct() {
        magic_number = 0x12345678;
        strcpy(identifier, "MEMTEST_STRUCT");
        value = 123.456789;
        pointer = this;
        array[0] = 10; array[1] = 20; array[2] = 30; array[3] = 40;
    }
};

// Global memory test structures
static MemoryTestStruct global_memory_struct;
static MemoryTestStruct* global_struct_pointer = &global_memory_struct;

// Memory pattern functions for testing different access patterns
void create_stack_patterns() {
    std::cout << "Creating stack memory patterns..." << std::endl;
    
    // Stack-based arrays with known patterns
    char stack_buffer[512];
    int stack_integers[64];
    double stack_doubles[32];
    
    // Fill with predictable patterns
    for (int i = 0; i < 512; i++) {
        stack_buffer[i] = static_cast<char>('A' + (i % 26));
    }
    
    for (int i = 0; i < 64; i++) {
        stack_integers[i] = i * 100;
    }
    
    for (int i = 0; i < 32; i++) {
        stack_doubles[i] = i * 3.14159;
    }
    
    // Create structures on stack
    MemoryTestStruct stack_struct1;
    MemoryTestStruct stack_struct2;
    stack_struct2.magic_number = 0x87654321;
    strcpy(stack_struct2.identifier, "STACK_STRUCT2");
    
    std::cout << "Stack patterns created:" << std::endl;
    std::cout << "  stack_buffer at: " << static_cast<void*>(stack_buffer) << std::endl;
    std::cout << "  stack_integers at: " << static_cast<void*>(stack_integers) << std::endl;
    std::cout << "  stack_doubles at: " << static_cast<void*>(stack_doubles) << std::endl;
    std::cout << "  stack_struct1 at: " << static_cast<void*>(&stack_struct1) << std::endl;
    std::cout << "  stack_struct2 at: " << static_cast<void*>(&stack_struct2) << std::endl;
    
    // Good breakpoint location for stack memory inspection
    volatile int breakpoint_marker = 1;
    (void)breakpoint_marker;
}

void create_heap_patterns() {
    std::cout << "Creating heap memory patterns..." << std::endl;
    
    // Various heap allocations for testing
    char* heap_buffer = new char[1024];
    int* heap_integers = new int[128];
    double* heap_doubles = new double[64];
    MemoryTestStruct* heap_struct = new MemoryTestStruct();
    
    // Fill heap memory with patterns
    for (int i = 0; i < 1024; i++) {
        heap_buffer[i] = static_cast<char>('a' + (i % 26));
    }
    
    for (int i = 0; i < 128; i++) {
        heap_integers[i] = i * 1000;
    }
    
    for (int i = 0; i < 64; i++) {
        heap_doubles[i] = i * 2.71828;
    }
    
    // Modify heap struct
    heap_struct->magic_number = 0xDEADBEEF;
    strcpy(heap_struct->identifier, "HEAP_STRUCT");
    heap_struct->value = 999.888777;
    
    std::cout << "Heap patterns created:" << std::endl;
    std::cout << "  heap_buffer at: " << static_cast<void*>(heap_buffer) << std::endl;
    std::cout << "  heap_integers at: " << static_cast<void*>(heap_integers) << std::endl;
    std::cout << "  heap_doubles at: " << static_cast<void*>(heap_doubles) << std::endl;
    std::cout << "  heap_struct at: " << static_cast<void*>(heap_struct) << std::endl;
    
    // Create some fragmentation
    char* small_alloc1 = new char[16];
    char* small_alloc2 = new char[32];
    char* small_alloc3 = new char[64];
    
    strcpy(small_alloc1, "Small1");
    strcpy(small_alloc2, "Small2");
    strcpy(small_alloc3, "Small3");
    
    std::cout << "  small_alloc1 at: " << static_cast<void*>(small_alloc1) << std::endl;
    std::cout << "  small_alloc2 at: " << static_cast<void*>(small_alloc2) << std::endl;
    std::cout << "  small_alloc3 at: " << static_cast<void*>(small_alloc3) << std::endl;
    
    // Good breakpoint location for heap memory inspection
    volatile int breakpoint_marker = 2;
    (void)breakpoint_marker;
    
    // Clean up some allocations (but not all, for memory leak testing)
    delete[] heap_buffer;
    delete[] heap_integers;
    // Intentionally leak heap_doubles, heap_struct, and small allocations for testing
    
    std::cout << "Some heap memory cleaned up, some intentionally leaked for testing" << std::endl;
}

void create_global_patterns() {
    std::cout << "Creating global memory patterns..." << std::endl;
    
    // Fill global buffer with pattern
    for (int i = 0; i < 1024; i++) {
        global_buffer[i] = static_cast<char>('0' + (i % 10));
    }
    
    // Fill global array with pattern
    for (int i = 0; i < 256; i++) {
        global_array[i] = i * i;
    }
    
    std::cout << "Global patterns created:" << std::endl;
    std::cout << "  global_buffer at: " << static_cast<void*>(global_buffer) << std::endl;
    std::cout << "  global_array at: " << static_cast<void*>(global_array) << std::endl;
    std::cout << "  global_memory_struct at: " << static_cast<void*>(&global_memory_struct) << std::endl;
    std::cout << "  const_string at: " << static_cast<const void*>(const_string) << std::endl;
    
    // Good breakpoint location for global memory inspection
    volatile int breakpoint_marker = 3;
    (void)breakpoint_marker;
}

void test_memory_access_patterns() {
    std::cout << "Testing memory access patterns..." << std::endl;
    
    // Sequential access pattern
    volatile int sequential_sum = 0;
    for (int i = 0; i < 256; i++) {
        sequential_sum += global_array[i];
    }
    
    // Random access pattern
    volatile int random_sum = 0;
    int indices[] = {5, 100, 50, 200, 25, 150, 75, 225, 10, 90};
    for (int i = 0; i < 10; i++) {
        random_sum += global_array[indices[i]];
    }
    
    // String operations
    char temp_buffer[256];
    strcpy(temp_buffer, "Memory access test string");
    strcat(temp_buffer, " - concatenated");
    int len = strlen(temp_buffer);
    
    std::cout << "Memory access patterns completed:" << std::endl;
    std::cout << "  Sequential sum: " << sequential_sum << std::endl;
    std::cout << "  Random sum: " << random_sum << std::endl;
    std::cout << "  String length: " << len << std::endl;
    
    // Good breakpoint location for memory access inspection
    volatile int breakpoint_marker = 4;
    (void)breakpoint_marker;
}

void demonstrate_memory_corruption() {
    std::cout << "Demonstrating controlled memory scenarios..." << std::endl;
    
    // Buffer with known content for corruption testing
    char test_buffer[128];
    strcpy(test_buffer, "CLEAN_BUFFER_CONTENT");
    std::cout << "Original buffer content: " << test_buffer << std::endl;
    
    // Simulate minor corruption (controlled)
    test_buffer[5] = 'X';
    test_buffer[10] = 'Y';
    std::cout << "Modified buffer content: " << test_buffer << std::endl;
    
    // Array bounds scenario (controlled)
    int bounds_test[10] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9};
    
    // Access patterns that debugger can inspect
    for (int i = 0; i < 10; i++) {
        bounds_test[i] *= 10;
    }
    
    std::cout << "Array modification completed" << std::endl;
    
    // Good breakpoint location for corruption inspection
    volatile int breakpoint_marker = 5;
    (void)breakpoint_marker;
}

void create_watchpoint_targets() {
    std::cout << "Creating watchpoint target variables..." << std::endl;
    
    // Static variables that can be watched
    static int watchpoint_int = 42;
    static double watchpoint_double = 3.14159;
    static char watchpoint_string[64] = "Initial watchpoint string";
    
    std::cout << "Watchpoint targets created:" << std::endl;
    std::cout << "  watchpoint_int at: " << static_cast<void*>(&watchpoint_int) << " = " << watchpoint_int << std::endl;
    std::cout << "  watchpoint_double at: " << static_cast<void*>(&watchpoint_double) << " = " << watchpoint_double << std::endl;
    std::cout << "  watchpoint_string at: " << static_cast<void*>(watchpoint_string) << " = " << watchpoint_string << std::endl;
    
    // Modify values to trigger watchpoints
    watchpoint_int = 100;
    watchpoint_double = 2.71828;
    strcpy(watchpoint_string, "Modified watchpoint string");
    
    std::cout << "Values modified:" << std::endl;
    std::cout << "  watchpoint_int = " << watchpoint_int << std::endl;
    std::cout << "  watchpoint_double = " << watchpoint_double << std::endl;
    std::cout << "  watchpoint_string = " << watchpoint_string << std::endl;
    
    // Good breakpoint location for watchpoint testing
    volatile int breakpoint_marker = 6;
    (void)breakpoint_marker;
}

// Main memory scenarios orchestrator
void run_memory_scenarios() {
    std::cout << "Starting memory inspection scenarios..." << std::endl;
    
    // Create different memory patterns
    create_stack_patterns();
    create_heap_patterns();
    create_global_patterns();
    
    // Test memory access patterns
    test_memory_access_patterns();
    
    // Demonstrate memory scenarios
    demonstrate_memory_corruption();
    
    // Create watchpoint targets
    create_watchpoint_targets();
    
    std::cout << "Memory scenarios complete." << std::endl;
    std::cout << "Memory regions available for inspection:" << std::endl;
    std::cout << "  Stack: Local variables in each function" << std::endl;
    std::cout << "  Heap: Allocated structures and arrays" << std::endl;
    std::cout << "  Global: global_buffer, global_array, global_memory_struct" << std::endl;
    std::cout << "  Constants: const_string and other read-only data" << std::endl;
}