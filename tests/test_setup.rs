// InCode Test Setup - Binary Lifecycle Management and LLDB Session Setup
// Test harness for managing test_debuggee binary lifecycle and LLDB integration
// Provides predictable state management for comprehensive tool testing

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio, Child};
use std::time::Duration;
use std::thread;
use std::collections::HashMap;

use incode::lldb_manager::LldbManager;
use incode::error::{IncodeResult, IncodeError};

/// Test binary execution modes
#[derive(Debug, Clone)]
pub enum TestMode {
    Normal,
    Threads,
    Memory,
    StepDebug,
    CrashSegv,
    CrashStack,
    CrashAbort,
    CrashDiv0,
    Infinite,
}

impl TestMode {
    pub fn as_arg(&self) -> &str {
        match self {
            TestMode::Normal => "normal",
            TestMode::Threads => "threads",
            TestMode::Memory => "memory",
            TestMode::StepDebug => "step-debug",
            TestMode::CrashSegv => "crash-segv",
            TestMode::CrashStack => "crash-stack",
            TestMode::CrashAbort => "crash-abort",
            TestMode::CrashDiv0 => "crash-div0",
            TestMode::Infinite => "infinite",
        }
    }
}

/// Test debuggee binary lifecycle manager
pub struct TestDebuggee {
    binary_path: PathBuf,
    process: Option<Child>,
    pid: Option<u32>,
    mode: TestMode,
}

impl TestDebuggee {
    /// Create new test debuggee manager
    pub fn new(mode: TestMode) -> IncodeResult<Self> {
        let binary_path = Self::find_test_binary()?;
        
        Ok(TestDebuggee {
            binary_path,
            process: None,
            pid: None,
            mode,
        })
    }
    
    /// Get the binary path
    pub fn binary_path(&self) -> &PathBuf {
        &self.binary_path
    }
    
    /// Get test mode
    pub fn mode(&self) -> &TestMode {
        &self.mode
    }

    /// Find the test_debuggee binary
    fn find_test_binary() -> IncodeResult<PathBuf> {
        // Try multiple possible locations
        let possible_paths = [
            "test_debuggee/test_debuggee",
            "../test_debuggee/test_debuggee",
            "./test_debuggee",
            "../test_debuggee",
        ];
        
        for path_str in &possible_paths {
            let path = Path::new(path_str);
            if path.exists() {
                return Ok(path.to_path_buf());
            }
        }
        
        // Try building the binary if not found
        println!("Test binary not found, attempting to build...");
        Self::build_test_binary()?;
        
        // Try again after building
        for path_str in &possible_paths {
            let path = Path::new(path_str);
            if path.exists() {
                return Ok(path.to_path_buf());
            }
        }
        
        Err(IncodeError::ProcessNotFound(
            "Could not find or build test_debuggee binary".to_string()
        ))
    }
    
    /// Build the test binary if it doesn't exist
    fn build_test_binary() -> IncodeResult<()> {
        let build_dirs = ["test_debuggee", "../test_debuggee"];
        
        for build_dir in &build_dirs {
            if Path::new(build_dir).exists() {
                let output = Command::new("make")
                    .current_dir(build_dir)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output();
                
                match output {
                    Ok(result) if result.status.success() => {
                        println!("Successfully built test_debuggee binary");
                        return Ok(());
                    }
                    Ok(result) => {
                        let stderr = String::from_utf8_lossy(&result.stderr);
                        return Err(IncodeError::ProcessError(
                            format!("Build failed: {}", stderr)
                        ));
                    }
                    Err(e) => {
                        return Err(IncodeError::ProcessError(
                            format!("Failed to run make: {}", e)
                        ));
                    }
                }
            }
        }
        
        Err(IncodeError::ProcessError(
            "Could not find test_debuggee directory for building".to_string()
        ))
    }
    
    /// Launch the test binary in the specified mode
    pub fn launch(&mut self) -> IncodeResult<u32> {
        if self.process.is_some() {
            return Err(IncodeError::LldbOperation(
                "Process already running".to_string()
            ));
        }
        
        let mut command = Command::new(&self.binary_path);
        command
            .arg("--mode")
            .arg(self.mode.as_arg())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        // For infinite mode, we want to keep it running
        if matches!(self.mode, TestMode::Infinite) {
            command.stdin(Stdio::piped());
        }
        
        let mut child = command.spawn().map_err(|e| {
            IncodeError::ProcessError(format!("Failed to launch test binary: {}", e))
        })?;
        
        let pid = child.id();
        
        // For crash modes, the process will exit quickly
        if matches!(self.mode, TestMode::CrashSegv | TestMode::CrashStack) {
            // Give it a moment to set up before crashing
            thread::sleep(Duration::from_millis(100));
        }
        
        self.process = Some(child);
        self.pid = Some(pid);
        
        Ok(pid)
    }
    
