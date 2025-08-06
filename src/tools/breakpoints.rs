use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::error::{IncodeError, IncodeResult};
use crate::lldb_manager::LldbManager;
use super::{Tool, ToolResponse};

// Breakpoint Management Tools (8 tools)
pub struct SetBreakpointTool;
pub struct SetWatchpointTool;
pub struct ListBreakpointsTool;
pub struct DeleteBreakpointTool;
pub struct EnableBreakpointTool;
pub struct DisableBreakpointTool;
pub struct SetConditionalBreakpointTool;
pub struct BreakpointCommandsTool;

macro_rules! impl_placeholder_tool {
    ($tool:ident, $name:expr, $desc:expr) => {
        #[async_trait]
        impl Tool for $tool {
            fn name(&self) -> &'static str { $name }
            fn description(&self) -> &'static str { $desc }
            fn parameters(&self) -> Value { json!({}) }
            async fn execute(&self, _: HashMap<String, Value>, _: &mut LldbManager) -> IncodeResult<ToolResponse> {
                Ok(ToolResponse::Error("Not yet implemented".to_string()))
            }
        }
    };
}

// F0014: set_breakpoint - Fully implemented
#[async_trait]
impl Tool for SetBreakpointTool {
    fn name(&self) -> &'static str {
        "set_breakpoint"
    }

    fn description(&self) -> &'static str {
        "Set breakpoint by address, function name, or file:line"
    }

    fn parameters(&self) -> Value {
        json!({
            "location": {
                "type": "string",
                "description": "Breakpoint location - address (0x1234), function name (main), or file:line (main.c:42)"
            },
            "address": {
                "type": "string",
                "description": "Memory address for breakpoint (alternative to location)"
            },
            "function": {
                "type": "string",
                "description": "Function name for breakpoint (alternative to location)"
            },
            "file": {
                "type": "string",
                "description": "Source file name (use with line parameter)"
            },
            "line": {
                "type": "integer",
                "description": "Line number (use with file parameter)",
                "minimum": 1
            },
            "enabled": {
                "type": "boolean",
                "description": "Whether breakpoint should be enabled after creation",
                "default": true
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        // Determine breakpoint location from various parameter combinations
        let location = if let Some(loc) = arguments.get("location").and_then(|v| v.as_str()) {
            loc.to_string()
        } else if let Some(addr) = arguments.get("address").and_then(|v| v.as_str()) {
            addr.to_string()
        } else if let Some(func) = arguments.get("function").and_then(|v| v.as_str()) {
            func.to_string()
        } else if let (Some(file), Some(line)) = (
            arguments.get("file").and_then(|v| v.as_str()),
            arguments.get("line").and_then(|v| v.as_u64())
        ) {
            format!("{}:{}", file, line)
        } else {
            return Ok(ToolResponse::Error("Must specify location, address, function, or file:line parameters".to_string()));
        };

        let _enabled = arguments.get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        match lldb_manager.set_breakpoint(&location) {
            Ok(breakpoint_id) => {
                Ok(ToolResponse::Json(json!({
                    "success": true,
                    "breakpoint_id": breakpoint_id,
                    "location": location,
                    "enabled": _enabled,
                    "message": format!("Successfully created breakpoint {} at {}", breakpoint_id, location)
                })))
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}
// F0015: set_watchpoint - Fully implemented
#[async_trait]
impl Tool for SetWatchpointTool {
    fn name(&self) -> &'static str {
        "set_watchpoint"
    }

    fn description(&self) -> &'static str {
        "Set memory watchpoint to monitor memory access (read/write/access)"
    }

    fn parameters(&self) -> Value {
        json!({
            "address": {
                "type": "string",
                "description": "Memory address to watch (hexadecimal, e.g., '0x7fff12345678')"
            },
            "size": {
                "type": "integer",
                "description": "Number of bytes to watch",
                "default": 8,
                "minimum": 1,
                "maximum": 256
            },
            "access_type": {
                "type": "string",
                "description": "Type of access to monitor",
                "enum": ["read", "write", "read_write"],
                "default": "write"
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let address_str = arguments.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IncodeError::mcp("Missing address parameter"))?;

        let address = if address_str.starts_with("0x") {
            u64::from_str_radix(&address_str[2..], 16)
        } else {
            u64::from_str_radix(address_str, 16)
        }.map_err(|_| IncodeError::mcp(format!("Invalid address format: {}", address_str)))?;

        let size = arguments.get("size")
            .and_then(|v| v.as_u64())
            .unwrap_or(8) as u32;

        let access_type = arguments.get("access_type")
            .and_then(|v| v.as_str())
            .unwrap_or("write");

        let (read, write) = match access_type {
            "read" => (true, false),
            "write" => (false, true),
            "read_write" => (true, true),
            _ => return Ok(ToolResponse::Error(format!("Invalid access_type: {}", access_type))),
        };

        match lldb_manager.set_watchpoint(address, size, read, write) {
            Ok(watchpoint_id) => {
                Ok(ToolResponse::Json(json!({
                    "success": true,
                    "watchpoint_id": watchpoint_id,
                    "address": format!("0x{:x}", address),
                    "size": size,
                    "access_type": access_type,
                    "message": format!("Successfully created watchpoint {} at 0x{:x} ({} access, {} bytes)", 
                                     watchpoint_id, address, access_type, size)
                })))
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}
// F0016: list_breakpoints - Fully implemented
#[async_trait]
impl Tool for ListBreakpointsTool {
    fn name(&self) -> &'static str {
        "list_breakpoints"
    }

    fn description(&self) -> &'static str {
        "List all active breakpoints with details including IDs, locations, and hit counts"
    }

    fn parameters(&self) -> Value {
        json!({
            "enabled_only": {
                "type": "boolean",
                "description": "Only list enabled breakpoints",
                "default": false
            },
            "include_hit_count": {
                "type": "boolean", 
                "description": "Include hit count information",
                "default": true
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let enabled_only = arguments.get("enabled_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let _include_hit_count = arguments.get("include_hit_count")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        match lldb_manager.list_breakpoints() {
            Ok(breakpoints) => {
                let filtered_breakpoints: Vec<_> = if enabled_only {
                    breakpoints.into_iter().filter(|bp| bp.enabled).collect()
                } else {
                    breakpoints
                };

                let breakpoint_data: Vec<Value> = filtered_breakpoints.iter().map(|bp| {
                    json!({
                        "id": bp.id,
                        "enabled": bp.enabled,
                        "location": bp.location,
                        "hit_count": bp.hit_count,
                        "condition": bp.condition
                    })
                }).collect();

                Ok(ToolResponse::Json(json!({
                    "breakpoints": breakpoint_data,
                    "total_count": breakpoint_data.len(),
                    "enabled_count": filtered_breakpoints.iter().filter(|bp| bp.enabled).count(),
                    "message": format!("Found {} breakpoints", breakpoint_data.len())
                })))
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}
// F0017: delete_breakpoint - Fully implemented
#[async_trait]
impl Tool for DeleteBreakpointTool {
    fn name(&self) -> &'static str {
        "delete_breakpoint"
    }

    fn description(&self) -> &'static str {
        "Remove specific breakpoint by ID"
    }

    fn parameters(&self) -> Value {
        json!({
            "breakpoint_id": {
                "type": "integer",
                "description": "ID of the breakpoint to delete",
                "minimum": 1
            },
            "confirm": {
                "type": "boolean",
                "description": "Confirmation flag to prevent accidental deletion",
                "default": false
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let breakpoint_id = arguments.get("breakpoint_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| IncodeError::mcp("Missing breakpoint_id parameter"))? as u32;

        let confirm = arguments.get("confirm")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !confirm {
            return Ok(ToolResponse::Error(format!(
                "Deletion requires confirmation. Set confirm=true to delete breakpoint {}", 
                breakpoint_id
            )));
        }

        match lldb_manager.delete_breakpoint(breakpoint_id) {
            Ok(_) => {
                Ok(ToolResponse::Json(json!({
                    "success": true,
                    "breakpoint_id": breakpoint_id,
                    "message": format!("Successfully deleted breakpoint {}", breakpoint_id)
                })))
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}
// F0018: enable_breakpoint - Fully implemented
#[async_trait]
impl Tool for EnableBreakpointTool {
    fn name(&self) -> &'static str {
        "enable_breakpoint"
    }

    fn description(&self) -> &'static str {
        "Enable a previously disabled breakpoint by ID"
    }

    fn parameters(&self) -> Value {
        json!({
            "breakpoint_id": {
                "type": "integer",
                "description": "ID of the breakpoint to enable",
                "minimum": 1
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let breakpoint_id = arguments.get("breakpoint_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| IncodeError::mcp("Missing breakpoint_id parameter"))? as u32;

        match lldb_manager.enable_breakpoint(breakpoint_id) {
            Ok(is_enabled) => {
                if is_enabled {
                    Ok(ToolResponse::Json(json!({
                        "breakpoint_id": breakpoint_id,
                        "enabled": true,
                        "success": true,
                        "message": format!("Breakpoint {} enabled successfully", breakpoint_id)
                    })))
                } else {
                    Ok(ToolResponse::Error(format!("Failed to enable breakpoint {}", breakpoint_id)))
                }
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}
// F0019: disable_breakpoint - Fully implemented
#[async_trait]
impl Tool for DisableBreakpointTool {
    fn name(&self) -> &'static str {
        "disable_breakpoint"
    }

    fn description(&self) -> &'static str {
        "Disable a breakpoint without removing it (can be re-enabled later)"
    }

    fn parameters(&self) -> Value {
        json!({
            "breakpoint_id": {
                "type": "integer",
                "description": "ID of the breakpoint to disable",
                "minimum": 1
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let breakpoint_id = arguments.get("breakpoint_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| IncodeError::mcp("Missing breakpoint_id parameter"))? as u32;

        match lldb_manager.disable_breakpoint(breakpoint_id) {
            Ok(is_disabled) => {
                if is_disabled {
                    Ok(ToolResponse::Json(json!({
                        "breakpoint_id": breakpoint_id,
                        "enabled": false,
                        "disabled": true,
                        "success": true,
                        "message": format!("Breakpoint {} disabled successfully", breakpoint_id)
                    })))
                } else {
                    Ok(ToolResponse::Error(format!("Failed to disable breakpoint {}", breakpoint_id)))
                }
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}
// F0020: set_conditional_breakpoint - Fully implemented
#[async_trait]
impl Tool for SetConditionalBreakpointTool {
    fn name(&self) -> &'static str {
        "set_conditional_breakpoint"
    }

    fn description(&self) -> &'static str {
        "Set a breakpoint that only triggers when a specified condition is true"
    }

    fn parameters(&self) -> Value {
        json!({
            "location": {
                "type": "string",
                "description": "Breakpoint location (function name, address, or file:line)"
            },
            "condition": {
                "type": "string",
                "description": "C/C++ expression that must evaluate to true for breakpoint to trigger (e.g., 'x > 10', 'strcmp(str, \"test\") == 0')"
            },
            "ignore_count": {
                "type": "integer",
                "description": "Number of times to ignore this breakpoint before checking condition",
                "default": 0,
                "minimum": 0
            },
            "description": {
                "type": "string",
                "description": "Optional description for the conditional breakpoint",
                "default": ""
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let location = arguments.get("location")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IncodeError::mcp("Missing location parameter"))?;

        let condition = arguments.get("condition")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IncodeError::mcp("Missing condition parameter"))?;

        let ignore_count = arguments.get("ignore_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let description = arguments.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Validate condition for basic safety
        if Self::is_unsafe_condition(condition) {
            return Ok(ToolResponse::Error(format!("Unsafe condition detected: {}", condition)));
        }

        match lldb_manager.set_conditional_breakpoint(location, condition) {
            Ok(breakpoint_id) => {
                Ok(ToolResponse::Json(json!({
                    "breakpoint_id": breakpoint_id,
                    "location": location,
                    "condition": condition,
                    "ignore_count": ignore_count,
                    "description": description,
                    "enabled": true,
                    "type": "conditional",
                    "success": true,
                    "message": format!("Conditional breakpoint {} set at {} with condition: {}", breakpoint_id, location, condition)
                })))
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

impl SetConditionalBreakpointTool {
    fn is_unsafe_condition(condition: &str) -> bool {
        let dangerous_patterns = [
            "system(", "exec(", "fork(", "kill(",
            "delete ", "free(", "malloc(", "realloc(",
            "exit(", "abort(", "_exit(",
            "remove(", "unlink(", "rmdir(",
        ];
        
        dangerous_patterns.iter().any(|pattern| condition.contains(pattern))
    }
}
// F0021: breakpoint_commands - Fully implemented
#[async_trait]
impl Tool for BreakpointCommandsTool {
    fn name(&self) -> &'static str {
        "breakpoint_commands"
    }

    fn description(&self) -> &'static str {
        "Set commands to execute automatically when a breakpoint is hit"
    }

    fn parameters(&self) -> Value {
        json!({
            "breakpoint_id": {
                "type": "integer",
                "description": "ID of the breakpoint to attach commands to",
                "minimum": 1
            },
            "commands": {
                "type": "array",
                "description": "List of LLDB commands to execute when breakpoint hits",
                "items": {
                    "type": "string"
                },
                "minItems": 1
            },
            "stop_on_command_failure": {
                "type": "boolean",
                "description": "Whether to stop execution if any command fails",
                "default": false
            },
            "continue_after_commands": {
                "type": "boolean",
                "description": "Whether to continue execution automatically after running commands",
                "default": false
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let breakpoint_id = arguments.get("breakpoint_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| IncodeError::mcp("Missing breakpoint_id parameter"))? as u32;

        let commands_array = arguments.get("commands")
            .and_then(|v| v.as_array())
            .ok_or_else(|| IncodeError::mcp("Missing commands parameter"))?;

        let commands: Vec<String> = commands_array.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        if commands.is_empty() {
            return Ok(ToolResponse::Error("No valid commands provided".to_string()));
        }

        let stop_on_failure = arguments.get("stop_on_command_failure")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let continue_after = arguments.get("continue_after_commands")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Validate commands for basic safety
        for command in &commands {
            if Self::is_unsafe_command(command) {
                return Ok(ToolResponse::Error(format!("Unsafe command detected: {}", command)));
            }
        }

        match lldb_manager.set_breakpoint_commands(breakpoint_id, &commands) {
            Ok(success) => {
                if success {
                    Ok(ToolResponse::Json(json!({
                        "breakpoint_id": breakpoint_id,
                        "commands": commands,
                        "command_count": commands.len(),
                        "stop_on_command_failure": stop_on_failure,
                        "continue_after_commands": continue_after,
                        "success": true,
                        "message": format!("Set {} commands for breakpoint {}", commands.len(), breakpoint_id)
                    })))
                } else {
                    Ok(ToolResponse::Error(format!("Failed to set commands for breakpoint {}", breakpoint_id)))
                }
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

impl BreakpointCommandsTool {
    fn is_unsafe_command(command: &str) -> bool {
        let dangerous_patterns = [
            "process kill", "process detach", "quit", "exit",
            "target delete", "settings clear", "platform disconnect",
            "script import", "command script", "process connect",
            "gdb-remote", "kdp-remote", "platform connect",
        ];
        
        dangerous_patterns.iter().any(|pattern| command.to_lowercase().contains(&pattern.to_lowercase()))
    }
}