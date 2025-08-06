use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::error::IncodeResult;
use crate::lldb_manager::LldbManager;
use super::{Tool, ToolResponse};

// TODO: Implement all specific tools for this category
// This is a placeholder structure that needs to be expanded

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
