use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::error::IncodeResult;
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

// Placeholder implementations - to be implemented
macro_rules! impl_placeholder_tool {
    ($tool:ident, $name:expr, $desc:expr) => {
        #[async_trait]
        impl Tool for $tool {
            fn name(&self) -> &'static str { $name }
            fn description(&self) -> &'static str { $desc }
            fn parameters(&self) -> Value { json!({}) }
            async fn execute(&self, _: HashMap<String, Value>, _: &mut LldbManager) -> IncodeResult<ToolResponse> {
                Ok(ToolResponse::Error("Not yet implemented".to_string()))
            }
        }
    };
}

impl_placeholder_tool!(ContinueExecutionTool, "continue_execution", "Continue process execution from current state");
impl_placeholder_tool!(StepOverTool, "step_over", "Step over current instruction");
impl_placeholder_tool!(StepIntoTool, "step_into", "Step into function calls");
impl_placeholder_tool!(StepOutTool, "step_out", "Step out of current function");
impl_placeholder_tool!(StepInstructionTool, "step_instruction", "Single instruction step");
impl_placeholder_tool!(RunUntilTool, "run_until", "Run until specific address or line");
impl_placeholder_tool!(InterruptExecutionTool, "interrupt_execution", "Pause/interrupt running process");