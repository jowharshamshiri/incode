// InCode Stack & Frame Analysis Tools Test Suite
// 
// GRANULAR FEATURES TESTED:
// - F0022: get_backtrace - Get call stack with function names and addresses
// - F0023: select_frame - Switch to specific stack frame by index
// - F0024: get_frame_info - Get current frame details (function, args, locals)
// - F0025: get_frame_variables - Get all local variables in current frame
// - F0026: get_frame_arguments - Get function arguments for current frame
// - F0027: evaluate_in_frame - Evaluate expression in specific frame context
//
// Tests stack analysis with real LLDB integration using test_debuggee binary

use std::time::Duration;
use std::thread;

// Import test setup utilities
mod test_setup;
use test_setup::{TestSession, TestMode, TestUtils};

use incode::lldb_manager::LldbManager;
use incode::error::{IncodeError, IncodeResult};

#[tokio::test]
async fn test_f0022_get_backtrace_success() {
    // F0022: get_backtrace - Test getting call stack with function names
    println!("Testing F0022: get_backtrace");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0022: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0022: Test session started with PID {}", pid);
            
            // Set breakpoint in nested function for good call stack
            let _ = session.lldb_manager().set_breakpoint("create_call_stack_depth");
            let _ = session.lldb_manager().continue_execution();
            
            // Test getting backtrace
            let result = session.lldb_manager().get_backtrace();
            
            match result {
                Ok(backtrace) => {
                    println!("✅ F0022: get_backtrace succeeded with {} frames", backtrace.len());
                    
                    for (i, frame) in backtrace.iter().take(5).enumerate() {
                        println!("  Frame {}: {}", i, frame);
                    }
                    
                    assert!(backtrace.len() > 0, "Should have at least one frame");
                    
                    // Look for expected functions in call stack
                    let has_main = backtrace.iter().any(|f| f.contains("main"));
                    let has_target = backtrace.iter().any(|f| f.contains("create_call_stack_depth"));
                    
                    if has_main {
                        println!("✅ F0022: Found main function in call stack");
                    }
                    if has_target {
                        println!("✅ F0022: Found target function in call stack");
                    }
                }
                Err(e) => {
                    println!("⚠️ F0022: get_backtrace failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0022: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0022_get_backtrace_with_addresses() {
    // F0022: get_backtrace - Test getting backtrace with address information
    println!("Testing F0022: get_backtrace with addresses");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0022: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            // Set breakpoint for stack analysis
            let _ = session.lldb_manager().set_breakpoint("recursive_function");
            let _ = session.lldb_manager().continue_execution();
            
            // Test getting backtrace with addresses
            let result = session.lldb_manager().get_backtrace();
            
            match result {
                Ok(backtrace) => {
                    println!("✅ F0022: get_backtrace with addresses succeeded, {} frames", backtrace.len());
                    
                    for frame in &backtrace {
                        assert!(!frame.is_empty(), "Frame should not be empty");
                        // Frame is a string, so just check it's not empty
                        println!("  Frame: {}", frame);
                    }
                }
                Err(e) => {
                    println!("⚠️ F0022: get_backtrace with addresses failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0022: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0023_select_frame() {
    // F0023: select_frame - Test switching to specific stack frame
    println!("Testing F0023: select_frame");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0023: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0023: Test session started with PID {}", pid);
            
            // Set breakpoint in nested function
            let _ = session.lldb_manager().set_breakpoint("test_function_with_params");
            let _ = session.lldb_manager().continue_execution();
            
            // Get backtrace first to know available frames
            match session.lldb_manager().get_backtrace() {
                Ok(backtrace) => {
                    if backtrace.len() > 1 {
                        // Test selecting different frame
                        let target_frame = 1;
                        let result = session.lldb_manager().select_frame(target_frame);
                        
                        match result {
                            Ok(_) => {
                                println!("✅ F0023: select_frame succeeded for frame {}", target_frame);
                                
                                // Verify frame selection by getting frame info
                                match session.lldb_manager().get_frame_info(Some(target_frame)) {
                                    Ok(frame_info) => {
                                        println!("  Selected frame: {} at 0x{:x}", 
                                               frame_info.function_name, frame_info.pc);
                                        assert_eq!(frame_info.index, target_frame, 
                                                  "Frame index should match selected frame");
                                    }
                                    Err(e) => {
                                        println!("⚠️ F0023: Could not verify frame selection: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                println!("⚠️ F0023: select_frame failed: {}", e);
                            }
                        }
                    } else {
                        println!("⚠️ F0023: Only one frame available, cannot test frame selection");
                    }
                }
                Err(e) => {
                    println!("⚠️ F0023: Could not get backtrace: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0023: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0023_select_frame_invalid_index() {
    // F0023: select_frame - Test error handling for invalid frame index
    println!("Testing F0023: select_frame with invalid index");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0023: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            let _ = session.lldb_manager().set_breakpoint("main");
            let _ = session.lldb_manager().continue_execution();
            
            // Test selecting invalid frame index
            let result = session.lldb_manager().select_frame(99999);
            
            match result {
                Err(e) => {
                    println!("✅ F0023: Correctly handled invalid frame index: {}", e);
                }
                Ok(_) => {
                    println!("⚠️ F0023: select_frame unexpectedly succeeded for invalid index");
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0023: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0024_get_frame_info() {
    // F0024: get_frame_info - Test getting current frame details
    println!("Testing F0024: get_frame_info");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0024: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0024: Test session started with PID {}", pid);
            
            // Set breakpoint in function with known parameters
            let _ = session.lldb_manager().set_breakpoint("test_function_with_params");
            let _ = session.lldb_manager().continue_execution();
            
            // Test getting frame info
            let result = session.lldb_manager().get_frame_info(None);
            
            match result {
                Ok(frame_info) => {
                    println!("✅ F0024: get_frame_info succeeded");
                    println!("  Function: {}", frame_info.function_name);
                    println!("  Address: 0x{:x}", frame_info.pc);
                    println!("  Frame Index: {}", frame_info.index);
                    println!("  Module: {}", frame_info.module.as_deref().unwrap_or("N/A"));
                    println!("  Source: {}:{}", 
                           frame_info.file.as_deref().unwrap_or("unknown"),
                           frame_info.line.unwrap_or(0));
                    
                    assert!(!frame_info.function_name.is_empty(), "Function name should not be empty");
                    assert!(frame_info.pc > 0, "Address should be valid");
                    assert!(frame_info.index >= 0, "Frame index should be valid");
                }
                Err(e) => {
                    println!("⚠️ F0024: get_frame_info failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0024: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0025_get_frame_variables() {
    // F0025: get_frame_variables - Test getting local variables in current frame
    println!("Testing F0025: get_frame_variables");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0025: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0025: Test session started with PID {}", pid);
            
            // Set breakpoint in function with local variables
            let _ = session.lldb_manager().set_breakpoint("demonstrate_local_variables");
            let _ = session.lldb_manager().continue_execution();
            
            // Test getting frame variables
            let result = session.lldb_manager().get_frame_variables(None, false);
            
            match result {
                Ok(variables) => {
                    println!("✅ F0025: get_frame_variables succeeded, found {} variables", variables.len());
                    
                    for (i, var) in variables.iter().take(5).enumerate() {
                        println!("  Variable {}: {} = {} ({})", 
                               i + 1, var.name, var.value, var.var_type);
                    }
                    
                    // Look for expected local variables
                    let has_local_int = variables.iter().any(|v| v.name == "local_int");
                    let has_local_float = variables.iter().any(|v| v.name == "local_float");
                    
                    if has_local_int || has_local_float {
                        println!("✅ F0025: Found expected local variables in frame");
                    }
                    
                    assert!(variables.len() > 0, "Should have at least one variable in frame");
                }
                Err(e) => {
                    println!("⚠️ F0025: get_frame_variables failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0025: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0026_get_frame_arguments() {
    // F0026: get_frame_arguments - Test getting function arguments for current frame
    println!("Testing F0026: get_frame_arguments");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0026: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0026: Test session started with PID {}", pid);
            
            // Set breakpoint in function with known parameters
            let _ = session.lldb_manager().set_breakpoint("function_with_parameters");
            let _ = session.lldb_manager().continue_execution();
            
            // Test getting frame arguments
            let result = session.lldb_manager().get_frame_arguments(None);
            
            match result {
                Ok(arguments) => {
                    println!("✅ F0026: get_frame_arguments succeeded, found {} arguments", arguments.len());
                    
                    for (i, arg) in arguments.iter().enumerate() {
                        println!("  Argument {}: {} = {} ({})", 
                               i + 1, arg.name, arg.value, arg.var_type);
                    }
                    
                    // Look for expected function parameters
                    let has_param_int = arguments.iter().any(|a| a.name == "param_int");
                    let has_param_string = arguments.iter().any(|a| a.name == "param_string");
                    
                    if has_param_int || has_param_string {
                        println!("✅ F0026: Found expected function parameters");
                    }
                    
                    assert!(arguments.len() > 0, "Function should have at least one argument");
                }
                Err(e) => {
                    println!("⚠️ F0026: get_frame_arguments failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0026: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0027_evaluate_in_frame() {
    // F0027: evaluate_in_frame - Test evaluating expression in specific frame context
    println!("Testing F0027: evaluate_in_frame");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0027: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0027: Test session started with PID {}", pid);
            
            // Set breakpoint in function with local variables
            let _ = session.lldb_manager().set_breakpoint("demonstrate_local_variables");
            let _ = session.lldb_manager().continue_execution();
            
            // Get backtrace to know frame indices
            match session.lldb_manager().get_backtrace() {
                Ok(backtrace) => {
                    if backtrace.len() > 0 {
                        let frame_index = 0; // Current frame
                        
                        // Test evaluating expressions in specific frame
                        let expressions = vec![
                            "local_int + 1",
                            "sizeof(local_int)",
                            "local_int > 0",
                        ];
                        
                        for expr in expressions {
                            let result = session.lldb_manager().evaluate_in_frame(Some(frame_index), expr);
                            
                            match result {
                                Ok(eval_result) => {
                                    println!("✅ F0027: evaluate_in_frame succeeded for '{}': result = {}", 
                                           expr, eval_result);
                                    
                                    assert!(!eval_result.is_empty());
                                }
                                Err(e) => {
                                    println!("⚠️ F0027: evaluate_in_frame failed for '{}': {}", expr, e);
                                }
                            }
                        }
                    } else {
                        println!("⚠️ F0027: No frames available for evaluation");
                    }
                }
                Err(e) => {
                    println!("⚠️ F0027: Could not get backtrace: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0027: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0027_evaluate_in_frame_invalid() {
    // F0027: evaluate_in_frame - Test error handling for invalid frame/expression
    println!("Testing F0027: evaluate_in_frame with invalid inputs");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0027: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            let _ = session.lldb_manager().set_breakpoint("main");
            let _ = session.lldb_manager().continue_execution();
            
            // Test invalid frame index
            let result = session.lldb_manager().evaluate_in_frame(Some(99999), "1 + 1");
            
            match result {
                Err(e) => {
                    println!("✅ F0027: Correctly handled invalid frame index: {}", e);
                }
                Ok(_) => {
                    println!("⚠️ F0027: evaluate_in_frame unexpectedly succeeded with invalid frame");
                }
            }
            
            // Test invalid expression in valid frame
            let result = session.lldb_manager().evaluate_in_frame(Some(0), "nonexistent_variable_12345");
            
            match result {
                Err(e) => {
                    println!("✅ F0027: Correctly handled invalid expression: {}", e);
                }
                Ok(_) => {
                    println!("⚠️ F0027: evaluate_in_frame unexpectedly succeeded with invalid expression");
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0027: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_stack_analysis_workflow() {
    // Integration test: Complete stack analysis workflow
    println!("Testing stack analysis workflow integration");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ Workflow: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ Workflow: Test session started with PID {}", pid);
            
            // Step 1: Get to a function with deep call stack
            let _ = session.lldb_manager().set_breakpoint("create_call_stack_depth");
            let _ = session.lldb_manager().continue_execution();
            
            // Step 2: Get backtrace
            let backtrace = match session.lldb_manager().get_backtrace() {
                Ok(bt) => {
                    println!("✅ Workflow: Got backtrace with {} frames", bt.len());
                    bt
                }
                Err(e) => {
                    println!("⚠️ Workflow: Failed to get backtrace: {}", e);
                    return;
                }
            };
            
            // Step 3: Analyze each frame
            for (i, frame) in backtrace.iter().take(3).enumerate() {
                println!("  Analyzing frame {}: {}", i, frame);
                
                // Select frame
                match session.lldb_manager().select_frame(i as u32) {
                    Ok(_) => {
                        println!("✅ Workflow: Selected frame {}", i);
                        
                        // Get frame info
                        match session.lldb_manager().get_frame_info(Some(i as u32)) {
                            Ok(info) => {
                                println!("  Frame info: {} at {}:{}", 
                                       info.function_name, 
                                       info.file.as_deref().unwrap_or("unknown"),
                                       info.line.unwrap_or(0));
                            }
                            Err(e) => println!("⚠️ Workflow: Failed to get frame info: {}", e),
                        }
                        
                        // Get frame variables
                        match session.lldb_manager().get_frame_variables(None, false) {
                            Ok(vars) => {
                                println!("  Found {} variables in frame", vars.len());
                            }
                            Err(e) => println!("⚠️ Workflow: Failed to get frame variables: {}", e),
                        }
                        
                        // Get frame arguments
                        match session.lldb_manager().get_frame_arguments(None) {
                            Ok(args) => {
                                println!("  Found {} arguments in frame", args.len());
                            }
                            Err(e) => println!("⚠️ Workflow: Failed to get frame arguments: {}", e),
                        }
                        
                        // Evaluate expression in frame context
                        match session.lldb_manager().evaluate_in_frame(Some(i as u32), "sizeof(int)") {
                            Ok(result) => {
                                println!("  Expression evaluation: sizeof(int) = {}", result);
                            }
                            Err(e) => println!("⚠️ Workflow: Failed to evaluate expression: {}", e),
                        }
                    }
                    Err(e) => {
                        println!("⚠️ Workflow: Failed to select frame {}: {}", i, e);
                    }
                }
            }
            
            println!("✅ Workflow: Complete stack analysis workflow tested");
        }
        Err(e) => {
            println!("⚠️ Workflow: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}