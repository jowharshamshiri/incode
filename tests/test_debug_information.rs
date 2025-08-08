// InCode Debug Information Tools - Comprehensive Test Suite
// Tests F0050-F0053: get_source_code, list_functions, get_line_info, get_debug_info
// Real LLDB integration testing with test_debuggee binary

use std::collections::HashMap;
use serde_json::Value;

mod test_setup;
use test_setup::{TestDebuggee, TestMode, TestSession};

use incode::tools::debug_information::{
    GetSourceCodeTool, ListFunctionsTool, GetLineInfoTool, GetDebugInfoTool
};
use incode::tools::{Tool, ToolResponse};

#[tokio::test]
async fn test_get_source_code_comprehensive() {
    let test_debuggee = TestDebuggee::new(TestMode::StepDebug).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::StepDebug).expect("Failed to create LLDB session");
    let _pid = session.start().expect("Failed to start session");
    
    // Set breakpoint and continue
    session.set_test_breakpoint("main").expect("Failed to set breakpoint");
    session.continue_execution().expect("Failed to continue");
    
    let tool = GetSourceCodeTool;
    
    // Test 1: Get source code around current location
    let mut args = HashMap::new();
    args.insert("context_lines".to_string(), Value::Number(5.into()));
    
    let result = tool.execute(args.clone(), session.lldb_manager()).await.expect("get_source_code failed");
    let result_str = match result {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response: Value = serde_json::from_str(&result_str).expect("Invalid JSON response");
    
    // Validate source code response structure
    assert!(response["success"].as_bool().unwrap_or(false), "get_source_code should succeed");
    assert!(response["source_lines"].is_array(), "Should return source_lines array");
    assert!(response["current_line"].is_number(), "Should return current_line number");
    assert!(response["file_path"].is_string(), "Should return file_path string");
    
    let source_lines = response["source_lines"].as_array().expect("source_lines should be array");
    assert!(source_lines.len() > 0, "Should return source lines");
    
    // Test 2: Get source code with different context sizes
    args.insert("context_lines".to_string(), Value::Number(10.into()));
    let result_large = tool.execute(args, session.lldb_manager()).await.expect("get_source_code with larger context failed");
    let result_large_str = match result_large {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response_large: Value = serde_json::from_str(&result_large_str).expect("Invalid JSON response");
    
    let source_lines_large = response_large["source_lines"].as_array().expect("source_lines should be array");
    assert!(source_lines_large.len() >= source_lines.len(), "Larger context should return more lines");
    
    // Test 3: Get source code for specific file
    let mut args_file = HashMap::new();
    args_file.insert("file_path".to_string(), Value::String("main.cpp".to_string()));
    args_file.insert("line_number".to_string(), Value::Number(1.into()));
    args_file.insert("context_lines".to_string(), Value::Number(5.into()));
    
    let result_file = tool.execute(args_file, session.lldb_manager()).await.expect("get_source_code for specific file failed");
    let result_file_str = match result_file {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response_file: Value = serde_json::from_str(&result_file_str).expect("Invalid JSON response");
    
    assert!(response_file["success"].as_bool().unwrap_or(false), "get_source_code for file should succeed");
    assert!(response_file["file_path"].as_str().unwrap().contains("main.cpp"), "Should return main.cpp file path");
    
    session.cleanup().expect("Failed to cleanup session");
}

#[tokio::test]
async fn test_list_functions_comprehensive() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::Normal).expect("Failed to create LLDB session");
    let _pid = session.start().expect("Failed to start session");
    
    // Set breakpoint and continue
    session.set_test_breakpoint("main").expect("Failed to set breakpoint");
    session.continue_execution().expect("Failed to continue");
    
    let tool = ListFunctionsTool;
    
    // Test 1: List all functions
    let args = HashMap::new();
    let result = tool.execute(args, session.lldb_manager()).await.expect("list_functions failed");
    let result_str = match result {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response: Value = serde_json::from_str(&result_str).expect("Invalid JSON response");
    
    // Validate functions list response
    assert!(response["success"].as_bool().unwrap_or(false), "list_functions should succeed");
    assert!(response["functions"].is_array(), "Should return functions array");
    assert!(response["total_count"].is_number(), "Should return total_count");
    
    let functions = response["functions"].as_array().expect("functions should be array");
    assert!(functions.len() > 0, "Should find functions in test binary");
    
    // Verify function structure
    let first_function = &functions[0];
    assert!(first_function["name"].is_string(), "Function should have name");
    assert!(first_function["address"].is_string(), "Function should have address");
    assert!(first_function["size"].is_number(), "Function should have size");
    
    // Test 2: List functions with name filter
    let mut args_filter = HashMap::new();
    args_filter.insert("name_pattern".to_string(), Value::String("main".to_string()));
    
    let result_filter = tool.execute(args_filter, session.lldb_manager()).await.expect("list_functions with filter failed");
    let result_filter_str = match result_filter {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response_filter: Value = serde_json::from_str(&result_filter_str).expect("Invalid JSON response");
    
    assert!(response_filter["success"].as_bool().unwrap_or(false), "Filtered list_functions should succeed");
    let filtered_functions = response_filter["functions"].as_array().expect("functions should be array");
    
    // Should find main function
    let has_main = filtered_functions.iter().any(|f| {
        f["name"].as_str().unwrap_or("").contains("main")
    });
    assert!(has_main, "Should find main function with name filter");
    
    // Test 3: List functions with address range
    let mut args_range = HashMap::new();
    args_range.insert("include_addresses".to_string(), Value::Bool(true));
    args_range.insert("limit".to_string(), Value::Number(10.into()));
    
    let result_range = tool.execute(args_range, session.lldb_manager()).await.expect("list_functions with address range failed");
    let result_range_str = match result_range {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response_range: Value = serde_json::from_str(&result_range_str).expect("Invalid JSON response");
    
    assert!(response_range["success"].as_bool().unwrap_or(false), "Address range list_functions should succeed");
    let range_functions = response_range["functions"].as_array().expect("functions should be array");
    assert!(range_functions.len() <= 10, "Should respect limit parameter");
    
    session.cleanup().expect("Failed to cleanup session");
}

#[tokio::test]  
async fn test_get_line_info_comprehensive() {
    let _test_debuggee = TestDebuggee::new(TestMode::StepDebug).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::StepDebug).expect("Failed to create LLDB session");
    session.start().expect("Failed to start session");
    
    // Launch and break at main
    // session.launch_and_break(&test_debuggee, Some("main")).expect("Failed to launch and break");
    
    let tool = GetLineInfoTool;
    
    // Test 1: Get line info for current location
    let args = HashMap::new();
    let result = tool.execute(args, session.lldb_manager()).await.expect("get_line_info failed");
    let result_str = match result {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response: Value = serde_json::from_str(&result_str).expect("Invalid JSON response");
    
    // Validate line info response
    assert!(response["success"].as_bool().unwrap_or(false), "get_line_info should succeed");
    assert!(response["file_path"].is_string(), "Should return file_path");
    assert!(response["line_number"].is_number(), "Should return line_number");
    assert!(response["column"].is_number(), "Should return column");
    assert!(response["address"].is_string(), "Should return address");
    
    let file_path = response["file_path"].as_str().expect("file_path should be string");
    let line_number = response["line_number"].as_u64().expect("line_number should be number");
    assert!(file_path.contains(".cpp"), "Should be C++ source file");
    assert!(line_number > 0, "Line number should be positive");
    
    // Test 2: Get line info for specific address
    let current_address = response["address"].as_str().expect("address should be string");
    
    let mut args_addr = HashMap::new();
    args_addr.insert("address".to_string(), Value::String(current_address.to_string()));
    
    let result_addr = tool.execute(args_addr, session.lldb_manager()).await.expect("get_line_info for address failed");
    let result_addr_str = match result_addr {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response_addr: Value = serde_json::from_str(&result_addr_str).expect("Invalid JSON response");
    
    assert!(response_addr["success"].as_bool().unwrap_or(false), "Address get_line_info should succeed");
    assert_eq!(response_addr["address"].as_str().unwrap(), current_address, "Should return same address");
    
    // Test 3: Get line info with context
    let mut args_context = HashMap::new();
    args_context.insert("include_context".to_string(), Value::Bool(true));
    
    let result_context = tool.execute(args_context, session.lldb_manager()).await.expect("get_line_info with context failed");
    let result_context_str = match result_context {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response_context: Value = serde_json::from_str(&result_context_str).expect("Invalid JSON response");
    
    assert!(response_context["success"].as_bool().unwrap_or(false), "Context get_line_info should succeed");
    if let Some(context) = response_context.get("context") {
        assert!(context.is_object(), "Context should be an object");
    }
    
    session.cleanup().expect("Failed to cleanup session");
}

#[tokio::test]
async fn test_get_debug_info_comprehensive() {
    let _test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::Normal).expect("Failed to create LLDB session");
    session.start().expect("Failed to start session");
    
    // Launch and analyze debug info
    // session.launch_and_break(&test_debuggee, Some("main")).expect("Failed to launch and break");
    
    let tool = GetDebugInfoTool;
    
    // Test 1: Get comprehensive debug info
    let args = HashMap::new();
    let result = tool.execute(args, session.lldb_manager()).await.expect("get_debug_info failed");
    let result_str = match result {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response: Value = serde_json::from_str(&result_str).expect("Invalid JSON response");
    
    // Validate debug info response structure
    assert!(response["success"].as_bool().unwrap_or(false), "get_debug_info should succeed");
    assert!(response["debug_format"].is_string(), "Should return debug_format");
    assert!(response["compilation_units"].is_array(), "Should return compilation_units array");
    assert!(response["symbol_count"].is_number(), "Should return symbol_count");
    assert!(response["has_debug_symbols"].is_boolean(), "Should return has_debug_symbols");
    
    let has_debug_symbols = response["has_debug_symbols"].as_bool().expect("has_debug_symbols should be boolean");
    assert!(has_debug_symbols, "Test binary should have debug symbols");
    
    let compilation_units = response["compilation_units"].as_array().expect("compilation_units should be array");
    assert!(compilation_units.len() > 0, "Should have compilation units");
    
    // Verify compilation unit structure
    let first_unit = &compilation_units[0];
    assert!(first_unit["file"].is_string(), "Compilation unit should have file");
    assert!(first_unit["language"].is_string(), "Compilation unit should have language");
    
    // Test 2: Get debug info with specific sections
    let mut args_sections = HashMap::new();
    args_sections.insert("include_symbols".to_string(), Value::Bool(true));
    args_sections.insert("include_types".to_string(), Value::Bool(true));
    
    let result_sections = tool.execute(args_sections, session.lldb_manager()).await.expect("get_debug_info with sections failed");
    let result_sections_str = match result_sections {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response_sections: Value = serde_json::from_str(&result_sections_str).expect("Invalid JSON response");
    
    assert!(response_sections["success"].as_bool().unwrap_or(false), "Sections debug info should succeed");
    
    // Should include additional sections when requested
    if let Some(symbols) = response_sections.get("symbols") {
        assert!(symbols.is_array(), "Symbols should be array when requested");
    }
    if let Some(types) = response_sections.get("types") {
        assert!(types.is_array(), "Types should be array when requested");
    }
    
    // Test 3: Get debug info summary only
    let mut args_summary = HashMap::new();
    args_summary.insert("summary_only".to_string(), Value::Bool(true));
    
    let result_summary = tool.execute(args_summary, session.lldb_manager()).await.expect("get_debug_info summary failed");
    let result_summary_str = match result_summary {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response_summary: Value = serde_json::from_str(&result_summary_str).expect("Invalid JSON response");
    
    assert!(response_summary["success"].as_bool().unwrap_or(false), "Summary debug info should succeed");
    
    // Summary should have core fields but may omit detailed arrays
    assert!(response_summary["debug_format"].is_string(), "Summary should include debug_format");
    assert!(response_summary["has_debug_symbols"].is_boolean(), "Summary should include has_debug_symbols");
    
    session.cleanup().expect("Failed to cleanup session");
}

#[tokio::test]
async fn test_debug_information_error_handling() {
    let _test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::Normal).expect("Failed to create LLDB session");
    session.start().expect("Failed to start session");
    
    // Launch process for error testing
    // session.launch_and_break(&test_debuggee, Some("main")).expect("Failed to launch and break");
    
    // Test invalid address for get_line_info
    let tool_line = GetLineInfoTool;
    let mut args_invalid = HashMap::new();
    args_invalid.insert("address".to_string(), Value::String("0xdeadbeef".to_string()));
    
    let result = tool_line.execute(args_invalid, session.lldb_manager()).await.expect("Tool should handle invalid address gracefully");
    let result_str = match result {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response: Value = serde_json::from_str(&result_str).expect("Invalid JSON response");
    
    // Should handle error gracefully
    if !response["success"].as_bool().unwrap_or(false) {
        assert!(response["error"].is_string(), "Should provide error message for invalid address");
    }
    
    // Test invalid file path for get_source_code  
    let tool_source = GetSourceCodeTool;
    let mut args_invalid_file = HashMap::new();
    args_invalid_file.insert("file_path".to_string(), Value::String("/nonexistent/file.cpp".to_string()));
    args_invalid_file.insert("line_number".to_string(), Value::Number(1.into()));
    
    let result_file = tool_source.execute(args_invalid_file, session.lldb_manager()).await.expect("Tool should handle invalid file gracefully");
    let result_file_str = match result_file {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response_file: Value = serde_json::from_str(&result_file_str).expect("Invalid JSON response");
    
    // Should handle file not found gracefully
    if !response_file["success"].as_bool().unwrap_or(false) {
        assert!(response_file["error"].is_string(), "Should provide error message for invalid file");
    }
    
    session.cleanup().expect("Failed to cleanup session");
}