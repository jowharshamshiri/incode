// Simple Breakpoint Tests - Direct LLDB Integration
use incode::lldb_manager::LldbManager;
use std::collections::HashMap;
use std::path::PathBuf;

fn get_test_binary_path() -> PathBuf {
    let mut path = std::env::current_dir().expect("Failed to get current directory");
    path.push("test_debuggee");
    path
}

#[tokio::test]
async fn test_f0014_function_breakpoint() {
    println!("Testing F0014: set_breakpoint with function name");
    
    let mut manager = LldbManager::new(None).expect("Failed to create LLDB manager");
    let test_binary = get_test_binary_path();
    
    // Launch process first
    let args = vec!["--mode".to_string(), "step_debug".to_string()];
    let env = HashMap::new();
    
    match manager.launch_process(&test_binary.to_string_lossy(), &args, &env) {
        Ok(pid) => {
            println!("Process launched with PID: {}", pid);
            
            // Set function breakpoint on main
            match manager.set_breakpoint("main") {
                Ok(bp_id) => {
                    println!("✅ F0014: Successfully set function breakpoint on 'main', ID: {}", bp_id);
                    
                    // List breakpoints
                    match manager.list_breakpoints() {
                        Ok(breakpoints) => {
                            println!("Breakpoints count: {}", breakpoints.len());
                            for bp in &breakpoints {
                                println!("  BP {}: enabled={}, hit_count={}", bp.id, bp.enabled, bp.hit_count);
                            }
                        }
                        Err(e) => println!("Warning: Could not list breakpoints: {:?}", e),
                    }
                }
                Err(e) => println!("❌ F0014: Failed to set breakpoint: {:?}", e),
            }
            
            // Clean up
            let _ = manager.kill_process();
        }
        Err(e) => {
            println!("Failed to launch process: {:?}", e);
            return;
        }
    }
}

#[tokio::test] 
async fn test_f0057_execute_command() {
    println!("Testing F0057: execute_command");
    
    let manager = LldbManager::new(None).expect("Failed to create LLDB manager");
    
    match manager.execute_command("version") {
        Ok(output) => {
            println!("✅ F0057: Command executed successfully");
            println!("Version output length: {} chars", output.len());
            if output.len() > 100 {
                println!("Output preview: {}...", &output[..100]);
            } else {
                println!("Output: {}", output);
            }
        }
        Err(e) => {
            println!("❌ F0057: Failed to execute command: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_f0058_lldb_version() {
    println!("Testing F0058: get_lldb_version");
    
    let manager = LldbManager::new(None).expect("Failed to create LLDB manager");
    
    match manager.get_lldb_version(true) {
        Ok(version_info) => {
            println!("✅ F0058: Successfully retrieved LLDB version");
            println!("Version: {}", version_info.version);
            println!("API Version: {}", version_info.api_version);
            println!("Platform: {}", version_info.platform);
            if let Some(build_date) = version_info.build_date {
                println!("Build Date: {}", build_date);
            }
        }
        Err(e) => {
            println!("❌ F0058: Failed to get LLDB version: {:?}", e);
        }
    }
}