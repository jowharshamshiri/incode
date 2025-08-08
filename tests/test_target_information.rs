// InCode Target Information Tools - Comprehensive Test Suite  
// Tests F0054-F0056: get_target_info, get_platform_info, list_modules
// Real LLDB integration testing with test_debuggee binary

use std::collections::HashMap;
use serde_json::Value;

mod test_setup;
use test_setup::{TestDebuggee, TestMode, TestSession};

use incode::tools::target_info::{GetTargetInfoTool, GetPlatformInfoTool, ListModulesTool};
use incode::tools::{Tool, ToolResponse};

#[tokio::test]
async fn test_get_target_info_comprehensive() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::Normal).expect("Failed to create LLDB session");
    
    // Launch process for target analysis
    session.start().expect("Failed to start debugging session");
    session.set_test_breakpoint("main").expect("Failed to set breakpoint at main");
    
    let tool = GetTargetInfoTool;
    
    // Test 1: Get comprehensive target info
    let args = HashMap::new();
    let result = tool.execute(args, session.lldb_manager()).await.expect("get_target_info failed");
    let result_str = match result {
        ToolResponse::Json(json) => json.to_string(),
        ToolResponse::Success(text) => text,
        ToolResponse::Error(err) => panic!("Tool execution failed: {}", err),
    };
    let response: Value = serde_json::from_str(&result_str).expect("Invalid JSON response");
    
    // Validate target info response structure
    assert!(response["success"].as_bool().unwrap_or(false), "get_target_info should succeed");
    assert!(response["executable_path"].is_string(), "Should return executable_path");
    assert!(response["architecture"].is_string(), "Should return architecture");
    assert!(response["platform"].is_string(), "Should return platform");
    assert!(response["executable_format"].is_string(), "Should return executable_format");
    assert!(response["has_debug_symbols"].is_boolean(), "Should return has_debug_symbols");
    
    // Validate target values
    let executable_path = response["executable_path"].as_str().expect("executable_path should be string");
    assert!(executable_path.contains("test_debuggee"), "Should reference test_debuggee binary");
    
    let architecture = response["architecture"].as_str().expect("architecture should be string");
    assert!(!architecture.is_empty(), "Architecture should not be empty");
    
    let has_debug_symbols = response["has_debug_symbols"].as_bool().expect("has_debug_symbols should be boolean");
    assert!(has_debug_symbols, "Test binary should have debug symbols");
    
    // Test 2: Get target info with metadata
    let mut args_metadata = HashMap::new();
    args_metadata.insert("include_metadata".to_string(), Value::Bool(true));
    
    let result_metadata = tool.execute(args_metadata, session.lldb_manager()).await.expect("get_target_info with metadata failed");
    let result_metadata_str = match result_metadata {
        ToolResponse::Json(json) => json.to_string(),
        ToolResponse::Success(text) => text,
        ToolResponse::Error(err) => panic!("Tool execution failed: {}", err),
    };
    let response_metadata: Value = serde_json::from_str(&result_metadata_str).expect("Invalid JSON response");
    
    assert!(response_metadata["success"].as_bool().unwrap_or(false), "Metadata target info should succeed");
    
    // Should include additional metadata when requested
    if let Some(file_size) = response_metadata.get("file_size") {
        assert!(file_size.is_number(), "File size should be number when included");
        assert!(file_size.as_u64().unwrap() > 0, "File size should be positive");
    }
    if let Some(creation_time) = response_metadata.get("creation_time") {
        assert!(creation_time.is_string(), "Creation time should be string when included");
    }
    
    // Test 3: Get target info with symbols analysis
    let mut args_symbols = HashMap::new();
    args_symbols.insert("analyze_symbols".to_string(), Value::Bool(true));
    
    let result_symbols = tool.execute(args_symbols, session.lldb_manager()).await.expect("get_target_info with symbols failed");
    let result_symbols_str = match result_symbols {
        ToolResponse::Json(json) => json.to_string(),
        ToolResponse::Success(text) => text,
        ToolResponse::Error(err) => panic!("Tool execution failed: {}", err),
    };
    let response_symbols: Value = serde_json::from_str(&result_symbols_str).expect("Invalid JSON response");
    
    assert!(response_symbols["success"].as_bool().unwrap_or(false), "Symbols target info should succeed");
    
    // Should include symbol analysis when requested
    if let Some(symbol_count) = response_symbols.get("symbol_count") {
        assert!(symbol_count.is_number(), "Symbol count should be number when included");
    }
    if let Some(debug_format) = response_symbols.get("debug_format") {
        assert!(debug_format.is_string(), "Debug format should be string when included");
    }
    
    session.cleanup().expect("Failed to cleanup session");
}

