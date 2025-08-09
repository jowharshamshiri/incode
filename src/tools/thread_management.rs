use crate::lldb_manager::{LldbManager, ThreadInfo};
use crate::error::IncodeResult;
use crate::tools::{Tool, ToolResponse};
use std::collections::HashMap;
use serde_json::{json, Value};
use async_trait::async_trait;
use tracing::{debug, error};

pub fn list_threads(
    lldb_manager: &LldbManager,
    arguments: HashMap<String, Value>,
) -> IncodeResult<Value> {
    debug!("Thread Management: list_threads called with args: {:?}", arguments);
    
    let include_details = arguments.get("include_details")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
        
    let filter_state = arguments.get("filter_state")
        .and_then(|v| v.as_str());
    
    match lldb_manager.list_threads() {
        Ok(threads) => {
            debug!("Found {} threads", threads.len());
            
            // Apply state filter if specified
            let filtered_threads: Vec<&ThreadInfo> = if let Some(state) = filter_state {
                threads.iter()
                    .filter(|t| t.state.contains(state))
                    .collect()
            } else {
                threads.iter().collect()
            };
            
            let thread_list: Vec<Value> = filtered_threads.iter().map(|thread| {
                if include_details {
                    json!({
                        "thread_id": thread.thread_id,
                        "index": thread.index,
                        "name": thread.name,
                        "state": thread.state,
                        "stop_reason": thread.stop_reason,
                        "queue_name": thread.queue_name,
                        "frame_count": thread.frame_count,
                        "current_frame": thread.current_frame.as_ref().map(|frame| json!({
                            "index": frame.index,
                            "function_name": frame.function_name,
                            "file_path": frame.file_path,
                            "line_number": frame.line_number,
                            "address": format!("0x{:x}", frame.address)
                        }))
                    })
                } else {
                    json!({
                        "thread_id": thread.thread_id,
                        "index": thread.index,
                        "name": thread.name,
                        "state": thread.state
                    })
                }
            }).collect();
            
            Ok(json!({
                "success": true,
                "threads": thread_list,
                "total_count": filtered_threads.len(),
                "filter_applied": filter_state
            }))
        }
        Err(e) => {
            error!("Failed to list threads: {}", e);
            Ok(json!({
                "success": false,
                "error": e.to_string(),
                "threads": []
            }))
        }
    }
}

