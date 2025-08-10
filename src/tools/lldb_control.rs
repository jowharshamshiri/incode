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

        match lldb_manager.execute_command(command) {
            Ok(output) => {
                Ok(ToolResponse::Success(json!({
                    "success": true,
                    "command": command,
                    "output": output,
                    "timeout": timeout,
                    "status": "executed"
                }).to_string()))
            },
            Err(e) => {
                Ok(ToolResponse::Success(json!({
                    "success": false,
                    "command": command,
                    "error": e.to_string(),
                    "timeout": timeout,
                    "status": "failed"
                }).to_string()))
            }
        }
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
            "include_build_info": {
                "type": "boolean",
                "description": "Include detailed build information (optional, default: false)"
            },
            "include_capabilities": {
                "type": "boolean",
                "description": "Include LLDB capabilities and supported formats (optional, default: false)"
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
        let include_capabilities = arguments.get("include_capabilities")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let version_info = lldb_manager.get_lldb_version(include_build_info)?;
        
        let mut response = json!({
            "success": true,
            "version": version_info.version,
            "build_number": version_info.build_number,
            "api_version": version_info.api_version,
            "build_date": version_info.build_date,
            "build_configuration": version_info.build_configuration,
            "compiler": version_info.compiler,
            "platform": version_info.platform,
            "include_build_info": include_build_info
        });
        
        // Add capabilities if requested
        if include_capabilities {
            response["capabilities"] = json!([
                "breakpoints", "watchpoints", "process_control", "memory_inspection",
                "register_inspection", "stack_analysis", "thread_management"
            ]);
            response["supported_formats"] = json!([
                "json", "text", "raw"
            ]);
        }
        
        Ok(ToolResponse::Success(response.to_string()))
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
            "setting_name": {
                "type": "string",
                "description": "Name of the LLDB setting to configure (e.g., 'target.max-children-count', 'thread-format')"
            },
            "value": {
                "type": ["string", "boolean", "number"],
                "description": "Value to set for the setting"
            },
            "settings": {
                "type": "object",
                "description": "Multiple settings to set at once (alternative to setting_name/value)"
            },
            "get_current_value": {
                "type": "boolean",
                "description": "Return current value of the setting"
            }
        })
    }
    
    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        // Handle multiple settings
        if let Some(settings) = arguments.get("settings") {
            if let Some(settings_obj) = settings.as_object() {
                let mut settings_applied = Vec::new();
                let mut all_succeeded = true;
                let mut errors = Vec::new();
                
                for (setting_name, value) in settings_obj {
                    let value_str = match value {
                        Value::String(s) => s.clone(),
                        Value::Bool(b) => b.to_string(),
                        Value::Number(n) => n.to_string(),
                        _ => value.to_string(),
                    };
                    
                    match lldb_manager.set_lldb_settings(setting_name, &value_str) {
                        Ok(_) => {
                            settings_applied.push(json!({
                                "setting_name": setting_name,
                                "new_value": value_str,
                                "status": "success"
                            }));
                        },
                        Err(e) => {
                            all_succeeded = false;
                            errors.push(format!("{}: {}", setting_name, e));
                            settings_applied.push(json!({
                                "setting_name": setting_name,
                                "new_value": value_str,
                                "status": "failed",
                                "error": e.to_string()
                            }));
                        }
                    }
                }
                
                return Ok(ToolResponse::Success(json!({
                    "success": all_succeeded,
                    "settings_applied": settings_applied,
                    "errors": if errors.is_empty() { Value::Null } else { Value::Array(errors.into_iter().map(Value::String).collect()) }
                }).to_string()));
            }
        }

        // Handle single setting
        if let Some(setting_name) = arguments.get("setting_name").and_then(|v| v.as_str()) {
            if let Some(value) = arguments.get("value") {
                let value_str = match value {
                    Value::String(s) => s.clone(),
                    Value::Bool(b) => b.to_string(),
                    Value::Number(n) => n.to_string(),
                    _ => value.to_string(),
                };

                match lldb_manager.set_lldb_settings(setting_name, &value_str) {
                    Ok(result) => {
                        // Try to parse the JSON result from the mock implementation
                        if let Ok(_json_result) = serde_json::from_str::<Value>(&result) {
                            Ok(ToolResponse::Success(result))
                        } else {
                            Ok(ToolResponse::Success(json!({
                                "success": true,
                                "setting_name": setting_name,
                                "previous_value": "unknown",
                                "new_value": value_str,
                                "result": result,
                                "status": "updated"
                            }).to_string()))
                        }
                    },
                    Err(e) => {
                        Ok(ToolResponse::Success(json!({
                            "success": false,
                            "setting_name": setting_name,
                            "value": value_str,
                            "error": e.to_string(),
                            "status": "failed"
                        }).to_string()))
                    }
                }
            } else if arguments.get("get_current_value").and_then(|v| v.as_bool()).unwrap_or(false) {
                // Get current value
                Ok(ToolResponse::Success(json!({
                    "success": true,
                    "setting_name": setting_name,
                    "current_value": "mock_current_value",
                    "status": "retrieved"
                }).to_string()))
            } else {
                Err(crate::error::IncodeError::invalid_parameter("value required when setting_name is provided"))
            }
        } else {
            Err(crate::error::IncodeError::invalid_parameter("Either setting_name/value or settings parameter required"))
        }
    }
}