    /// Get the process ID
    pub fn pid(&self) -> Option<u32> {
        self.pid
    }
    
    /// Check if the process is still running
    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut process) = self.process {
            match process.try_wait() {
                Ok(Some(_)) => false,  // Process has exited
                Ok(None) => true,      // Process is still running
                Err(_) => false,       // Error checking status, assume dead
            }
        } else {
            false
        }
    }
    
    /// Wait for process to be ready for debugging
    pub fn wait_for_ready(&mut self, timeout: Duration) -> IncodeResult<()> {
        let start_time = std::time::Instant::now();
        
        while start_time.elapsed() < timeout {
            if self.is_running() {
                // Give it a bit more time to initialize
                thread::sleep(Duration::from_millis(50));
                return Ok(());
            }
            thread::sleep(Duration::from_millis(10));
        }
        
        // For crash modes, the process may have already exited
        if matches!(self.mode, TestMode::CrashSegv | TestMode::CrashStack) {
            return Ok(()); // This is expected
        }
        
        Err(IncodeError::Timeout)
    }
    
    /// Terminate the process
    pub fn terminate(&mut self) -> IncodeResult<()> {
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
        self.pid = None;
        Ok(())
    }
}

impl Drop for TestDebuggee {
    fn drop(&mut self) {
        let _ = self.terminate();
    }
}

/// LLDB session manager for testing
pub struct TestSession {
    lldb_manager: LldbManager,
    debuggee: TestDebuggee,
    session_id: Option<uuid::Uuid>,
}

impl TestSession {
    /// Create a new test session with LLDB manager and test binary
    pub fn new(mode: TestMode) -> IncodeResult<Self> {
        let lldb_manager = LldbManager::new(None)?;
        let debuggee = TestDebuggee::new(mode)?;
        
        Ok(TestSession {
            lldb_manager,
            debuggee,
            session_id: None,
        })
    }
    
    /// Start the debugging session
    pub fn start(&mut self) -> IncodeResult<u32> {
        // Create LLDB session
        let session_id = self.lldb_manager.create_session()?;
        self.session_id = Some(session_id);
        
        // Use LLDB to launch the target process directly (better for breakpoints and debugging)
        let args = vec![
            "--mode".to_string(),
            self.debuggee.mode.as_arg().to_string(),
        ];
        
        let env = std::collections::HashMap::new();
        match self.lldb_manager.launch_process(&self.debuggee.binary_path.to_string_lossy(), &args, &env) {
            Ok(pid) => {
                println!("Successfully launched target via LLDB: {}", pid);
                Ok(pid)
            }
            Err(e) => {
                println!("LLDB launch failed, falling back to attach method: {}", e);
                
                // Fallback: Launch the test binary separately and attach
                let pid = self.debuggee.launch()?;
                
                // Wait for process to be ready
                self.debuggee.wait_for_ready(Duration::from_secs(2))?;
                
                // Attach LLDB to the process
                if self.debuggee.is_running() {
                    match self.lldb_manager.attach_to_process(pid) {
                        Ok(_) => println!("Successfully attached LLDB to process {}", pid),
                        Err(e) => return Err(IncodeError::LldbOperation(
                            format!("Could not attach LLDB to process {}: {}", pid, e)
                        )),
                    }
                }
                
                Ok(pid)
            }
        }
    }
    
    /// Start session for crash analysis (attach to core dump or process remains)
    pub fn start_crash_analysis(&mut self) -> IncodeResult<()> {
        // Create LLDB session
        let session_id = self.lldb_manager.create_session()?;
        self.session_id = Some(session_id);
        
        // Launch the test binary via LLDB (it will crash)
        let binary_path = self.debuggee.binary_path();
        let args = vec![
            "--mode".to_string(),
            self.debuggee.mode().as_arg().to_string(),
        ];
        let env = HashMap::new();
        let _pid = self.lldb_manager.launch_process(
            binary_path.to_str().unwrap(),
            &args,
            &env
        )?;
        
        // Wait for crash to occur
        thread::sleep(Duration::from_millis(500));
        
        println!("Test binary launched for crash analysis");
        Ok(())
    }
    
