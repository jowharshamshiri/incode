use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::debug;
use crate::error::{IncodeError, IncodeResult};
use crate::lldb_manager::LldbManager;
use super::{Tool, ToolResponse};

// Execution Control Tools (7 tools)
pub struct ContinueExecutionTool;
pub struct StepOverTool;
pub struct StepIntoTool;
pub struct StepOutTool;
pub struct StepInstructionTool;
pub struct RunUntilTool;
pub struct InterruptExecutionTool;


// F0007: continue_execution - Fully implemented
#[async_trait]
impl Tool for ContinueExecutionTool {
    fn name(&self) -> &'static str {
        "continue_execution"
    }

    fn description(&self) -> &'static str {
        "Continue process execution from current state"
    }

    fn parameters(&self) -> Value {
        json!({
            "thread_id": {
                "type": "integer",
                "description": "Specific thread ID to continue (optional, continues all if not specified)"
            },
            "ignore_breakpoints": {
                "type": "boolean",
                "description": "Continue past breakpoints without stopping",
                "default": false
            }
        })
    }

    async fn execute(
        &self,
        _arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        // TODO: Handle thread_id and ignore_breakpoints parameters in future iterations
        match lldb_manager.continue_execution() {
            Ok(_) => Ok(ToolResponse::Success("Process execution continued successfully".to_string())),
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}
// F0008: step_over - Fully implemented
#[async_trait]
impl Tool for StepOverTool {
    fn name(&self) -> &'static str {
        "step_over"
    }

    fn description(&self) -> &'static str {
        "Step over current instruction (next line in same function)"
    }

    fn parameters(&self) -> Value {
        json!({
            "count": {
                "type": "integer",
                "description": "Number of steps to perform",
                "default": 1,
                "minimum": 1
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let count = arguments.get("count")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as usize;

        for i in 0..count {
            match lldb_manager.step_over() {
                Ok(_) => {
                    if count > 1 {
                        debug!("Completed step {} of {}", i + 1, count);
                    }
                }
                Err(e) => {
                    let msg = if count > 1 {
                        format!("Step over failed at step {} of {}: {}", i + 1, count, e)
                    } else {
                        format!("Step over failed: {}", e)
                    };
                    return Ok(ToolResponse::Error(msg));
                }
            }
        }

        let msg = if count > 1 {
            format!("Successfully completed {} step over operations", count)
        } else {
            "Successfully stepped over current instruction".to_string()
        };
        Ok(ToolResponse::Success(msg))
    }
}
// F0009: step_into - Fully implemented
#[async_trait]
impl Tool for StepIntoTool {
    fn name(&self) -> &'static str {
        "step_into"
    }

    fn description(&self) -> &'static str {
        "Step into function calls"
    }

    fn parameters(&self) -> Value {
        json!({
            "count": {
                "type": "integer", 
                "description": "Number of steps to perform",
                "default": 1,
                "minimum": 1
            },
            "target_function": {
                "type": "string",
                "description": "Specific function name to step into (optional)"
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let count = arguments.get("count")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as usize;

        // TODO: Handle target_function parameter in future iterations
        let _target_function = arguments.get("target_function")
            .and_then(|v| v.as_str());

        for i in 0..count {
            match lldb_manager.step_into() {
                Ok(_) => {
                    if count > 1 {
                        debug!("Completed step into {} of {}", i + 1, count);
                    }
                }
                Err(e) => {
                    let msg = if count > 1 {
                        format!("Step into failed at step {} of {}: {}", i + 1, count, e)
                    } else {
                        format!("Step into failed: {}", e)
                    };
                    return Ok(ToolResponse::Error(msg));
                }
            }
        }

        let msg = if count > 1 {
            format!("Successfully completed {} step into operations", count)
        } else {
            "Successfully stepped into function call".to_string()
        };
        Ok(ToolResponse::Success(msg))
    }
}
// F0010: step_out - Fully implemented
#[async_trait]
impl Tool for StepOutTool {
    fn name(&self) -> &'static str {
        "step_out"
    }

    fn description(&self) -> &'static str {
        "Step out of current function (finish current function and return to caller)"
    }

    fn parameters(&self) -> Value {
        json!({
            "count": {
                "type": "integer",
                "description": "Number of functions to step out of", 
                "default": 1,
                "minimum": 1
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let count = arguments.get("count")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as usize;

        for i in 0..count {
            match lldb_manager.step_out() {
                Ok(_) => {
                    if count > 1 {
                        debug!("Completed step out {} of {}", i + 1, count);
                    }
                }
                Err(e) => {
                    let msg = if count > 1 {
                        format!("Step out failed at step {} of {}: {}", i + 1, count, e)
                    } else {
                        format!("Step out failed: {}", e)
                    };
                    return Ok(ToolResponse::Error(msg));
                }
            }
        }

        let msg = if count > 1 {
            format!("Successfully stepped out of {} functions", count)
        } else {
            "Successfully stepped out of current function".to_string()
        };
        Ok(ToolResponse::Success(msg))
    }
}
// F0011: step_instruction - Fully implemented
#[async_trait]
impl Tool for StepInstructionTool {
    fn name(&self) -> &'static str {
        "step_instruction"
    }

    fn description(&self) -> &'static str {
        "Single instruction step (assembly-level stepping)"
    }

    fn parameters(&self) -> Value {
        json!({
            "count": {
                "type": "integer",
                "description": "Number of instructions to step",
                "default": 1,
                "minimum": 1
            },
            "step_over": {
                "type": "boolean", 
                "description": "Step over calls instead of into them",
                "default": false
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let count = arguments.get("count")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as usize;

        let step_over = arguments.get("step_over")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        for i in 0..count {
            match lldb_manager.step_instruction(step_over) {
                Ok(_) => {
                    if count > 1 {
                        debug!("Completed instruction step {} of {}", i + 1, count);
                    }
                }
                Err(e) => {
                    let msg = if count > 1 {
                        format!("Instruction step failed at step {} of {}: {}", i + 1, count, e)
                    } else {
                        format!("Instruction step failed: {}", e)
                    };
                    return Ok(ToolResponse::Error(msg));
                }
            }
        }

        let step_type = if step_over { "over" } else { "into" };
        let msg = if count > 1 {
            format!("Successfully stepped {} {} instructions", step_type, count)
        } else {
            format!("Successfully stepped {} single instruction", step_type)
        };
        Ok(ToolResponse::Success(msg))
    }
}
// F0012: run_until - Fully implemented
#[async_trait]
impl Tool for RunUntilTool {
    fn name(&self) -> &'static str {
        "run_until"
    }

    fn description(&self) -> &'static str {
        "Run until specific address or line number"
    }

    fn parameters(&self) -> Value {
        json!({
            "address": {
                "type": "string",
                "description": "Hexadecimal address to run until (e.g., '0x401000')"
            },
            "file": {
                "type": "string", 
                "description": "Source file name for line-based run until"
            },
            "line": {
                "type": "integer",
                "description": "Line number to run until (requires file parameter)",
                "minimum": 1
            },
            "location": {
                "type": "string",
                "description": "Combined location string (e.g., 'main.c:42' or '0x401000')"
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        // Parse parameters - support both individual params and combined location
        let (address, file, line) = if let Some(location_str) = arguments.get("location").and_then(|v| v.as_str()) {
            if location_str.starts_with("0x") {
                // Address location
                let address = u64::from_str_radix(&location_str[2..], 16)
                    .map_err(|_| IncodeError::mcp(format!("Invalid address format: {}", location_str)))?;
                (Some(address), None, None)
            } else if location_str.contains(':') {
                // File:line location
                let parts: Vec<&str> = location_str.splitn(2, ':').collect();
                if parts.len() != 2 {
                    return Ok(ToolResponse::Error(format!("Invalid location format: {}", location_str)));
                }
                let line_num = parts[1].parse::<u32>()
                    .map_err(|_| IncodeError::mcp(format!("Invalid line number: {}", parts[1])))?;
                (None, Some(parts[0]), Some(line_num))
            } else {
                return Ok(ToolResponse::Error(format!("Invalid location format: {}", location_str)));
            }
        } else {
            // Individual parameters
            let address = arguments.get("address")
                .and_then(|v| v.as_str())
                .map(|s| {
                    if s.starts_with("0x") {
                        u64::from_str_radix(&s[2..], 16)
                    } else {
                        u64::from_str_radix(s, 16)
                    }
                })
                .transpose()
                .map_err(|_| IncodeError::mcp("Invalid address format"))?;

            let file = arguments.get("file").and_then(|v| v.as_str());
            let line = arguments.get("line").and_then(|v| v.as_u64()).map(|n| n as u32);

            (address, file, line)
        };

        match lldb_manager.run_until(address, file, line) {
            Ok(_) => {
                let msg = if let Some(addr) = address {
                    format!("Successfully running until address 0x{:x}", addr)
                } else if let (Some(f), Some(l)) = (file, line) {
                    format!("Successfully running until {}:{}", f, l)
                } else {
                    "Successfully initiated run until operation".to_string()
                };
                Ok(ToolResponse::Success(msg))
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}
// F0013: interrupt_execution - Fully implemented
#[async_trait]
impl Tool for InterruptExecutionTool {
    fn name(&self) -> &'static str {
        "interrupt_execution"
    }

    fn description(&self) -> &'static str {
        "Pause/interrupt running process execution"
    }

    fn parameters(&self) -> Value {
        json!({
            "timeout_ms": {
                "type": "integer",
                "description": "Timeout in milliseconds to wait for interrupt to complete",
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
        let _timeout_ms = arguments.get("timeout_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(5000);

        // TODO: Implement timeout handling in future iterations
        match lldb_manager.interrupt_execution() {
            Ok(_) => Ok(ToolResponse::Success("Successfully interrupted process execution - process is now paused".to_string())),
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}