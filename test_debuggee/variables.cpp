// InCode Test Debuggee - Variable Showcase
// Variable patterns and types for Variable Inspection tools (F0035-F0040)

#include <iostream>
#include <string>
#include <vector>
#include <map>
#include <memory>
#include <functional>
#include <complex>
#include <cstring>

// Forward declarations for recursive structures
struct Node;
struct TreeNode;

// Enum for testing
enum class Status {
    INACTIVE = 0,
    ACTIVE = 1,
    PENDING = 2,
    ERROR = 99
};

// Union for testing
union DataUnion {
    int int_value;
    float float_value;
    char char_array[4];
    
    DataUnion(int val) : int_value(val) {}
};

// Complex structure for comprehensive testing
struct ComplexStruct {
    int id;
    std::string name;
    double value;
    Status status;
    std::vector<int> numbers;
    std::map<std::string, int> mapping;
    DataUnion data;
    Node* node_ptr;
    
    ComplexStruct(int id_val, const std::string& name_val) 
        : id(id_val), name(name_val), value(123.456), status(Status::ACTIVE), data(42) {
        numbers = {1, 2, 3, 4, 5};
        mapping["key1"] = 10;
        mapping["key2"] = 20;
        node_ptr = nullptr;
    }
};

// Node structure for pointer testing
struct Node {
    int data;
    Node* next;
    Node* prev;
    
    Node(int val) : data(val), next(nullptr), prev(nullptr) {}
};

// Tree structure for recursive testing
struct TreeNode {
    int value;
    TreeNode* left;
    TreeNode* right;
    int depth;
    
    TreeNode(int val, int d = 0) : value(val), left(nullptr), right(nullptr), depth(d) {}
};

// Global variables for testing
int global_int = 42;
float global_float = 3.14159f;
double global_double = 2.71828;
bool global_bool = true;
char global_char = 'G';
const char* global_string = "Global String Value";
std::string global_std_string = "Global std::string Value";

// Static variables
static int static_int = 100;
static double static_double = 999.888;
static ComplexStruct static_complex(999, "StaticComplex");

// Extern variables (declared elsewhere)
extern int external_variable;
int external_variable = 777;

// Array variables
int global_int_array[10] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9};
double global_double_array[] = {1.1, 2.2, 3.3, 4.4, 5.5};
char global_char_array[32] = "Global Character Array";

// Pointer variables
int* global_int_ptr = &global_int;
ComplexStruct* global_struct_ptr = nullptr;

// Complex type variables
std::vector<int> global_vector = {10, 20, 30, 40, 50};
std::map<std::string, double> global_map = {
    {"pi", 3.14159},
    {"e", 2.71828},
    {"sqrt2", 1.41421}
};

// Function pointer
std::function<int(int, int)> global_function = [](int a, int b) { return a + b; };

// Complex number
std::complex<double> global_complex(3.0, 4.0);

// Function to demonstrate local variables
void demonstrate_local_variables() {
    std::cout << "Demonstrating local variables..." << std::endl;
    
    // Basic types
    int local_int = 123;
    float local_float = 45.67f;
    double local_double = 890.123;
    bool local_bool = false;
    char local_char = 'L';
    const char* local_string = "Local String";
    std::string local_std_string = "Local std::string";
    
    // Arrays
    int local_array[5] = {10, 20, 30, 40, 50};
    char local_char_array[16] = "LocalCharArray";
    
    // Pointers
    int* local_int_ptr = &local_int;
    int** local_int_ptr_ptr = &local_int_ptr;
    
    // Complex structures
    ComplexStruct local_complex(456, "LocalComplex");
    Status local_status = Status::PENDING;
    DataUnion local_union(789);
    
    // STL containers
    std::vector<std::string> local_string_vector = {"one", "two", "three"};
    std::map<int, std::string> local_int_string_map = {
        {1, "first"},
        {2, "second"},
        {3, "third"}
    };
    
    // Dynamic allocations
    int* heap_int = new int(999);
    ComplexStruct* heap_complex = new ComplexStruct(888, "HeapComplex");
    
    std::cout << "Local variables created. Values:" << std::endl;
    std::cout << "  local_int: " << local_int << std::endl;
    std::cout << "  local_float: " << local_float << std::endl;
    std::cout << "  local_double: " << local_double << std::endl;
    std::cout << "  local_bool: " << std::boolalpha << local_bool << std::endl;
    std::cout << "  local_char: " << local_char << std::endl;
    std::cout << "  local_string: " << local_string << std::endl;
    std::cout << "  local_std_string: " << local_std_string << std::endl;
    
    // Good breakpoint location for local variable inspection
    volatile int local_breakpoint_marker = 1;
    (void)local_breakpoint_marker;
    
    // Cleanup
    delete heap_int;
    delete heap_complex;
}

