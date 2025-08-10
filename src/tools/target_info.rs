use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::error::IncodeResult;
use crate::lldb_manager::LldbManager;
use super::{Tool, ToolResponse};
use super::target_information;

pub struct GetTargetInfoTool;

#[async_trait]
impl Tool for GetTargetInfoTool {
    fn name(&self) -> &'static str {
        "get_target_info"
    }
    
    fn description(&self) -> &'static str {
        "Get comprehensive target information including architecture, platform, and executable details"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "include_debug_info": {
                "type": "boolean",
                "description": "Include debug symbol information",
                "default": true
            },
            "include_file_details": {
                "type": "boolean", 
                "description": "Include file size, timestamps, and other file details",
                "default": true
            },
            "analyze_symbols": {
                "type": "boolean",
                "description": "Perform symbol analysis and include detailed symbol information",
                "default": false
            }
        })
    }
    
    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        match target_information::get_target_info(lldb_manager, arguments).await {
            Ok(result) => {
                let mut response = json!({
                    "success": true
                });
                if let Value::Object(obj) = result {
                    for (key, value) in obj {
                        response[key] = value;
                    }
                }
                Ok(ToolResponse::Success(response.to_string()))
            },
            Err(e) => Ok(ToolResponse::Success(json!({
                "success": false,
                "error": format!("Failed to get target info: {}", e)
            }).to_string()))
        }
    }
}

pub struct GetPlatformInfoTool;

#[async_trait]
impl Tool for GetPlatformInfoTool {
    fn name(&self) -> &'static str {
        "get_platform_info"
    }
    
    fn description(&self) -> &'static str {
        "Get comprehensive platform information including OS version, architecture, and development environment details"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "include_development_info": {
                "type": "boolean",
                "description": "Include SDK version and deployment target information",
                "default": true
            },
            "include_capabilities": {
                "type": "boolean",
                "description": "Include platform capabilities like JIT support, simulator status",
                "default": true
            }
        })
    }
    
    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        match target_information::get_platform_info(lldb_manager, arguments).await {
            Ok(result) => {
                let mut response = json!({
                    "success": true
                });
                if let Value::Object(obj) = result {
                    for (key, value) in obj {
                        response[key] = value;
                    }
                }
                Ok(ToolResponse::Success(response.to_string()))
            },
            Err(e) => Ok(ToolResponse::Success(json!({
                "success": false,
                "error": format!("Failed to get platform info: {}", e)
            }).to_string()))
        }
    }
}

pub struct ListModulesTool;

#[async_trait]
impl Tool for ListModulesTool {
    fn name(&self) -> &'static str {
        "list_modules"
    }
    
    fn description(&self) -> &'static str {
        "List all loaded modules/libraries with their addresses and debug information"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "filter_name": {
                "type": "string",
                "description": "Filter modules by name (substring match)"
            },
            "name_pattern": {
                "type": "string",
                "description": "Filter modules by name pattern (alias for filter_name)"
            },
            "limit": {
                "type": "number",
                "description": "Maximum number of modules to return"
            },
            "include_debug_info": {
                "type": "boolean",
                "description": "Include debug information such as compile units",
                "default": true
            },
            "include_addresses": {
                "type": "boolean",
                "description": "Include load addresses and ASLR slide information", 
                "default": true
            },
            "include_symbols": {
                "type": "boolean",
                "description": "Include symbol count information",
                "default": false
            }
        })
    }
    
    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        match target_information::list_modules(lldb_manager, arguments).await {
            Ok(result) => {
                let mut response = json!({
                    "success": true
                });
                if let Value::Object(obj) = result {
                    for (key, value) in obj {
                        response[key] = value;
                    }
                }
                Ok(ToolResponse::Success(response.to_string()))
            },
            Err(e) => Ok(ToolResponse::Success(json!({
                "success": false,
                "error": format!("Failed to list modules: {}", e)
            }).to_string()))
        }
    }
}

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