pub fn select_thread(
    lldb_manager: &mut LldbManager,
    arguments: HashMap<String, Value>,
) -> IncodeResult<Value> {
    debug!("Thread Management: select_thread called with args: {:?}", arguments);
    
    let thread_id = arguments.get("thread_id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| crate::error::IncodeError::invalid_parameter("thread_id is required and must be a number"))?
        as u32;
    
    let include_frames = arguments.get("include_frames")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    match lldb_manager.select_thread(thread_id) {
        Ok(thread_info) => {
            debug!("Selected thread {}: {}", thread_id, thread_info.name.as_deref().unwrap_or("unnamed"));
            
            let mut result = json!({
                "success": true,
                "selected_thread": {
                    "thread_id": thread_info.thread_id,
                    "index": thread_info.index,
                    "name": thread_info.name,
                    "state": thread_info.state,
                    "stop_reason": thread_info.stop_reason,
                    "queue_name": thread_info.queue_name,
                    "frame_count": thread_info.frame_count
                }
            });
            
            if include_frames && thread_info.current_frame.is_some() {
                result["selected_thread"]["current_frame"] = json!({
                    "index": thread_info.current_frame.as_ref().unwrap().index,
                    "function_name": thread_info.current_frame.as_ref().unwrap().function_name,
                    "file_path": thread_info.current_frame.as_ref().unwrap().file_path,
                    "line_number": thread_info.current_frame.as_ref().unwrap().line_number,
                    "address": format!("0x{:x}", thread_info.current_frame.as_ref().unwrap().address)
                });
            }
            
            Ok(result)
        }
        Err(e) => {
            error!("Failed to select thread {}: {}", thread_id, e);
            Ok(json!({
                "success": false,
                "error": e.to_string(),
                "thread_id": thread_id
            }))
        }
    }
}

pub fn get_thread_info(
    lldb_manager: &LldbManager,
    arguments: HashMap<String, Value>,
) -> IncodeResult<Value> {
    debug!("Thread Management: get_thread_info called with args: {:?}", arguments);
    
    let thread_id = arguments.get("thread_id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| crate::error::IncodeError::invalid_parameter("thread_id is required and must be a number"))?
        as u32;
    
    let include_stack = arguments.get("include_stack")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    let include_registers = arguments.get("include_registers")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    // First, get all threads to find the requested one
    match lldb_manager.list_threads() {
        Ok(threads) => {
            if let Some(thread) = threads.iter().find(|t| t.thread_id == thread_id) {
                let mut result = json!({
                    "success": true,
                    "thread_info": {
                        "thread_id": thread.thread_id,
                        "index": thread.index,
                        "name": thread.name,
                        "state": thread.state,
                        "stop_reason": thread.stop_reason,
                        "queue_name": thread.queue_name,
                        "frame_count": thread.frame_count,
                        "current_frame": thread.current_frame.as_ref().map(|frame| json!({
                            "index": frame.index,
                            "function_name": frame.function_name,
                            "file_path": frame.file_path,
                            "line_number": frame.line_number,
                            "address": format!("0x{:x}", frame.address),
                            "is_inlined": frame.is_inlined
                        }))
                    }
                });
                
                // Add stack information if requested
                if include_stack {
                    // TODO: Implement stack frame enumeration
                    result["thread_info"]["stack_frames"] = json!([]);
                }
                
                // Add register information if requested
                if include_registers {
                    // TODO: Implement register reading for specific thread
                    result["thread_info"]["registers"] = json!({});
                }
                
                Ok(result)
            } else {
                Ok(json!({
                    "success": false,
                    "error": format!("Thread {} not found", thread_id),
                    "thread_id": thread_id
                }))
            }
        }
        Err(e) => {
            error!("Failed to get thread info for {}: {}", thread_id, e);
            Ok(json!({
                "success": false,
                "error": e.to_string(),
                "thread_id": thread_id
            }))
        }
    }
}

pub fn suspend_thread(
    _lldb_manager: &mut LldbManager,
    arguments: HashMap<String, Value>,
) -> IncodeResult<Value> {
    debug!("Thread Management: suspend_thread called with args: {:?}", arguments);
    
    let thread_id = arguments.get("thread_id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| crate::error::IncodeError::invalid_parameter("thread_id is required and must be a number"))?
        as u32;
    
    // TODO: Implement actual thread suspension
    debug!("Mock: Suspending thread {}", thread_id);
    
    Ok(json!({
        "success": true,
        "thread_id": thread_id,
        "status": "suspended",
        "message": format!("Thread {} suspended (mock implementation)", thread_id)
    }))
}

pub fn resume_thread(
    _lldb_manager: &mut LldbManager,
    arguments: HashMap<String, Value>,
) -> IncodeResult<Value> {
    debug!("Thread Management: resume_thread called with args: {:?}", arguments);
    
    let thread_id = arguments.get("thread_id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| crate::error::IncodeError::invalid_parameter("thread_id is required and must be a number"))?
        as u32;
    
    // TODO: Implement actual thread resumption
    debug!("Mock: Resuming thread {}", thread_id);
    
    Ok(json!({
        "success": true,
        "thread_id": thread_id,
        "status": "running",
        "message": format!("Thread {} resumed (mock implementation)", thread_id)
    }))
}

// Tool implementations for MCP protocol

pub struct ListThreadsTool;

#[async_trait]
impl Tool for ListThreadsTool {
    fn name(&self) -> &'static str {
        "list_threads"
    }
    
    fn description(&self) -> &'static str {
        "List all threads with IDs and states"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "include_details": {
                "type": "boolean",
                "description": "Include detailed thread information including frames",
                "default": false
            },
            "filter_state": {
                "type": "string",
                "description": "Filter threads by state (stopped, running, etc.)"
            }
        })
    }
    
    async fn execute(&self, arguments: HashMap<String, Value>, manager: &mut LldbManager) -> IncodeResult<ToolResponse> {
        match list_threads(manager, arguments) {
            Ok(result) => Ok(ToolResponse::Success(result.to_string())),
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

pub struct SelectThreadTool;

#[async_trait]
impl Tool for SelectThreadTool {
    fn name(&self) -> &'static str {
        "select_thread"
    }
    
    fn description(&self) -> &'static str {
        "Switch to specific thread for debugging"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "thread_id": {
                "type": "number",
                "description": "Thread ID to select"
            },
            "include_frames": {
                "type": "boolean",
                "description": "Include current frame information",
                "default": false
            }
        })
    }
    
    async fn execute(&self, arguments: HashMap<String, Value>, manager: &mut LldbManager) -> IncodeResult<ToolResponse> {
        match select_thread(manager, arguments) {
            Ok(result) => Ok(ToolResponse::Success(result.to_string())),
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

pub struct GetThreadInfoTool;

#[async_trait]
impl Tool for GetThreadInfoTool {
    fn name(&self) -> &'static str {
        "get_thread_info"
    }
    
    fn description(&self) -> &'static str {
        "Get thread details (state, stack, registers)"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "thread_id": {
                "type": "number",
                "description": "Thread ID to get info for"
            },
            "include_stack": {
                "type": "boolean",
                "description": "Include stack frame information",
                "default": false
            },
            "include_registers": {
                "type": "boolean",
                "description": "Include register information",
                "default": false
            }
        })
    }
    
    async fn execute(&self, arguments: HashMap<String, Value>, manager: &mut LldbManager) -> IncodeResult<ToolResponse> {
        match get_thread_info(manager, arguments) {
            Ok(result) => Ok(ToolResponse::Success(result.to_string())),
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

pub struct SuspendThreadTool;

#[async_trait]
impl Tool for SuspendThreadTool {
    fn name(&self) -> &'static str {
        "suspend_thread"
    }
    
    fn description(&self) -> &'static str {
        "Suspend specific thread execution"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "thread_id": {
                "type": "number",
                "description": "Thread ID to suspend"
            }
        })
    }
    
    async fn execute(&self, arguments: HashMap<String, Value>, manager: &mut LldbManager) -> IncodeResult<ToolResponse> {
        match suspend_thread(manager, arguments) {
            Ok(result) => Ok(ToolResponse::Success(result.to_string())),
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

pub struct ResumeThreadTool;

#[async_trait]
impl Tool for ResumeThreadTool {
    fn name(&self) -> &'static str {
        "resume_thread"
    }
    
    fn description(&self) -> &'static str {
        "Resume suspended thread"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "thread_id": {
                "type": "number",
                "description": "Thread ID to resume"
            }
        })
    }
    
    async fn execute(&self, arguments: HashMap<String, Value>, manager: &mut LldbManager) -> IncodeResult<ToolResponse> {
        match resume_thread(manager, arguments) {
            Ok(result) => Ok(ToolResponse::Success(result.to_string())),
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}