// Function with parameters for parameter inspection
int function_with_parameters(int param_int, const std::string& param_string, 
                           ComplexStruct* param_struct, const std::vector<int>& param_vector) {
    std::cout << "Function with parameters called:" << std::endl;
    std::cout << "  param_int: " << param_int << std::endl;
    std::cout << "  param_string: " << param_string << std::endl;
    std::cout << "  param_struct ptr: " << param_struct << std::endl;
    if (param_struct) {
        std::cout << "  param_struct->id: " << param_struct->id << std::endl;
        std::cout << "  param_struct->name: " << param_struct->name << std::endl;
    }
    std::cout << "  param_vector size: " << param_vector.size() << std::endl;
    
    // Local variables that reference parameters
    int local_param_copy = param_int;
    std::string local_string_copy = param_string;
    ComplexStruct* local_struct_ptr = param_struct;
    
    // Good breakpoint location for parameter inspection
    volatile int param_breakpoint_marker = 2;
    (void)param_breakpoint_marker;
    
    return local_param_copy + static_cast<int>(param_vector.size());
}

// Function to create linked list for pointer chain testing
Node* create_linked_list(int count) {
    if (count <= 0) return nullptr;
    
    Node* head = new Node(1);
    Node* current = head;
    
    for (int i = 2; i <= count; i++) {
        Node* new_node = new Node(i);
        current->next = new_node;
        new_node->prev = current;
        current = new_node;
    }
    
    return head;
}

// Function to create binary tree for recursive structure testing
TreeNode* create_binary_tree(int depth, int value = 1) {
    if (depth <= 0) return nullptr;
    
    TreeNode* root = new TreeNode(value, depth);
    root->left = create_binary_tree(depth - 1, value * 2);
    root->right = create_binary_tree(depth - 1, value * 2 + 1);
    
    return root;
}

// Function to demonstrate variable modifications
void demonstrate_variable_modifications() {
    std::cout << "Demonstrating variable modifications..." << std::endl;
    
    int modification_test = 10;
    std::cout << "Initial value: " << modification_test << std::endl;
    
    // Various modifications
    modification_test += 5;
    std::cout << "After += 5: " << modification_test << std::endl;
    
    modification_test *= 2;
    std::cout << "After *= 2: " << modification_test << std::endl;
    
    modification_test = modification_test % 7;
    std::cout << "After % 7: " << modification_test << std::endl;
    
    // Array modifications
    int mod_array[5] = {1, 2, 3, 4, 5};
    for (int i = 0; i < 5; i++) {
        mod_array[i] *= 10;
    }
    
    // Structure modifications
    ComplexStruct mod_struct(123, "ModificationTest");
    mod_struct.value = 456.789;
    mod_struct.status = Status::ERROR;
    mod_struct.numbers.push_back(100);
    mod_struct.mapping["new_key"] = 999;
    
    // Good breakpoint location for modification inspection
    volatile int mod_breakpoint_marker = 3;
    (void)mod_breakpoint_marker;
}

// Function for const and volatile variable testing
void demonstrate_const_volatile() {
    std::cout << "Demonstrating const and volatile variables..." << std::endl;
    
    const int const_int = 999;
    const std::string const_string = "Constant String";
    const ComplexStruct const_struct(777, "ConstStruct");
    
    volatile int volatile_int = 555;
    volatile bool volatile_bool = true;
    
    const volatile int const_volatile_int = 888;
    
    // References
    int reference_target = 444;
    int& int_reference = reference_target;
    const int& const_reference = const_int;
    
    std::cout << "Const and volatile variables created" << std::endl;
    
    // Good breakpoint location for const/volatile inspection
    volatile int const_vol_breakpoint_marker = 4;
    (void)const_vol_breakpoint_marker;
}

