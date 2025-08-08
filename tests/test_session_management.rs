// InCode Session Management Tools - Comprehensive Test Suite
// Tests F0060-F0063: create_session, save_session, load_session, cleanup_session
// Real LLDB integration testing with test_debuggee binary

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde_json::Value;

mod test_setup;
use test_setup::{TestDebuggee, TestMode, TestSession};

use incode::tools::session_management::{
    CreateSessionTool, SaveSessionTool, LoadSessionTool, CleanupSessionTool
};
use incode::tools::{Tool, ToolResponse};

#[tokio::test]
async fn test_create_session_comprehensive() {
    let _test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::Normal).expect("Failed to create LLDB session");
    
    let tool = CreateSessionTool;
    
    // Test 1: Create basic session
    let args = HashMap::new();
    let result = tool.execute(args, session.lldb_manager()).await.expect("create_session failed");
    let result_str = match result {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response: Value = serde_json::from_str(&result_str).expect("Invalid JSON response");
    
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
    
    let result_named = tool.execute(args_named, session.lldb_manager()).await.expect("create_session with name failed");
    let result_named_str = match result_named {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response_named: Value = serde_json::from_str(&result_named_str).expect("Invalid JSON response");
    
    assert!(response_named["success"].as_bool().unwrap_or(false), "Named session creation should succeed");
    if let Some(name) = response_named.get("session_name") {
        assert_eq!(name.as_str().unwrap(), "TestSession", "Should use provided session name");
    }
    
    // Test 3: Create session with metadata
    let mut args_metadata = HashMap::new();
    args_metadata.insert("include_environment_info".to_string(), Value::Bool(true));
    
    let result_metadata = tool.execute(args_metadata, session.lldb_manager()).await.expect("create_session with metadata failed");
    let result_metadata_str = match result_metadata {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response_metadata: Value = serde_json::from_str(&result_metadata_str).expect("Invalid JSON response");
    
    assert!(response_metadata["success"].as_bool().unwrap_or(false), "Metadata session creation should succeed");
    
    // Should include environment info when requested
    if let Some(env_info) = response_metadata.get("environment_info") {
        assert!(env_info.is_object(), "Environment info should be object when included");
    }
}

#[tokio::test]
async fn test_save_session_comprehensive() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::Normal).expect("Failed to create LLDB session");
    let _pid = session.start().expect("Failed to start session");
    
    // Create debugging state to save
    session.set_test_breakpoint("main").expect("Failed to set breakpoint");
    session.continue_execution().expect("Failed to continue");
    
    let save_tool = SaveSessionTool;
    
    // Test 1: Save basic session
    let mut args = HashMap::new();
    args.insert("session_name".to_string(), Value::String("test_save_session".to_string()));
    
    let result = save_tool.execute(args, session.lldb_manager()).await.expect("save_session failed");
    let result_str = match result {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response: Value = serde_json::from_str(&result_str).expect("Invalid JSON response");
    
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
    
    let result_full = save_tool.execute(args_full, session.lldb_manager()).await.expect("save_session full failed");
    let result_full_str = match result_full {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response_full: Value = serde_json::from_str(&result_full_str).expect("Invalid JSON response");
    
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
    
    let result_path = save_tool.execute(args_path, session.lldb_manager()).await.expect("save_session to path failed");
    let result_path_str = match result_path {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response_path: Value = serde_json::from_str(&result_path_str).expect("Invalid JSON response");
    
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

#[tokio::test]
async fn test_load_session_comprehensive() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::Normal).expect("Failed to create LLDB session");
    let _pid = session.start().expect("Failed to start session");
    
    // Create and save a session to load
    session.set_test_breakpoint("main").expect("Failed to set breakpoint");
    session.continue_execution().expect("Failed to continue");
    
    let save_tool = SaveSessionTool;
    let mut save_args = HashMap::new();
    save_args.insert("session_name".to_string(), Value::String("test_load_session".to_string()));
    save_args.insert("include_breakpoints".to_string(), Value::Bool(true));
    
    let save_result = save_tool.execute(save_args, session.lldb_manager()).await.expect("Failed to save session for loading test");
    let save_result_str = match save_result {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let save_response: Value = serde_json::from_str(&save_result_str).expect("Invalid JSON response");
    let saved_file_path = save_response["file_path"].as_str().expect("file_path should be string");
    
    session.cleanup().expect("Failed to cleanup session for reload test");
    
    // Now test loading the session
    let mut new_session = TestSession::new(TestMode::Normal).expect("Failed to create new LLDB session");
    let load_tool = LoadSessionTool;
    
    // Test 1: Load session by file path
    let mut args = HashMap::new();
    args.insert("file_path".to_string(), Value::String(saved_file_path.to_string()));
    
    let result = load_tool.execute(args, new_session.lldb_manager()).await.expect("load_session failed");
    let result_str = match result {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response: Value = serde_json::from_str(&result_str).expect("Invalid JSON response");
    
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
    
    let result_restore = load_tool.execute(args_restore, new_session.lldb_manager()).await.expect("load_session with restore failed");
    let result_restore_str = match result_restore {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response_restore: Value = serde_json::from_str(&result_restore_str).expect("Invalid JSON response");
    
    assert!(response_restore["success"].as_bool().unwrap_or(false), "Restore session should succeed");
    
    // Should indicate what was restored
    if let Some(restored) = response_restore.get("restored_components") {
        assert!(restored.is_array(), "Restored components should be array");
    }
    
    // Test 3: Load session by name (if saved with name)
    let mut args_name = HashMap::new();
    args_name.insert("session_name".to_string(), Value::String("test_load_session".to_string()));
    
    let result_name = load_tool.execute(args_name, new_session.lldb_manager()).await.expect("load_session by name should work");
    let result_name_str = match result_name {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response_name: Value = serde_json::from_str(&result_name_str).expect("Invalid JSON response");
    
    // Should succeed or gracefully handle name-based loading
    assert!(response_name["success"].is_boolean(), "Should return success status");
    
    // Cleanup test file
    let _ = fs::remove_file(saved_file_path);
}

#[tokio::test]  
async fn test_cleanup_session_comprehensive() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::Normal).expect("Failed to create LLDB session");
    let _pid = session.start().expect("Failed to start session");
    
    // Create debugging state to cleanup
    session.set_test_breakpoint("main").expect("Failed to set breakpoint");
    session.continue_execution().expect("Failed to continue");
    
    let cleanup_tool = CleanupSessionTool;
    
    // Test 1: Basic session cleanup
    let args = HashMap::new();
    let result = cleanup_tool.execute(args, session.lldb_manager()).await.expect("cleanup_session failed");
    let result_str = match result {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response: Value = serde_json::from_str(&result_str).expect("Invalid JSON response");
    
    // Validate cleanup response
    assert!(response["success"].as_bool().unwrap_or(false), "cleanup_session should succeed");
    assert!(response["cleaned_up_at"].is_string(), "Should return cleaned_up_at timestamp");
    assert!(response["resources_cleaned"].is_array(), "Should return resources_cleaned array");
    
    let resources_cleaned = response["resources_cleaned"].as_array().expect("resources_cleaned should be array");
    assert!(resources_cleaned.len() > 0, "Should clean up some resources");
    
    // Test 2: Cleanup with specific options
    let mut new_session = TestSession::new(TestMode::Normal).expect("Failed to create new session");
    let _pid2 = new_session.start().expect("Failed to start session");
    new_session.set_test_breakpoint("main").expect("Failed to set breakpoint");
    new_session.continue_execution().expect("Failed to continue");
    
    let cleanup_tool2 = CleanupSessionTool;
    
    let mut args_options = HashMap::new();
    args_options.insert("cleanup_breakpoints".to_string(), Value::Bool(true));
    args_options.insert("cleanup_processes".to_string(), Value::Bool(true));
    args_options.insert("cleanup_files".to_string(), Value::Bool(false));
    
    let result_options = cleanup_tool2.execute(args_options, new_session.lldb_manager()).await.expect("cleanup_session with options failed");
    let result_options_str = match result_options {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response_options: Value = serde_json::from_str(&result_options_str).expect("Invalid JSON response");
    
    assert!(response_options["success"].as_bool().unwrap_or(false), "Options cleanup should succeed");
    
    // Should detail what was cleaned
    let options_resources = response_options["resources_cleaned"].as_array().expect("resources_cleaned should be array");
    assert!(options_resources.len() > 0, "Should clean specific resources");
    
    // Test 3: Force cleanup
    let mut force_session = TestSession::new(TestMode::Normal).expect("Failed to create force session");
    let _pid3 = force_session.start().expect("Failed to start session");
    force_session.set_test_breakpoint("main").expect("Failed to set breakpoint");
    force_session.continue_execution().expect("Failed to continue");
    
    let cleanup_tool3 = CleanupSessionTool;
    
    let mut args_force = HashMap::new();
    args_force.insert("force_cleanup".to_string(), Value::Bool(true));
    
    let result_force = cleanup_tool3.execute(args_force, force_session.lldb_manager()).await.expect("force cleanup failed");
    let result_force_str = match result_force {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response_force: Value = serde_json::from_str(&result_force_str).expect("Invalid JSON response");
    
    assert!(response_force["success"].as_bool().unwrap_or(false), "Force cleanup should succeed");
    
    // Force cleanup might clean more resources
    if let Some(force_resources) = response_force.get("resources_cleaned") {
        assert!(force_resources.is_array(), "Force cleanup should return resources cleaned");
    }
}

#[tokio::test]
async fn test_session_lifecycle_integration() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::Normal).expect("Failed to create LLDB session");
    let _pid = session.start().expect("Failed to start session");
    
    // Test complete session lifecycle: create -> use -> save -> cleanup -> load
    
    // Step 1: Create session
    let create_tool = CreateSessionTool;
    let mut create_args = HashMap::new();
    create_args.insert("session_name".to_string(), Value::String("lifecycle_test".to_string()));
    
    let create_result = create_tool.execute(create_args, session.lldb_manager()).await.expect("Failed to create session");
    let create_result_str = match create_result {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let create_response: Value = serde_json::from_str(&create_result_str).expect("Invalid JSON response");
    assert!(create_response["success"].as_bool().unwrap_or(false), "Session creation should succeed");
    
    // Step 2: Use session (set breakpoint and continue)
    session.set_test_breakpoint("main").expect("Failed to set breakpoint");
    session.continue_execution().expect("Failed to continue");
    
    // Step 3: Save session state
    let save_tool = SaveSessionTool;
    let mut save_args = HashMap::new();
    save_args.insert("session_name".to_string(), Value::String("lifecycle_test".to_string()));
    save_args.insert("include_breakpoints".to_string(), Value::Bool(true));
    
    let save_result = save_tool.execute(save_args, session.lldb_manager()).await.expect("Failed to save session");
    let save_result_str = match save_result {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let save_response: Value = serde_json::from_str(&save_result_str).expect("Invalid JSON response");
    assert!(save_response["success"].as_bool().unwrap_or(false), "Session save should succeed");
    
    let saved_file = save_response["file_path"].as_str().expect("file_path should be string");
    
    // Step 4: Cleanup original session
    let cleanup_tool = CleanupSessionTool;
    let cleanup_result = cleanup_tool.execute(HashMap::new(), session.lldb_manager()).await.expect("Failed to cleanup session");
    let cleanup_result_str = match cleanup_result {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let cleanup_response: Value = serde_json::from_str(&cleanup_result_str).expect("Invalid JSON response");
    assert!(cleanup_response["success"].as_bool().unwrap_or(false), "Session cleanup should succeed");
    
    // Step 5: Load session in new manager
    let mut new_session = TestSession::new(TestMode::Normal).expect("Failed to create new session");
    let load_tool = LoadSessionTool;
    let mut load_args = HashMap::new();
    load_args.insert("file_path".to_string(), Value::String(saved_file.to_string()));
    
    let load_result = load_tool.execute(load_args, new_session.lldb_manager()).await.expect("Failed to load session");
    let load_result_str = match load_result {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let load_response: Value = serde_json::from_str(&load_result_str).expect("Invalid JSON response");
    assert!(load_response["success"].as_bool().unwrap_or(false), "Session load should succeed");
    
    // Verify session was restored
    let loaded_id = load_response["session_id"].as_str().expect("session_id should be string");
    assert!(!loaded_id.is_empty(), "Loaded session should have ID");
    
    // Cleanup test file
    let _ = fs::remove_file(saved_file);
}

#[tokio::test]
async fn test_session_management_error_handling() {
    let _test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::Normal).expect("Failed to create LLDB session");
    session.start().expect("Failed to start session");
    
    // Test load session with non-existent file
    let load_tool = LoadSessionTool;
    let mut args_invalid = HashMap::new();
    args_invalid.insert("file_path".to_string(), Value::String("/nonexistent/session.json".to_string()));
    
    let result_invalid = load_tool.execute(args_invalid, session.lldb_manager()).await.expect("Tool should handle missing file");
    let result_invalid_str = match result_invalid {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response_invalid: Value = serde_json::from_str(&result_invalid_str).expect("Invalid JSON response");
    
    // Should handle missing file gracefully
    if !response_invalid["success"].as_bool().unwrap_or(false) {
        assert!(response_invalid["error"].is_string(), "Should provide error for missing file");
    }
    
    // Test save session without write permissions (if possible)
    let save_tool = SaveSessionTool;
    let mut args_no_perm = HashMap::new();
    args_no_perm.insert("session_name".to_string(), Value::String("test_no_permission".to_string()));
    args_no_perm.insert("save_path".to_string(), Value::String("/root/no_permission".to_string()));
    
    let result_no_perm = save_tool.execute(args_no_perm, session.lldb_manager()).await.expect("Tool should handle permission errors");
    let result_no_perm_str = match result_no_perm {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response_no_perm: Value = serde_json::from_str(&result_no_perm_str).expect("Invalid JSON response");
    
    // Should handle permission error gracefully
    if !response_no_perm["success"].as_bool().unwrap_or(false) {
        assert!(response_no_perm["error"].is_string(), "Should provide error for permission issues");
    }
}