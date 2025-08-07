// InCode Target Information Tools - Comprehensive Test Suite  
// Tests F0054-F0056: get_target_info, get_platform_info, list_modules
// Real LLDB integration testing with test_debuggee binary

use std::collections::HashMap;
use serde_json::Value;

mod test_setup;
use test_setup::{TestDebuggee, TestMode, LldbTestSession};

use incode::tools::target_info::{GetTargetInfoTool, GetPlatformInfoTool, ListModulesTool};
use incode::mcp_server::McpTool;

#[test]
fn test_get_target_info_comprehensive() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = LldbTestSession::new().expect("Failed to create LLDB session");
    
    // Launch process for target analysis
    session.launch_and_break(&test_debuggee, Some("main")).expect("Failed to launch and break");
    
    let tool = GetTargetInfoTool::new(session.manager());
    
    // Test 1: Get comprehensive target info
    let args = HashMap::new();
    let result = tool.call(args).expect("get_target_info failed");
    let response: Value = serde_json::from_str(&result).expect("Invalid JSON response");
    
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
    
    let result_metadata = tool.call(args_metadata).expect("get_target_info with metadata failed");
    let response_metadata: Value = serde_json::from_str(&result_metadata).expect("Invalid JSON response");
    
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
    
    let result_symbols = tool.call(args_symbols).expect("get_target_info with symbols failed");
    let response_symbols: Value = serde_json::from_str(&result_symbols).expect("Invalid JSON response");
    
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

#[test]
fn test_get_platform_info_comprehensive() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = LldbTestSession::new().expect("Failed to create LLDB session");
    
    // Launch process for platform analysis
    session.launch_and_break(&test_debuggee, Some("main")).expect("Failed to launch and break");
    
    let tool = GetPlatformInfoTool::new(session.manager());
    
    // Test 1: Get basic platform info
    let args = HashMap::new();
    let result = tool.call(args).expect("get_platform_info failed");
    let response: Value = serde_json::from_str(&result).expect("Invalid JSON response");
    
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
    
    let result_dev = tool.call(args_dev).expect("get_platform_info with dev info failed");
    let response_dev: Value = serde_json::from_str(&result_dev).expect("Invalid JSON response");
    
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
    
    let result_caps = tool.call(args_caps).expect("get_platform_info with capabilities failed");
    let response_caps: Value = serde_json::from_str(&result_caps).expect("Invalid JSON response");
    
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

#[test]  
fn test_list_modules_comprehensive() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = LldbTestSession::new().expect("Failed to create LLDB session");
    
    // Launch process for module analysis
    session.launch_and_break(&test_debuggee, Some("main")).expect("Failed to launch and break");
    
    let tool = ListModulesTool::new(session.manager());
    
    // Test 1: List all modules
    let args = HashMap::new();
    let result = tool.call(args).expect("list_modules failed");
    let response: Value = serde_json::from_str(&result).expect("Invalid JSON response");
    
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
    
    let result_debug = tool.call(args_debug).expect("list_modules with debug filter failed");
    let response_debug: Value = serde_json::from_str(&result_debug).expect("Invalid JSON response");
    
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
    
    let result_aslr = tool.call(args_aslr).expect("list_modules with ASLR failed");
    let response_aslr: Value = serde_json::from_str(&result_aslr).expect("Invalid JSON response");
    
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
    
    let result_symbols = tool.call(args_symbols).expect("list_modules with symbols failed");
    let response_symbols: Value = serde_json::from_str(&result_symbols).expect("Invalid JSON response");
    
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

#[test]
fn test_target_information_filtering_and_limits() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = LldbTestSession::new().expect("Failed to create LLDB session");
    
    // Launch process for filtering tests
    session.launch_and_break(&test_debuggee, Some("main")).expect("Failed to launch and break");
    
    // Test list_modules with name pattern filter
    let tool = ListModulesTool::new(session.manager());
    let mut args_pattern = HashMap::new();
    args_pattern.insert("name_pattern".to_string(), Value::String("lib".to_string()));
    
    let result_pattern = tool.call(args_pattern).expect("list_modules with pattern failed");
    let response_pattern: Value = serde_json::from_str(&result_pattern).expect("Invalid JSON response");
    
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
    
    let result_limit = tool.call(args_limit).expect("list_modules with limit failed");
    let response_limit: Value = serde_json::from_str(&result_limit).expect("Invalid JSON response");
    
    assert!(response_limit["success"].as_bool().unwrap_or(false), "Limit should succeed");
    let limited_modules = response_limit["modules"].as_array().expect("modules should be array");
    assert!(limited_modules.len() <= 3, "Should respect limit parameter");
    
    session.cleanup().expect("Failed to cleanup session");
}

#[test]
fn test_target_information_error_handling() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = LldbTestSession::new().expect("Failed to create LLDB session");
    
    // Launch process for error testing
    session.launch_and_break(&test_debuggee, Some("main")).expect("Failed to launch and break");
    
    // Test target info tools with no active target (after cleanup)
    session.cleanup().expect("Failed to cleanup session");
    
    // Tools should handle missing target gracefully
    let target_tool = GetTargetInfoTool::new(session.manager());
    let result_target = target_tool.call(HashMap::new()).expect("Tool should handle missing target gracefully");
    let response_target: Value = serde_json::from_str(&result_target).expect("Invalid JSON response");
    
    // Should handle error gracefully
    if !response_target["success"].as_bool().unwrap_or(false) {
        assert!(response_target["error"].is_string(), "Should provide error message for missing target");
    }
    
    let platform_tool = GetPlatformInfoTool::new(session.manager());
    let result_platform = platform_tool.call(HashMap::new()).expect("Tool should handle missing target gracefully");
    let response_platform: Value = serde_json::from_str(&result_platform).expect("Invalid JSON response");
    
    // Platform info might still work without active target, but should handle gracefully
    assert!(response_platform["success"].is_boolean(), "Should return success status");
    
    let modules_tool = ListModulesTool::new(session.manager());
    let result_modules = modules_tool.call(HashMap::new()).expect("Tool should handle missing target gracefully");
    let response_modules: Value = serde_json::from_str(&result_modules).expect("Invalid JSON response");
    
    // Should handle error gracefully for missing target
    if !response_modules["success"].as_bool().unwrap_or(false) {
        assert!(response_modules["error"].is_string(), "Should provide error message for missing target");
    }
}