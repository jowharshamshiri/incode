use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::error::{IncodeError, IncodeResult};
use crate::lldb_manager::LldbManager;
use super::{Tool, ToolResponse};

// Variable & Symbol Inspection Tools (6 tools)
pub struct GetVariablesTool;
pub struct GetGlobalVariablesTool;
pub struct EvaluateExpressionTool;
pub struct GetVariableInfoTool;
pub struct SetVariableTool;
pub struct LookupSymbolTool;

// F0035: get_variables - Fully implemented
#[async_trait]
impl Tool for GetVariablesTool {
    fn name(&self) -> &'static str {
        "get_variables"
    }

    fn description(&self) -> &'static str {
        "Get variables in current scope with types and values (local, global, static)"
    }

    fn parameters(&self) -> Value {
        json!({
            "scope": {
                "type": "string",
                "description": "Variable scope filter",
                "enum": ["all", "local", "global", "parameter", "static"],
                "default": "all"
            },
            "filter": {
                "type": "string",
                "description": "Filter variables by name pattern (optional)",
                "default": ""
            },
            "include_types": {
                "type": "boolean",
                "description": "Include detailed type information",
                "default": true
            },
            "format": {
                "type": "string",
                "description": "Output format for variable information",
                "enum": ["detailed", "compact", "names_only", "types_only"],
                "default": "detailed"
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let scope = arguments.get("scope")
            .and_then(|v| v.as_str())
            .unwrap_or("all");

        let filter = arguments.get("filter")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());

        let include_types = arguments.get("include_types")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let format = arguments.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("detailed");

        let scope_filter = if scope == "all" { None } else { Some(scope) };

        match lldb_manager.get_variables(scope_filter, filter) {
            Ok(variables) => {
                let formatted_variables = Self::format_variables(&variables, format, include_types);
                
                // Count by scope
                let local_count = variables.iter().filter(|v| v.scope == "local").count();
                let global_count = variables.iter().filter(|v| v.scope == "global").count();
                let param_count = variables.iter().filter(|v| v.is_argument).count();
                let static_count = variables.iter().filter(|v| v.scope == "static").count();

                Ok(ToolResponse::Json(json!({
                    "total_variables": variables.len(),
                    "local_variables": local_count,
                    "global_variables": global_count,
                    "parameters": param_count,
                    "static_variables": static_count,
                    "scope_filter": scope,
                    "name_filter": filter.unwrap_or(""),
                    "include_types": include_types,
                    "format": format,
                    "variables": formatted_variables,
                    "message": format!("Found {} variables in scope: {}", variables.len(), scope)
                })))
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

impl GetVariablesTool {
    fn format_variables(variables: &[crate::lldb_manager::Variable], format: &str, include_types: bool) -> Value {
        match format {
            "compact" => {
                let compact: Vec<String> = variables.iter().map(|var| {
                    if include_types {
                        format!("{}: {} = {} ({})", var.name, var.var_type, var.value, var.scope)
                    } else {
                        format!("{} = {} ({})", var.name, var.value, var.scope)
                    }
                }).collect();
                json!(compact)
            },
            "names_only" => {
                let names: Vec<String> = variables.iter().map(|var| var.name.clone()).collect();
                json!(names)
            },
            "types_only" => {
                let types: Vec<String> = variables.iter().map(|var| {
                    format!("{}: {}", var.name, var.var_type)
                }).collect();
                json!(types)
            },
            _ => { // "detailed" format
                let detailed: Vec<Value> = variables.iter().map(|var| {
                    let mut obj = json!({
                        "name": var.name,
                        "value": var.value,
                        "scope": var.scope,
                        "is_argument": var.is_argument,
                        "category": if var.is_argument { "parameter" } else { &var.scope }
                    });
                    
                    if include_types {
                        obj["type"] = json!(var.var_type);
                    }
                    
                    obj
                }).collect();
                json!(detailed)
            }
        }
    }
}

// F0036: get_global_variables - Fully implemented
#[async_trait]
impl Tool for GetGlobalVariablesTool {
    fn name(&self) -> &'static str {
        "get_global_variables"
    }

    fn description(&self) -> &'static str {
        "Get global and static variables with their current values"
    }

    fn parameters(&self) -> Value {
        json!({
            "module_filter": {
                "type": "string",
                "description": "Filter globals by module/library name (optional)",
                "default": ""
            },
            "name_pattern": {
                "type": "string",
                "description": "Filter globals by name pattern (optional)",
                "default": ""
            },
            "include_static": {
                "type": "boolean",
                "description": "Include static variables",
                "default": true
            },
            "format": {
                "type": "string",
                "description": "Output format for global variable information",
                "enum": ["detailed", "compact", "names_only", "addresses_only"],
                "default": "detailed"
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let module_filter = arguments.get("module_filter")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());

        let name_pattern = arguments.get("name_pattern")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());

        let include_static = arguments.get("include_static")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let format = arguments.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("detailed");

        match lldb_manager.get_global_variables(module_filter) {
            Ok(mut global_vars) => {
                // Apply name filter
                if let Some(pattern) = name_pattern {
                    global_vars.retain(|var| var.name.contains(pattern));
                }

                // Apply static filter
                if !include_static {
                    global_vars.retain(|var| var.scope != "static");
                }

                let formatted_globals = Self::format_global_variables(&global_vars, format);
                
                let global_count = global_vars.iter().filter(|v| v.scope == "global").count();
                let static_count = global_vars.iter().filter(|v| v.scope == "static").count();

                Ok(ToolResponse::Json(json!({
                    "total_globals": global_vars.len(),
                    "global_variables": global_count,
                    "static_variables": static_count,
                    "module_filter": module_filter.unwrap_or(""),
                    "name_pattern": name_pattern.unwrap_or(""),
                    "include_static": include_static,
                    "format": format,
                    "variables": formatted_globals,
                    "message": format!("Found {} global variables", global_vars.len())
                })))
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

impl GetGlobalVariablesTool {
    fn format_global_variables(variables: &[crate::lldb_manager::Variable], format: &str) -> Value {
        match format {
            "compact" => {
                let compact: Vec<String> = variables.iter().map(|var| {
                    format!("{}: {} = {} [{}]", var.name, var.var_type, var.value, var.scope)
                }).collect();
                json!(compact)
            },
            "names_only" => {
                let names: Vec<String> = variables.iter().map(|var| var.name.clone()).collect();
                json!(names)
            },
            "addresses_only" => {
                let addresses: Vec<String> = variables.iter()
                    .filter_map(|var| {
                        if var.value.starts_with("0x") {
                            Some(var.value.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                json!(addresses)
            },
            _ => { // "detailed" format
                let detailed: Vec<Value> = variables.iter().map(|var| {
                    json!({
                        "name": var.name,
                        "type": var.var_type,
                        "value": var.value,
                        "scope": var.scope,
                        "is_pointer": var.value.starts_with("0x"),
                        "is_string": var.value.starts_with("\"") && var.value.ends_with("\""),
                        "storage_class": if var.scope == "static" { "static" } else { "global" }
                    })
                }).collect();
                json!(detailed)
            }
        }
    }
}

// F0037: evaluate_expression - Fully implemented
#[async_trait]
impl Tool for EvaluateExpressionTool {
    fn name(&self) -> &'static str {
        "evaluate_expression"
    }

    fn description(&self) -> &'static str {
        "Evaluate C/C++ expressions in the current debugging context"
    }

    fn parameters(&self) -> Value {
        json!({
            "expression": {
                "type": "string",
                "description": "C/C++ expression to evaluate (e.g., 'variable_name', 'ptr->field', 'array[index]')"
            },
            "format": {
                "type": "string",
                "description": "Output format for expression result",
                "enum": ["auto", "decimal", "hex", "binary", "string", "pointer", "boolean"],
                "default": "auto"
            },
            "timeout_ms": {
                "type": "integer",
                "description": "Expression evaluation timeout in milliseconds",
                "default": 5000,
                "minimum": 100,
                "maximum": 30000
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let expression = arguments.get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IncodeError::mcp("Missing expression parameter"))?;

        let format = arguments.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("auto");

        let timeout_ms = arguments.get("timeout_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(5000);

        // Validate expression for safety
        if Self::is_unsafe_expression(expression) {
            return Ok(ToolResponse::Error(format!("Unsafe expression detected: {}", expression)));
        }

        match lldb_manager.evaluate_expression(expression) {
            Ok(raw_result) => {
                let formatted_result = Self::format_expression_result(&raw_result, format);
                
                Ok(ToolResponse::Json(json!({
                    "expression": expression,
                    "raw_result": raw_result,
                    "formatted_result": formatted_result,
                    "format": format,
                    "timeout_ms": timeout_ms,
                    "success": true,
                    "message": format!("Expression '{}' evaluated successfully", expression)
                })))
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

impl EvaluateExpressionTool {
    fn is_unsafe_expression(expr: &str) -> bool {
        let dangerous_patterns = [
            "system(", "exec(", "fork(", "kill(", 
            "delete ", "free(", "malloc(", "realloc(",
            "memcpy(", "memset(", "strcpy(",
            "exit(", "abort(", "_exit(",
            "remove(", "unlink(", "rmdir(",
        ];
        
        dangerous_patterns.iter().any(|pattern| expr.contains(pattern))
    }

    fn format_expression_result(result: &str, format: &str) -> Value {
        match format {
            "decimal" => {
                if let Ok(num) = result.parse::<i64>() {
                    json!(num)
                } else {
                    json!(result)
                }
            },
            "hex" => {
                if let Ok(num) = result.parse::<u64>() {
                    json!(format!("0x{:x}", num))
                } else if result.starts_with("0x") {
                    json!(result)
                } else {
                    json!(format!("0x{}", result))
                }
            },
            "binary" => {
                if let Ok(num) = result.parse::<u64>() {
                    json!(format!("0b{:b}", num))
                } else {
                    json!(result)
                }
            },
            "string" => {
                let cleaned = result.trim_matches('"');
                json!(cleaned)
            },
            "pointer" => {
                if result.starts_with("0x") {
                    json!(result)
                } else {
                    json!(format!("0x{}", result))
                }
            },
            "boolean" => {
                let bool_val = match result.to_lowercase().as_str() {
                    "true" | "1" | "yes" => true,
                    "false" | "0" | "no" => false,
                    _ => result.parse::<i64>().unwrap_or(0) != 0,
                };
                json!(bool_val)
            },
            _ => { // "auto" format
                if result.starts_with("0x") {
                    json!({
                        "type": "pointer",
                        "value": result
                    })
                } else if result.starts_with('"') && result.ends_with('"') {
                    json!({
                        "type": "string",
                        "value": result.trim_matches('"')
                    })
                } else if let Ok(num) = result.parse::<i64>() {
                    json!({
                        "type": "integer",
                        "value": num,
                        "hex": format!("0x{:x}", num as u64)
                    })
                } else {
                    json!({
                        "type": "expression",
                        "value": result
                    })
                }
            }
        }
    }
}

// F0038: get_variable_info - Fully implemented
#[async_trait]
impl Tool for GetVariableInfoTool {
    fn name(&self) -> &'static str {
        "get_variable_info"
    }

    fn description(&self) -> &'static str {
        "Get detailed information about a specific variable including type, location, and metadata"
    }

    fn parameters(&self) -> Value {
        json!({
            "variable_name": {
                "type": "string",
                "description": "Name of the variable to inspect"
            },
            "include_address": {
                "type": "boolean",
                "description": "Include memory address information",
                "default": true
            },
            "include_declaration": {
                "type": "boolean",
                "description": "Include declaration file and line information",
                "default": true
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let variable_name = arguments.get("variable_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IncodeError::mcp("Missing variable_name parameter"))?;

        let include_address = arguments.get("include_address")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let include_declaration = arguments.get("include_declaration")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        match lldb_manager.get_variable_info(variable_name) {
            Ok(var_info) => {
                let mut response = json!({
                    "name": var_info.name,
                    "full_name": var_info.full_name,
                    "type": var_info.var_type,
                    "type_class": var_info.type_class,
                    "value": var_info.value,
                    "size": var_info.size,
                    "is_valid": var_info.is_valid,
                    "is_in_scope": var_info.is_in_scope,
                    "location": var_info.location,
                    "include_address": include_address,
                    "include_declaration": include_declaration
                });

                if include_address {
                    response["address"] = json!(format!("0x{:x}", var_info.address));
                    response["address_decimal"] = json!(var_info.address);
                }

                if include_declaration {
                    response["declaration"] = json!({
                        "file": var_info.declaration_file,
                        "line": var_info.declaration_line
                    });
                }

                Ok(ToolResponse::Json(json!({
                    "variable_info": response,
                    "message": format!("Retrieved detailed information for variable '{}'", variable_name)
                })))
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

// Placeholder implementations for remaining tools
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

impl_placeholder_tool!(SetVariableTool, "set_variable", "Modify variable value during debugging");
impl_placeholder_tool!(LookupSymbolTool, "lookup_symbol", "Find symbol information by name");

// Keep the old PlaceholderTool for compatibility with tool registry
pub struct PlaceholderTool;

#[async_trait]
impl Tool for PlaceholderTool {
    fn name(&self) -> &'static str {
        "variables_placeholder"
    }
    
    fn description(&self) -> &'static str {
        "Variable inspection placeholder - use specific variable tools instead"
    }
    
    fn parameters(&self) -> Value {
        json!({})
    }
    
    async fn execute(
        &self,
        _: HashMap<String, Value>,
        _: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        Ok(ToolResponse::Error("Use specific variable inspection tools like get_variables instead".to_string()))
    }
}