#[tokio::test]
async fn test_get_platform_info_comprehensive() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::Normal).expect("Failed to create LLDB session");
    
    // Launch process for platform analysis
    session.start().expect("Failed to start debugging session");
    session.set_test_breakpoint("main").expect("Failed to set breakpoint at main");
    
    let tool = GetPlatformInfoTool;
    
    // Test 1: Get basic platform info
    let args = HashMap::new();
    let result = tool.execute(args, session.lldb_manager()).await.expect("get_platform_info failed");
    let result_str = match result {
        ToolResponse::Json(json) => json.to_string(),
        ToolResponse::Success(text) => text,
        ToolResponse::Error(err) => panic!("Tool execution failed: {}", err),
    };
    let response: Value = serde_json::from_str(&result_str).expect("Invalid JSON response");
    
    // Validate platform info response structure
    assert!(response["success"].as_bool().unwrap_or(false), "get_platform_info should succeed");
    assert!(response["platform_name"].is_string(), "Should return platform_name");
    assert!(response["os_version"].is_string(), "Should return os_version");
    assert!(response["architecture"].is_string(), "Should return architecture");
    assert!(response["byte_order"].is_string(), "Should return byte_order");
    
    // Validate platform values
    let platform_name = response["platform_name"].as_str().expect("platform_name should be string");
    assert!(!platform_name.is_empty(), "Platform name should not be empty");
    
    let os_version = response["os_version"].as_str().expect("os_version should be string");
    assert!(!os_version.is_empty(), "OS version should not be empty");
    
    let byte_order = response["byte_order"].as_str().expect("byte_order should be string");
    assert!(byte_order == "little" || byte_order == "big", "Byte order should be little or big endian");
    
    // Test 2: Get platform info with development environment details
    let mut args_dev = HashMap::new();
    args_dev.insert("include_development_info".to_string(), Value::Bool(true));
    
    let result_dev = tool.execute(args_dev, session.lldb_manager()).await.expect("get_platform_info with dev info failed");
    let result_dev_str = match result_dev {
        ToolResponse::Json(json) => json.to_string(),
        ToolResponse::Success(text) => text,
        ToolResponse::Error(err) => panic!("Tool execution failed: {}", err),
    };
    let response_dev: Value = serde_json::from_str(&result_dev_str).expect("Invalid JSON response");
    
    assert!(response_dev["success"].as_bool().unwrap_or(false), "Dev platform info should succeed");
    
    // Should include development environment details when requested
    if let Some(sdk_version) = response_dev.get("sdk_version") {
        assert!(sdk_version.is_string(), "SDK version should be string when included");
    }
    if let Some(compiler_info) = response_dev.get("compiler_info") {
        assert!(compiler_info.is_object(), "Compiler info should be object when included");
    }
    
    // Test 3: Get platform info with capabilities
    let mut args_caps = HashMap::new();
    args_caps.insert("include_capabilities".to_string(), Value::Bool(true));
    
    let result_caps = tool.execute(args_caps, session.lldb_manager()).await.expect("get_platform_info with capabilities failed");
    let result_caps_str = match result_caps {
        ToolResponse::Json(json) => json.to_string(),
        ToolResponse::Success(text) => text,
        ToolResponse::Error(err) => panic!("Tool execution failed: {}", err),
    };
    let response_caps: Value = serde_json::from_str(&result_caps_str).expect("Invalid JSON response");
    
    assert!(response_caps["success"].as_bool().unwrap_or(false), "Capabilities platform info should succeed");
    
    // Should include platform capabilities when requested
    if let Some(capabilities) = response_caps.get("capabilities") {
        assert!(capabilities.is_array(), "Capabilities should be array when included");
    }
    if let Some(supported_architectures) = response_caps.get("supported_architectures") {
        assert!(supported_architectures.is_array(), "Supported architectures should be array when included");
    }
    
    session.cleanup().expect("Failed to cleanup session");
}

