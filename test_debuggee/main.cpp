// InCode Test Debuggee - Main Program
// Comprehensive test binary for all 65 debugging tools across 13 categories
// 
// Execution Modes:
// --mode normal     - Standard execution with predictable flow
// --mode threads    - Multi-threading scenarios  
// --mode memory     - Memory operations and patterns
// --mode crash-segv - Controlled segmentation fault
// --mode crash-stack - Stack overflow scenario
// --mode infinite   - Infinite loop for interruption testing
// --mode step-debug - Step-friendly execution paths

#include <iostream>
#include <string>
#include <vector>
#include <thread>
#include <chrono>
#include <csignal>
#include <cstring>
#include <unistd.h>

// Forward declarations from other modules
extern void run_threading_scenarios();
extern void run_memory_scenarios();
extern void showcase_variables();
extern void create_call_stack_depth(int depth);
extern void trigger_segmentation_fault();
extern void trigger_stack_overflow();

// Global variables for Variable Inspection testing (main module)
int main_global_int = 42;
float main_global_float = 3.14159f;
char main_global_string[] = "Test Global String";
static int main_static_global = 123;

// Structure for Variable Inspection testing
struct TestStruct {
    int id;
    char name[32];
    double value;
    bool active;
};

TestStruct global_struct = {1001, "GlobalStruct", 99.99, true};

// Function with various parameter types for Stack Analysis testing
int test_function_with_params(int param_int, float param_float, const char* param_str, TestStruct* param_struct) {
    // Local variables for Variable Inspection
    int local_int = param_int * 2;
    float local_float = param_float + 1.0f;
    char local_buffer[64];
    strcpy(local_buffer, param_str);
    
    // More locals for comprehensive testing
    double local_double = 123.456;
    bool local_bool = true;
    int local_array[5] = {1, 2, 3, 4, 5};
    
    // Pointer operations for memory testing
    int* heap_memory = new int[10];
    for (int i = 0; i < 10; i++) {
        heap_memory[i] = i * 10;
    }
    
    // Breakpoint-friendly operations
    std::cout << "Function parameters: int=" << param_int 
              << ", float=" << param_float 
              << ", str=" << param_str << std::endl;
    
    // Use param_struct to avoid warning
    if (param_struct) {
        std::cout << "Struct param ID: " << param_struct->id << std::endl;
    }
    
    // Clean up
    delete[] heap_memory;
    
    return local_int + static_cast<int>(local_float);
}

// Recursive function for Stack Analysis testing
int recursive_function(int depth, int accumulator = 0) {
    if (depth <= 0) {
        return accumulator;
    }
    
    // Local variables at each recursion level
    int local_depth = depth;
    int local_result = accumulator + depth;
    
    // Tail recursion
    return recursive_function(depth - 1, local_result);
}

// Step-through friendly function for Execution Control testing
void step_debug_function() {
    std::cout << "Step 1: Initialize variables" << std::endl;
    int step_var1 = 10;
    
    std::cout << "Step 2: Conditional branch" << std::endl;
    if (step_var1 > 5) {
        std::cout << "Step 3: In true branch" << std::endl;
        step_var1 += 5;
    } else {
        std::cout << "Step 3: In false branch" << std::endl;
        step_var1 -= 5;
    }
    
    std::cout << "Step 4: Loop operations" << std::endl;
    for (int i = 0; i < 3; i++) {
        std::cout << "  Loop iteration: " << i << ", step_var1: " << step_var1 << std::endl;
        step_var1 *= 2;
    }
    
    std::cout << "Step 5: Function call" << std::endl;
    int result = test_function_with_params(step_var1, 2.5f, "step_debug", &global_struct);
    
    std::cout << "Step 6: Function complete, result: " << result << std::endl;
}

// Signal handler for controlled interruption
void signal_handler(int signal) {
    std::cout << "Received signal: " << signal << std::endl;
    exit(0);
}

