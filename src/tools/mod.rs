use serde_json::{json, Value};
use std::collections::HashMap;
use async_trait::async_trait;

use crate::error::{IncodeError, IncodeResult};
use crate::lldb_manager::LldbManager;

pub mod process_control;
pub mod execution_control;
pub mod breakpoints;
pub mod stack_analysis;
pub mod memory_inspection;
pub mod variables;
pub mod thread_management;
pub mod threads;
pub mod register_inspection;
pub mod registers;
pub mod debug_information;
pub mod debug_info;
pub mod target_information;
pub mod target_info;
pub mod lldb_control;
pub mod session_management;
pub mod advanced_analysis;

#[derive(Debug)]
pub enum ToolResponse {
    Success(String),
    Error(String),
    Json(Value),
}

#[async_trait]
pub trait Tool {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn parameters(&self) -> Value;
    
    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool + Send + Sync>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };
        
        // Register all tools from all categories
        registry.register_process_control_tools();
        registry.register_execution_control_tools();
        registry.register_breakpoint_tools();
        registry.register_stack_analysis_tools();
        registry.register_memory_inspection_tools();
        registry.register_variable_tools();
        registry.register_thread_tools();
        registry.register_register_tools();
        registry.register_debug_info_tools();
        registry.register_target_info_tools();
        registry.register_lldb_control_tools();
        registry.register_session_management_tools();
        registry.register_advanced_analysis_tools();
        
        registry
    }

    fn register_tool(&mut self, tool: Box<dyn Tool + Send + Sync>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    pub fn get_tool_list(&self) -> Vec<Value> {
        self.tools.values().map(|tool| {
            json!({
                "name": tool.name(),
                "description": tool.description(),
                "inputSchema": {
                    "type": "object",
                    "properties": tool.parameters(),
                    "required": []
                }
            })
        }).collect()
    }

    pub async fn execute_tool(
        &self,
        name: &str,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let tool = self.tools.get(name)
            .ok_or_else(|| IncodeError::mcp(format!("Unknown tool: {}", name)))?;
        
        tool.execute(arguments, lldb_manager).await
    }

    // Tool registration methods for each category
    fn register_process_control_tools(&mut self) {
        self.register_tool(Box::new(process_control::LaunchProcessTool));
        self.register_tool(Box::new(process_control::AttachToProcessTool));
        self.register_tool(Box::new(process_control::DetachProcessTool));
        self.register_tool(Box::new(process_control::KillProcessTool));
        self.register_tool(Box::new(process_control::GetProcessInfoTool));
        self.register_tool(Box::new(process_control::ListProcessesTool));
    }

    fn register_execution_control_tools(&mut self) {
        self.register_tool(Box::new(execution_control::ContinueExecutionTool));
        self.register_tool(Box::new(execution_control::StepOverTool));
        self.register_tool(Box::new(execution_control::StepIntoTool));
        self.register_tool(Box::new(execution_control::StepOutTool));
        self.register_tool(Box::new(execution_control::StepInstructionTool));
        self.register_tool(Box::new(execution_control::RunUntilTool));
        self.register_tool(Box::new(execution_control::InterruptExecutionTool));
    }

    fn register_breakpoint_tools(&mut self) {
        self.register_tool(Box::new(breakpoints::SetBreakpointTool));
        self.register_tool(Box::new(breakpoints::SetWatchpointTool));
        self.register_tool(Box::new(breakpoints::ListBreakpointsTool));
        self.register_tool(Box::new(breakpoints::DeleteBreakpointTool));
        self.register_tool(Box::new(breakpoints::EnableBreakpointTool));
        self.register_tool(Box::new(breakpoints::DisableBreakpointTool));
        self.register_tool(Box::new(breakpoints::SetConditionalBreakpointTool));
        self.register_tool(Box::new(breakpoints::BreakpointCommandsTool));
    }

    fn register_stack_analysis_tools(&mut self) {
        self.register_tool(Box::new(stack_analysis::GetBacktraceTool));
        self.register_tool(Box::new(stack_analysis::SelectFrameTool));
        self.register_tool(Box::new(stack_analysis::GetFrameInfoTool));
        self.register_tool(Box::new(stack_analysis::GetFrameVariablesTool));
        self.register_tool(Box::new(stack_analysis::GetFrameArgumentsTool));
        self.register_tool(Box::new(stack_analysis::EvaluateInFrameTool));
        // Keep placeholder for backward compatibility
        self.register_tool(Box::new(stack_analysis::PlaceholderTool));
    }

    fn register_memory_inspection_tools(&mut self) {
        self.register_tool(Box::new(memory_inspection::ReadMemoryTool));
        self.register_tool(Box::new(memory_inspection::WriteMemoryTool));
        self.register_tool(Box::new(memory_inspection::DisassembleTool));
        self.register_tool(Box::new(memory_inspection::SearchMemoryTool));
        self.register_tool(Box::new(memory_inspection::GetMemoryRegionsTool));
        self.register_tool(Box::new(memory_inspection::DumpMemoryTool));
        self.register_tool(Box::new(memory_inspection::MemoryMapTool));
        // Keep placeholder for backward compatibility
        self.register_tool(Box::new(memory_inspection::PlaceholderTool));
    }

    fn register_variable_tools(&mut self) {
        self.register_tool(Box::new(variables::GetVariablesTool));
        self.register_tool(Box::new(variables::GetGlobalVariablesTool));
        self.register_tool(Box::new(variables::EvaluateExpressionTool));
        self.register_tool(Box::new(variables::GetVariableInfoTool));
        self.register_tool(Box::new(variables::SetVariableTool));
        self.register_tool(Box::new(variables::LookupSymbolTool));
        // Keep placeholder for backward compatibility
        self.register_tool(Box::new(variables::PlaceholderTool));
    }

    fn register_thread_tools(&mut self) {
        self.register_tool(Box::new(thread_management::ListThreadsTool));
        self.register_tool(Box::new(thread_management::SelectThreadTool));
        self.register_tool(Box::new(thread_management::GetThreadInfoTool));
        self.register_tool(Box::new(thread_management::SuspendThreadTool));
        self.register_tool(Box::new(thread_management::ResumeThreadTool));
        // Keep placeholder for backward compatibility
        self.register_tool(Box::new(threads::PlaceholderTool));
    }

    fn register_register_tools(&mut self) {
        self.register_tool(Box::new(register_inspection::GetRegistersTool));
        self.register_tool(Box::new(register_inspection::SetRegisterTool));
        self.register_tool(Box::new(register_inspection::GetRegisterInfoTool));
        self.register_tool(Box::new(register_inspection::SaveRegisterStateTool));
        // Keep placeholder for backward compatibility
        self.register_tool(Box::new(registers::PlaceholderTool));
    }

    fn register_debug_info_tools(&mut self) {
        self.register_tool(Box::new(debug_information::GetSourceCodeTool));
        self.register_tool(Box::new(debug_information::ListFunctionsTool));
        self.register_tool(Box::new(debug_information::GetLineInfoTool));
        self.register_tool(Box::new(debug_information::GetDebugInfoTool));
        // Keep placeholder for backward compatibility
        self.register_tool(Box::new(debug_info::PlaceholderTool));
    }

    fn register_target_info_tools(&mut self) {
        self.register_tool(Box::new(target_info::GetTargetInfoTool));
        self.register_tool(Box::new(target_info::GetPlatformInfoTool));
        self.register_tool(Box::new(target_info::ListModulesTool));
        self.register_tool(Box::new(target_info::PlaceholderTool));
    }

    fn register_lldb_control_tools(&mut self) {
        // TODO: Register actual tools when implemented
        self.register_tool(Box::new(lldb_control::PlaceholderTool));
    }

    fn register_session_management_tools(&mut self) {
        // TODO: Register actual tools when implemented
        self.register_tool(Box::new(session_management::PlaceholderTool));
    }

    fn register_advanced_analysis_tools(&mut self) {
        // TODO: Register actual tools when implemented  
        self.register_tool(Box::new(advanced_analysis::PlaceholderTool));
    }
}