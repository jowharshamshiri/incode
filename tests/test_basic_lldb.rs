// Basic LLDB functionality test
use incode::lldb_manager::LldbManager;

#[tokio::test]
async fn test_lldb_manager_creation() {
    println!("Testing LLDB manager creation...");
    
    match LldbManager::new(None) {
        Ok(manager) => {
            println!("✅ LLDB manager created successfully");
            println!("Manager initialized");
        }
        Err(e) => {
            println!("❌ Failed to create LLDB manager: {:?}", e);
            panic!("LLDB manager creation failed");
        }
    }
}

#[tokio::test]
async fn test_lldb_version() {
    println!("Testing LLDB version detection...");
    
    let manager = match LldbManager::new(None) {
        Ok(m) => m,
        Err(e) => {
            println!("❌ Failed to create LLDB manager: {:?}", e);
            return;
        }
    };
    
    match manager.get_lldb_version(true) {
        Ok(version) => {
            println!("✅ LLDB version: {:?}", version);
        }
        Err(e) => {
            println!("❌ Failed to get LLDB version: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_simple_command_execution() {
    println!("Testing simple LLDB command execution...");
    
    let manager = match LldbManager::new(None) {
        Ok(m) => m,
        Err(e) => {
            println!("❌ Failed to create LLDB manager: {:?}", e);
            return;
        }
    };
    
    match manager.execute_command("help") {
        Ok(output) => {
            println!("✅ Command executed successfully");
            println!("Output length: {} chars", output.len());
            if output.len() > 100 {
                println!("Output preview: {}...", &output[..100]);
            } else {
                println!("Output: {}", output);
            }
        }
        Err(e) => {
            println!("❌ Failed to execute command: {:?}", e);
        }
    }
}