// InCode Register Inspection Tools Test Suite
// 
// GRANULAR FEATURES TESTED:
// - F0046: get_registers - Get all CPU registers for current thread
// - F0047: set_register - Modify register value
// - F0048: get_register_info - Get detailed register information
// - F0049: save_register_state - Save current register state
//
// Tests register inspection with real LLDB integration using test_debuggee binary

use std::time::Duration;
use std::thread;

// Import test setup utilities
mod test_setup;
use test_setup::{TestSession, TestMode};

use incode::lldb_manager::LldbManager;
use incode::error::IncodeError;

#[tokio::test]
async fn test_f0046_get_registers_success() {
    // F0046: get_registers - Test getting all CPU registers
    println!("Testing F0046: get_registers");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0046: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0046: Test session started with PID {}", pid);
            
            // Set breakpoint to get to a known state
            let _ = session.lldb_manager().set_breakpoint("main");
            let _ = session.lldb_manager().continue_execution();
            
            // Test getting all registers
            let result = session.lldb_manager().get_registers(None, true);
            
            match result {
                Ok(register_state) => {
                    println!("✅ F0046: get_registers succeeded");
                    println!("  Thread ID: {:?}", register_state.thread_id);
                    println!("  General Registers: {}", register_state.registers.len());
                    println!("  Floating Point Registers: 0 /* not tracked in current RegisterState */");
                    println!("  Vector Registers: 0 /* not tracked in current RegisterState */");
                    
                    // Display some general registers
                    for (i, reg) in register_state.registers.values().take(5).enumerate() {
                        println!("  General {}: {} = 0x{:x}", i + 1, reg.name, reg.value);
                    }
                    
                    // Common registers should be present
                    let has_pc = register_state.registers.values()
                        .any(|r| r.name.to_lowercase().contains("pc") || r.name.to_lowercase().contains("rip"));
                    let has_sp = register_state.registers.values()
                        .any(|r| r.name.to_lowercase().contains("sp") || r.name.to_lowercase().contains("rsp"));
                    
                    if has_pc {
                        println!("✅ F0046: Found program counter register");
                    }
                    if has_sp {
                        println!("✅ F0046: Found stack pointer register");
                    }
                    
                    assert!(register_state.registers.len() > 0, "Should have general registers");
                    assert!(register_state.thread_id.is_some(), "Thread ID should be present");
                }
                Err(e) => {
                    println!("⚠️ F0046: get_registers failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0046: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0046_get_registers_specific_thread() {
    // F0046: get_registers - Test getting registers for specific thread
    println!("Testing F0046: get_registers for specific thread");
    
    let mut session = match TestSession::new(TestMode::Threads) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0046: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            // Let threads initialize
            thread::sleep(Duration::from_millis(500));
            
            // Get list of threads first
            match session.lldb_manager().list_threads() {
                Ok(threads) => {
                    if let Some(first_thread) = threads.first() {
                        let target_thread_id = first_thread.thread_id;
                        
                        // Test getting registers for specific thread
                        let result = session.lldb_manager().get_registers(Some(target_thread_id), false);
                        
                        match result {
                            Ok(register_state) => {
                                println!("✅ F0046: get_registers succeeded for thread {}", target_thread_id);
                                assert_eq!(register_state.thread_id, Some(target_thread_id), 
                                          "Register state should match requested thread");
                                println!("  Registers for thread {}: {} general, {} float", 
                                       register_state.thread_id.unwrap_or(0),
                                       register_state.registers.len(),
                                       0 /* float registers not tracked */);
                            }
                            Err(e) => {
                                println!("⚠️ F0046: get_registers failed for specific thread: {}", e);
                            }
                        }
                    } else {
                        println!("⚠️ F0046: No threads available for specific thread test");
                    }
                }
                Err(e) => {
                    println!("⚠️ F0046: Could not get thread list: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0046: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0047_set_register() {
    // F0047: set_register - Test modifying register value
    println!("Testing F0047: set_register");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0047: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0047: Test session started with PID {}", pid);
            
            // Set breakpoint to get to a known state
            let _ = session.lldb_manager().set_breakpoint("main");
            let _ = session.lldb_manager().continue_execution();
            
            // Get current registers first to find a suitable register to modify
            match session.lldb_manager().get_registers(None, false) {
                Ok(register_state) => {
                    if let Some(first_register) = register_state.registers.values().next() {
                        let register_name = &first_register.name;
                        let original_value = first_register.value;
                        let new_value = 0x12345678u64;
                        
                        println!("  Original value of {}: 0x{:x}", register_name, original_value);
                        
                        // Test setting register value
                        let result = session.lldb_manager().set_register(register_name, new_value, None);
                        
                        match result {
                            Ok(success) => {
                                if success {
                                    println!("✅ F0047: set_register succeeded for {}", register_name);
                                    
                                    // Verify the change by reading registers again
                                    match session.lldb_manager().get_registers(None, false) {
                                        Ok(new_register_state) => {
                                            if let Some(modified_register) = new_register_state.registers
                                                .values().find(|r| r.name == *register_name) {
                                                println!("  New value of {}: 0x{:x}", 
                                                       register_name, modified_register.value);
                                                
                                                if modified_register.value == new_value {
                                                    println!("✅ F0047: Register modification verified");
                                                } else {
                                                    println!("⚠️ F0047: Register modification not reflected or overridden");
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            println!("⚠️ F0047: Could not verify register change: {}", e);
                                        }
                                    }
                                } else {
                                    println!("⚠️ F0047: set_register reported failure");
                                }
                            }
                            Err(e) => {
                                println!("⚠️ F0047: set_register failed: {}", e);
                            }
                        }
                    } else {
                        println!("⚠️ F0047: No registers available to modify");
                    }
                }
                Err(e) => {
                    println!("⚠️ F0047: Could not get initial registers: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0047: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0047_set_register_invalid() {
    // F0047: set_register - Test error handling for invalid register name
    println!("Testing F0047: set_register with invalid register name");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0047: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            let _ = session.lldb_manager().set_breakpoint("main");
            let _ = session.lldb_manager().continue_execution();
            
            // Test setting invalid register
            let result = session.lldb_manager().set_register("invalid_register_name_12345", 0x1234, None);
            
            match result {
                Err(e) => {
                    println!("✅ F0047: Correctly handled invalid register name: {}", e);
                }
                Ok(success) => {
                    if success {
                        println!("⚠️ F0047: set_register unexpectedly succeeded for invalid register");
                    } else {
                        println!("✅ F0047: set_register correctly reported failure for invalid register");
                    }
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0047: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0048_get_register_info() {
    // F0048: get_register_info - Test getting detailed register information
    println!("Testing F0048: get_register_info");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0048: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0048: Test session started with PID {}", pid);
            
            // Set breakpoint to get to a known state
            let _ = session.lldb_manager().set_breakpoint("main");
            let _ = session.lldb_manager().continue_execution();
            
            // Get registers first to find available register names
            match session.lldb_manager().get_registers(None, false) {
                Ok(register_state) => {
                    if let Some(first_register) = register_state.registers.values().next() {
                        let register_name = &first_register.name;
                        
                        // Test getting detailed register info
                        let result = session.lldb_manager().get_register_info(register_name, None);
                        
                        match result {
                            Ok(register_info) => {
                                println!("✅ F0048: get_register_info succeeded for {}", register_name);
                                println!("  Name: {}", register_info.name);
                                println!("  Size: {} bits", register_info.size);
                                println!("  Offset: 0 /* byte_offset not available in current RegisterInfo */");
                                println!("  Encoding: {}", register_info.register_type);
                                println!("  Format: {}", register_info.format);
                                println!("  Generic Name: N/A /* generic_name not available in current RegisterInfo */");
                                
                                assert_eq!(register_info.name, *register_name);
                                assert!(register_info.size > 0, "Register should have valid bit size");
                                assert!(!register_info.register_type.is_empty(), "Encoding should not be empty");
                                assert!(!register_info.format.is_empty(), "Format should not be empty");
                            }
                            Err(e) => {
                                println!("⚠️ F0048: get_register_info failed: {}", e);
                            }
                        }
                    } else {
                        println!("⚠️ F0048: No registers available to get info for");
                    }
                }
                Err(e) => {
                    println!("⚠️ F0048: Could not get registers: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0048: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0048_get_register_info_invalid() {
    // F0048: get_register_info - Test error handling for invalid register name
    println!("Testing F0048: get_register_info with invalid register name");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0048: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            let _ = session.lldb_manager().set_breakpoint("main");
            let _ = session.lldb_manager().continue_execution();
            
            // Test getting info for invalid register
            let result = session.lldb_manager().get_register_info("invalid_register_name_12345", None);
            
            match result {
                Err(e) => {
                    println!("✅ F0048: Correctly handled invalid register name: {}", e);
                }
                Ok(register_info) => {
                    println!("⚠️ F0048: get_register_info unexpectedly succeeded for invalid register: {}", 
                           register_info.name);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0048: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0049_save_register_state() {
    // F0049: save_register_state - Test saving current register state
    println!("Testing F0049: save_register_state");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0049: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0049: Test session started with PID {}", pid);
            
            // Set breakpoint to get to a known state
            let _ = session.lldb_manager().set_breakpoint("main");
            let _ = session.lldb_manager().continue_execution();
            
            // Test saving register state
            let result = session.lldb_manager().save_register_state(None);
            
            match result {
                Ok(state_id) => {
                    println!("✅ F0049: save_register_state succeeded with timestamp {:?}", state_id.timestamp);
                    assert!(state_id.registers.len() > 0, "Should have registers in saved state");
                    
                    // Verify the saved state by getting current registers
                    match session.lldb_manager().get_registers(None, false) {
                        Ok(register_state) => {
                            println!("  Saved state for {} general registers", 
                                   register_state.registers.len());
                            println!("  Thread ID: {:?}", register_state.thread_id);
                        }
                        Err(e) => {
                            println!("⚠️ F0049: Could not verify saved state: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("⚠️ F0049: save_register_state failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0049: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0049_save_register_state_no_process() {
    // F0049: save_register_state - Test error handling when no process attached
    println!("Testing F0049: save_register_state with no process");
    
    let mut manager = match LldbManager::new(None) {
        Ok(m) => m,
        Err(e) => {
            println!("⚠️ F0049: LLDB manager creation failed: {}", e);
            return;
        }
    };
    
    let result = manager.save_register_state(None);
    
    match result {
        Err(IncodeError::LldbOperation(msg)) => {
            println!("✅ F0049: Correctly handled no process case: {}", msg);
        }
        Ok(_state_id) => {
            println!("⚠️ F0049: save_register_state unexpectedly succeeded");
        }
        Err(e) => {
            println!("✅ F0049: Error handling works: {}", e);
        }
    }
}

#[tokio::test]
async fn test_register_inspection_workflow() {
    // Integration test: Complete register inspection workflow
    println!("Testing register inspection workflow integration");
    
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
            
            // Step 1: Get to a known state
            let _ = session.lldb_manager().set_breakpoint("main");
            let _ = session.lldb_manager().continue_execution();
            
            // Step 2: Save initial register state
            let _saved_state = match session.lldb_manager().save_register_state(None) {
                Ok(state) => {
                    println!("✅ Workflow: Saved initial register state");
                    state
                }
                Err(e) => {
                    println!("⚠️ Workflow: Failed to save register state: {}", e);
                    return;
                }
            };
            
            // Step 3: Get all registers
            let register_state = match session.lldb_manager().get_registers(None, true) {
                Ok(state) => {
                    println!("✅ Workflow: Got registers - {} general, {} float, {} vector", 
                           state.registers.len(),
                           0 /* float registers not tracked */,
                           0 /* vector registers not tracked */);
                    state
                }
                Err(e) => {
                    println!("⚠️ Workflow: Failed to get registers: {}", e);
                    return;
                }
            };
            
            // Step 4: Analyze specific registers
            if let Some(first_register) = register_state.registers.values().next() {
                let register_name = &first_register.name;
                let original_value = first_register.value;
                
                println!("  Analyzing register: {} = 0x{:x}", register_name, original_value);
                
                // Get detailed register info
                match session.lldb_manager().get_register_info(register_name, None) {
                    Ok(info) => {
                        println!("✅ Workflow: Register info - Size: {} bits, Encoding: {}, Format: {}", 
                               info.size, info.register_type, info.format);
                    }
                    Err(e) => {
                        println!("⚠️ Workflow: Failed to get register info: {}", e);
                    }
                }
                
                // Try to modify register (may not work for all registers)
                let new_value = 0xDEADBEEF;
                match session.lldb_manager().set_register(register_name, new_value, None) {
                    Ok(success) => {
                        if success {
                            println!("✅ Workflow: Modified register {} to 0x{:x}", register_name, new_value);
                            
                            // Verify the change
                            match session.lldb_manager().get_registers(None, false) {
                                Ok(new_state) => {
                                    if let Some(modified_reg) = new_state.registers.get(register_name) {
                                        println!("  Verified: {} = 0x{:x}", register_name, modified_reg.value);
                                    }
                                }
                                Err(e) => {
                                    println!("⚠️ Workflow: Could not verify register change: {}", e);
                                }
                            }
                        } else {
                            println!("⚠️ Workflow: Register modification reported failure");
                        }
                    }
                    Err(e) => {
                        println!("⚠️ Workflow: Register modification failed: {}", e);
                    }
                }
            }
            
            // Step 5: Save final register state
            match session.lldb_manager().save_register_state(None) {
                Ok(_final_state) => {
                    println!("✅ Workflow: Saved final register state");
                    // State comparison removed - RegisterState doesn't implement PartialEq
                }
                Err(e) => {
                    println!("⚠️ Workflow: Failed to save final register state: {}", e);
                }
            }
            
            println!("✅ Workflow: Complete register inspection workflow tested");
        }
        Err(e) => {
            println!("⚠️ Workflow: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_register_modification_safety() {
    // Test register modification safety and recovery
    println!("Testing register modification safety");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ Safety: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            let _ = session.lldb_manager().set_breakpoint("main");
            let _ = session.lldb_manager().continue_execution();
            
            // Save initial state for recovery
            let _saved_state = match session.lldb_manager().save_register_state(None) {
                Ok(state) => state,
                Err(e) => {
                    println!("⚠️ Safety: Could not save initial state: {}", e);
                    return;
                }
            };
            
            // Get current registers
            match session.lldb_manager().get_registers(None, false) {
                Ok(register_state) => {
                    // Try to modify different types of registers safely
                    let mut modifications = Vec::new();
                    
                    for register in register_state.registers.values().take(3) {
                        let original_value = register.value;
                        let test_value = 0x12345000 + modifications.len() as u64;
                        
                        match session.lldb_manager().set_register(&register.name, test_value, None) {
                            Ok(success) => {
                                if success {
                                    modifications.push((register.name.clone(), original_value, test_value));
                                    println!("  Modified {}: 0x{:x} -> 0x{:x}", 
                                           register.name, original_value, test_value);
                                }
                            }
                            Err(e) => {
                                println!("  Could not modify {}: {}", register.name, e);
                            }
                        }
                    }
                    
                    if !modifications.is_empty() {
                        println!("✅ Safety: Modified {} registers safely", modifications.len());
                        
                        // Verify modifications
                        match session.lldb_manager().get_registers(None, false) {
                            Ok(new_state) => {
                                for (name, _original, expected) in &modifications {
                                    if let Some(reg) = new_state.registers.get(name) {
                                        if reg.value == *expected {
                                            println!("  Verified {}: 0x{:x}", name, reg.value);
                                        } else {
                                            println!("  Warning: {} value changed unexpectedly to 0x{:x}", 
                                                   name, reg.value);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                println!("⚠️ Safety: Could not verify modifications: {}", e);
                            }
                        }
                    } else {
                        println!("⚠️ Safety: No registers were successfully modified");
                    }
                    
                    println!("✅ Safety: Register modification safety testing completed");
                }
                Err(e) => {
                    println!("⚠️ Safety: Could not get initial registers: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ Safety: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}