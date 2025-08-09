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
            }
        })
    }
    
    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let result = target_information::get_target_info(lldb_manager, arguments).await?;
        Ok(ToolResponse::Json(result))
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
        let result = target_information::get_platform_info(lldb_manager, arguments).await?;
        Ok(ToolResponse::Json(result))
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
        let result = target_information::list_modules(lldb_manager, arguments).await?;
        Ok(ToolResponse::Json(result))
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
