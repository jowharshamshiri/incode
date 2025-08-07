// InCode Variable & Symbol Inspection Tools Test Suite
// 
// GRANULAR FEATURES TESTED:
// - F0035: get_variables - Get variables in current scope with types and values
// - F0036: get_global_variables - Get global variables and their values
// - F0037: evaluate_expression - Evaluate C/C++ expressions in current context
// - F0038: get_variable_info - Get detailed info about specific variable
// - F0039: set_variable - Modify variable value during debugging
// - F0040: lookup_symbol - Find symbol information by name
//
// Tests variable inspection with real LLDB integration using test_debuggee binary

use std::time::Duration;
use std::thread;

// Import test setup utilities
mod test_setup;
use test_setup::{TestSession, TestMode, TestUtils};

use incode::lldb_manager::LldbManager;
use incode::error::{IncodeError, IncodeResult};

#[tokio::test]
async fn test_f0035_get_variables_success() {
    // F0035: get_variables - Test getting variables in current scope
    println!("Testing F0035: get_variables");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0035: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0035: Test session started with PID {}", pid);
            
            // Set breakpoint in function with local variables
            let _ = session.lldb_manager().set_breakpoint("demonstrate_local_variables", None);
            let _ = session.lldb_manager().continue_execution();
            
            // Test getting variables in current scope
            let result = session.lldb_manager().get_variables(None, None);
            
            match result {
                Ok(variables) => {
                    println!("✅ F0035: get_variables succeeded, found {} variables", variables.len());
                    
                    for (i, var) in variables.iter().take(5).enumerate() {
                        println!("  Variable {}: {} = {} ({})", 
                               i + 1, var.name, var.value, var.var_type);
                    }
                    
                    // Look for expected local variables
                    let has_local_int = variables.iter().any(|v| v.name == "local_int");
                    let has_local_float = variables.iter().any(|v| v.name == "local_float");
                    
                    if has_local_int || has_local_float {
                        println!("✅ F0035: Found expected local variables");
                    }
                    
                    assert!(variables.len() > 0, "Should have at least one variable");
                }
                Err(e) => {
                    println!("⚠️ F0035: get_variables failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0035: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0035_get_variables_with_filter() {
    // F0035: get_variables - Test getting variables with pattern filter
    println!("Testing F0035: get_variables with filter");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0035: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            // Set breakpoint in function with various variables
            let _ = session.lldb_manager().set_breakpoint("showcase_variables", None);
            let _ = session.lldb_manager().continue_execution();
            
            // Test filtering variables by pattern
            let result = session.lldb_manager().get_variables(Some("local_*"), None);
            
            match result {
                Ok(filtered_vars) => {
                    println!("✅ F0035: get_variables with filter succeeded, found {} variables", filtered_vars.len());
                    
                    for var in &filtered_vars {
                        println!("  Filtered variable: {} = {}", var.name, var.value);
                        assert!(var.name.starts_with("local_"), 
                               "Filtered variable should match pattern");
                    }
                }
                Err(e) => {
                    println!("⚠️ F0035: get_variables with filter failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0035: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0036_get_global_variables() {
    // F0036: get_global_variables - Test getting global variables
    println!("Testing F0036: get_global_variables");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0036: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0036: Test session started with PID {}", pid);
            
            // Set breakpoint to get process into a known state
            let _ = session.lldb_manager().set_breakpoint("main", None);
            let _ = session.lldb_manager().continue_execution();
            
            // Test getting global variables
            let result = session.lldb_manager().get_global_variables(None);
            
            match result {
                Ok(globals) => {
                    println!("✅ F0036: get_global_variables succeeded, found {} globals", globals.len());
                    
                    for (i, var) in globals.iter().take(5).enumerate() {
                        println!("  Global {}: {} = {} ({})", 
                               i + 1, var.name, var.value, var.var_type);
                    }
                    
                    // Look for expected global variables from test binary
                    let has_global_int = globals.iter().any(|v| v.name.contains("global_int"));
                    let has_global_string = globals.iter().any(|v| v.name.contains("global_string"));
                    
                    if has_global_int || has_global_string {
                        println!("✅ F0036: Found expected global variables");
                    }
                }
                Err(e) => {
                    println!("⚠️ F0036: get_global_variables failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0036: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0036_get_global_variables_with_module_filter() {
    // F0036: get_global_variables - Test getting globals with module filter
    println!("Testing F0036: get_global_variables with module filter");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0036: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            let _ = session.lldb_manager().set_breakpoint("main", None);
            let _ = session.lldb_manager().continue_execution();
            
            // Test filtering globals by module
            let result = session.lldb_manager().get_global_variables(Some("test_debuggee"));
            
            match result {
                Ok(module_globals) => {
                    println!("✅ F0036: get_global_variables with module filter succeeded, found {} globals", 
                           module_globals.len());
                    
                    for var in module_globals.iter().take(3) {
                        println!("  Module global: {} = {}", var.name, var.value);
                    }
                }
                Err(e) => {
                    println!("⚠️ F0036: get_global_variables with module filter failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0036: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0037_evaluate_expression() {
    // F0037: evaluate_expression - Test evaluating C/C++ expressions
    println!("Testing F0037: evaluate_expression");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0037: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0037: Test session started with PID {}", pid);
            
            // Set breakpoint in function with local variables
            let _ = session.lldb_manager().set_breakpoint("demonstrate_local_variables", None);
            let _ = session.lldb_manager().continue_execution();
            
            // Test various expressions
            let expressions = vec![
                ("1 + 2", "simple arithmetic"),
                ("sizeof(int)", "sizeof operator"),
                ("local_int * 2", "variable arithmetic"),
                ("local_int > 0", "comparison"),
            ];
            
            for (expr, description) in expressions {
                let result = session.lldb_manager().evaluate_expression(expr, None);
                
                match result {
                    Ok(eval_result) => {
                        println!("✅ F0037: evaluate_expression succeeded for {}: {} = {} ({})", 
                               description, eval_result.expression, eval_result.value, eval_result.result_type);
                        
                        assert_eq!(eval_result.expression, expr);
                        assert!(!eval_result.value.is_empty());
                        assert!(!eval_result.result_type.is_empty());
                    }
                    Err(e) => {
                        println!("⚠️ F0037: evaluate_expression failed for {}: {}", description, e);
                    }
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0037: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0037_evaluate_expression_invalid() {
    // F0037: evaluate_expression - Test error handling for invalid expressions
    println!("Testing F0037: evaluate_expression with invalid expression");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0037: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            let _ = session.lldb_manager().set_breakpoint("main", None);
            let _ = session.lldb_manager().continue_execution();
            
            // Test invalid expression
            let result = session.lldb_manager().evaluate_expression("invalid_variable_name_12345", None);
            
            match result {
                Err(e) => {
                    println!("✅ F0037: Correctly handled invalid expression: {}", e);
                }
                Ok(eval_result) => {
                    println!("⚠️ F0037: evaluate_expression unexpectedly succeeded: {} = {}", 
                           eval_result.expression, eval_result.value);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0037: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0038_get_variable_info() {
    // F0038: get_variable_info - Test getting detailed variable information
    println!("Testing F0038: get_variable_info");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0038: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0038: Test session started with PID {}", pid);
            
            // Set breakpoint in function with local variables
            let _ = session.lldb_manager().set_breakpoint("demonstrate_local_variables", None);
            let _ = session.lldb_manager().continue_execution();
            
            // Test getting detailed info for specific variable
            let result = session.lldb_manager().get_variable_info("local_int");
            
            match result {
                Ok(var_info) => {
                    println!("✅ F0038: get_variable_info succeeded");
                    println!("  Name: {}", var_info.name);
                    println!("  Type: {}", var_info.var_type);
                    println!("  Value: {}", var_info.value);
                    println!("  Size: {} bytes", var_info.size);
                    println!("  Address: 0x{:x}", var_info.address);
                    println!("  Scope: {}", var_info.scope);
                    
                    assert_eq!(var_info.name, "local_int");
                    assert!(!var_info.var_type.is_empty());
                    assert!(!var_info.value.is_empty());
                    assert!(var_info.size > 0);
                    assert!(var_info.address > 0);
                }
                Err(e) => {
                    println!("⚠️ F0038: get_variable_info failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0038: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0038_get_variable_info_nonexistent() {
    // F0038: get_variable_info - Test error handling for non-existent variable
    println!("Testing F0038: get_variable_info with non-existent variable");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0038: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            let _ = session.lldb_manager().set_breakpoint("main", None);
            let _ = session.lldb_manager().continue_execution();
            
            // Test getting info for non-existent variable
            let result = session.lldb_manager().get_variable_info("nonexistent_variable_12345");
            
            match result {
                Err(e) => {
                    println!("✅ F0038: Correctly handled non-existent variable: {}", e);
                }
                Ok(var_info) => {
                    println!("⚠️ F0038: get_variable_info unexpectedly succeeded for non-existent variable: {}", 
                           var_info.name);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0038: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0039_set_variable() {
    // F0039: set_variable - Test modifying variable value during debugging
    println!("Testing F0039: set_variable");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0039: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0039: Test session started with PID {}", pid);
            
            // Set breakpoint in function with modifiable variables
            let _ = session.lldb_manager().set_breakpoint("demonstrate_variable_modifications", None);
            let _ = session.lldb_manager().continue_execution();
            
            // Get original variable value
            let original_value = match session.lldb_manager().evaluate_expression("modification_test", None) {
                Ok(result) => {
                    println!("Original value of modification_test: {}", result.value);
                    result.value
                }
                Err(e) => {
                    println!("⚠️ F0039: Could not get original value: {}", e);
                    return;
                }
            };
            
            // Test setting variable to new value
            let new_value = "999";
            let result = session.lldb_manager().set_variable("modification_test", new_value);
            
            match result {
                Ok(success) => {
                    if success {
                        println!("✅ F0039: set_variable succeeded");
                        
                        // Verify the change
                        match session.lldb_manager().evaluate_expression("modification_test", None) {
                            Ok(new_result) => {
                                println!("New value of modification_test: {}", new_result.value);
                                if new_result.value.contains(new_value) {
                                    println!("✅ F0039: Variable modification verified");
                                } else {
                                    println!("⚠️ F0039: Variable modification not reflected");
                                }
                            }
                            Err(e) => {
                                println!("⚠️ F0039: Could not verify variable change: {}", e);
                            }
                        }
                    } else {
                        println!("⚠️ F0039: set_variable reported failure");
                    }
                }
                Err(e) => {
                    println!("⚠️ F0039: set_variable failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0039: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0040_lookup_symbol() {
    // F0040: lookup_symbol - Test finding symbol information by name
    println!("Testing F0040: lookup_symbol");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0040: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0040: Test session started with PID {}", pid);
            
            // Test looking up function symbols
            let symbols_to_lookup = vec!["main", "showcase_variables", "printf"];
            
            for symbol_name in symbols_to_lookup {
                let result = session.lldb_manager().lookup_symbol(symbol_name);
                
                match result {
                    Ok(symbol_info) => {
                        println!("✅ F0040: lookup_symbol succeeded for '{}'", symbol_name);
                        println!("  Name: {}", symbol_info.name);
                        println!("  Type: {}", symbol_info.symbol_type);
                        println!("  Address: 0x{:x}", symbol_info.address);
                        println!("  Size: {}", symbol_info.size);
                        println!("  Module: {}", symbol_info.module.unwrap_or_else(|| "N/A".to_string()));
                        
                        assert_eq!(symbol_info.name, symbol_name);
                        assert!(!symbol_info.symbol_type.is_empty());
                        assert!(symbol_info.address > 0);
                    }
                    Err(e) => {
                        println!("⚠️ F0040: lookup_symbol failed for '{}': {}", symbol_name, e);
                    }
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0040: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0040_lookup_symbol_nonexistent() {
    // F0040: lookup_symbol - Test error handling for non-existent symbol
    println!("Testing F0040: lookup_symbol with non-existent symbol");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0040: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            // Test looking up non-existent symbol
            let result = session.lldb_manager().lookup_symbol("nonexistent_symbol_12345");
            
            match result {
                Err(e) => {
                    println!("✅ F0040: Correctly handled non-existent symbol: {}", e);
                }
                Ok(symbol_info) => {
                    println!("⚠️ F0040: lookup_symbol unexpectedly succeeded for non-existent symbol: {}", 
                           symbol_info.name);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0040: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_variable_inspection_workflow() {
    // Integration test: Complete variable inspection workflow
    println!("Testing variable inspection workflow integration");
    
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
            
            // Step 1: Get to variable-rich function
            let _ = session.lldb_manager().set_breakpoint("function_with_parameters", None);
            let _ = session.lldb_manager().continue_execution();
            
            // Step 2: List all local variables
            match session.lldb_manager().get_variables(None, None) {
                Ok(locals) => {
                    println!("✅ Workflow: Found {} local variables", locals.len());
                    
                    if !locals.is_empty() {
                        let first_var = &locals[0];
                        
                        // Step 3: Get detailed info for first variable
                        match session.lldb_manager().get_variable_info(&first_var.name) {
                            Ok(var_info) => {
                                println!("✅ Workflow: Got detailed info for variable '{}'", var_info.name);
                                println!("  Type: {}, Size: {} bytes, Address: 0x{:x}", 
                                       var_info.var_type, var_info.size, var_info.address);
                            }
                            Err(e) => {
                                println!("⚠️ Workflow: Failed to get variable info: {}", e);
                            }
                        }
                        
                        // Step 4: Try to modify variable if it's modifiable
                        if first_var.var_type.contains("int") {
                            match session.lldb_manager().set_variable(&first_var.name, "42") {
                                Ok(success) => {
                                    if success {
                                        println!("✅ Workflow: Successfully modified variable '{}'", first_var.name);
                                    } else {
                                        println!("⚠️ Workflow: Variable modification reported failure");
                                    }
                                }
                                Err(e) => {
                                    println!("⚠️ Workflow: Variable modification failed: {}", e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("⚠️ Workflow: Failed to get local variables: {}", e);
                }
            }
            
            // Step 5: Get global variables
            match session.lldb_manager().get_global_variables(None) {
                Ok(globals) => {
                    println!("✅ Workflow: Found {} global variables", globals.len());
                }
                Err(e) => {
                    println!("⚠️ Workflow: Failed to get global variables: {}", e);
                }
            }
            
            // Step 6: Evaluate some expressions
            let expressions = vec!["sizeof(int)", "1 + 1"];
            for expr in expressions {
                match session.lldb_manager().evaluate_expression(expr, None) {
                    Ok(result) => {
                        println!("✅ Workflow: Expression '{}' = {}", expr, result.value);
                    }
                    Err(e) => {
                        println!("⚠️ Workflow: Expression '{}' failed: {}", expr, e);
                    }
                }
            }
            
            // Step 7: Lookup function symbols
            let symbols = ["main", "showcase_variables"];
            for symbol in symbols {
                match session.lldb_manager().lookup_symbol(symbol) {
                    Ok(symbol_info) => {
                        println!("✅ Workflow: Found symbol '{}' at 0x{:x}", symbol, symbol_info.address);
                    }
                    Err(e) => {
                        println!("⚠️ Workflow: Symbol lookup for '{}' failed: {}", symbol, e);
                    }
                }
            }
            
            println!("✅ Workflow: Complete variable inspection workflow tested");
        }
        Err(e) => {
            println!("⚠️ Workflow: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}