#[tokio::test]
async fn test_list_modules_comprehensive() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::Normal).expect("Failed to create LLDB session");
    
    // Launch process for module analysis
    session.start().expect("Failed to start debugging session");
    session.set_test_breakpoint("main").expect("Failed to set breakpoint at main");
    
    let tool = ListModulesTool;
    
    // Test 1: List all modules
    let args = HashMap::new();
    let result = tool.execute(args, session.lldb_manager()).await.expect("list_modules failed");
    let result_str = match result {
        ToolResponse::Json(json) => json.to_string(),
        ToolResponse::Success(text) => text,
        ToolResponse::Error(err) => panic!("Tool execution failed: {}", err),
    };
    let response: Value = serde_json::from_str(&result_str).expect("Invalid JSON response");
    
    // Validate modules list response
    assert!(response["success"].as_bool().unwrap_or(false), "list_modules should succeed");
    assert!(response["modules"].is_array(), "Should return modules array");
    assert!(response["total_count"].is_number(), "Should return total_count");
    
    let modules = response["modules"].as_array().expect("modules should be array");
    assert!(modules.len() > 0, "Should find modules in test process");
    
    // Verify module structure
    let first_module = &modules[0];
    assert!(first_module["name"].is_string(), "Module should have name");
    assert!(first_module["path"].is_string(), "Module should have path");
    assert!(first_module["load_address"].is_string(), "Module should have load_address");
    assert!(first_module["has_debug_symbols"].is_boolean(), "Module should have has_debug_symbols");
    
    // Should find the test_debuggee module
    let has_test_module = modules.iter().any(|m| {
        m["name"].as_str().unwrap_or("").contains("test_debuggee") ||
        m["path"].as_str().unwrap_or("").contains("test_debuggee")
    });
    assert!(has_test_module, "Should find test_debuggee module in list");
    
    // Test 2: List modules with debug symbols filter
    let mut args_debug = HashMap::new();
    args_debug.insert("debug_symbols_only".to_string(), Value::Bool(true));
    
    let result_debug = tool.execute(args_debug, session.lldb_manager()).await.expect("list_modules with debug filter failed");
    let result_debug_str = match result_debug {
        ToolResponse::Json(json) => json.to_string(),
        ToolResponse::Success(text) => text,
        ToolResponse::Error(err) => panic!("Tool execution failed: {}", err),
    };
    let response_debug: Value = serde_json::from_str(&result_debug_str).expect("Invalid JSON response");
    
    assert!(response_debug["success"].as_bool().unwrap_or(false), "Debug modules should succeed");
    let debug_modules = response_debug["modules"].as_array().expect("modules should be array");
    
    // All returned modules should have debug symbols when filtered
    for module in debug_modules {
        let has_debug = module["has_debug_symbols"].as_bool().expect("has_debug_symbols should be boolean");
        assert!(has_debug, "All modules should have debug symbols when filtered");
    }
    
    // Test 3: List modules with ASLR slide information
    let mut args_aslr = HashMap::new();
    args_aslr.insert("include_aslr_slide".to_string(), Value::Bool(true));
    
    let result_aslr = tool.execute(args_aslr, session.lldb_manager()).await.expect("list_modules with ASLR failed");
    let result_aslr_str = match result_aslr {
        ToolResponse::Json(json) => json.to_string(),
        ToolResponse::Success(text) => text,
        ToolResponse::Error(err) => panic!("Tool execution failed: {}", err),
    };
    let response_aslr: Value = serde_json::from_str(&result_aslr_str).expect("Invalid JSON response");
    
    assert!(response_aslr["success"].as_bool().unwrap_or(false), "ASLR modules should succeed");
    let aslr_modules = response_aslr["modules"].as_array().expect("modules should be array");
    
    // Should include ASLR slide information when requested
    for module in aslr_modules {
        if let Some(aslr_slide) = module.get("aslr_slide") {
            assert!(aslr_slide.is_string(), "ASLR slide should be string when included");
        }
        if let Some(file_address) = module.get("file_address") {
            assert!(file_address.is_string(), "File address should be string when included");
        }
    }
    
    // Test 4: List modules with symbol count
    let mut args_symbols = HashMap::new();
    args_symbols.insert("include_symbol_count".to_string(), Value::Bool(true));
    
    let result_symbols = tool.execute(args_symbols, session.lldb_manager()).await.expect("list_modules with symbols failed");
    let result_symbols_str = match result_symbols {
        ToolResponse::Json(json) => json.to_string(),
        ToolResponse::Success(text) => text,
        ToolResponse::Error(err) => panic!("Tool execution failed: {}", err),
    };
    let response_symbols: Value = serde_json::from_str(&result_symbols_str).expect("Invalid JSON response");
    
    assert!(response_symbols["success"].as_bool().unwrap_or(false), "Symbol count modules should succeed");
    let symbol_modules = response_symbols["modules"].as_array().expect("modules should be array");
    
    // Should include symbol count when requested
    for module in symbol_modules {
        if let Some(symbol_count) = module.get("symbol_count") {
            assert!(symbol_count.is_number(), "Symbol count should be number when included");
        }
    }
    
    session.cleanup().expect("Failed to cleanup session");
}

