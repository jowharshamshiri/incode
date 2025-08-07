// InCode Session Management Tools - Comprehensive Test Suite
// Tests F0060-F0063: create_session, save_session, load_session, cleanup_session
// Real LLDB integration testing with test_debuggee binary

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde_json::Value;

mod test_setup;
use test_setup::{TestDebuggee, TestMode, LldbTestSession};

use incode::tools::session_management::{
    CreateSessionTool, SaveSessionTool, LoadSessionTool, CleanupSessionTool
};
use incode::mcp_server::McpTool;

#[test]
fn test_create_session_comprehensive() {
    let _test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let session = LldbTestSession::new().expect("Failed to create LLDB session");
    
    let tool = CreateSessionTool::new(session.manager());
    
    // Test 1: Create basic session
    let args = HashMap::new();
    let result = tool.call(args).expect("create_session failed");
    let response: Value = serde_json::from_str(&result).expect("Invalid JSON response");
    
    // Validate session creation response
    assert!(response["success"].as_bool().unwrap_or(false), "create_session should succeed");
    assert!(response["session_id"].is_string(), "Should return session_id");
    assert!(response["created_at"].is_string(), "Should return created_at timestamp");
    assert!(response["session_state"].is_string(), "Should return session_state");
    
    let session_id = response["session_id"].as_str().expect("session_id should be string");
    assert!(!session_id.is_empty(), "Session ID should not be empty");
    assert!(session_id.len() >= 32, "Session ID should be UUID-like"); // UUID is typically 36 chars
    
    let session_state = response["session_state"].as_str().expect("session_state should be string");
    assert!(session_state == "initialized" || session_state == "active", "Session should be initialized or active");
    
    // Test 2: Create session with custom name
    let mut args_named = HashMap::new();
    args_named.insert("session_name".to_string(), Value::String("TestSession".to_string()));
    
    let result_named = tool.call(args_named).expect("create_session with name failed");
    let response_named: Value = serde_json::from_str(&result_named).expect("Invalid JSON response");
    
    assert!(response_named["success"].as_bool().unwrap_or(false), "Named session creation should succeed");
    if let Some(name) = response_named.get("session_name") {
        assert_eq!(name.as_str().unwrap(), "TestSession", "Should use provided session name");
    }
    
    // Test 3: Create session with metadata
    let mut args_metadata = HashMap::new();
    args_metadata.insert("include_environment_info".to_string(), Value::Bool(true));
    
    let result_metadata = tool.call(args_metadata).expect("create_session with metadata failed");
    let response_metadata: Value = serde_json::from_str(&result_metadata).expect("Invalid JSON response");
    
    assert!(response_metadata["success"].as_bool().unwrap_or(false), "Metadata session creation should succeed");
    
    // Should include environment info when requested
    if let Some(env_info) = response_metadata.get("environment_info") {
        assert!(env_info.is_object(), "Environment info should be object when included");
    }
}

