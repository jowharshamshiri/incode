use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::error::{IncodeError, IncodeResult};
use crate::lldb_manager::LldbManager;
use super::{Tool, ToolResponse};

// Advanced Analysis Tools (2 tools)
pub struct AnalyzeCrashTool;
pub struct GenerateCoreDumpTool;

/// Analyze crash dumps and provide detailed crash information
#[async_trait]
impl Tool for AnalyzeCrashTool {
    fn name(&self) -> &'static str {
        "analyze_crash"
    }
    
    fn description(&self) -> &'static str {
        "Analyze crash dumps and provide detailed crash information"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "core_file_path": {
                "type": "string",
                "description": "Path to core dump file to analyze (optional, uses current process if not specified)"
            },
            "include_recommendations": {
                "type": "boolean",
                "description": "Include debugging recommendations in the analysis",
                "default": true
            },
            "max_backtrace_depth": {
                "type": "number",
                "description": "Maximum depth for backtrace analysis",
                "default": 10,
                "minimum": 1,
                "maximum": 100
            }
        })
    }
    
    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let core_file_path = arguments.get("core_file_path")
            .and_then(|v| v.as_str());
        
        let include_recommendations = arguments.get("include_recommendations")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        let max_backtrace_depth = arguments.get("max_backtrace_depth")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        let analysis = lldb_manager.analyze_crash(core_file_path)?;
        
        let has_crash = analysis.crash_type != "No crash";
        
        let mut response = json!({
            "success": has_crash,
            "crash_info": {
                "signal": analysis.signal_name,
                "signal_number": analysis.signal_number,
                "address": analysis.crash_address.map(|addr| format!("0x{:x}", addr)),
                "type": analysis.crash_type,
                "thread_id": analysis.faulting_thread,
                "exception_type": analysis.exception_type,
                "exception_codes": analysis.exception_codes,
            },
            "crash_type": analysis.crash_type,
            "crash_address": analysis.crash_address.map(|addr| format!("0x{:x}", addr)),
            "faulting_thread": analysis.faulting_thread,
            "signal_number": analysis.signal_number,
            "signal_name": analysis.signal_name,
            "exception_type": analysis.exception_type,
            "exception_codes": analysis.exception_codes,
            "crashed_thread_backtrace": analysis.crashed_thread_backtrace.into_iter()
                .take(max_backtrace_depth)
                .collect::<Vec<_>>(),
            "register_state": analysis.register_state,
            "memory_regions": analysis.memory_regions,
            "loaded_modules": analysis.loaded_modules,
            "crash_summary": analysis.crash_summary,
            "analysis_summary": analysis.crash_summary,
            "core_file": core_file_path,
            "analyzed_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_secs(),
            "status": if has_crash { "analyzed" } else { "no_crash" }
        });
        
        if !has_crash {
            response["error"] = json!("No crash to analyze - no active process or core file provided");
        }
        
        if include_recommendations {
            response["recommendations"] = json!(analysis.recommendations);
        }
        
        Ok(ToolResponse::Success(response.to_string()))
    }
}

/// Generate core dump files for current process state
#[async_trait]
impl Tool for GenerateCoreDumpTool {
    fn name(&self) -> &'static str {
        "generate_core_dump"
    }
    
    fn description(&self) -> &'static str {
        "Generate core dump files for current process state"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "output_path": {
                "type": "string",
                "description": "Path where core dump file should be saved"
            },
            "format": {
                "type": "string",
                "enum": ["auto", "elf", "macho", "minidump"],
                "description": "Core dump format (default: auto - detect from platform)",
                "default": "auto"
            },
            "include_memory": {
                "type": "boolean",
                "description": "Include full memory contents in core dump",
                "default": true
            }
        })
    }
    
    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let output_path = arguments.get("output_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IncodeError::invalid_parameter("output_path required"))?;
        
        let format = arguments.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("auto");
        
        let include_memory = arguments.get("include_memory")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let result = match lldb_manager.generate_core_dump(output_path) {
            Ok(r) => r,
            Err(IncodeError::ProcessError(_)) => {
                // Handle no process gracefully
                return Ok(ToolResponse::Success(json!({
                    "success": false,
                    "error": "No process attached",
                    "output_path": output_path,
                    "core_dump_path": output_path,
                    "format": format,
                    "include_memory": include_memory,
                    "file_size": 0,
                    "file_exists": false,
                    "status": "no_process"
                }).to_string()));
            },
            Err(e) => return Err(e),
        };
        
        // Check if file was actually created
        let file_exists = std::path::Path::new(output_path).exists();
        let file_size = if file_exists {
            std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };
        
        Ok(ToolResponse::Success(json!({
            "success": true,
            "output_path": output_path,
            "core_dump_path": output_path,
            "format": format,
            "include_memory": include_memory,
            "file_exists": file_exists,
            "file_size": file_size,
            "result": result,
            "generated_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_secs().to_string(),
            "status": if file_exists { "generated" } else { "completed" }
        }).to_string()))
    }
}

// Keep the old PlaceholderTool for compatibility
pub struct PlaceholderTool;

#[async_trait]
impl Tool for PlaceholderTool {
    fn name(&self) -> &'static str {
        "placeholder"
    }
    
    fn description(&self) -> &'static str {
        "Placeholder tool - needs implementation"
    }
    
    fn parameters(&self) -> Value {
        json!({})
    }
    
    async fn execute(
        &self,
        _: HashMap<String, Value>,
        _: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        Ok(ToolResponse::Error("Not implemented".to_string()))
    }
}
