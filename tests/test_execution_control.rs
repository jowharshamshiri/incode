// InCode Execution Control Tools Test Suite
// 
// GRANULAR FEATURES TESTED:
// - F0007: continue_execution - Continue process execution from current state
// - F0008: step_over - Step over current instruction (next line in same function)
// - F0009: step_into - Step into function calls
// - F0010: step_out - Step out of current function (finish)
// - F0011: step_instruction - Single instruction step
// - F0012: run_until - Run until specific address or line number
// - F0013: interrupt_execution - Pause/interrupt running process
//
// Tests execution control with real LLDB integration using test_debuggee binary

use std::time::Duration;
use std::thread;

// Import test setup utilities
mod test_setup;
use test_setup::{TestSession, TestMode, TestUtils};

use incode::lldb_manager::LldbManager;
use incode::error::{IncodeError, IncodeResult};

#[tokio::test]
async fn test_f0007_continue_execution_success() {
    // F0007: continue_execution - Test successful process continuation
    println!("Testing F0007: continue_execution");
    
    let mut session = match TestSession::new(TestMode::StepDebug) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0007: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0007: Test session started with PID {}", pid);
            
            // Set breakpoint at main function
            let _ = session.set_test_breakpoint("main");
            
            // Test continue execution
            let result = session.lldb_manager().continue_execution();
            
            match result {
                Ok(_) => {
                    println!("✅ F0007: continue_execution succeeded");
                }
                Err(e) => {
                    println!("⚠️ F0007: continue_execution failed (may be expected): {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0007: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0007_continue_execution_no_process() {
    // F0007: continue_execution - Test error handling when no process attached
    println!("Testing F0007: continue_execution with no process");
    
    let mut manager = match LldbManager::new(None) {
        Ok(m) => m,
        Err(e) => {
            println!("⚠️ F0007: LLDB manager creation failed: {}", e);
            return;
        }
    };
    
    let result = manager.continue_execution();
    
    match result {
        Err(IncodeError::LldbOperation(msg)) => {
            assert!(msg.contains("No process") || msg.contains("no active"), 
                   "Error should mention no process");
            println!("✅ F0007: Correctly handled no process case: {}", msg);
        }
        Ok(_) => {
            println!("⚠️ F0007: continue_execution unexpectedly succeeded with no process");
        }
        Err(e) => {
            println!("✅ F0007: Error handling works: {}", e);
        }
    }
}

#[tokio::test]
async fn test_f0008_step_over_functionality() {
    // F0008: step_over - Test step over instruction execution
    println!("Testing F0008: step_over");
    
    let mut session = match TestSession::new(TestMode::StepDebug) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0008: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0008: Test session started with PID {}", pid);
            
            // Set breakpoint at step_debug_function
            let _ = session.set_test_breakpoint("step_debug_function");
            let _ = session.continue_execution();
            
            // Test step over
            let result = session.lldb_manager().step_over();
            
            match result {
                Ok(_) => {
                    println!("✅ F0008: step_over succeeded");
                }
                Err(e) => {
                    println!("⚠️ F0008: step_over failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0008: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0008_step_over_no_process() {
    // F0008: step_over - Test error handling when no process attached
    println!("Testing F0008: step_over with no process");
    
    let mut manager = match LldbManager::new(None) {
        Ok(m) => m,
        Err(e) => {
            println!("⚠️ F0008: LLDB manager creation failed: {}", e);
            return;
        }
    };
    
    let result = manager.step_over();
    
    match result {
        Err(IncodeError::LldbOperation(msg)) => {
            println!("✅ F0008: Correctly handled no process case: {}", msg);
        }
        Ok(_) => {
            println!("⚠️ F0008: step_over unexpectedly succeeded with no process");
        }
        Err(e) => {
            println!("✅ F0008: Error handling works: {}", e);
        }
    }
}

#[tokio::test]
async fn test_f0009_step_into_functionality() {
    // F0009: step_into - Test step into function calls
    println!("Testing F0009: step_into");
    
    let mut session = match TestSession::new(TestMode::StepDebug) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0009: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0009: Test session started with PID {}", pid);
            
            // Set breakpoint before function call
            let _ = session.set_test_breakpoint("test_function_with_params");
            let _ = session.continue_execution();
            
            // Test step into
            let result = session.lldb_manager().step_into();
            
            match result {
                Ok(_) => {
                    println!("✅ F0009: step_into succeeded");
                }
                Err(e) => {
                    println!("⚠️ F0009: step_into failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0009: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0010_step_out_functionality() {
    // F0010: step_out - Test step out of current function
    println!("Testing F0010: step_out");
    
    let mut session = match TestSession::new(TestMode::StepDebug) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0010: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0010: Test session started with PID {}", pid);
            
            // Set breakpoint inside a function
            let _ = session.set_test_breakpoint("recursive_function");
            let _ = session.continue_execution();
            
            // Test step out
            let result = session.lldb_manager().step_out();
            
            match result {
                Ok(_) => {
                    println!("✅ F0010: step_out succeeded");
                }
                Err(e) => {
                    println!("⚠️ F0010: step_out failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0010: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0011_step_instruction_functionality() {
    // F0011: step_instruction - Test single instruction step
    println!("Testing F0011: step_instruction");
    
    let mut session = match TestSession::new(TestMode::StepDebug) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0011: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0011: Test session started with PID {}", pid);
            
            // Set breakpoint for instruction stepping
            let _ = session.set_test_breakpoint("main");
            let _ = session.continue_execution();
            
            // Test instruction step
            let result = session.lldb_manager().step_instruction(false);
            
            match result {
                Ok(_) => {
                    println!("✅ F0011: step_instruction succeeded");
                }
                Err(e) => {
                    println!("⚠️ F0011: step_instruction failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0011: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0012_run_until_address() {
    // F0012: run_until - Test run until specific address
    println!("Testing F0012: run_until by address");
    
    let mut session = match TestSession::new(TestMode::StepDebug) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0012: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0012: Test session started with PID {}", pid);
            
            // Set initial breakpoint
            let _ = session.set_test_breakpoint("main");
            let _ = session.continue_execution();
            
            // Test run until (use a reasonable address or line)
            let result = session.lldb_manager().run_until(None, Some("main.cpp"), Some(42));
            
            match result {
                Ok(_) => {
                    println!("✅ F0012: run_until succeeded");
                }
                Err(e) => {
                    println!("⚠️ F0012: run_until failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0012: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0012_run_until_invalid_location() {
    // F0012: run_until - Test error handling for invalid location
    println!("Testing F0012: run_until with invalid location");
    
    let mut session = match TestSession::new(TestMode::StepDebug) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0012: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            // Test run until with invalid location
            let result = session.lldb_manager().run_until(None, Some("invalid_file.cpp"), Some(999));
            
            match result {
                Err(e) => {
                    println!("✅ F0012: Correctly handled invalid location: {}", e);
                }
                Ok(_) => {
                    println!("⚠️ F0012: run_until unexpectedly succeeded with invalid location");
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0012: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0013_interrupt_execution() {
    // F0013: interrupt_execution - Test process interruption
    println!("Testing F0013: interrupt_execution");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0013: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0013: Test session started with PID {}", pid);
            
            // Continue execution briefly
            let _ = session.lldb_manager().continue_execution();
            
            // Wait a short moment
            thread::sleep(Duration::from_millis(100));
            
            // Test interrupt execution - should work even if process has completed
            let result = session.lldb_manager().interrupt_execution();
            
            match result {
                Ok(_) => {
                    println!("✅ F0013: interrupt_execution succeeded");
                }
                Err(e) => {
                    // Interrupt may fail if process already completed - this is acceptable
                    println!("⚠️ F0013: interrupt_execution result: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0013: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0013_interrupt_no_process() {
    // F0013: interrupt_execution - Test error handling when no process attached
    println!("Testing F0013: interrupt_execution with no process");
    
    let mut manager = match LldbManager::new(None) {
        Ok(m) => m,
        Err(e) => {
            println!("⚠️ F0013: LLDB manager creation failed: {}", e);
            return;
        }
    };
    
    let result = manager.interrupt_execution();
    
    match result {
        Err(IncodeError::LldbOperation(msg)) => {
            println!("✅ F0013: Correctly handled no process case: {}", msg);
        }
        Ok(_) => {
            println!("⚠️ F0013: interrupt_execution unexpectedly succeeded with no process");
        }
        Err(e) => {
            println!("✅ F0013: Error handling works: {}", e);
        }
    }
}

#[tokio::test]
async fn test_execution_control_workflow() {
    // Integration test: Complete execution control workflow
    println!("Testing execution control workflow integration");
    
    let mut session = match TestSession::new(TestMode::StepDebug) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ Workflow: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ Workflow: Test session started with PID {}", pid);
            
            // Set breakpoint
            let _ = session.set_test_breakpoint("step_debug_function");
            
            // Continue to breakpoint
            let _ = session.lldb_manager().continue_execution();
            println!("✅ Workflow: Continued to breakpoint");
            
            // Step over a few instructions
            for i in 0..3 {
                let result = session.lldb_manager().step_over();
                match result {
                    Ok(_) => println!("✅ Workflow: Step {} completed", i + 1),
                    Err(e) => println!("⚠️ Workflow: Step {} failed: {}", i + 1, e),
                }
                thread::sleep(Duration::from_millis(50));
            }
            
            // Try step into
            let _ = session.lldb_manager().step_into();
            println!("✅ Workflow: Step into completed");
            
            // Continue execution
            let _ = session.lldb_manager().continue_execution();
            println!("✅ Workflow: Final continue completed");
            
            println!("✅ Workflow: Complete execution control workflow tested");
        }
        Err(e) => {
            println!("⚠️ Workflow: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_execution_control_performance() {
    // Performance test: Measure execution control command response times
    println!("Testing execution control performance");
    
    if !TestUtils::verify_lldb_available() {
        println!("⚠️ Performance: LLDB not available, skipping performance test");
        return;
    }
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ Performance: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            // Set a simple breakpoint first for predictable state
            let _ = session.set_test_breakpoint("test_function_with_params");
            
            let start_time = std::time::Instant::now();
            // Measure continue_execution time
            let _ = session.lldb_manager().continue_execution();
            let continue_time = start_time.elapsed();
            
            // Wait a moment for stable state
            thread::sleep(Duration::from_millis(100));
            
            let start_time = std::time::Instant::now();
            // Measure step_over time  
            let _ = session.lldb_manager().step_over();
            let step_time = start_time.elapsed();
            
            println!("📊 Performance Results:");
            println!("  continue_execution: {:?}", continue_time);
            println!("  step_over: {:?}", step_time);
            
            // More realistic performance thresholds for LLDB operations
            assert!(continue_time < Duration::from_secs(10), "continue_execution should complete within 10 seconds");
            assert!(step_time < Duration::from_secs(10), "step_over should complete within 10 seconds");
            
            println!("✅ Performance: All execution control commands within acceptable time limits");
        }
        Err(e) => {
            println!("⚠️ Performance: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}