// Mode execution functions
void run_normal_mode() {
    std::cout << "=== Normal Mode Execution ===" << std::endl;
    
    // Showcase variables
    showcase_variables();
    
    // Test function calls for Stack Analysis
    std::cout << "\nTesting function calls and stack analysis..." << std::endl;
    int result = test_function_with_params(100, 25.5f, "normal_mode", &global_struct);
    std::cout << "Function result: " << result << std::endl;
    
    // Test recursion
    std::cout << "\nTesting recursive function..." << std::endl;
    int recursive_result = recursive_function(5);
    std::cout << "Recursive result: " << recursive_result << std::endl;
    
    // Create some call stack depth
    create_call_stack_depth(3);
    
    std::cout << "\nNormal mode execution complete." << std::endl;
}

void run_threads_mode() {
    std::cout << "=== Threading Mode Execution ===" << std::endl;
    run_threading_scenarios();
}

void run_memory_mode() {
    std::cout << "=== Memory Mode Execution ===" << std::endl;
    run_memory_scenarios();
}

void run_crash_segv_mode() {
    std::cout << "=== Crash (Segmentation Fault) Mode ===" << std::endl;
    std::cout << "Triggering controlled segmentation fault in 2 seconds..." << std::endl;
    std::this_thread::sleep_for(std::chrono::seconds(2));
    trigger_segmentation_fault();
}

void run_crash_stack_mode() {
    std::cout << "=== Crash (Stack Overflow) Mode ===" << std::endl;
    std::cout << "Triggering controlled stack overflow in 2 seconds..." << std::endl;
    std::this_thread::sleep_for(std::chrono::seconds(2));
    trigger_stack_overflow();
}

void run_infinite_mode() {
    std::cout << "=== Infinite Loop Mode ===" << std::endl;
    std::cout << "Starting infinite loop for interruption testing..." << std::endl;
    std::cout << "Use Ctrl+C or debugging interrupt to stop." << std::endl;
    
    signal(SIGINT, signal_handler);
    
    int counter = 0;
    while (true) {
        counter++;
        if (counter % 1000000 == 0) {
            std::cout << "Loop iteration: " << counter << std::endl;
        }
        
        // Brief pause every million iterations to make interruption easier
        if (counter % 10000000 == 0) {
            std::this_thread::sleep_for(std::chrono::milliseconds(100));
        }
    }
}

void run_step_debug_mode() {
    std::cout << "=== Step Debug Mode ===" << std::endl;
    std::cout << "Executing step-friendly function for Execution Control testing..." << std::endl;
    step_debug_function();
}

int main(int argc, char* argv[]) {
    std::cout << "InCode Test Debuggee - Comprehensive Debugging Test Binary" << std::endl;
    std::cout << "Process ID: " << getpid() << std::endl;
    std::cout << "Arguments: " << argc << std::endl;
    
    for (int i = 0; i < argc; i++) {
        std::cout << "  argv[" << i << "]: " << argv[i] << std::endl;
    }
    
    // Default mode
    std::string mode = "normal";
    
    // Parse command line arguments
    for (int i = 1; i < argc - 1; i++) {
        if (std::string(argv[i]) == "--mode" && i + 1 < argc) {
            mode = std::string(argv[i + 1]);
            break;
        }
    }
    
    std::cout << "Execution mode: " << mode << std::endl << std::endl;
    
    // Execute based on mode
    if (mode == "normal") {
        run_normal_mode();
    } else if (mode == "threads") {
        run_threads_mode();
    } else if (mode == "memory") {
        run_memory_mode();
    } else if (mode == "crash-segv") {
        run_crash_segv_mode();
    } else if (mode == "crash-stack") {
        run_crash_stack_mode();
    } else if (mode == "infinite") {
        run_infinite_mode();
    } else if (mode == "step-debug") {
        run_step_debug_mode();
    } else {
        std::cout << "Unknown mode: " << mode << std::endl;
        std::cout << "Available modes: normal, threads, memory, crash-segv, crash-stack, infinite, step-debug" << std::endl;
        return 1;
    }
    
    return 0;
}