#[test]
fn test_save_session_comprehensive() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = LldbTestSession::new().expect("Failed to create LLDB session");
    
    // Create debugging state to save
    session.launch_and_break(&test_debuggee, Some("main")).expect("Failed to launch and break");
    
    let save_tool = SaveSessionTool::new(session.manager());
    
    // Test 1: Save basic session
    let mut args = HashMap::new();
    args.insert("session_name".to_string(), Value::String("test_save_session".to_string()));
    
    let result = save_tool.call(args).expect("save_session failed");
    let response: Value = serde_json::from_str(&result).expect("Invalid JSON response");
    
    // Validate session save response
    assert!(response["success"].as_bool().unwrap_or(false), "save_session should succeed");
    assert!(response["session_id"].is_string(), "Should return session_id");
    assert!(response["saved_at"].is_string(), "Should return saved_at timestamp");
    assert!(response["file_path"].is_string(), "Should return saved file_path");
    
    let file_path = response["file_path"].as_str().expect("file_path should be string");
    assert!(Path::new(file_path).exists(), "Saved session file should exist");
    
    // Test 2: Save session with full state
    let mut args_full = HashMap::new();
    args_full.insert("session_name".to_string(), Value::String("test_full_session".to_string()));
    args_full.insert("include_breakpoints".to_string(), Value::Bool(true));
    args_full.insert("include_variables".to_string(), Value::Bool(true));
    
    let result_full = save_tool.call(args_full).expect("save_session full failed");
    let response_full: Value = serde_json::from_str(&result_full).expect("Invalid JSON response");
    
    assert!(response_full["success"].as_bool().unwrap_or(false), "Full session save should succeed");
    
    let full_file_path = response_full["file_path"].as_str().expect("file_path should be string");
    assert!(Path::new(full_file_path).exists(), "Full session file should exist");
    
    // Verify saved file contains JSON data
    let saved_content = fs::read_to_string(full_file_path).expect("Should read saved session file");
    let saved_data: Value = serde_json::from_str(&saved_content).expect("Saved file should contain valid JSON");
    
    assert!(saved_data["session_id"].is_string(), "Saved data should contain session_id");
    assert!(saved_data["process_info"].is_object(), "Saved data should contain process_info");
    
    // Test 3: Save session to specific path
    let session_dir = std::env::temp_dir().join("incode_test_sessions");
    fs::create_dir_all(&session_dir).expect("Should create session directory");
    
    let mut args_path = HashMap::new();
    args_path.insert("session_name".to_string(), Value::String("test_custom_path".to_string()));
    args_path.insert("save_path".to_string(), Value::String(session_dir.to_string_lossy().to_string()));
    
    let result_path = save_tool.call(args_path).expect("save_session to path failed");
    let response_path: Value = serde_json::from_str(&result_path).expect("Invalid JSON response");
    
    assert!(response_path["success"].as_bool().unwrap_or(false), "Custom path save should succeed");
    
    let custom_file_path = response_path["file_path"].as_str().expect("file_path should be string");
    assert!(custom_file_path.starts_with(&session_dir.to_string_lossy().to_string()), 
           "Should save to custom path");
    
    // Cleanup test files
    let _ = fs::remove_file(file_path);
    let _ = fs::remove_file(full_file_path);
    let _ = fs::remove_file(custom_file_path);
    let _ = fs::remove_dir(session_dir);
    
    session.cleanup().expect("Failed to cleanup session");
}

#[test]
fn test_load_session_comprehensive() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = LldbTestSession::new().expect("Failed to create LLDB session");
    
    // Create and save a session to load
    session.launch_and_break(&test_debuggee, Some("main")).expect("Failed to launch and break");
    
    let save_tool = SaveSessionTool::new(session.manager());
    let mut save_args = HashMap::new();
    save_args.insert("session_name".to_string(), Value::String("test_load_session".to_string()));
    save_args.insert("include_breakpoints".to_string(), Value::Bool(true));
    
    let save_result = save_tool.call(save_args).expect("Failed to save session for loading test");
    let save_response: Value = serde_json::from_str(&save_result).expect("Invalid JSON response");
    let saved_file_path = save_response["file_path"].as_str().expect("file_path should be string");
    
    session.cleanup().expect("Failed to cleanup session for reload test");
    
    // Now test loading the session
    let new_session = LldbTestSession::new().expect("Failed to create new LLDB session");
    let load_tool = LoadSessionTool::new(new_session.manager());
    
    // Test 1: Load session by file path
    let mut args = HashMap::new();
    args.insert("file_path".to_string(), Value::String(saved_file_path.to_string()));
    
    let result = load_tool.call(args).expect("load_session failed");
    let response: Value = serde_json::from_str(&result).expect("Invalid JSON response");
    
    // Validate session load response
    assert!(response["success"].as_bool().unwrap_or(false), "load_session should succeed");
    assert!(response["session_id"].is_string(), "Should return loaded session_id");
    assert!(response["loaded_at"].is_string(), "Should return loaded_at timestamp");
    assert!(response["session_state"].is_string(), "Should return session_state");
    
    let loaded_session_id = response["session_id"].as_str().expect("session_id should be string");
    assert!(!loaded_session_id.is_empty(), "Loaded session ID should not be empty");
    
    // Test 2: Load session with restoration options
    let mut args_restore = HashMap::new();
    args_restore.insert("file_path".to_string(), Value::String(saved_file_path.to_string()));
    args_restore.insert("restore_breakpoints".to_string(), Value::Bool(true));
    args_restore.insert("restore_target".to_string(), Value::Bool(false)); // Don't restart target
    
    let result_restore = load_tool.call(args_restore).expect("load_session with restore failed");
    let response_restore: Value = serde_json::from_str(&result_restore).expect("Invalid JSON response");
    
    assert!(response_restore["success"].as_bool().unwrap_or(false), "Restore session should succeed");
    
    // Should indicate what was restored
    if let Some(restored) = response_restore.get("restored_components") {
        assert!(restored.is_array(), "Restored components should be array");
    }
    
    // Test 3: Load session by name (if saved with name)
    let mut args_name = HashMap::new();
    args_name.insert("session_name".to_string(), Value::String("test_load_session".to_string()));
    
    let result_name = load_tool.call(args_name).expect("load_session by name should work");
    let response_name: Value = serde_json::from_str(&result_name).expect("Invalid JSON response");
    
    // Should succeed or gracefully handle name-based loading
    assert!(response_name["success"].is_boolean(), "Should return success status");
    
    // Cleanup test file
    let _ = fs::remove_file(saved_file_path);
}