// Main variable showcase orchestrator
void showcase_variables() {
    std::cout << "Starting variable showcase..." << std::endl;
    
    // Initialize global struct pointer
    global_struct_ptr = new ComplexStruct(666, "GlobalStructPtr");
    
    // Display global variable information
    std::cout << "Global variables:" << std::endl;
    std::cout << "  global_int: " << global_int << std::endl;
    std::cout << "  global_float: " << global_float << std::endl;
    std::cout << "  global_double: " << global_double << std::endl;
    std::cout << "  global_bool: " << std::boolalpha << global_bool << std::endl;
    std::cout << "  global_string: " << global_string << std::endl;
    std::cout << "  global_std_string: " << global_std_string << std::endl;
    
    // Demonstrate local variables
    demonstrate_local_variables();
    
    // Demonstrate function parameters
    std::vector<int> param_vector = {100, 200, 300};
    int param_result = function_with_parameters(123, "ParameterTest", global_struct_ptr, param_vector);
    std::cout << "Parameter function result: " << param_result << std::endl;
    
    // Create complex data structures
    Node* linked_list = create_linked_list(5);
    TreeNode* binary_tree = create_binary_tree(3);
    
    std::cout << "Complex structures created:" << std::endl;
    std::cout << "  linked_list head: " << linked_list << std::endl;
    std::cout << "  binary_tree root: " << binary_tree << std::endl;
    
    // Demonstrate variable modifications
    demonstrate_variable_modifications();
    
    // Demonstrate const and volatile
    demonstrate_const_volatile();
    
    // Good breakpoint location for comprehensive variable inspection
    volatile int showcase_breakpoint_marker = 5;
    (void)showcase_breakpoint_marker;
    
    std::cout << "Variable showcase complete." << std::endl;
    std::cout << "Available variable types:" << std::endl;
    std::cout << "  Basic types: int, float, double, bool, char" << std::endl;
    std::cout << "  Strings: C-style and std::string" << std::endl;
    std::cout << "  Arrays: static and dynamic" << std::endl;
    std::cout << "  Pointers: single and multi-level" << std::endl;
    std::cout << "  Structures: simple and complex" << std::endl;
    std::cout << "  STL containers: vector, map" << std::endl;
    std::cout << "  Linked structures: lists and trees" << std::endl;
}

// Function to create call stack depth for Stack Analysis testing
void create_call_stack_depth(int depth) {
    if (depth <= 0) {
        std::cout << "Maximum call stack depth reached" << std::endl;
        
        // Local variables at maximum depth
        int depth_var = 999;
        std::string depth_string = "MaxDepth";
        ComplexStruct depth_struct(depth, "DepthStruct");
        
        // Good breakpoint location for call stack inspection
        volatile int stack_depth_marker = depth;
        (void)stack_depth_marker;
        return;
    }
    
    // Local variables at this depth level
    int current_depth = depth;
    std::string level_name = "Level" + std::to_string(depth);
    
    std::cout << "Call stack depth: " << current_depth << " (" << level_name << ")" << std::endl;
    
    // Recursive call
    create_call_stack_depth(depth - 1);
}

// Crash scenario functions
void trigger_segmentation_fault() {
    std::cout << "Triggering segmentation fault..." << std::endl;
    
    // Null pointer dereference
    int* null_ptr = nullptr;
    *null_ptr = 42;  // This will cause segmentation fault
}

void trigger_stack_overflow() {
    std::cout << "Triggering stack overflow..." << std::endl;
    
    // Infinite recursion to cause stack overflow
    static int overflow_counter = 0;
    overflow_counter++;
    
    char large_buffer[1024];  // Consume stack space
    memset(large_buffer, overflow_counter % 256, sizeof(large_buffer));
    
    trigger_stack_overflow();  // Infinite recursion
}