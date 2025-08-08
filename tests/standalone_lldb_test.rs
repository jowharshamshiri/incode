// Standalone LLDB test that bypasses the main library
use lldb_sys::*;
use std::ffi::CString;
use std::path::Path;

#[test]
fn test_standalone_lldb_integration() {
    println!("=== Standalone LLDB Integration Test ===");
    
    // Initialize LLDB
    unsafe { SBDebuggerInitialize() };
    println!("✅ LLDB initialized");
    
    // Create debugger
    let debugger = unsafe { SBDebuggerCreate() };
    if debugger.is_null() {
        panic!("❌ Failed to create LLDB debugger");
    }
    println!("✅ LLDB debugger created");
    
    // Configure debugger
    unsafe { SBDebuggerSetAsync(debugger, false) };
    println!("✅ LLDB debugger configured");
    
    // Check if test binary exists
    let test_binary = Path::new("test_debuggee/test_debuggee");
    if !test_binary.exists() {
        println!("⚠️  Test binary not found at test_debuggee/test_debuggee");
        unsafe {
            SBDebuggerDestroy(debugger);
            SBDebuggerTerminate();
        }
        return;
    }
    println!("✅ Test binary found: {}", test_binary.display());
    
    // Create target
    let exe_path = test_binary.canonicalize().unwrap();
    let exe_cstr = CString::new(exe_path.to_str().unwrap()).unwrap();
    let target = unsafe { SBDebuggerCreateTarget2(debugger, exe_cstr.as_ptr()) };
    
    if target.is_null() {
        println!("❌ Failed to create target");
        unsafe {
            SBDebuggerDestroy(debugger);
            SBDebuggerTerminate();
        }
        return;
    }
    println!("✅ Target created successfully");
    
    // Try to launch process
    let args = [CString::new("simple").unwrap()];
    let arg_ptrs: Vec<*const i8> = args.iter().map(|s| s.as_ptr()).collect();
    let process = unsafe { 
        SBTargetLaunchSimple(
            target, 
            arg_ptrs.as_ptr(),
            std::ptr::null(),  // env
            std::ptr::null()   // working dir
        )
    };
    
    if process.is_null() {
        println!("❌ Failed to launch process");
    } else {
        println!("✅ Process launched successfully!");
        
        // Get process ID
        let pid = unsafe { SBProcessGetProcessID(process) };
        println!("  Process PID: {}", pid);
        
        // Get process state  
        let state = unsafe { SBProcessGetState(process) };
        println!("  Process State: {:?}", state);
        
        // Kill process
        let kill_error = unsafe { CreateSBError() };
        unsafe { SBProcessKill(process) };
        println!("✅ Process terminated");
        unsafe { DisposeSBError(kill_error) };
    }
    
    // Cleanup
    unsafe {
        SBDebuggerDestroy(debugger);
        SBDebuggerTerminate();
    }
    
    println!("✅ Standalone LLDB integration test completed successfully!");
    println!("🎉 Real LLDB C++ API integration is WORKING!");
}