#[test]  
fn test_cleanup_session_comprehensive() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = LldbTestSession::new().expect("Failed to create LLDB session");
    
    // Create debugging state to cleanup
    session.launch_and_break(&test_debuggee, Some("main")).expect("Failed to launch and break");
    
    let cleanup_tool = CleanupSessionTool::new(session.manager());
    
    // Test 1: Basic session cleanup
    let args = HashMap::new();
    let result = cleanup_tool.call(args).expect("cleanup_session failed");
    let response: Value = serde_json::from_str(&result).expect("Invalid JSON response");
    
    // Validate cleanup response
    assert!(response["success"].as_bool().unwrap_or(false), "cleanup_session should succeed");
    assert!(response["cleaned_up_at"].is_string(), "Should return cleaned_up_at timestamp");
    assert!(response["resources_cleaned"].is_array(), "Should return resources_cleaned array");
    
    let resources_cleaned = response["resources_cleaned"].as_array().expect("resources_cleaned should be array");
    assert!(resources_cleaned.len() > 0, "Should clean up some resources");
    
    // Test 2: Cleanup with specific options
    let mut new_session = LldbTestSession::new().expect("Failed to create new session");
    new_session.launch_and_break(&test_debuggee, Some("main")).expect("Failed to launch and break");
    
    let cleanup_tool2 = CleanupSessionTool::new(new_session.manager());
    
    let mut args_options = HashMap::new();
    args_options.insert("cleanup_breakpoints".to_string(), Value::Bool(true));
    args_options.insert("cleanup_processes".to_string(), Value::Bool(true));
    args_options.insert("cleanup_files".to_string(), Value::Bool(false));
    
    let result_options = cleanup_tool2.call(args_options).expect("cleanup_session with options failed");
    let response_options: Value = serde_json::from_str(&result_options).expect("Invalid JSON response");
    
    assert!(response_options["success"].as_bool().unwrap_or(false), "Options cleanup should succeed");
    
    // Should detail what was cleaned
    let options_resources = response_options["resources_cleaned"].as_array().expect("resources_cleaned should be array");
    assert!(options_resources.len() > 0, "Should clean specific resources");
    
    // Test 3: Force cleanup
    let mut force_session = LldbTestSession::new().expect("Failed to create force session");
    force_session.launch_and_break(&test_debuggee, Some("main")).expect("Failed to launch and break");
    
    let cleanup_tool3 = CleanupSessionTool::new(force_session.manager());
    
    let mut args_force = HashMap::new();
    args_force.insert("force_cleanup".to_string(), Value::Bool(true));
    
    let result_force = cleanup_tool3.call(args_force).expect("force cleanup failed");
    let response_force: Value = serde_json::from_str(&result_force).expect("Invalid JSON response");
    
    assert!(response_force["success"].as_bool().unwrap_or(false), "Force cleanup should succeed");
    
    // Force cleanup might clean more resources
    if let Some(force_resources) = response_force.get("resources_cleaned") {
        assert!(force_resources.is_array(), "Force cleanup should return resources cleaned");
    }
}

