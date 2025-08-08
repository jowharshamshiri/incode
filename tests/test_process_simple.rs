// Simple Process Control Tests - Direct LLDB Integration
use incode::lldb_manager::LldbManager;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

fn get_test_binary_path() -> PathBuf {
    let mut path = std::env::current_dir().expect("Failed to get current directory");
    path.push("test_debuggee");
    path
}

#[tokio::test]
async fn test_f0001_launch_simple_executable() {
    println!("Testing F0001: launch_process with simple executable");
    
    let mut manager = LldbManager::new(None).expect("Failed to create LLDB manager");
    let test_binary = get_test_binary_path();
    
    // Verify test binary exists
    assert!(test_binary.exists(), "Test binary should exist: {:?}", test_binary);
    
    let args = vec!["--mode".to_string(), "normal".to_string()];
    let env = HashMap::new();
    
    println!("Attempting to launch: {:?}", test_binary);
    
    match manager.launch_process(&test_binary.to_string_lossy(), &args, &env) {
        Ok(pid) => {
            println!("✅ F0001: Successfully launched process with PID {}", pid);
            assert!(pid > 0, "PID should be greater than 0");
            
            // Try to get process info
            match manager.get_process_info() {
                Ok(info) => {
                    println!("Process info - PID: {}, State: {}", info.pid, info.state);
                    assert_eq!(info.pid, pid, "PID should match");
                }
                Err(e) => {
                    println!("Warning: Could not get process info: {:?}", e);
                }
            }
            
            // Try to kill the process to clean up
            match manager.kill_process() {
                Ok(_) => println!("Process terminated successfully"),
                Err(e) => println!("Warning: Could not terminate process: {:?}", e),
            }
        }
        Err(e) => {
            println!("❌ F0001: Failed to launch process: {:?}", e);
            panic!("Process launch failed: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_f0001_launch_invalid_executable() {
    println!("Testing F0001: launch_process with invalid executable");
    
    let mut manager = LldbManager::new(None).expect("Failed to create LLDB manager");
    let args = vec![];
    let env = HashMap::new();
    
    match manager.launch_process("/nonexistent/executable", &args, &env) {
        Ok(_) => {
            panic!("Should not succeed with invalid executable");
        }
        Err(e) => {
            println!("✅ F0001: Correctly handled invalid executable: {}", e);
        }
    }
}

#[tokio::test] 
async fn test_f0005_process_info_without_process() {
    println!("Testing F0005: get_process_info without process");
    
    let manager = LldbManager::new(None).expect("Failed to create LLDB manager");
    
    match manager.get_process_info() {
        Ok(_) => {
            println!("Warning: get_process_info succeeded without process");
        }
        Err(e) => {
            println!("✅ F0005: Correctly handled no process: {}", e);
        }
    }
}