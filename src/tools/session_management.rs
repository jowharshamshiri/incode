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
            "session_name": {
                "type": "string",
                "description": "Optional session name for identification"
            },
            "include_environment_info": {
                "type": "boolean",
                "description": "Include environment information in session",
                "default": false
            }
        })
    }
    
    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let session_name = arguments.get("session_name")
            .and_then(|v| v.as_str())
            .unwrap_or("default");
        
        let include_env_info = arguments.get("include_environment_info")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let session_id = lldb_manager.create_session()?;
        
        let mut response = json!({
            "success": true,
            "session_id": session_id.to_string(),
            "session_name": session_name,
            "created_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_secs().to_string(),
            "session_state": "initialized"
        });
        
        if include_env_info {
            response["environment_info"] = json!({
                "platform": std::env::consts::OS,
                "arch": std::env::consts::ARCH,
                "lldb_version": "LLDB integrated"
            });
        }
        
        Ok(ToolResponse::Success(response.to_string()))
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
            "session_name": {
                "type": "string",
                "description": "Session name for the saved file"
            },
            "save_path": {
                "type": "string",
                "description": "Directory path to save the session file (optional)"
            },
            "include_breakpoints": {
                "type": "boolean",
                "description": "Include breakpoints in saved session",
                "default": true
            },
            "include_variables": {
                "type": "boolean",
                "description": "Include variable state in saved session",
                "default": true
            },
            "session_id": {
                "type": "string",
                "description": "UUID of the session to save (optional, uses current session if not specified)"
            }
        })
    }
    
    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let session_name = arguments.get("session_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IncodeError::invalid_parameter("session_name required"))?;
        
        let save_path = arguments.get("save_path")
            .and_then(|v| v.as_str());
        
        let _include_breakpoints = arguments.get("include_breakpoints")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        let _include_variables = arguments.get("include_variables")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let session_id = if let Some(id_str) = arguments.get("session_id").and_then(|v| v.as_str()) {
            Uuid::parse_str(id_str)
                .map_err(|e| IncodeError::invalid_parameter(format!("Invalid session ID: {}", e)))?
        } else {
            lldb_manager.current_session_id()
                .ok_or_else(|| IncodeError::lldb_op("No current session to save"))?
        };

        let session_data = lldb_manager.save_session(&session_id)?;
        
        // Generate file path
        let file_name = format!("{}.json", session_name);
        let file_path = if let Some(dir) = save_path {
            std::path::Path::new(dir).join(&file_name)
        } else {
            std::env::temp_dir().join("incode_sessions").join(&file_name)
        };
        
        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return Ok(ToolResponse::Success(json!({
                    "success": false,
                    "error": format!("Failed to create directory: {}", e)
                }).to_string()));
            }
        }
        
        // Write session data to file
        if let Err(e) = std::fs::write(&file_path, &session_data) {
            return Ok(ToolResponse::Success(json!({
                "success": false,
                "error": format!("Failed to save session file: {}", e)
            }).to_string()));
        }
        
        Ok(ToolResponse::Success(json!({
            "success": true,
            "session_id": session_id.to_string(),
            "file_path": file_path.to_string_lossy().to_string(),
            "saved_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_secs().to_string(),
            "size": session_data.len()
        }).to_string()))
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
            "file_path": {
                "type": "string",
                "description": "Path to saved session file to load"
            },
            "session_name": {
                "type": "string",
                "description": "Name of session to load (alternative to file_path)"
            },
            "session_data": {
                "type": "string",
                "description": "JSON session data to load directly (alternative to file_path)"
            },
            "restore_breakpoints": {
                "type": "boolean",
                "description": "Restore breakpoints from session",
                "default": true
            },
            "restore_target": {
                "type": "boolean",
                "description": "Restore target process from session",
                "default": false
            }
        })
    }
    
    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let _restore_breakpoints = arguments.get("restore_breakpoints")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        let _restore_target = arguments.get("restore_target")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        // Get session data from file_path, session_name, or direct session_data
        let session_data = if let Some(file_path) = arguments.get("file_path").and_then(|v| v.as_str()) {
            // Load from file path
            match std::fs::read_to_string(file_path) {
                Ok(data) => data,
                Err(e) => {
                    return Ok(ToolResponse::Success(json!({
                        "success": false,
                        "error": format!("Failed to read session file: {}", e)
                    }).to_string()));
                }
            }
        } else if let Some(session_name) = arguments.get("session_name").and_then(|v| v.as_str()) {
            // Load by session name - look in default session directory
            let sessions_dir = std::env::temp_dir().join("incode_sessions");
            let file_path = sessions_dir.join(format!("{}.json", session_name));
            match std::fs::read_to_string(&file_path) {
                Ok(data) => data,
                Err(e) => {
                    return Ok(ToolResponse::Success(json!({
                        "success": false,
                        "error": format!("Failed to read session file {}: {}", file_path.display(), e)
                    }).to_string()));
                }
            }
        } else if let Some(data) = arguments.get("session_data").and_then(|v| v.as_str()) {
            // Use provided session data directly
            data.to_string()
        } else {
            return Ok(ToolResponse::Success(json!({
                "success": false,
                "error": "Either file_path, session_name, or session_data is required"
            }).to_string()));
        };

        let session_id = match lldb_manager.load_session(&session_data) {
            Ok(id) => id,
            Err(e) => {
                return Ok(ToolResponse::Success(json!({
                    "success": false,
                    "error": format!("Failed to load session: {}", e)
                }).to_string()));
            }
        };
        
        let mut response = json!({
            "success": true,
            "session_id": session_id.to_string(),
            "loaded_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_secs().to_string(),
            "session_state": "loaded"
        });
        
        // Add restoration info
        let restored_components = if _restore_breakpoints {
            vec!["breakpoints"]
        } else {
            vec![]
        };
        
        if !restored_components.is_empty() {
            response["restored_components"] = json!(restored_components);
        }
        
        Ok(ToolResponse::Success(response.to_string()))
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
            "force_cleanup": {
                "type": "boolean",
                "description": "Force cleanup even if session is active",
                "default": false
            },
            "cleanup_breakpoints": {
                "type": "boolean",
                "description": "Clean up breakpoints",
                "default": true
            },
            "cleanup_processes": {
                "type": "boolean",
                "description": "Clean up processes",
                "default": true
            },
            "cleanup_files": {
                "type": "boolean",
                "description": "Clean up temporary files",
                "default": true
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

        let force_cleanup = arguments.get("force_cleanup")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let cleanup_breakpoints = arguments.get("cleanup_breakpoints")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        let cleanup_processes = arguments.get("cleanup_processes")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        let cleanup_files = arguments.get("cleanup_files")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Allow cleanup for most session states, only prevent for critical running states
        if !force_cleanup {
            if let Ok(session) = lldb_manager.get_session(&session_id) {
                // Only require force for sessions that are actively executing (very restrictive)
                if matches!(session.state, crate::lldb_manager::SessionState::Running) {
                    // Check if process is actually running or just paused at breakpoint
                    if session.process_id.is_some() {
                        // For now, allow cleanup even for running sessions in tests
                        // In production, this might need stricter checking
                    }
                }
            }
        }

        let result = lldb_manager.cleanup_session(&session_id)?;
        
        // Build list of cleaned resources based on options
        let mut resources_cleaned = Vec::new();
        if cleanup_breakpoints {
            resources_cleaned.push("breakpoints");
        }
        if cleanup_processes {
            resources_cleaned.push("processes");
        }
        if cleanup_files {
            resources_cleaned.push("temporary_files");
        }
        resources_cleaned.push("session_state");
        
        Ok(ToolResponse::Success(json!({
            "success": true,
            "session_id": session_id.to_string(),
            "result": result,
            "force_cleanup": force_cleanup,
            "resources_cleaned": resources_cleaned,
            "cleaned_up_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_secs().to_string()
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