#[test]
fn test_session_lifecycle_integration() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = LldbTestSession::new().expect("Failed to create LLDB session");
    
    // Test complete session lifecycle: create -> use -> save -> cleanup -> load
    
    // Step 1: Create session
    let create_tool = CreateSessionTool::new(session.manager());
    let mut create_args = HashMap::new();
    create_args.insert("session_name".to_string(), Value::String("lifecycle_test".to_string()));
    
    let create_result = create_tool.call(create_args).expect("Failed to create session");
    let create_response: Value = serde_json::from_str(&create_result).expect("Invalid JSON response");
    assert!(create_response["success"].as_bool().unwrap_or(false), "Session creation should succeed");
    
    // Step 2: Use session (launch and break)
    session.launch_and_break(&test_debuggee, Some("main")).expect("Failed to launch and break");
    
    // Step 3: Save session state
    let save_tool = SaveSessionTool::new(session.manager());
    let mut save_args = HashMap::new();
    save_args.insert("session_name".to_string(), Value::String("lifecycle_test".to_string()));
    save_args.insert("include_breakpoints".to_string(), Value::Bool(true));
    
    let save_result = save_tool.call(save_args).expect("Failed to save session");
    let save_response: Value = serde_json::from_str(&save_result).expect("Invalid JSON response");
    assert!(save_response["success"].as_bool().unwrap_or(false), "Session save should succeed");
    
    let saved_file = save_response["file_path"].as_str().expect("file_path should be string");
    
    // Step 4: Cleanup original session
    let cleanup_tool = CleanupSessionTool::new(session.manager());
    let cleanup_result = cleanup_tool.call(HashMap::new()).expect("Failed to cleanup session");
    let cleanup_response: Value = serde_json::from_str(&cleanup_result).expect("Invalid JSON response");
    assert!(cleanup_response["success"].as_bool().unwrap_or(false), "Session cleanup should succeed");
    
    // Step 5: Load session in new manager
    let new_session = LldbTestSession::new().expect("Failed to create new session");
    let load_tool = LoadSessionTool::new(new_session.manager());
    let mut load_args = HashMap::new();
    load_args.insert("file_path".to_string(), Value::String(saved_file.to_string()));
    
    let load_result = load_tool.call(load_args).expect("Failed to load session");
    let load_response: Value = serde_json::from_str(&load_result).expect("Invalid JSON response");
    assert!(load_response["success"].as_bool().unwrap_or(false), "Session load should succeed");
    
    // Verify session was restored
    let loaded_id = load_response["session_id"].as_str().expect("session_id should be string");
    assert!(!loaded_id.is_empty(), "Loaded session should have ID");
    
    // Cleanup test file
    let _ = fs::remove_file(saved_file);
}

#[test]
fn test_session_management_error_handling() {
    let _test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let session = LldbTestSession::new().expect("Failed to create LLDB session");
    
    // Test load session with non-existent file
    let load_tool = LoadSessionTool::new(session.manager());
    let mut args_invalid = HashMap::new();
    args_invalid.insert("file_path".to_string(), Value::String("/nonexistent/session.json".to_string()));
    
    let result_invalid = load_tool.call(args_invalid).expect("Tool should handle missing file");
    let response_invalid: Value = serde_json::from_str(&result_invalid).expect("Invalid JSON response");
    
    // Should handle missing file gracefully
    if !response_invalid["success"].as_bool().unwrap_or(false) {
        assert!(response_invalid["error"].is_string(), "Should provide error for missing file");
    }
    
    // Test save session without write permissions (if possible)
    let save_tool = SaveSessionTool::new(session.manager());
    let mut args_no_perm = HashMap::new();
    args_no_perm.insert("session_name".to_string(), Value::String("test_no_permission".to_string()));
    args_no_perm.insert("save_path".to_string(), Value::String("/root/no_permission".to_string()));
    
    let result_no_perm = save_tool.call(args_no_perm).expect("Tool should handle permission errors");
    let response_no_perm: Value = serde_json::from_str(&result_no_perm).expect("Invalid JSON response");
    
    // Should handle permission error gracefully
    if !response_no_perm["success"].as_bool().unwrap_or(false) {
        assert!(response_no_perm["error"].is_string(), "Should provide error for permission issues");
    }
}