#[tokio::test]
async fn test_target_information_filtering_and_limits() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::Normal).expect("Failed to create LLDB session");
    
    // Launch process for filtering tests
    session.start().expect("Failed to start debugging session");
    session.set_test_breakpoint("main").expect("Failed to set breakpoint at main");
    
    // Test list_modules with name pattern filter
    let tool = ListModulesTool;
    let mut args_pattern = HashMap::new();
    args_pattern.insert("name_pattern".to_string(), Value::String("lib".to_string()));
    
    let result_pattern = tool.execute(args_pattern, session.lldb_manager()).await.expect("list_modules with pattern failed");
    let result_pattern_str = match result_pattern {
        ToolResponse::Json(json) => json.to_string(),
        ToolResponse::Success(text) => text,
        ToolResponse::Error(err) => panic!("Tool execution failed: {}", err),
    };
    let response_pattern: Value = serde_json::from_str(&result_pattern_str).expect("Invalid JSON response");
    
    assert!(response_pattern["success"].as_bool().unwrap_or(false), "Pattern filter should succeed");
    let filtered_modules = response_pattern["modules"].as_array().expect("modules should be array");
    
    // All returned modules should match the pattern
    for module in filtered_modules {
        let name = module["name"].as_str().expect("Module name should be string");
        assert!(name.contains("lib") || name.to_lowercase().contains("lib"), 
               "Module name should match pattern filter");
    }
    
    // Test list_modules with limit
    let mut args_limit = HashMap::new();
    args_limit.insert("limit".to_string(), Value::Number(3.into()));
    
    let result_limit = tool.execute(args_limit, session.lldb_manager()).await.expect("list_modules with limit failed");
    let result_limit_str = match result_limit {
        ToolResponse::Json(json) => json.to_string(),
        ToolResponse::Success(text) => text,
        ToolResponse::Error(err) => panic!("Tool execution failed: {}", err),
    };
    let response_limit: Value = serde_json::from_str(&result_limit_str).expect("Invalid JSON response");
    
    assert!(response_limit["success"].as_bool().unwrap_or(false), "Limit should succeed");
    let limited_modules = response_limit["modules"].as_array().expect("modules should be array");
    assert!(limited_modules.len() <= 3, "Should respect limit parameter");
    
    session.cleanup().expect("Failed to cleanup session");
}

#[tokio::test]
async fn test_target_information_error_handling() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::Normal).expect("Failed to create LLDB session");
    
    // Launch process for error testing
    session.start().expect("Failed to start debugging session");
    session.set_test_breakpoint("main").expect("Failed to set breakpoint at main");
    
    // Test target info tools with no active target (after cleanup)
    session.cleanup().expect("Failed to cleanup session");
    
    // Tools should handle missing target gracefully
    let target_tool = GetTargetInfoTool;
    let result_target = target_tool.execute(HashMap::new(), session.lldb_manager()).await.expect("Tool should handle missing target gracefully");
    let result_target_str = match result_target {
        ToolResponse::Json(json) => json.to_string(),
        ToolResponse::Success(text) => text,
        ToolResponse::Error(err) => format!("{{\"success\": false, \"error\": \"{}\"}}", err),
    };
    let response_target: Value = serde_json::from_str(&result_target_str).expect("Invalid JSON response");
    
    // Should handle error gracefully
    if !response_target["success"].as_bool().unwrap_or(false) {
        assert!(response_target["error"].is_string(), "Should provide error message for missing target");
    }
    
    let platform_tool = GetPlatformInfoTool;
    let result_platform = platform_tool.execute(HashMap::new(), session.lldb_manager()).await.expect("Tool should handle missing target gracefully");
    let result_platform_str = match result_platform {
        ToolResponse::Json(json) => json.to_string(),
        ToolResponse::Success(text) => text,
        ToolResponse::Error(err) => format!("{{\"success\": false, \"error\": \"{}\"}}", err),
    };
    let response_platform: Value = serde_json::from_str(&result_platform_str).expect("Invalid JSON response");
    
    // Platform info might still work without active target, but should handle gracefully
    assert!(response_platform["success"].is_boolean(), "Should return success status");
    
    let modules_tool = ListModulesTool;
    let result_modules = modules_tool.execute(HashMap::new(), session.lldb_manager()).await.expect("Tool should handle missing target gracefully");
    let result_modules_str = match result_modules {
        ToolResponse::Json(json) => json.to_string(),
        ToolResponse::Success(text) => text,
        ToolResponse::Error(err) => format!("{{\"success\": false, \"error\": \"{}\"}}", err),
    };
    let response_modules: Value = serde_json::from_str(&result_modules_str).expect("Invalid JSON response");
    
    // Should handle error gracefully for missing target
    if !response_modules["success"].as_bool().unwrap_or(false) {
        assert!(response_modules["error"].is_string(), "Should provide error message for missing target");
    }
}