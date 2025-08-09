use crate::lldb_manager::{LldbManager, RegisterInfo, RegisterState};
use crate::error::IncodeResult;
use crate::tools::{Tool, ToolResponse};
use std::collections::HashMap;
use serde_json::{json, Value};
use async_trait::async_trait;
use tracing::{debug, error};

pub fn get_registers(
    lldb_manager: &LldbManager,
    arguments: HashMap<String, Value>,
) -> IncodeResult<Value> {
    debug!("Register Inspection: get_registers called with args: {:?}", arguments);
    
    let thread_id = arguments.get("thread_id")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);
        
    let include_metadata = arguments.get("include_metadata")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
        
    let register_filter = arguments.get("register_filter")
        .and_then(|v| v.as_str());
    
    match lldb_manager.get_registers(thread_id, include_metadata) {
        Ok(register_state) => {
            debug!("Found {} registers", register_state.registers.len());
            
            // Apply filter if specified
            let filtered_registers: Vec<(&String, &RegisterInfo)> = if let Some(filter) = register_filter {
                register_state.registers.iter()
                    .filter(|(name, _)| name.to_lowercase().contains(&filter.to_lowercase()))
                    .collect()
            } else {
                register_state.registers.iter().collect()
            };
            
            let register_list: Vec<Value> = filtered_registers.iter().map(|(_, reg_info)| {
                if include_metadata {
                    json!({
                        "name": reg_info.name,
                        "value": format!("0x{:x}", reg_info.value),
                        "decimal_value": reg_info.value,
                        "size": reg_info.size,
                        "type": reg_info.register_type,
                        "format": reg_info.format,
                        "is_valid": reg_info.is_valid
                    })
                } else {
                    json!({
                        "name": reg_info.name,
                        "value": format!("0x{:x}", reg_info.value),
                        "decimal_value": reg_info.value
                    })
                }
            }).collect();
            
            Ok(json!({
                "success": true,
                "registers": register_list,
                "total_count": filtered_registers.len(),
                "thread_id": register_state.thread_id,
                "frame_index": register_state.frame_index,
                "timestamp": register_state.timestamp.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default().as_secs(),
                "filter_applied": register_filter
            }))
        }
        Err(e) => {
            error!("Failed to get registers: {}", e);
            Ok(json!({
                "success": false,
                "error": e.to_string(),
                "registers": []
            }))
        }
    }
}

pub fn set_register(
    lldb_manager: &mut LldbManager,
    arguments: HashMap<String, Value>,
) -> IncodeResult<Value> {
    debug!("Register Inspection: set_register called with args: {:?}", arguments);
    
    let register_name = arguments.get("register_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| crate::error::IncodeError::invalid_parameter("register_name is required"))?;
    
    let value = arguments.get("value")
        .and_then(|v| {
            // Handle both string hex values and numbers
            if let Some(s) = v.as_str() {
                if s.starts_with("0x") || s.starts_with("0X") {
                    u64::from_str_radix(&s[2..], 16).ok()
                } else {
                    s.parse::<u64>().ok()
                }
            } else {
                v.as_u64()
            }
        })
        .ok_or_else(|| crate::error::IncodeError::invalid_parameter("value is required and must be a number or hex string"))?;
    
    let thread_id = arguments.get("thread_id")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);
    
    match lldb_manager.set_register(register_name, value, thread_id) {
        Ok(success) => {
            debug!("Register {} set to 0x{:x}, success: {}", register_name, value, success);
            
            Ok(json!({
                "success": success,
                "register_name": register_name,
                "new_value": format!("0x{:x}", value),
                "decimal_value": value,
                "thread_id": thread_id,
                "message": if success { 
                    format!("Register {} successfully set to 0x{:x}", register_name, value)
                } else { 
                    format!("Failed to set register {} to 0x{:x}", register_name, value)
                }
            }))
        }
        Err(e) => {
            error!("Failed to set register {}: {}", register_name, e);
            Ok(json!({
                "success": false,
                "error": e.to_string(),
                "register_name": register_name,
                "attempted_value": format!("0x{:x}", value)
            }))
        }
    }
}

pub fn get_register_info(
    lldb_manager: &LldbManager,
    arguments: HashMap<String, Value>,
) -> IncodeResult<Value> {
    debug!("Register Inspection: get_register_info called with args: {:?}", arguments);
    
    let register_name = arguments.get("register_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| crate::error::IncodeError::invalid_parameter("register_name is required"))?;
    
    let thread_id = arguments.get("thread_id")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);
    
    match lldb_manager.get_register_info(register_name, thread_id) {
        Ok(reg_info) => {
            debug!("Got register info for {}: 0x{:x}", register_name, reg_info.value);
            
            Ok(json!({
                "success": true,
                "register_info": {
                    "name": reg_info.name,
                    "value": format!("0x{:x}", reg_info.value),
                    "decimal_value": reg_info.value,
                    "size": reg_info.size,
                    "size_description": match reg_info.size {
                        1 => "8-bit",
                        2 => "16-bit", 
                        4 => "32-bit",
                        8 => "64-bit",
                        16 => "128-bit",
                        _ => "unknown"
                    },
                    "type": reg_info.register_type,
                    "format": reg_info.format,
                    "is_valid": reg_info.is_valid,
                    "binary_representation": format!("{:064b}", reg_info.value),
                    "bits_set": reg_info.value.count_ones(),
                    "bits_clear": reg_info.value.count_zeros()
                },
                "thread_id": thread_id
            }))
        }
        Err(e) => {
            error!("Failed to get register info for {}: {}", register_name, e);
            Ok(json!({
                "success": false,
                "error": e.to_string(),
                "register_name": register_name
            }))
        }
    }
}

