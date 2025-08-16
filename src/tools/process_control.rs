use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::error::{IncodeError, IncodeResult};
use crate::lldb_manager::LldbManager;
use super::{Tool, ToolResponse};

// F0001: launch_process
pub struct LaunchProcessTool;

#[async_trait]
impl Tool for LaunchProcessTool {
    fn name(&self) -> &'static str {
        "launch_process"
    }

    fn description(&self) -> &'static str {
        "Launch executable with arguments, environment variables, and working directory"
    }

    fn parameters(&self) -> Value {
        json!({
            "executable": {
                "type": "string",
                "description": "Path to executable to launch"
            },
            "args": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Command line arguments"
            },
            "env": {
                "type": "object",
                "description": "Environment variables"
            },
            "working_dir": {
                "type": "string",
                "description": "Working directory for the process"
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let executable = arguments.get("executable")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IncodeError::mcp("Missing executable parameter"))?;

        let args: Vec<String> = arguments.get("args")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        let env: HashMap<String, String> = arguments.get("env")
            .and_then(|v| v.as_object())
            .map(|obj| obj.iter().filter_map(|(k, v)| {
                v.as_str().map(|s| (k.clone(), s.to_string()))
            }).collect())
            .unwrap_or_default();

        let working_dir = arguments.get("working_dir")
            .and_then(|v| v.as_str());

        // Handle working directory by changing current directory if specified
        let original_dir = if let Some(wd) = working_dir {
            let current = std::env::current_dir().ok();
            if let Err(e) = std::env::set_current_dir(wd) {
                return Ok(ToolResponse::Error(format!("Failed to change to working directory {}: {}", wd, e)));
            }
            current
        } else {
            None
        };

        // Launch the process
        let result = lldb_manager.launch_process(executable, &args, &env);

        // Restore original directory if we changed it
        if let Some(orig_dir) = original_dir {
            if let Err(e) = std::env::set_current_dir(&orig_dir) {
                eprintln!("Warning: Failed to restore original directory: {}", e);
            }
        }

        match result {
            Ok(pid) => {
                // Get initial console output from the launched process
                let console_output = lldb_manager.get_console_output()
                    .unwrap_or_else(|_| "No console output available yet".to_string());
                
                Ok(ToolResponse::Json(json!({
                    "success": true,
                    "pid": pid,
                    "executable": executable,
                    "working_dir": working_dir,
                    "message": format!("Successfully launched process {} with PID {} - process running independently", executable, pid),
                    "console_output": console_output,
                    "note": "Process is running independently. Use get_console_output to read more output."
                })))
            },
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

// Console output tool for reading from running process
pub struct GetConsoleOutputTool;

#[async_trait]
impl Tool for GetConsoleOutputTool {
    fn name(&self) -> &'static str {
        "get_console_output"
    }

    fn description(&self) -> &'static str {
        "Get current console output (stdout/stderr) from the running debugged process"
    }

    fn parameters(&self) -> Value {
        json!({})
    }

    async fn execute(
        &self,
        _arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        match lldb_manager.get_console_output() {
            Ok(output) => Ok(ToolResponse::Json(json!({
                "success": true,
                "console_output": output,
                "message": "Retrieved console output from running process"
            }))),
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

// F0002: attach_to_process
pub struct AttachToProcessTool;

#[async_trait]
impl Tool for AttachToProcessTool {
    fn name(&self) -> &'static str {
        "attach_to_process"
    }

    fn description(&self) -> &'static str {
        "Attach to running process by PID or name"
    }

    fn parameters(&self) -> Value {
        json!({
            "pid": {
                "type": "integer", 
                "description": "Process ID to attach to"
            },
            "name": {
                "type": "string",
                "description": "Process name to attach to (alternative to PID)"
            },
            "wait": {
                "type": "boolean",
                "description": "Wait for process to launch if it doesn't exist",
                "default": false
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        if let Some(pid_val) = arguments.get("pid") {
            let pid = pid_val.as_u64()
                .ok_or_else(|| IncodeError::mcp("Invalid PID value"))? as u32;

            match lldb_manager.attach_to_process(pid) {
                Ok(_) => Ok(ToolResponse::Json(json!({
                    "success": true,
                    "pid": pid,
                    "message": format!("Successfully attached to process {}", pid)
                }))),
                Err(e) => Ok(ToolResponse::Error(e.to_string())),
            }
        } else {
            Ok(ToolResponse::Error("Either pid or name parameter required".to_string()))
        }
    }
}

// F0003: detach_process
pub struct DetachProcessTool;

#[async_trait]
impl Tool for DetachProcessTool {
    fn name(&self) -> &'static str {
        "detach_process"
    }

    fn description(&self) -> &'static str {
        "Safely detach from current debugging target"
    }

    fn parameters(&self) -> Value {
        json!({
            "keep_stopped": {
                "type": "boolean",
                "description": "Keep process stopped after detaching",
                "default": false
            }
        })
    }

    async fn execute(
        &self,
        _arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        match lldb_manager.detach_process() {
            Ok(_) => Ok(ToolResponse::Success("Successfully detached from process".to_string())),
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

// F0004: kill_process
pub struct KillProcessTool;

#[async_trait]
impl Tool for KillProcessTool {
    fn name(&self) -> &'static str {
        "kill_process"
    }

    fn description(&self) -> &'static str {
        "Terminate debugging target process"
    }

    fn parameters(&self) -> Value {
        json!({
            "signal": {
                "type": "string",
                "description": "Signal to send (SIGTERM, SIGKILL, etc.)",
                "default": "SIGTERM"
            }
        })
    }

    async fn execute(
        &self,
        _arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        match lldb_manager.kill_process() {
            Ok(_) => Ok(ToolResponse::Success("Successfully killed process".to_string())),
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

// F0005: get_process_info
pub struct GetProcessInfoTool;

#[async_trait]
impl Tool for GetProcessInfoTool {
    fn name(&self) -> &'static str {
        "get_process_info"
    }

    fn description(&self) -> &'static str {
        "Get process PID, executable path, state, memory usage"
    }

    fn parameters(&self) -> Value {
        json!({})
    }

    async fn execute(
        &self,
        _arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        match lldb_manager.get_process_info() {
            Ok(info) => Ok(ToolResponse::Json(json!({
                "pid": info.pid,
                "state": info.state,
                "executable_path": info.executable_path,
                "memory_usage": info.memory_usage
            }))),
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

// F0006: list_processes
pub struct ListProcessesTool;

#[async_trait]
impl Tool for ListProcessesTool {
    fn name(&self) -> &'static str {
        "list_processes"
    }

    fn description(&self) -> &'static str {
        "List all debuggable processes on system"
    }

    fn parameters(&self) -> Value {
        json!({
            "filter": {
                "type": "string",
                "description": "Filter processes by name pattern"
            },
            "include_system": {
                "type": "boolean",
                "description": "Include system processes",
                "default": false
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let filter = arguments.get("filter")
            .and_then(|v| v.as_str());

        let include_system = arguments.get("include_system")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        match lldb_manager.list_processes(filter, include_system) {
            Ok(processes) => {
                let process_data: Vec<Value> = processes.into_iter().map(|p| json!({
                    "pid": p.pid,
                    "state": p.state,
                    "executable_path": p.executable_path,
                    "memory_usage": p.memory_usage
                })).collect();

                Ok(ToolResponse::Json(json!({
                    "processes": process_data,
                    "count": process_data.len()
                })))
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}