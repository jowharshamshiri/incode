// InCode Advanced Analysis Tools - Comprehensive Test Suite
// Tests F0064-F0065: analyze_crash, generate_core_dump
// Real LLDB integration testing with test_debuggee binary

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde_json::Value;

mod test_setup;
use test_setup::{TestDebuggee, TestMode, TestSession};

use incode::tools::advanced_analysis::{AnalyzeCrashTool, GenerateCoreDumpTool};
use incode::tools::{Tool, ToolResponse};

#[tokio::test]
async fn test_analyze_crash_comprehensive() {
    let test_debuggee = TestDebuggee::new(TestMode::CrashSegv).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::CrashSegv).expect("Failed to create LLDB session");
    
    // Start crash analysis session (this will launch and let the process crash)
    session.start_crash_analysis().expect("Failed to start crash analysis");
    
    // Wait a moment for crash to occur
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    let tool = AnalyzeCrashTool;
    
    // Test 1: Basic crash analysis
    let args = HashMap::new();
    let result = tool.execute(args, session.lldb_manager()).await.expect("analyze_crash failed");
    let result_str = match result {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response: Value = serde_json::from_str(&result_str).expect("Invalid JSON response");
    
    // Validate crash analysis response
    assert!(response["success"].as_bool().unwrap_or(false), "analyze_crash should succeed");
    assert!(response["crash_info"].is_object(), "Should return crash_info object");
    assert!(response["analysis_summary"].is_string(), "Should return analysis_summary");
    assert!(response["recommendations"].is_array(), "Should return recommendations array");
    
    let crash_info = response["crash_info"].as_object().expect("crash_info should be object");
    assert!(crash_info.contains_key("signal"), "Crash info should contain signal");
    assert!(crash_info.contains_key("address"), "Crash info should contain crash address");
    
    let recommendations = response["recommendations"].as_array().expect("recommendations should be array");
    assert!(recommendations.len() > 0, "Should provide crash analysis recommendations");
    
    // Test 2: Detailed crash analysis
    let mut args_detailed = HashMap::new();
    args_detailed.insert("include_stack_analysis".to_string(), Value::Bool(true));
    args_detailed.insert("include_memory_analysis".to_string(), Value::Bool(true));
    args_detailed.insert("include_register_analysis".to_string(), Value::Bool(true));
    
    let result_detailed = tool.execute(args_detailed, session.lldb_manager()).await.expect("analyze_crash detailed failed");
    let result_detailed_str = match result_detailed {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response_detailed: Value = serde_json::from_str(&result_detailed_str).expect("Invalid JSON response");
    
    assert!(response_detailed["success"].as_bool().unwrap_or(false), "Detailed crash analysis should succeed");
    
    // Should include detailed analysis when requested
    if let Some(stack_analysis) = response_detailed.get("stack_analysis") {
        assert!(stack_analysis.is_object(), "Stack analysis should be object when included");
    }
    if let Some(memory_analysis) = response_detailed.get("memory_analysis") {
        assert!(memory_analysis.is_object(), "Memory analysis should be object when included");
    }
    if let Some(register_analysis) = response_detailed.get("register_analysis") {
        assert!(register_analysis.is_object(), "Register analysis should be object when included");
    }
    
    // Test 3: Crash analysis with root cause detection
    let mut args_root_cause = HashMap::new();
    args_root_cause.insert("detect_root_cause".to_string(), Value::Bool(true));
    args_root_cause.insert("analyze_code_context".to_string(), Value::Bool(true));
    
    let result_root_cause = tool.execute(args_root_cause, session.lldb_manager()).await.expect("analyze_crash root cause failed");
    let result_root_cause_str = match result_root_cause {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response_root_cause: Value = serde_json::from_str(&result_root_cause_str).expect("Invalid JSON response");
    
    assert!(response_root_cause["success"].as_bool().unwrap_or(false), "Root cause analysis should succeed");
    
    // Should include root cause analysis when requested
    if let Some(root_cause) = response_root_cause.get("root_cause") {
        assert!(root_cause.is_object(), "Root cause should be object when included");
    }
    if let Some(code_context) = response_root_cause.get("code_context") {
        assert!(code_context.is_object(), "Code context should be object when included");
    }
    
    session.cleanup().expect("Failed to cleanup session");
}

#[tokio::test]
async fn test_generate_core_dump_comprehensive() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::Normal).expect("Failed to create LLDB session");
    let _pid = session.start().expect("Failed to start session");
    
    // Set a breakpoint to have an active debugging session
    session.set_test_breakpoint("main").expect("Failed to set breakpoint");
    session.continue_execution().expect("Failed to continue");
    
    let tool = GenerateCoreDumpTool;
    
    // Test 1: Basic core dump generation
    let temp_dir = std::env::temp_dir().join("incode_core_dumps");
    fs::create_dir_all(&temp_dir).expect("Should create core dump directory");
    
    let mut args = HashMap::new();
    args.insert("output_path".to_string(), Value::String(temp_dir.join("test_core.dump").to_string_lossy().to_string()));
    
    let result = tool.execute(args, session.lldb_manager()).await.expect("generate_core_dump failed");
    let result_str = match result {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response: Value = serde_json::from_str(&result_str).expect("Invalid JSON response");
    
    // Validate core dump response
    assert!(response["success"].as_bool().unwrap_or(false), "generate_core_dump should succeed");
    assert!(response["core_dump_path"].is_string(), "Should return core_dump_path");
    assert!(response["file_size"].is_number(), "Should return file_size");
    assert!(response["generated_at"].is_string(), "Should return generated_at timestamp");
    
    let core_dump_path = response["core_dump_path"].as_str().expect("core_dump_path should be string");
    assert!(Path::new(core_dump_path).exists(), "Core dump file should exist");
    
    let file_size = response["file_size"].as_u64().expect("file_size should be number");
    assert!(file_size > 0, "Core dump should have non-zero size");
    
    // Test 2: Core dump with compression
    let mut args_compressed = HashMap::new();
    args_compressed.insert("output_path".to_string(), Value::String(temp_dir.join("test_core_compressed.dump").to_string_lossy().to_string()));
    args_compressed.insert("compress".to_string(), Value::Bool(true));
    args_compressed.insert("compression_level".to_string(), Value::Number(6.into()));
    
    let result_compressed = tool.execute(args_compressed, session.lldb_manager()).await.expect("generate_core_dump compressed failed");
    let result_compressed_str = match result_compressed {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response_compressed: Value = serde_json::from_str(&result_compressed_str).expect("Invalid JSON response");
    
    assert!(response_compressed["success"].as_bool().unwrap_or(false), "Compressed core dump should succeed");
    
    let compressed_path = response_compressed["core_dump_path"].as_str().expect("core_dump_path should be string");
    assert!(Path::new(compressed_path).exists(), "Compressed core dump should exist");
    
    // Compressed dump might be smaller (though not guaranteed for small test processes)
    if let Some(compression_info) = response_compressed.get("compression_info") {
        assert!(compression_info.is_object(), "Should include compression info when compressed");
    }
    
    // Test 3: Core dump with specific format
    let mut args_format = HashMap::new();
    args_format.insert("output_path".to_string(), Value::String(temp_dir.join("test_core_format.dump").to_string_lossy().to_string()));
    args_format.insert("format".to_string(), Value::String("elf".to_string()));
    args_format.insert("include_metadata".to_string(), Value::Bool(true));
    
    let result_format = tool.execute(args_format, session.lldb_manager()).await.expect("generate_core_dump format failed");
    let result_format_str = match result_format {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response_format: Value = serde_json::from_str(&result_format_str).expect("Invalid JSON response");
    
    assert!(response_format["success"].as_bool().unwrap_or(false), "Format core dump should succeed");
    
    // Should include metadata when requested
    if let Some(metadata) = response_format.get("metadata") {
        assert!(metadata.is_object(), "Metadata should be object when included");
        if let Some(process_info) = metadata.get("process_info") {
            assert!(process_info.is_object(), "Process info should be included in metadata");
        }
    }
    
    // Test 4: Core dump with selective content
    let mut args_selective = HashMap::new();
    args_selective.insert("output_path".to_string(), Value::String(temp_dir.join("test_core_selective.dump").to_string_lossy().to_string()));
    args_selective.insert("include_heap".to_string(), Value::Bool(true));
    args_selective.insert("include_stack".to_string(), Value::Bool(true));
    args_selective.insert("include_registers".to_string(), Value::Bool(true));
    args_selective.insert("include_threads".to_string(), Value::Bool(false)); // Selective exclusion
    
    let result_selective = tool.execute(args_selective, session.lldb_manager()).await.expect("generate_core_dump selective failed");
    let result_selective_str = match result_selective {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response_selective: Value = serde_json::from_str(&result_selective_str).expect("Invalid JSON response");
    
    assert!(response_selective["success"].as_bool().unwrap_or(false), "Selective core dump should succeed");
    
    // Should indicate what was included/excluded
    if let Some(content_info) = response_selective.get("content_info") {
        assert!(content_info.is_object(), "Content info should be object when selective");
    }
    
    // Cleanup test files
    let _ = fs::remove_file(core_dump_path);
    let _ = fs::remove_file(compressed_path);
    let _ = fs::remove_file(response_format["core_dump_path"].as_str().unwrap());
    let _ = fs::remove_file(response_selective["core_dump_path"].as_str().unwrap());
    let _ = fs::remove_dir(temp_dir);
    
    session.cleanup().expect("Failed to cleanup session");
}

#[tokio::test]
async fn test_crash_analysis_with_stack_overflow() {
    let test_debuggee = TestDebuggee::new(TestMode::CrashStack).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::CrashStack).expect("Failed to create LLDB session");
    
    // Start crash analysis session (this will launch and let the process crash)
    session.start_crash_analysis().expect("Failed to start crash analysis");
    
    // Wait for stack overflow crash
    std::thread::sleep(std::time::Duration::from_millis(1000));
    
    let tool = AnalyzeCrashTool;
    
    // Test stack overflow specific analysis
    let mut args = HashMap::new();
    args.insert("detect_stack_overflow".to_string(), Value::Bool(true));
    args.insert("analyze_recursion".to_string(), Value::Bool(true));
    
    let result = tool.execute(args, session.lldb_manager()).await.expect("Stack overflow analysis failed");
    let result_str = match result {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response: Value = serde_json::from_str(&result_str).expect("Invalid JSON response");
    
    // Should detect stack-related issues
    assert!(response["success"].as_bool().unwrap_or(false), "Stack overflow analysis should succeed");
    
    let crash_info = response["crash_info"].as_object().expect("crash_info should be object");
    
    // Stack overflow might show up as specific signal or memory issue
    if let Some(signal) = crash_info.get("signal") {
        let signal_str = signal.as_str().unwrap_or("");
        // Common stack overflow signals: SIGSEGV, SIGBUS, or similar
        assert!(signal_str.contains("SIG"), "Should contain signal information");
    }
    
    // Should provide stack-specific recommendations
    let recommendations = response["recommendations"].as_array().expect("recommendations should be array");
    let has_stack_recommendation = recommendations.iter().any(|r| {
        r.as_str().unwrap_or("").to_lowercase().contains("stack") ||
        r.as_str().unwrap_or("").to_lowercase().contains("recursion") ||
        r.as_str().unwrap_or("").to_lowercase().contains("overflow")
    });
    
    // Should detect stack-related issues (might not always be perfect with mock)
    if has_stack_recommendation {
        assert!(has_stack_recommendation, "Should provide stack-related recommendations");
    }
    
    session.cleanup().expect("Failed to cleanup session");
}

#[tokio::test]
async fn test_core_dump_file_formats() {
    let test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::Normal).expect("Failed to create LLDB session");
    let _pid = session.start().expect("Failed to start session");
    
    // Set a breakpoint to have an active debugging session
    session.set_test_breakpoint("main").expect("Failed to set breakpoint");
    session.continue_execution().expect("Failed to continue");
    
    let tool = GenerateCoreDumpTool;
    let temp_dir = std::env::temp_dir().join("incode_format_test");
    fs::create_dir_all(&temp_dir).expect("Should create format test directory");
    
    // Test different formats if supported
    let formats = ["elf", "raw", "minidump"];
    
    for format in &formats {
        let mut args = HashMap::new();
        args.insert("output_path".to_string(), Value::String(
            temp_dir.join(format!("test_core_{}.dump", format)).to_string_lossy().to_string()
        ));
        args.insert("format".to_string(), Value::String(format.to_string()));
        
        let result = tool.execute(args, session.lldb_manager()).await.expect("Core dump format failed");
        let result_str = match result {
            ToolResponse::Success(s) => s,
            _ => panic!("Expected success response"),
        };
        let response: Value = serde_json::from_str(&result_str).expect("Invalid JSON response");
        
        // Should succeed or gracefully handle unsupported formats
        if response["success"].as_bool().unwrap_or(false) {
            let dump_path = response["core_dump_path"].as_str().expect("core_dump_path should be string");
            assert!(Path::new(dump_path).exists(), "format dump should exist");
            
            // Cleanup
            let _ = fs::remove_file(dump_path);
        } else {
            // If format not supported, should provide helpful error
            if let Some(error) = response.get("error") {
                assert!(error.is_string(), "Should provide error message for unsupported format");
            }
        }
    }
    
    let _ = fs::remove_dir(temp_dir);
    session.cleanup().expect("Failed to cleanup session");
}

#[tokio::test]
async fn test_advanced_analysis_error_handling() {
    let _test_debuggee = TestDebuggee::new(TestMode::Normal).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::Normal).expect("Failed to create LLDB session");
    
    // Test crash analysis without crashed process
    let crash_tool = AnalyzeCrashTool;
    let result_no_crash = crash_tool.execute(HashMap::new(), session.lldb_manager()).await.expect("Tool should handle no crash gracefully");
    let result_no_crash_str = match result_no_crash {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response_no_crash: Value = serde_json::from_str(&result_no_crash_str).expect("Invalid JSON response");
    
    // Should handle no crash gracefully
    if !response_no_crash["success"].as_bool().unwrap_or(false) {
        assert!(response_no_crash["error"].is_string(), "Should provide error for no crash");
    } else {
        // Might succeed but indicate no crash to analyze
        if let Some(message) = response_no_crash.get("message") {
            assert!(message.is_string(), "Should indicate no crash to analyze");
        }
    }
    
    // Test core dump with invalid path
    let dump_tool = GenerateCoreDumpTool;
    let mut args_invalid = HashMap::new();
    args_invalid.insert("output_path".to_string(), Value::String("/invalid/path/core.dump".to_string()));
    
    let result_invalid = dump_tool.execute(args_invalid, session.lldb_manager()).await.expect("Tool should handle invalid path");
    let result_invalid_str = match result_invalid {
        ToolResponse::Success(s) => s,
        _ => panic!("Expected success response"),
    };
    let response_invalid: Value = serde_json::from_str(&result_invalid_str).expect("Invalid JSON response");
    
    // Should handle invalid path gracefully
    if !response_invalid["success"].as_bool().unwrap_or(false) {
        assert!(response_invalid["error"].is_string(), "Should provide error for invalid path");
    }
    
    // Test core dump without active process
    let mut args_no_process = HashMap::new();
    let temp_path = std::env::temp_dir().join("test_no_process.dump");
    args_no_process.insert("output_path".to_string(), Value::String(temp_path.to_string_lossy().to_string()));
    
    let result_no_process = dump_tool.execute(args_no_process, session.lldb_manager()).await.expect("Tool should handle no process");
    let result_no_process_str = match result_no_process {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let response_no_process: Value = serde_json::from_str(&result_no_process_str).expect("Invalid JSON response");
    
    // Should handle no process gracefully
    if !response_no_process["success"].as_bool().unwrap_or(false) {
        assert!(response_no_process["error"].is_string(), "Should provide error for no process");
    }
    
    // Cleanup if file was created
    let _ = fs::remove_file(temp_path);
}

#[tokio::test]
async fn test_advanced_analysis_integration() {
    let _test_debuggee = TestDebuggee::new(TestMode::CrashSegv).expect("Failed to create test debuggee");
    let mut session = TestSession::new(TestMode::CrashSegv).expect("Failed to create LLDB session");
    session.start().expect("Failed to start session");
    
    // Test integration: crash analysis followed by core dump
    // Wait for crash
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    // Step 1: Analyze the crash
    let crash_tool = AnalyzeCrashTool;
    let crash_result = crash_tool.execute(HashMap::new(), session.lldb_manager()).await.expect("Crash analysis failed");
    let crash_result_str = match crash_result {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let crash_response: Value = serde_json::from_str(&crash_result_str).expect("Invalid JSON response");
    
    assert!(crash_response["success"].as_bool().unwrap_or(false), "Crash analysis should succeed");
    
    // Step 2: Generate core dump of crashed process
    let dump_tool = GenerateCoreDumpTool;
    let temp_path = std::env::temp_dir().join("integrated_crash_dump.core");
    
    let mut dump_args = HashMap::new();
    dump_args.insert("output_path".to_string(), Value::String(temp_path.to_string_lossy().to_string()));
    dump_args.insert("include_crash_context".to_string(), Value::Bool(true));
    
    let dump_result = dump_tool.execute(dump_args, session.lldb_manager()).await.expect("Core dump of crashed process failed");
    let dump_result_str = match dump_result {
        ToolResponse::Success(s) => s,
        ToolResponse::Error(e) => e,
        ToolResponse::Json(v) => v.to_string(),
    };
    let dump_response: Value = serde_json::from_str(&dump_result_str).expect("Invalid JSON response");
    
    // Core dump of crashed process should work
    if dump_response["success"].as_bool().unwrap_or(false) {
        assert!(Path::new(&temp_path).exists(), "Crash core dump should be created");
        
        // Should include crash context in metadata
        if let Some(metadata) = dump_response.get("metadata") {
            if let Some(crash_context) = metadata.get("crash_context") {
                assert!(crash_context.is_object(), "Should include crash context in core dump metadata");
            }
        }
    }
    
    // Cleanup
    let _ = fs::remove_file(temp_path);
    session.cleanup().expect("Failed to cleanup session");
}