pub fn save_register_state(
    lldb_manager: &LldbManager,
    arguments: HashMap<String, Value>,
) -> IncodeResult<Value> {
    debug!("Register Inspection: save_register_state called with args: {:?}", arguments);
    
    let thread_id = arguments.get("thread_id")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);
        
    let include_metadata = arguments.get("include_metadata")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    
    match lldb_manager.save_register_state(thread_id) {
        Ok(register_state) => {
            debug!("Saved register state with {} registers", register_state.registers.len());
            
            let registers_json: HashMap<String, Value> = register_state.registers.iter()
                .map(|(name, reg_info)| {
                    let value = if include_metadata {
                        json!({
                            "value": format!("0x{:x}", reg_info.value),
                            "decimal_value": reg_info.value,
                            "size": reg_info.size,
                            "type": reg_info.register_type,
                            "format": reg_info.format,
                            "is_valid": reg_info.is_valid
                        })
                    } else {
                        json!({
                            "value": format!("0x{:x}", reg_info.value),
                            "decimal_value": reg_info.value
                        })
                    };
                    (name.clone(), value)
                })
                .collect();
            
            Ok(json!({
                "success": true,
                "register_state": {
                    "registers": registers_json,
                    "register_count": register_state.registers.len(),
                    "thread_id": register_state.thread_id,
                    "frame_index": register_state.frame_index,
                    "timestamp": register_state.timestamp.duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default().as_secs(),
                    "state_id": format!("{:x}", 
                        register_state.timestamp.duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default().as_nanos() % u64::MAX as u128
                    )
                },
                "message": format!("Register state saved with {} registers", register_state.registers.len())
            }))
        }
        Err(e) => {
            error!("Failed to save register state: {}", e);
            Ok(json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

// Tool implementations for MCP protocol

pub struct GetRegistersTool;

#[async_trait]
impl Tool for GetRegistersTool {
    fn name(&self) -> &'static str {
        "get_registers"
    }
    
    fn description(&self) -> &'static str {
        "Get all CPU registers for current thread"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "thread_id": {
                "type": "number",
                "description": "Thread ID to get registers for (uses current thread if not specified)"
            },
            "include_metadata": {
                "type": "boolean",
                "description": "Include register metadata (size, type, format)",
                "default": false
            },
            "register_filter": {
                "type": "string",
                "description": "Filter registers by name (case-insensitive substring match)"
            }
        })
    }
    
    async fn execute(&self, arguments: HashMap<String, Value>, manager: &mut LldbManager) -> IncodeResult<ToolResponse> {
        match get_registers(manager, arguments) {
            Ok(result) => Ok(ToolResponse::Success(result.to_string())),
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

pub struct SetRegisterTool;

#[async_trait]
impl Tool for SetRegisterTool {
    fn name(&self) -> &'static str {
        "set_register"
    }
    
    fn description(&self) -> &'static str {
        "Modify register value"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "register_name": {
                "type": "string",
                "description": "Name of the register to modify (e.g., 'rax', 'eip', 'esp')"
            },
            "value": {
                "type": ["number", "string"],
                "description": "New register value (can be decimal number or hex string like '0x1234')"
            },
            "thread_id": {
                "type": "number",
                "description": "Thread ID (uses current thread if not specified)"
            }
        })
    }
    
    async fn execute(&self, arguments: HashMap<String, Value>, manager: &mut LldbManager) -> IncodeResult<ToolResponse> {
        match set_register(manager, arguments) {
            Ok(result) => Ok(ToolResponse::Success(result.to_string())),
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

pub struct GetRegisterInfoTool;

#[async_trait]
impl Tool for GetRegisterInfoTool {
    fn name(&self) -> &'static str {
        "get_register_info"
    }
    
    fn description(&self) -> &'static str {
        "Get detailed register information"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "register_name": {
                "type": "string",
                "description": "Name of the register to inspect (e.g., 'rax', 'eip', 'esp')"
            },
            "thread_id": {
                "type": "number",
                "description": "Thread ID (uses current thread if not specified)"
            }
        })
    }
    
    async fn execute(&self, arguments: HashMap<String, Value>, manager: &mut LldbManager) -> IncodeResult<ToolResponse> {
        match get_register_info(manager, arguments) {
            Ok(result) => Ok(ToolResponse::Success(result.to_string())),
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

pub struct SaveRegisterStateTool;

#[async_trait]
impl Tool for SaveRegisterStateTool {
    fn name(&self) -> &'static str {
        "save_register_state"
    }
    
    fn description(&self) -> &'static str {
        "Save current register state"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "thread_id": {
                "type": "number",
                "description": "Thread ID to save registers for (uses current thread if not specified)"
            },
            "include_metadata": {
                "type": "boolean",
                "description": "Include register metadata in saved state",
                "default": true
            }
        })
    }
    
    async fn execute(&self, arguments: HashMap<String, Value>, manager: &mut LldbManager) -> IncodeResult<ToolResponse> {
        match save_register_state(manager, arguments) {
            Ok(result) => Ok(ToolResponse::Success(result.to_string())),
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}