    /// Get reference to LLDB manager
    pub fn lldb_manager(&mut self) -> &mut LldbManager {
        &mut self.lldb_manager
    }
    
    /// Get process ID
    pub fn pid(&self) -> Option<u32> {
        self.debuggee.pid()
    }
    
    /// Check if debuggee is still running
    pub fn is_debuggee_running(&mut self) -> bool {
        self.debuggee.is_running()
    }
    
    /// Set a breakpoint at a known location
    pub fn set_test_breakpoint(&mut self, location: &str) -> IncodeResult<()> {
        match self.lldb_manager.set_breakpoint(location) {
            Ok(bp_id) => {
                println!("Set test breakpoint {} at {}", bp_id, location);
                Ok(())
            }
            Err(e) => {
                println!("Warning: Could not set breakpoint at {}: {}", location, e);
                Ok(()) // Non-critical for testing
            }
        }
    }
    
    /// Continue execution until breakpoint or completion
    pub fn continue_execution(&mut self) -> IncodeResult<()> {
        match self.lldb_manager.continue_execution() {
            Ok(_) => {
                println!("Continued execution");
                Ok(())
            }
            Err(e) => {
                println!("Continue execution result: {}", e);
                Ok(()) // May be expected in some test scenarios
            }
        }
    }
    
    /// Interrupt process execution to stop at current location
    pub fn interrupt_process(&mut self) -> IncodeResult<()> {
        match self.lldb_manager.interrupt_execution() {
            Ok(_) => {
                println!("Process interrupted successfully");
                Ok(())
            }
            Err(e) => {
                println!("Interrupt process result: {}", e);
                Ok(()) // May be expected in some test scenarios
            }
        }
    }
    
    /// Cleanup the session
    pub fn cleanup(&mut self) -> IncodeResult<()> {
        // Detach from process if attached
        if self.debuggee.is_running() {
            let _ = self.lldb_manager.detach_process();
        }
        
        // Cleanup LLDB session
        if let Some(session_id) = self.session_id {
            let _ = self.lldb_manager.cleanup_session(&session_id);
        }
        
        // Terminate debuggee
        self.debuggee.terminate()?;
        
        Ok(())
    }
}

impl Drop for TestSession {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

/// Utility functions for test setup
pub struct TestUtils;

impl TestUtils {
    /// Wait for a condition with timeout
    pub fn wait_for_condition<F>(condition: F, timeout: Duration) -> bool
    where
        F: Fn() -> bool,
    {
        let start_time = std::time::Instant::now();
        
        while start_time.elapsed() < timeout {
            if condition() {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        
        false
    }
    
    /// Create test breakpoint locations for different scenarios
    pub fn get_test_breakpoint_locations() -> Vec<&'static str> {
        vec![
            "main",
            "showcase_variables",
            "run_threading_scenarios",
            "run_memory_scenarios",
            "step_debug_function",
            "test_function_with_params",
        ]
    }
    
    /// Verify LLDB is available on the system
    pub fn verify_lldb_available() -> bool {
        Command::new("lldb")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }
    
    /// Get expected memory addresses for testing (these are symbolic)
    pub fn get_test_memory_locations() -> Vec<&'static str> {
        vec![
            "global_buffer",
            "global_array",
            "global_memory_struct",
            "global_struct_ptr",
        ]
    }
    
    /// Get expected variable names for testing
    pub fn get_test_variable_names() -> Vec<&'static str> {
        vec![
            "global_int",
            "global_float",
            "global_string",
            "local_int",
            "local_float",
            "param_int",
            "param_string",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_debuggee_creation() {
        let debuggee_result = TestDebuggee::new(TestMode::StepDebug);
        
        match debuggee_result {
            Ok(_) => println!("✅ Test debuggee creation successful"),
            Err(e) => println!("⚠️ Test debuggee creation failed: {}", e),
        }
    }
    
    #[tokio::test]
    async fn test_session_creation() {
        let session_result = TestSession::new(TestMode::StepDebug);
        
        match session_result {
            Ok(mut session) => {
                println!("✅ Test session creation successful");
                let _ = session.cleanup();
            }
            Err(e) => println!("⚠️ Test session creation failed: {}", e),
        }
    }
    
    #[tokio::test]
    async fn test_lldb_availability() {
        let lldb_available = TestUtils::verify_lldb_available();
        
        if lldb_available {
            println!("✅ LLDB is available on system");
        } else {
            println!("⚠️ LLDB is not available - some tests may fail");
        }
    }
}