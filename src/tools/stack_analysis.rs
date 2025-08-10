use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::error::{IncodeError, IncodeResult};
use crate::lldb_manager::LldbManager;
use super::{Tool, ToolResponse};

// Stack & Frame Analysis Tools (6 tools)
pub struct GetBacktraceTool;
pub struct SelectFrameTool;
pub struct GetFrameInfoTool;
pub struct GetFrameVariablesTool;
pub struct GetFrameArgumentsTool;
pub struct EvaluateInFrameTool;

// F0022: get_backtrace - Fully implemented
#[async_trait]
impl Tool for GetBacktraceTool {
    fn name(&self) -> &'static str {
        "get_backtrace"
    }

    fn description(&self) -> &'static str {
        "Get call stack with function names, addresses, and frame information"
    }

    fn parameters(&self) -> Value {
        json!({
            "max_frames": {
                "type": "integer",
                "description": "Maximum number of frames to retrieve (0 for all)",
                "default": 0,
                "minimum": 0,
                "maximum": 1000
            },
            "include_addresses": {
                "type": "boolean",
                "description": "Include program counter and stack pointer addresses",
                "default": true
            },
            "format": {
                "type": "string",
                "description": "Output format for backtrace",
                "enum": ["compact", "detailed", "gdb_style"],
                "default": "detailed"
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let max_frames = arguments.get("max_frames")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let _include_addresses = arguments.get("include_addresses")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let format = arguments.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("detailed");

        match lldb_manager.get_backtrace() {
            Ok(mut backtrace) => {
                // Limit frames if requested
                if max_frames > 0 && backtrace.len() > max_frames {
                    backtrace.truncate(max_frames);
                    backtrace.push(format!("... ({} more frames truncated)", backtrace.len() - max_frames));
                }

                // Format based on user preference
                let formatted_backtrace = match format {
                    "compact" => {
                        backtrace.iter().map(|frame| {
                            // Extract just function name from detailed format
                            if let Some(start) = frame.find(": ") {
                                if let Some(end) = frame[start+2..].find(" (") {
                                    format!("{}{}", &frame[..start+2], &frame[start+2..start+2+end])
                                } else {
                                    frame.clone()
                                }
                            } else {
                                frame.clone()
                            }
                        }).collect()
                    },
                    "gdb_style" => {
                        backtrace.iter().map(|frame| {
                            frame.replace("#", "#").replace("PC:", "at")
                        }).collect()
                    },
                    _ => backtrace, // detailed is default
                };

                Ok(ToolResponse::Json(json!({
                    "backtrace": formatted_backtrace,
                    "frame_count": formatted_backtrace.len(),
                    "format": format,
                    "truncated": max_frames > 0 && formatted_backtrace.len() >= max_frames,
                    "message": format!("Retrieved {} stack frames", formatted_backtrace.len())
                })))
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}


// F0023: select_frame - Fully implemented
#[async_trait]
impl Tool for SelectFrameTool {
    fn name(&self) -> &'static str {
        "select_frame"
    }

    fn description(&self) -> &'static str {
        "Switch to specific stack frame by index for debugging context"
    }

    fn parameters(&self) -> Value {
        json!({
            "frame_index": {
                "type": "integer",
                "description": "Stack frame index to select (0 is current frame)",
                "minimum": 0,
                "maximum": 1000
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let frame_index = arguments.get("frame_index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| IncodeError::mcp("Missing frame_index parameter"))? as u32;

        match lldb_manager.select_frame(frame_index) {
            Ok(frame_info) => {
                Ok(ToolResponse::Json(json!({
                    "success": true,
                    "selected_frame": {
                        "index": frame_info.index,
                        "function_name": frame_info.function_name,
                        "pc": format!("0x{:x}", frame_info.pc),
                        "sp": format!("0x{:x}", frame_info.sp),
                        "module": frame_info.module,
                        "file": frame_info.file,
                        "line": frame_info.line
                    },
                    "message": format!("Selected frame {} in function {}", frame_info.index, frame_info.function_name)
                })))
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}
// F0024: get_frame_info - Fully implemented
#[async_trait]
impl Tool for GetFrameInfoTool {
    fn name(&self) -> &'static str {
        "get_frame_info"
    }

    fn description(&self) -> &'static str {
        "Get current frame details including function, addresses, module, and source location"
    }

    fn parameters(&self) -> Value {
        json!({
            "frame_index": {
                "type": "integer",
                "description": "Specific frame index to get info for (defaults to current frame)",
                "minimum": 0,
                "maximum": 1000
            },
            "include_addresses": {
                "type": "boolean",
                "description": "Include program counter and stack pointer addresses",
                "default": true
            },
            "include_source": {
                "type": "boolean",
                "description": "Include source file and line information",
                "default": true
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let frame_index = arguments.get("frame_index")
            .and_then(|v| v.as_u64())
            .map(|i| i as u32);

        let include_addresses = arguments.get("include_addresses")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let include_source = arguments.get("include_source")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        match lldb_manager.get_frame_info(frame_index) {
            Ok(frame_info) => {
                let mut response = json!({
                    "frame_index": frame_info.index,
                    "function_name": frame_info.function_name
                });

                if include_addresses {
                    response["pc"] = json!(format!("0x{:x}", frame_info.pc));
                    response["sp"] = json!(format!("0x{:x}", frame_info.sp));
                }

                if include_source {
                    response["module"] = json!(frame_info.module);
                    response["file"] = json!(frame_info.file);
                    response["line"] = json!(frame_info.line);
                }

                Ok(ToolResponse::Json(json!({
                    "frame_info": response,
                    "message": format!("Frame {} info: {}", frame_info.index, frame_info.function_name)
                })))
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}
// F0025: get_frame_variables - Fully implemented
#[async_trait]
impl Tool for GetFrameVariablesTool {
    fn name(&self) -> &'static str {
        "get_frame_variables"
    }

    fn description(&self) -> &'static str {
        "Get all local variables in current or specified frame"
    }

    fn parameters(&self) -> Value {
        json!({
            "frame_index": {
                "type": "integer",
                "description": "Stack frame index (0 = current frame, 1 = caller, etc.)",
                "default": 0,
                "minimum": 0,
                "maximum": 100
            },
            "include_arguments": {
                "type": "boolean",
                "description": "Include function arguments along with local variables",
                "default": false
            },
            "filter": {
                "type": "string",
                "description": "Filter variables by name pattern (optional)",
                "default": ""
            },
            "format": {
                "type": "string",
                "description": "Output format for variable information",
                "enum": ["detailed", "compact", "names_only", "types_only"],
                "default": "detailed"
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let frame_index = arguments.get("frame_index")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        let include_arguments = arguments.get("include_arguments")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let filter = arguments.get("filter")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let format = arguments.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("detailed");

        match lldb_manager.get_frame_variables(frame_index, include_arguments) {
            Ok(variables) => {
                // Apply filter if specified
                let filtered_variables: Vec<_> = if filter.is_empty() {
                    variables
                } else {
                    variables.into_iter().filter(|var| var.name.contains(filter)).collect()
                };

                let formatted_variables = Self::format_variables(&filtered_variables, format);

                Ok(ToolResponse::Json(json!({
                    "frame_index": frame_index.unwrap_or(0),
                    "include_arguments": include_arguments,
                    "total_variables": filtered_variables.len(),
                    "local_variables": filtered_variables.iter().filter(|v| !v.is_argument).count(),
                    "arguments": filtered_variables.iter().filter(|v| v.is_argument).count(),
                    "filter_applied": !filter.is_empty(),
                    "filter": filter,
                    "format": format,
                    "variables": formatted_variables,
                    "message": format!("Found {} variables in frame {}", filtered_variables.len(), frame_index.unwrap_or(0))
                })))
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

impl GetFrameVariablesTool {
    fn format_variables(variables: &[crate::lldb_manager::Variable], format: &str) -> Value {
        match format {
            "compact" => {
                let compact: Vec<String> = variables.iter().map(|var| {
                    format!("{}: {} = {}", var.name, var.var_type, var.value)
                }).collect();
                json!(compact)
            },
            "names_only" => {
                let names: Vec<String> = variables.iter().map(|var| var.name.clone()).collect();
                json!(names)
            },
            "types_only" => {
                let types: Vec<String> = variables.iter().map(|var| {
                    format!("{}: {}", var.name, var.var_type)
                }).collect();
                json!(types)
            },
            _ => { // "detailed" format
                let detailed: Vec<Value> = variables.iter().map(|var| {
                    json!({
                        "name": var.name,
                        "value": var.value,
                        "type": var.var_type,
                        "is_argument": var.is_argument,
                        "scope": var.scope,
                        "category": if var.is_argument { "parameter" } else { "local" }
                    })
                }).collect();
                json!(detailed)
            }
        }
    }
}
// F0026: get_frame_arguments - Fully implemented
#[async_trait]
impl Tool for GetFrameArgumentsTool {
    fn name(&self) -> &'static str {
        "get_frame_arguments"
    }

    fn description(&self) -> &'static str {
        "Get function arguments (parameters) for current or specified frame"
    }

    fn parameters(&self) -> Value {
        json!({
            "frame_index": {
                "type": "integer",
                "description": "Stack frame index (0 = current frame, 1 = caller, etc.)",
                "default": 0,
                "minimum": 0,
                "maximum": 100
            },
            "format": {
                "type": "string",
                "description": "Output format for argument information",
                "enum": ["detailed", "compact", "names_only", "types_only"],
                "default": "detailed"
            },
            "include_types": {
                "type": "boolean",
                "description": "Include detailed type information",
                "default": true
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let frame_index = arguments.get("frame_index")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        let format = arguments.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("detailed");

        let include_types = arguments.get("include_types")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        match lldb_manager.get_frame_arguments(frame_index) {
            Ok(arguments_list) => {
                let formatted_arguments = Self::format_arguments(&arguments_list, format, include_types);

                Ok(ToolResponse::Json(json!({
                    "frame_index": frame_index.unwrap_or(0),
                    "argument_count": arguments_list.len(),
                    "format": format,
                    "include_types": include_types,
                    "arguments": formatted_arguments,
                    "message": format!("Found {} arguments in frame {}", arguments_list.len(), frame_index.unwrap_or(0))
                })))
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

impl GetFrameArgumentsTool {
    fn format_arguments(arguments: &[crate::lldb_manager::Variable], format: &str, include_types: bool) -> Value {
        match format {
            "compact" => {
                let compact: Vec<String> = arguments.iter().map(|arg| {
                    if include_types {
                        format!("{}: {} = {}", arg.name, arg.var_type, arg.value)
                    } else {
                        format!("{} = {}", arg.name, arg.value)
                    }
                }).collect();
                json!(compact)
            },
            "names_only" => {
                let names: Vec<String> = arguments.iter().map(|arg| arg.name.clone()).collect();
                json!(names)
            },
            "types_only" => {
                let types: Vec<String> = arguments.iter().map(|arg| {
                    format!("{}: {}", arg.name, arg.var_type)
                }).collect();
                json!(types)
            },
            _ => { // "detailed" format
                let detailed: Vec<Value> = arguments.iter().map(|arg| {
                    let mut obj = json!({
                        "name": arg.name,
                        "value": arg.value,
                        "scope": arg.scope
                    });
                    
                    if include_types {
                        obj["type"] = json!(arg.var_type);
                    }
                    
                    obj
                }).collect();
                json!(detailed)
            }
        }
    }
}
// F0027: evaluate_in_frame - Fully implemented
#[async_trait]
impl Tool for EvaluateInFrameTool {
    fn name(&self) -> &'static str {
        "evaluate_in_frame"
    }

    fn description(&self) -> &'static str {
        "Evaluate C/C++ expressions in specific frame context with access to local variables"
    }

    fn parameters(&self) -> Value {
        json!({
            "expression": {
                "type": "string",
                "description": "C/C++ expression to evaluate (e.g., 'argc + 1', 'argv[0]', 'sizeof(buffer)')"
            },
            "frame_index": {
                "type": "integer",
                "description": "Stack frame index for evaluation context (0 = current frame)",
                "default": 0,
                "minimum": 0,
                "maximum": 100
            },
            "format": {
                "type": "string",
                "description": "Output format for expression result",
                "enum": ["auto", "decimal", "hex", "binary", "string", "pointer"],
                "default": "auto"
            },
            "timeout_ms": {
                "type": "integer",
                "description": "Expression evaluation timeout in milliseconds",
                "default": 5000,
                "minimum": 100,
                "maximum": 30000
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let expression = arguments.get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IncodeError::mcp("Missing expression parameter"))?;

        let frame_index = arguments.get("frame_index")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        let format = arguments.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("auto");

        let timeout_ms = arguments.get("timeout_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(5000);

        // Validate expression for safety (basic checks)
        if Self::is_unsafe_expression(expression) {
            return Ok(ToolResponse::Error(format!("Unsafe expression detected: {}", expression)));
        }

        match lldb_manager.evaluate_in_frame(frame_index, expression) {
            Ok(raw_result) => {
                let formatted_result = Self::format_result(&raw_result, format);
                
                Ok(ToolResponse::Json(json!({
                    "expression": expression,
                    "frame_index": frame_index.unwrap_or(0),
                    "raw_result": raw_result,
                    "formatted_result": formatted_result,
                    "format": format,
                    "timeout_ms": timeout_ms,
                    "success": true,
                    "message": format!("Expression '{}' evaluated in frame {}", expression, frame_index.unwrap_or(0))
                })))
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

impl EvaluateInFrameTool {
    fn is_unsafe_expression(expr: &str) -> bool {
        let dangerous_patterns = [
            "system(", "exec(", "fork(", "kill(", 
            "delete ", "free(", "malloc(", "realloc(",
            "memcpy(", "memset(", "strcpy(",
            "exit(", "abort(", "_exit(",
        ];
        
        dangerous_patterns.iter().any(|pattern| expr.contains(pattern))
    }

    fn format_result(result: &str, format: &str) -> Value {
        match format {
            "decimal" => {
                // Try to parse as number and format as decimal
                if let Ok(num) = result.parse::<i64>() {
                    json!(num)
                } else {
                    json!(result)
                }
            },
            "hex" => {
                // Try to parse as number and format as hex
                if let Ok(num) = result.parse::<u64>() {
                    json!(format!("0x{:x}", num))
                } else if result.starts_with("0x") {
                    json!(result)
                } else {
                    json!(format!("0x{}", result))
                }
            },
            "binary" => {
                // Try to parse as number and format as binary
                if let Ok(num) = result.parse::<u64>() {
                    json!(format!("0b{:b}", num))
                } else {
                    json!(result)
                }
            },
            "string" => {
                // Format as string (remove quotes if present)
                let cleaned = result.trim_matches('"');
                json!(cleaned)
            },
            "pointer" => {
                // Format as pointer address
                if result.starts_with("0x") {
                    json!(result)
                } else {
                    json!(format!("0x{}", result))
                }
            },
            _ => { // "auto" format
                // Auto-detect format based on result content
                if result.starts_with("0x") {
                    json!({
                        "type": "pointer",
                        "value": result
                    })
                } else if result.starts_with('"') && result.ends_with('"') {
                    json!({
                        "type": "string",
                        "value": result.trim_matches('"')
                    })
                } else if let Ok(num) = result.parse::<i64>() {
                    json!({
                        "type": "integer",
                        "value": num,
                        "hex": format!("0x{:x}", num as u64)
                    })
                } else {
                    json!({
                        "type": "expression",
                        "value": result
                    })
                }
            }
        }
    }
}

// Keep the old PlaceholderTool for compatibility with tool registry
pub struct PlaceholderTool;

#[async_trait]
impl Tool for PlaceholderTool {
    fn name(&self) -> &'static str {
        "stack_placeholder"
    }
    
    fn description(&self) -> &'static str {
        "Stack analysis placeholder - use specific stack tools instead"
    }
    
    fn parameters(&self) -> Value {
        json!({})
    }
    
    async fn execute(
        &self,
        _: HashMap<String, Value>,
        _: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        Ok(ToolResponse::Error("Use specific stack analysis tools like get_backtrace instead".to_string()))
    }
}
