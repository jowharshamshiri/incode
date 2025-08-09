use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::error::{IncodeError, IncodeResult};
use crate::lldb_manager::LldbManager;
use super::{Tool, ToolResponse};
use uuid::Uuid;

// Session Management Tools (4 tools)
pub struct CreateSessionTool;
pub struct SaveSessionTool;
pub struct LoadSessionTool;
pub struct CleanupSessionTool;

/// Create new debugging session
#[async_trait]
impl Tool for CreateSessionTool {
    fn name(&self) -> &'static str {
        "create_session"
    }
    
    fn description(&self) -> &'static str {
        "Create new debugging session"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "name": {
                "type": "string",
                "description": "Optional session name for identification"
            }
        })
    }
    
    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let session_name = arguments.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let session_id = lldb_manager.create_session()?;
        
        Ok(ToolResponse::Success(json!({
            "session_id": session_id.to_string(),
            "name": session_name,
            "created_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_secs(),
            "status": "created"
        }).to_string()))
    }
}

/// Save debugging session state
#[async_trait]
impl Tool for SaveSessionTool {
    fn name(&self) -> &'static str {
        "save_session"
    }
    
    fn description(&self) -> &'static str {
        "Save debugging session state"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "session_id": {
                "type": "string",
                "description": "UUID of the session to save (optional, uses current session if not specified)"
            },
            "format": {
                "type": "string",
                "enum": ["json", "compact"],
                "description": "Output format (default: json)",
                "default": "json"
            }
        })
    }
    
    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let session_id = if let Some(id_str) = arguments.get("session_id").and_then(|v| v.as_str()) {
            Uuid::parse_str(id_str)
                .map_err(|e| IncodeError::invalid_parameter(format!("Invalid session ID: {}", e)))?
        } else {
            lldb_manager.current_session_id()
                .ok_or_else(|| IncodeError::lldb_op("No current session to save"))?
        };

        let format = arguments.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("json");

        let session_data = lldb_manager.save_session(&session_id)?;
        
        let response = match format {
            "compact" => {
                let compact_data: Value = serde_json::from_str(&session_data)?;
                json!({
                    "session_id": session_id.to_string(),
                    "data": compact_data,
                    "size": session_data.len(),
                    "status": "saved"
                }).to_string()
            },
            _ => {
                json!({
                    "session_id": session_id.to_string(),
                    "session_data": session_data,
                    "size": session_data.len(),
                    "status": "saved"
                }).to_string()
            }
        };
        
        Ok(ToolResponse::Success(response))
    }
}

/// Load debugging session state
#[async_trait]
impl Tool for LoadSessionTool {
    fn name(&self) -> &'static str {
        "load_session"
    }
    
    fn description(&self) -> &'static str {
        "Load debugging session state"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "session_data": {
                "type": "string",
                "description": "JSON session data to load"
            }
        })
    }
    
    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let session_data = arguments.get("session_data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IncodeError::invalid_parameter("session_data required"))?;

        let session_id = lldb_manager.load_session(session_data)?;
        
        Ok(ToolResponse::Success(json!({
            "session_id": session_id.to_string(),
            "loaded_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_secs(),
            "status": "loaded"
        }).to_string()))
    }
}

/// Clean up debugging session resources
#[async_trait]
impl Tool for CleanupSessionTool {
    fn name(&self) -> &'static str {
        "cleanup_session"
    }
    
    fn description(&self) -> &'static str {
        "Clean up debugging session resources"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "session_id": {
                "type": "string",
                "description": "UUID of the session to cleanup (optional, uses current session if not specified)"
            },
            "force": {
                "type": "boolean",
                "description": "Force cleanup even if session is active",
                "default": false
            }
        })
    }
    
    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let session_id = if let Some(id_str) = arguments.get("session_id").and_then(|v| v.as_str()) {
            Uuid::parse_str(id_str)
                .map_err(|e| IncodeError::invalid_parameter(format!("Invalid session ID: {}", e)))?
        } else {
            lldb_manager.current_session_id()
                .ok_or_else(|| IncodeError::lldb_op("No current session to cleanup"))?
        };

        let force = arguments.get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Check if session is active and force is not enabled
        if !force {
            let session = lldb_manager.get_session(&session_id)?;
            if matches!(session.state, crate::lldb_manager::SessionState::Running | crate::lldb_manager::SessionState::Attached) {
                return Err(IncodeError::lldb_op("Session is active. Use force=true to cleanup active session"));
            }
        }

        let result = lldb_manager.cleanup_session(&session_id)?;
        
        Ok(ToolResponse::Success(json!({
            "session_id": session_id.to_string(),
            "result": result,
            "force": force,
            "cleaned_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_secs(),
            "status": "cleaned"
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
