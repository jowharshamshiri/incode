use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::error::IncodeResult;
use crate::lldb_manager::LldbManager;
use super::{Tool, ToolResponse};

/// Execute raw LLDB command and return output
pub struct ExecuteCommandTool;

#[async_trait]
impl Tool for ExecuteCommandTool {
    fn name(&self) -> &'static str {
        "execute_command"
    }
    
    fn description(&self) -> &'static str {
        "Execute raw LLDB command and return output"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "LLDB command to execute (e.g., 'frame info', 'bt', 'p variable_name')"
                },
                "timeout": {
                    "type": "number",
                    "description": "Command timeout in seconds (optional, default: 30)",
                    "minimum": 1,
                    "maximum": 300
                }
            },
            "required": ["command"]
        })
    }
    
    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let command = arguments.get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::IncodeError::invalid_parameter("command required"))?;

        let timeout = arguments.get("timeout")
            .and_then(|v| v.as_f64())
            .unwrap_or(30.0) as u32;

        if command.trim().is_empty() {
            return Err(crate::error::IncodeError::invalid_parameter("command cannot be empty"));
        }

        // Safety validation - block potentially dangerous commands
        let dangerous_commands = ["quit", "exit", "kill", "detach", "attach"];
        for dangerous in &dangerous_commands {
            if command.trim().to_lowercase().starts_with(dangerous) {
                return Err(crate::error::IncodeError::invalid_parameter(
                    "dangerous command blocked - use specific tools instead"
                ));
            }
        }

        let output = lldb_manager.execute_command(command)?;
        
        Ok(ToolResponse::Success(json!({
            "command": command,
            "output": output,
            "timeout": timeout,
            "status": "executed"
        }).to_string()))
    }
}

/// Get LLDB version and build information
pub struct GetLldbVersionTool;

#[async_trait]
impl Tool for GetLldbVersionTool {
    fn name(&self) -> &'static str {
        "get_lldb_version"
    }
    
    fn description(&self) -> &'static str {
        "Get LLDB version and build information"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "include_build_info": {
                    "type": "boolean",
                    "description": "Include detailed build information (optional, default: false)"
                }
            }
        })
    }
    
    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let include_build_info = arguments.get("include_build_info")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let version_info = lldb_manager.get_lldb_version(include_build_info)?;
        
        Ok(ToolResponse::Success(json!({
            "version": version_info.version,
            "build_number": version_info.build_number,
            "api_version": version_info.api_version,
            "build_date": version_info.build_date,
            "build_configuration": version_info.build_configuration,
            "compiler": version_info.compiler,
            "platform": version_info.platform,
            "include_build_info": include_build_info
        }).to_string()))
    }
}

/// Configure LLDB settings
pub struct SetLldbSettingsTool;

#[async_trait]
impl Tool for SetLldbSettingsTool {
    fn name(&self) -> &'static str {
        "set_lldb_settings"
    }
    
    fn description(&self) -> &'static str {
        "Configure LLDB settings"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "setting_name": {
                    "type": "string",
                    "description": "Name of the LLDB setting to configure (e.g., 'target.max-children-count', 'thread-format')"
                },
                "value": {
                    "type": "string",
                    "description": "Value to set for the setting"
                }
            },
            "required": ["setting_name", "value"]
        })
    }
    
    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let setting_name = arguments.get("setting_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::IncodeError::invalid_parameter("setting_name required"))?;

        let value = arguments.get("value")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::IncodeError::invalid_parameter("value required"))?;

        let result = lldb_manager.set_lldb_settings(setting_name, value)?;
        
        Ok(ToolResponse::Success(json!({
            "setting_name": setting_name,
            "value": value,
            "result": result,
            "status": "updated"
        }).to_string()))
    }
}
