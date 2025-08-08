// InCode LLDB Control Tools - Comprehensive Test Suite
// Tests F0057-F0059: execute_command, get_lldb_version, set_lldb_settings
// Real LLDB integration testing with test_debuggee binary

use std::collections::HashMap;
use serde_json::Value;

mod test_setup;
use test_setup::{TestDebuggee, TestMode, TestSession};

use incode::tools::lldb_control::{ExecuteCommandTool, GetLldbVersionTool, SetLldbSettingsTool};
use incode::tools::{Tool, ToolResponse};

#[tokio::test]
async fn test_execute_command_comprehensive() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::Normal).expect("Failed to create LLDB session");
    let _pid = session.start().expect("Failed to start session");
    
    // Set a breakpoint to have an active debugging session
    session.set_test_breakpoint("main").expect("Failed to set breakpoint");
    session.continue_execution().expect("Failed to continue");
    
    let tool = ExecuteCommandTool;
    
    // Test 1: Execute basic LLDB command
    let mut args = HashMap::new();
    args.insert("command".to_string(), Value::String("help".to_string()));
    
    let result = tool.execute(args, session.lldb_manager()).await.expect("execute_command help failed");
    let result_str = match result {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response: Value = serde_json::from_str(&result_str).expect("Invalid JSON response");
    
    // Validate command execution response
    assert_eq!(response["status"].as_str().unwrap_or(""), "executed", "execute_command should execute successfully");
    assert!(response["output"].is_string(), "Should return command output");
    assert!(response["command"].is_string(), "Should echo back command");
    
    let output = response["output"].as_str().expect("output should be string");
    assert!(!output.is_empty(), "Help command should produce output");
    assert!(output.to_lowercase().contains("help"), "Help output should contain help information");
    
    // Test 2: Execute target info command
    let mut args_target = HashMap::new();
    args_target.insert("command".to_string(), Value::String("target list".to_string()));
    
    let result_target = tool.execute(args_target, session.lldb_manager()).await.expect("execute_command target list failed");
    let result_target_str = match result_target {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response_target: Value = serde_json::from_str(&result_target_str).expect("Invalid JSON response");
    
    assert!(response_target["success"].as_bool().unwrap_or(false), "execute_command target list should succeed");
    let target_output = response_target["output"].as_str().expect("output should be string");
    assert!(target_output.contains("test_debuggee"), "Target list should show test_debuggee");
    
    // Test 3: Execute thread info command
    let mut args_thread = HashMap::new();
    args_thread.insert("command".to_string(), Value::String("thread list".to_string()));
    
    let result_thread = tool.execute(args_thread, session.lldb_manager()).await.expect("execute_command thread list failed");
    let result_thread_str = match result_thread {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response_thread: Value = serde_json::from_str(&result_thread_str).expect("Invalid JSON response");
    
    assert!(response_thread["success"].as_bool().unwrap_or(false), "execute_command thread list should succeed");
    let thread_output = response_thread["output"].as_str().expect("output should be string");
    assert!(thread_output.contains("thread"), "Thread list should contain thread information");
    
    // Test 4: Execute backtrace command
    let mut args_bt = HashMap::new();
    args_bt.insert("command".to_string(), Value::String("bt".to_string()));
    
    let result_bt = tool.execute(args_bt, session.lldb_manager()).await.expect("execute_command bt failed");
    let result_bt_str = match result_bt {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response_bt: Value = serde_json::from_str(&result_bt_str).expect("Invalid JSON response");
    
    assert!(response_bt["success"].as_bool().unwrap_or(false), "execute_command bt should succeed");
    let bt_output = response_bt["output"].as_str().expect("output should be string");
    assert!(bt_output.contains("main"), "Backtrace should contain main function");
    
    session.cleanup().expect("Failed to cleanup session");
}

#[tokio::test]
async fn test_get_lldb_version_comprehensive() {
    let _test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::Normal).expect("Failed to create LLDB session");
    
    // Note: Version info should be available without launching a target
    let tool = GetLldbVersionTool;
    
    // Test 1: Get basic version information
    let args = HashMap::new();
    let result = tool.execute(args, session.lldb_manager()).await.expect("get_lldb_version failed");
    let result_str = match result {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response: Value = serde_json::from_str(&result_str).expect("Invalid JSON response");
    // Validate version response structure - successful response contains version field
    assert!(response["version"].is_string(), "get_lldb_version should return version field");
    assert!(response["api_version"].is_string(), "Should return api_version string");
    assert!(response.get("platform").is_some(), "Should return platform information");
    
    let version = response["version"].as_str().expect("version should be string");
    assert!(!version.is_empty(), "Version should not be empty");
    assert!(version.chars().any(|c| c.is_ascii_digit()), "Version should contain numbers");
    
    // Validate individual build fields exist
    assert!(response.get("build_configuration").is_some(), "Should have build_configuration field");
    assert!(response.get("build_date").is_some(), "Should have build_date field");
    
    // Test 2: Get detailed version information
    let mut args_detailed = HashMap::new();
    args_detailed.insert("include_build_info".to_string(), Value::Bool(true));
    
    let result_detailed = tool.execute(args_detailed, session.lldb_manager()).await.expect("get_lldb_version detailed failed");
    let result_detailed_str = match result_detailed {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response_detailed: Value = serde_json::from_str(&result_detailed_str).expect("Invalid JSON response");
    
    // Verify detailed response contains version information
    assert!(response_detailed["version"].is_string(), "Detailed version should have version field");
    
    // Should include additional details when requested
    if let Some(compiler) = response_detailed.get("compiler_info") {
        assert!(compiler.is_object(), "Compiler info should be object when included");
    }
    if let Some(llvm_version) = response_detailed.get("llvm_version") {
        assert!(llvm_version.is_string(), "LLVM version should be string when included");
    }
    
    // Test 3: Get version with capabilities
    let mut args_caps = HashMap::new();
    args_caps.insert("include_capabilities".to_string(), Value::Bool(true));
    
    let result_caps = tool.execute(args_caps, session.lldb_manager()).await.expect("get_lldb_version capabilities failed");
    let result_caps_str = match result_caps {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response_caps: Value = serde_json::from_str(&result_caps_str).expect("Invalid JSON response");
    
    assert!(response_caps["success"].as_bool().unwrap_or(false), "Capabilities version should succeed");
    
    // Should include capabilities when requested
    if let Some(capabilities) = response_caps.get("capabilities") {
        assert!(capabilities.is_array(), "Capabilities should be array when included");
    }
    if let Some(supported_formats) = response_caps.get("supported_formats") {
        assert!(supported_formats.is_array(), "Supported formats should be array when included");
    }
}

#[tokio::test]
async fn test_set_lldb_settings_comprehensive() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::Normal).expect("Failed to create LLDB session");
    let _pid = session.start().expect("Failed to start session");
    
    // Set a breakpoint to have an active debugging session
    session.set_test_breakpoint("main").expect("Failed to set breakpoint");
    session.continue_execution().expect("Failed to continue");
    
    let tool = SetLldbSettingsTool;
    
    // Test 1: Set basic LLDB setting
    let mut args = HashMap::new();
    args.insert("setting_name".to_string(), Value::String("target.process.thread.step-avoid-regexp".to_string()));
    args.insert("value".to_string(), Value::String("^std::".to_string()));
    
    let result = tool.execute(args, session.lldb_manager()).await.expect("set_lldb_settings failed");
    let result_str = match result {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response: Value = serde_json::from_str(&result_str).expect("Invalid JSON response");
    
    // Validate settings response
    assert!(response["success"].as_bool().unwrap_or(false), "set_lldb_settings should succeed");
    assert!(response["setting_name"].is_string(), "Should return setting_name");
    assert!(response["previous_value"].is_string(), "Should return previous_value");
    assert!(response["new_value"].is_string(), "Should return new_value");
    
    let setting_name = response["setting_name"].as_str().expect("setting_name should be string");
    assert_eq!(setting_name, "target.process.thread.step-avoid-regexp", "Should match requested setting");
    
    // Test 2: Set multiple settings
    let mut args_multi = HashMap::new();
    let mut settings = HashMap::new();
    settings.insert("target.process.thread.step-avoid-regexp".to_string(), Value::String("^boost::".to_string()));
    settings.insert("target.process.disable-stdio".to_string(), Value::Bool(false));
    
    args_multi.insert("settings".to_string(), Value::Object(settings.into_iter().collect()));
    
    let result_multi = tool.execute(args_multi, session.lldb_manager()).await.expect("set_lldb_settings multiple failed");
    let result_multi_str = match result_multi {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response_multi: Value = serde_json::from_str(&result_multi_str).expect("Invalid JSON response");
    
    assert!(response_multi["success"].as_bool().unwrap_or(false), "Multiple settings should succeed");
    assert!(response_multi["settings_applied"].is_array(), "Should return settings_applied array");
    
    let settings_applied = response_multi["settings_applied"].as_array().expect("settings_applied should be array");
    assert_eq!(settings_applied.len(), 2, "Should apply both settings");
    
    // Test 3: Verify setting was applied by retrieving it
    let mut args_get = HashMap::new();
    args_get.insert("setting_name".to_string(), Value::String("target.process.thread.step-avoid-regexp".to_string()));
    args_get.insert("get_current_value".to_string(), Value::Bool(true));
    
    let result_get = tool.execute(args_get, session.lldb_manager()).await.expect("get setting value failed");
    let result_get_str = match result_get {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response_get: Value = serde_json::from_str(&result_get_str).expect("Invalid JSON response");
    
    assert!(response_get["success"].as_bool().unwrap_or(false), "Get setting should succeed");
    if let Some(current_value) = response_get.get("current_value") {
        assert!(current_value.is_string(), "Current value should be string");
    }
    
    session.cleanup().expect("Failed to cleanup session");
}

#[tokio::test]
async fn test_lldb_control_command_safety() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::Normal).expect("Failed to create LLDB session");
    let _pid = session.start().expect("Failed to start session");
    
    // Set a breakpoint to have an active debugging session
    session.set_test_breakpoint("main").expect("Failed to set breakpoint");
    session.continue_execution().expect("Failed to continue");
    
    let tool = ExecuteCommandTool;
    
    // Test 1: Safe command execution
    let mut args_safe = HashMap::new();
    args_safe.insert("command".to_string(), Value::String("frame info".to_string()));
    
    let result_safe = tool.execute(args_safe, session.lldb_manager()).await.expect("Safe command should work");
    let result_safe_str = match result_safe {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response_safe: Value = serde_json::from_str(&result_safe_str).expect("Invalid JSON response");
    
    assert!(response_safe["success"].as_bool().unwrap_or(false), "Safe command should succeed");
    
    // Test 2: Command with safety validation
    let mut args_validate = HashMap::new();
    args_validate.insert("command".to_string(), Value::String("register read".to_string()));
    args_validate.insert("validate_safety".to_string(), Value::Bool(true));
    
    let result_validate = tool.execute(args_validate, session.lldb_manager()).await.expect("Validated command should work");
    let result_validate_str = match result_validate {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response_validate: Value = serde_json::from_str(&result_validate_str).expect("Invalid JSON response");
    
    // Should succeed or provide safety warning
    assert!(response_validate["success"].is_boolean(), "Should return success status");
    
    // Test 3: Potentially dangerous command (should be handled safely)
    let mut args_dangerous = HashMap::new();
    args_dangerous.insert("command".to_string(), Value::String("process kill".to_string()));
    args_dangerous.insert("validate_safety".to_string(), Value::Bool(true));
    
    let result_dangerous = tool.execute(args_dangerous, session.lldb_manager()).await.expect("Tool should handle dangerous commands");
    let result_dangerous_str = match result_dangerous {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response_dangerous: Value = serde_json::from_str(&result_dangerous_str).expect("Invalid JSON response");
    
    // Should either block the command or warn about it
    if !response_dangerous["success"].as_bool().unwrap_or(false) {
        assert!(response_dangerous["error"].is_string(), "Should provide error for dangerous command");
    } else if let Some(warning) = response_dangerous.get("warning") {
        assert!(warning.is_string(), "Should provide warning for dangerous command");
    }
    
    session.cleanup().expect("Failed to cleanup session");
}

#[tokio::test]
async fn test_lldb_control_error_handling() {
    let _test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::Normal).expect("Failed to create LLDB session");
    
    // Test error handling without active target
    let command_tool = ExecuteCommandTool;
    
    // Test 1: Invalid command
    let mut args_invalid = HashMap::new();
    args_invalid.insert("command".to_string(), Value::String("invalidcommandname".to_string()));
    
    let result_invalid = command_tool.execute(args_invalid, session.lldb_manager()).await.expect("Tool should handle invalid commands");
    let result_invalid_str = match result_invalid {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response_invalid: Value = serde_json::from_str(&result_invalid_str).expect("Invalid JSON response");
    
    // Should handle invalid command gracefully
    if !response_invalid["success"].as_bool().unwrap_or(false) {
        assert!(response_invalid["error"].is_string(), "Should provide error for invalid command");
    } else {
        // LLDB might provide help for invalid commands
        let output = response_invalid["output"].as_str().expect("output should be string");
        assert!(output.to_lowercase().contains("error") || 
               output.to_lowercase().contains("unknown") ||
               output.to_lowercase().contains("help"), "Should indicate invalid command");
    }
    
    // Test 2: Settings tool with invalid setting
    let settings_tool = SetLldbSettingsTool;
    let mut args_invalid_setting = HashMap::new();
    args_invalid_setting.insert("setting_name".to_string(), Value::String("invalid.setting.name".to_string()));
    args_invalid_setting.insert("value".to_string(), Value::String("value".to_string()));
    
    let result_invalid_setting = settings_tool.execute(args_invalid_setting, session.lldb_manager()).await.expect("Tool should handle invalid settings");
    let result_invalid_setting_str = match result_invalid_setting {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response_invalid_setting: Value = serde_json::from_str(&result_invalid_setting_str).expect("Invalid JSON response");
    
    // Should handle invalid setting gracefully
    if !response_invalid_setting["success"].as_bool().unwrap_or(false) {
        assert!(response_invalid_setting["error"].is_string(), "Should provide error for invalid setting");
    }
}