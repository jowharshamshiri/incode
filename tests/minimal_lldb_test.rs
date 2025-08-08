use lldb_sys::*;

#[test]
fn test_minimal_lldb_init() {
    println!("Testing minimal LLDB initialization...");
    
    // Initialize LLDB
    unsafe { SBDebuggerInitialize() };
    
    // Create debugger
    let debugger = unsafe { SBDebuggerCreate() };
    
    if debugger.is_null() {
        println!("❌ Failed to create LLDB debugger");
        panic!("LLDB debugger creation failed");
    } else {
        println!("✅ LLDB debugger created successfully");
    }
    
    // Cleanup
    unsafe {
        SBDebuggerDestroy(debugger);
        SBDebuggerTerminate();
    }
    
    println!("✅ LLDB minimal test completed");
}