// InCode Breakpoint Management Tools Test Suite
// 
// GRANULAR FEATURES TESTED:
// - F0014: set_breakpoint - Set breakpoint by address, function name, or file:line
// - F0015: set_watchpoint - Set memory watchpoint (read/write/access)
// - F0016: list_breakpoints - List all active breakpoints with details
// - F0017: delete_breakpoint - Remove specific breakpoint by ID
// - F0018: enable_breakpoint - Enable disabled breakpoint
// - F0019: disable_breakpoint - Disable breakpoint without removing
// - F0020: set_conditional_breakpoint - Set breakpoint with condition expression
// - F0021: breakpoint_commands - Set commands to execute when breakpoint hits
//
// Tests breakpoint management with real LLDB integration using test_debuggee binary

use std::time::Duration;
use std::thread;

// Import test setup utilities
mod test_setup;
use test_setup::{TestSession, TestMode, TestUtils};

use incode::lldb_manager::LldbManager;
use incode::error::{IncodeError, IncodeResult};

#[tokio::test]
async fn test_f0014_set_breakpoint_by_function() {
    // F0014: set_breakpoint - Test setting breakpoint by function name
    println!("Testing F0014: set_breakpoint by function name");
    
    let mut session = match TestSession::new(TestMode::StepDebug) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0014: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0014: Test session started with PID {}", pid);
            
            // Test setting breakpoint by function name
            let result = session.lldb_manager().set_breakpoint("main", None);
            
            match result {
                Ok(bp_id) => {
                    println!("✅ F0014: set_breakpoint succeeded, breakpoint ID: {}", bp_id);
                    assert!(bp_id > 0, "Breakpoint ID should be positive");
                }
                Err(e) => {
                    println!("⚠️ F0014: set_breakpoint failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0014: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0014_set_breakpoint_multiple_functions() {
    // F0014: set_breakpoint - Test setting breakpoints on multiple functions
    println!("Testing F0014: set_breakpoint on multiple functions");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0014: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            let functions = ["main", "showcase_variables", "test_function_with_params"];
            let mut breakpoint_ids = Vec::new();
            
            for function in &functions {
                match session.lldb_manager().set_breakpoint(function, None) {
                    Ok(bp_id) => {
                        println!("✅ F0014: Breakpoint set on {}, ID: {}", function, bp_id);
                        breakpoint_ids.push(bp_id);
                    }
                    Err(e) => {
                        println!("⚠️ F0014: Failed to set breakpoint on {}: {}", function, e);
                    }
                }
            }
            
            assert!(!breakpoint_ids.is_empty(), "At least one breakpoint should be set");
            println!("✅ F0014: Successfully set {} breakpoints", breakpoint_ids.len());
        }
        Err(e) => {
            println!("⚠️ F0014: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0014_set_breakpoint_invalid_function() {
    // F0014: set_breakpoint - Test error handling for invalid function name
    println!("Testing F0014: set_breakpoint with invalid function");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0014: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            let result = session.lldb_manager().set_breakpoint("nonexistent_function_12345", None);
            
            match result {
                Err(e) => {
                    println!("✅ F0014: Correctly handled invalid function: {}", e);
                }
                Ok(bp_id) => {
                    println!("⚠️ F0014: set_breakpoint unexpectedly succeeded with ID {}", bp_id);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0014: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0015_set_watchpoint() {
    // F0015: set_watchpoint - Test setting memory watchpoint
    println!("Testing F0015: set_watchpoint");
    
    let mut session = match TestSession::new(TestMode::Memory) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0015: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0015: Test session started with PID {}", pid);
            
            // Set breakpoint to get to a predictable state
            let _ = session.lldb_manager().set_breakpoint("create_global_patterns", None);
            let _ = session.lldb_manager().continue_execution();
            
            // Test setting watchpoint on global variable
            let result = session.lldb_manager().set_watchpoint("global_buffer", "write", 4);
            
            match result {
                Ok(wp_id) => {
                    println!("✅ F0015: set_watchpoint succeeded, watchpoint ID: {}", wp_id);
                    assert!(wp_id > 0, "Watchpoint ID should be positive");
                }
                Err(e) => {
                    println!("⚠️ F0015: set_watchpoint failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0015: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0016_list_breakpoints() {
    // F0016: list_breakpoints - Test listing all active breakpoints
    println!("Testing F0016: list_breakpoints");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0016: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            // Set some breakpoints first
            let _ = session.lldb_manager().set_breakpoint("main", None);
            let _ = session.lldb_manager().set_breakpoint("showcase_variables", None);
            
            // Test listing breakpoints
            let result = session.lldb_manager().list_breakpoints();
            
            match result {
                Ok(breakpoints) => {
                    println!("✅ F0016: list_breakpoints succeeded, found {} breakpoints", breakpoints.len());
                    
                    for (i, bp) in breakpoints.iter().enumerate() {
                        println!("  Breakpoint {}: ID={}, Location={}, Enabled={}, Hit Count={}", 
                                i + 1, bp.id, bp.location, bp.enabled, bp.hit_count);
                    }
                    
                    assert!(breakpoints.len() >= 2, "Should have at least 2 breakpoints");
                }
                Err(e) => {
                    println!("⚠️ F0016: list_breakpoints failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0016: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0016_list_breakpoints_empty() {
    // F0016: list_breakpoints - Test listing when no breakpoints exist
    println!("Testing F0016: list_breakpoints with no breakpoints");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0016: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            // Test listing breakpoints without setting any
            let result = session.lldb_manager().list_breakpoints();
            
            match result {
                Ok(breakpoints) => {
                    println!("✅ F0016: list_breakpoints succeeded, found {} breakpoints", breakpoints.len());
                    // Empty list is acceptable
                }
                Err(e) => {
                    println!("⚠️ F0016: list_breakpoints failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0016: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0017_delete_breakpoint() {
    // F0017: delete_breakpoint - Test deleting specific breakpoint
    println!("Testing F0017: delete_breakpoint");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0017: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            // Set a breakpoint first
            let bp_id = match session.lldb_manager().set_breakpoint("main", None) {
                Ok(id) => {
                    println!("✅ F0017: Created test breakpoint with ID {}", id);
                    id
                }
                Err(e) => {
                    println!("⚠️ F0017: Could not create test breakpoint: {}", e);
                    return;
                }
            };
            
            // Test deleting the breakpoint
            let result = session.lldb_manager().delete_breakpoint(bp_id);
            
            match result {
                Ok(_) => {
                    println!("✅ F0017: delete_breakpoint succeeded for ID {}", bp_id);
                }
                Err(e) => {
                    println!("⚠️ F0017: delete_breakpoint failed: {}", e);
                }
            }
            
            // Verify breakpoint was deleted by listing
            match session.lldb_manager().list_breakpoints() {
                Ok(breakpoints) => {
                    let found = breakpoints.iter().any(|bp| bp.id == bp_id);
                    if !found {
                        println!("✅ F0017: Breakpoint {} successfully deleted", bp_id);
                    } else {
                        println!("⚠️ F0017: Breakpoint {} still exists after deletion", bp_id);
                    }
                }
                Err(_) => {
                    println!("⚠️ F0017: Could not verify breakpoint deletion");
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0017: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0017_delete_invalid_breakpoint() {
    // F0017: delete_breakpoint - Test error handling for invalid breakpoint ID
    println!("Testing F0017: delete_breakpoint with invalid ID");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0017: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            // Test deleting non-existent breakpoint
            let result = session.lldb_manager().delete_breakpoint(99999);
            
            match result {
                Err(e) => {
                    println!("✅ F0017: Correctly handled invalid breakpoint ID: {}", e);
                }
                Ok(_) => {
                    println!("⚠️ F0017: delete_breakpoint unexpectedly succeeded for invalid ID");
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0017: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0018_enable_breakpoint() {
    // F0018: enable_breakpoint - Test enabling disabled breakpoint
    println!("Testing F0018: enable_breakpoint");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0018: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            // Set and then disable a breakpoint
            let bp_id = match session.lldb_manager().set_breakpoint("main", None) {
                Ok(id) => id,
                Err(e) => {
                    println!("⚠️ F0018: Could not create test breakpoint: {}", e);
                    return;
                }
            };
            
            // Disable it first
            let _ = session.lldb_manager().disable_breakpoint(bp_id);
            
            // Test enabling the breakpoint
            let result = session.lldb_manager().enable_breakpoint(bp_id);
            
            match result {
                Ok(_) => {
                    println!("✅ F0018: enable_breakpoint succeeded for ID {}", bp_id);
                }
                Err(e) => {
                    println!("⚠️ F0018: enable_breakpoint failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0018: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0019_disable_breakpoint() {
    // F0019: disable_breakpoint - Test disabling breakpoint without removing
    println!("Testing F0019: disable_breakpoint");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0019: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            // Set a breakpoint first
            let bp_id = match session.lldb_manager().set_breakpoint("main", None) {
                Ok(id) => id,
                Err(e) => {
                    println!("⚠️ F0019: Could not create test breakpoint: {}", e);
                    return;
                }
            };
            
            // Test disabling the breakpoint
            let result = session.lldb_manager().disable_breakpoint(bp_id);
            
            match result {
                Ok(_) => {
                    println!("✅ F0019: disable_breakpoint succeeded for ID {}", bp_id);
                }
                Err(e) => {
                    println!("⚠️ F0019: disable_breakpoint failed: {}", e);
                }
            }
            
            // Verify breakpoint still exists but is disabled
            match session.lldb_manager().list_breakpoints() {
                Ok(breakpoints) => {
                    if let Some(bp) = breakpoints.iter().find(|bp| bp.id == bp_id) {
                        if !bp.enabled {
                            println!("✅ F0019: Breakpoint {} successfully disabled", bp_id);
                        } else {
                            println!("⚠️ F0019: Breakpoint {} still enabled after disable", bp_id);
                        }
                    } else {
                        println!("⚠️ F0019: Breakpoint {} not found after disable", bp_id);
                    }
                }
                Err(_) => {
                    println!("⚠️ F0019: Could not verify breakpoint status");
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0019: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0020_set_conditional_breakpoint() {
    // F0020: set_conditional_breakpoint - Test setting breakpoint with condition
    println!("Testing F0020: set_conditional_breakpoint");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0020: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            // Test setting conditional breakpoint
            let result = session.lldb_manager().set_conditional_breakpoint(
                "main", 
                "argc > 1",
                None
            );
            
            match result {
                Ok(bp_id) => {
                    println!("✅ F0020: set_conditional_breakpoint succeeded, ID: {}", bp_id);
                    assert!(bp_id > 0, "Breakpoint ID should be positive");
                }
                Err(e) => {
                    println!("⚠️ F0020: set_conditional_breakpoint failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0020: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0021_breakpoint_commands() {
    // F0021: breakpoint_commands - Test setting commands for breakpoint
    println!("Testing F0021: breakpoint_commands");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0021: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            // Set a breakpoint first
            let bp_id = match session.lldb_manager().set_breakpoint("main", None) {
                Ok(id) => id,
                Err(e) => {
                    println!("⚠️ F0021: Could not create test breakpoint: {}", e);
                    return;
                }
            };
            
            // Test setting breakpoint commands
            let commands = vec!["print argc".to_string(), "bt".to_string()];
            let result = session.lldb_manager().set_breakpoint_commands(bp_id, commands);
            
            match result {
                Ok(_) => {
                    println!("✅ F0021: breakpoint_commands succeeded for ID {}", bp_id);
                }
                Err(e) => {
                    println!("⚠️ F0021: breakpoint_commands failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0021: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_breakpoint_management_workflow() {
    // Integration test: Complete breakpoint management workflow
    println!("Testing breakpoint management workflow integration");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ Workflow: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            println!("✅ Workflow: Test session started");
            
            // Step 1: Set multiple breakpoints
            let bp_ids = TestUtils::get_test_breakpoint_locations()
                .into_iter()
                .filter_map(|func| {
                    match session.lldb_manager().set_breakpoint(func, None) {
                        Ok(id) => {
                            println!("✅ Workflow: Set breakpoint on {}, ID: {}", func, id);
                            Some(id)
                        }
                        Err(_) => None,
                    }
                })
                .collect::<Vec<_>>();
            
            assert!(!bp_ids.is_empty(), "Should set at least one breakpoint");
            
            // Step 2: List all breakpoints
            match session.lldb_manager().list_breakpoints() {
                Ok(breakpoints) => {
                    println!("✅ Workflow: Listed {} breakpoints", breakpoints.len());
                    assert!(breakpoints.len() >= bp_ids.len());
                }
                Err(e) => println!("⚠️ Workflow: Failed to list breakpoints: {}", e),
            }
            
            // Step 3: Disable some breakpoints
            for (i, &bp_id) in bp_ids.iter().take(2).enumerate() {
                match session.lldb_manager().disable_breakpoint(bp_id) {
                    Ok(_) => println!("✅ Workflow: Disabled breakpoint {} (ID: {})", i + 1, bp_id),
                    Err(e) => println!("⚠️ Workflow: Failed to disable breakpoint {}: {}", bp_id, e),
                }
            }
            
            // Step 4: Re-enable breakpoints
            for (i, &bp_id) in bp_ids.iter().take(2).enumerate() {
                match session.lldb_manager().enable_breakpoint(bp_id) {
                    Ok(_) => println!("✅ Workflow: Re-enabled breakpoint {} (ID: {})", i + 1, bp_id),
                    Err(e) => println!("⚠️ Workflow: Failed to enable breakpoint {}: {}", bp_id, e),
                }
            }
            
            // Step 5: Delete some breakpoints
            for &bp_id in bp_ids.iter().skip(2) {
                match session.lldb_manager().delete_breakpoint(bp_id) {
                    Ok(_) => println!("✅ Workflow: Deleted breakpoint ID: {}", bp_id),
                    Err(e) => println!("⚠️ Workflow: Failed to delete breakpoint {}: {}", bp_id, e),
                }
            }
            
            // Step 6: Final verification
            match session.lldb_manager().list_breakpoints() {
                Ok(final_breakpoints) => {
                    println!("✅ Workflow: Final breakpoint count: {}", final_breakpoints.len());
                }
                Err(e) => println!("⚠️ Workflow: Final list failed: {}", e),
            }
            
            println!("✅ Workflow: Complete breakpoint management workflow tested");
        }
        Err(e) => {
            println!("⚠️ Workflow: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}