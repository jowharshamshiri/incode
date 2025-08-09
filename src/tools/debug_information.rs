use crate::lldb_manager::{LldbManager, SourceCode, FunctionInfo, SourceLocation, DebugInfo};
use crate::error::IncodeResult;
use crate::tools::{Tool, ToolResponse};
use std::collections::HashMap;
use serde_json::{json, Value};
use async_trait::async_trait;
use tracing::{debug, error};

pub fn get_source_code(
    lldb_manager: &LldbManager,
    arguments: HashMap<String, Value>,
) -> IncodeResult<Value> {
    debug!("Debug Information: get_source_code called with args: {:?}", arguments);
    
    let address = arguments.get("address")
        .and_then(|v| {
            if let Some(s) = v.as_str() {
                if s.starts_with("0x") || s.starts_with("0X") {
                    u64::from_str_radix(&s[2..], 16).ok()
                } else {
                    s.parse::<u64>().ok()
                }
            } else {
                v.as_u64()
            }
        });
        
    let context_lines = arguments.get("context_lines")
        .and_then(|v| v.as_u64())
        .unwrap_or(5) as u32;
        
    let include_addresses = arguments.get("include_addresses")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    match lldb_manager.get_source_code(address, context_lines) {
        Ok(source_code) => {
            debug!("Retrieved source code from {} with {} lines", 
                source_code.file_path, source_code.lines.len());
            
            let lines_json: Vec<Value> = source_code.lines.iter().map(|line| {
                let mut line_obj = json!({
                    "line_number": line.line_number,
                    "content": line.content,
                    "is_current": line.is_current,
                    "has_breakpoint": line.has_breakpoint
                });
                
                if include_addresses {
                    line_obj["address"] = if let Some(addr) = line.address {
                        json!(format!("0x{:x}", addr))
                    } else {
                        json!(null)
                    };
                }
                
                line_obj
            }).collect();
            
            Ok(json!({
                "success": true,
                "source_code": {
                    "file_path": source_code.file_path,
                    "lines": lines_json,
                    "start_line": source_code.start_line,
                    "end_line": source_code.end_line,
                    "current_line": source_code.current_line,
                    "total_lines": source_code.total_lines,
                    "context_lines": context_lines
                },
                "metadata": {
                    "address_requested": address.map(|a| format!("0x{:x}", a)),
                    "lines_returned": source_code.lines.len(),
                    "has_current_location": source_code.current_line.is_some()
                }
            }))
        }
        Err(e) => {
            error!("Failed to get source code: {}", e);
            Ok(json!({
                "success": false,
                "error": e.to_string(),
                "address": address.map(|a| format!("0x{:x}", a))
            }))
        }
    }
}

pub fn list_functions(
    lldb_manager: &LldbManager,
    arguments: HashMap<String, Value>,
) -> IncodeResult<Value> {
    debug!("Debug Information: list_functions called with args: {:?}", arguments);
    
    let module_filter = arguments.get("module_filter")
        .and_then(|v| v.as_str());
        
    let name_filter = arguments.get("name_filter")
        .and_then(|v| v.as_str());
        
    let include_addresses = arguments.get("include_addresses")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
        
    let include_source_info = arguments.get("include_source_info")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    match lldb_manager.list_functions(module_filter) {
        Ok(functions) => {
            debug!("Found {} functions", functions.len());
            
            // Apply name filter if specified
            let filtered_functions: Vec<&FunctionInfo> = if let Some(filter) = name_filter {
                functions.iter()
                    .filter(|f| f.name.to_lowercase().contains(&filter.to_lowercase()))
                    .collect()
            } else {
                functions.iter().collect()
            };
            
            let functions_json: Vec<Value> = filtered_functions.iter().map(|func| {
                let mut func_obj = json!({
                    "name": func.name,
                    "mangled_name": func.mangled_name,
                    "is_inline": func.is_inline,
                    "return_type": func.return_type
                });
                
                if include_addresses {
                    func_obj["start_address"] = json!(format!("0x{:x}", func.start_address));
                    func_obj["end_address"] = if let Some(end) = func.end_address {
                        json!(format!("0x{:x}", end))
                    } else {
                        json!(null)
                    };
                    func_obj["size"] = if let Some(size) = func.size {
                        json!(size)
                    } else {
                        json!(null)
                    };
                }
                
                if include_source_info {
                    func_obj["file_path"] = json!(func.file_path);
                    func_obj["line_number"] = json!(func.line_number);
                }
                
                func_obj
            }).collect();
            
            Ok(json!({
                "success": true,
                "functions": functions_json,
                "total_count": filtered_functions.len(),
                "filters_applied": {
                    "module_filter": module_filter,
                    "name_filter": name_filter
                },
                "metadata": {
                    "include_addresses": include_addresses,
                    "include_source_info": include_source_info
                }
            }))
        }
        Err(e) => {
            error!("Failed to list functions: {}", e);
            Ok(json!({
                "success": false,
                "error": e.to_string(),
                "functions": []
            }))
        }
    }
}

pub fn get_line_info(
    lldb_manager: &LldbManager,
    arguments: HashMap<String, Value>,
) -> IncodeResult<Value> {
    debug!("Debug Information: get_line_info called with args: {:?}", arguments);
    
    let address = arguments.get("address")
        .and_then(|v| {
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
        .ok_or_else(|| crate::error::IncodeError::invalid_parameter("address is required and must be a number or hex string"))?;
    
    match lldb_manager.get_line_info(address) {
        Ok(location) => {
            debug!("Got line info for 0x{:x}: {}:{}", address, location.file_path, location.line_number);
            
            let basename = location.file_path.split('/').last().unwrap_or(&location.file_path);
            let path_parts: Vec<&str> = location.file_path.split('/').collect();
            let directory = if path_parts.len() > 1 {
                path_parts[..path_parts.len()-1].join("/")
            } else {
                "".to_string()
            };
            let location_string = format!("{}:{}", location.file_path, location.line_number);
            
            Ok(json!({
                "success": true,
                "line_info": {
                    "address": format!("0x{:x}", location.address),
                    "file_path": location.file_path,
                    "line_number": location.line_number,
                    "column": location.column,
                    "function_name": location.function_name,
                    "is_valid": location.is_valid
                },
                "source_location": {
                    "basename": basename,
                    "directory": directory,
                    "location_string": location_string
                }
            }))
        }
        Err(e) => {
            error!("Failed to get line info for 0x{:x}: {}", address, e);
            Ok(json!({
                "success": false,
                "error": e.to_string(),
                "address": format!("0x{:x}", address)
            }))
        }
    }
}

pub fn get_debug_info(
    lldb_manager: &LldbManager,
    arguments: HashMap<String, Value>,
) -> IncodeResult<Value> {
    debug!("Debug Information: get_debug_info called with args: {:?}", arguments);
    
    let include_compilation_units = arguments.get("include_compilation_units")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
        
    let include_detailed_stats = arguments.get("include_detailed_stats")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    match lldb_manager.get_debug_info() {
        Ok(debug_info) => {
            debug!("Retrieved debug info: {} symbols, {} format", 
                debug_info.symbol_count, debug_info.debug_format);
            
            let mut result = json!({
                "success": true,
                "debug_info": {
                    "has_debug_symbols": debug_info.has_debug_symbols,
                    "debug_format": debug_info.debug_format,
                    "symbol_count": debug_info.symbol_count,
                    "line_table_count": debug_info.line_table_count,
                    "function_count": debug_info.function_count,
                    "compilation_unit_count": debug_info.compilation_units.len()
                }
            });
            
            if include_compilation_units {
                let comp_units: Vec<Value> = debug_info.compilation_units.iter().map(|cu| {
                    json!({
                        "file_path": cu.file_path,
                        "producer": cu.producer,
                        "language": cu.language,
                        "low_pc": format!("0x{:x}", cu.low_pc),
                        "high_pc": format!("0x{:x}", cu.high_pc),
                        "line_count": cu.line_count,
                        "address_range_size": cu.high_pc - cu.low_pc
                    })
                }).collect();
                
                result["debug_info"]["compilation_units"] = json!(comp_units);
            }
            
            if include_detailed_stats {
                let total_address_space: u64 = debug_info.compilation_units.iter()
                    .map(|cu| cu.high_pc - cu.low_pc)
                    .sum();
                    
                let languages: Vec<String> = debug_info.compilation_units.iter()
                    .filter_map(|cu| cu.language.as_ref())
                    .cloned()
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();
                    
                result["debug_info"]["detailed_stats"] = json!({
                    "total_address_space": total_address_space,
                    "average_symbols_per_unit": if debug_info.compilation_units.len() > 0 {
                        debug_info.symbol_count / debug_info.compilation_units.len() as u32
                    } else {
                        0
                    },
                    "languages_detected": languages,
                    "has_line_tables": debug_info.line_table_count > 0,
                    "debug_density": if total_address_space > 0 {
                        debug_info.symbol_count as f64 / total_address_space as f64
                    } else {
                        0.0
                    }
                });
            }
            
            Ok(result)
        }
        Err(e) => {
            error!("Failed to get debug info: {}", e);
            Ok(json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

// Tool implementations for MCP protocol

pub struct GetSourceCodeTool;

#[async_trait]
impl Tool for GetSourceCodeTool {
    fn name(&self) -> &'static str {
        "get_source_code"
    }
    
    fn description(&self) -> &'static str {
        "Get source code around current location"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "address": {
                "type": ["number", "string"],
                "description": "Specific address to get source for (hex string or number, uses current location if not provided)"
            },
            "context_lines": {
                "type": "number", 
                "description": "Number of context lines before/after current line",
                "default": 5,
                "minimum": 1,
                "maximum": 50
            },
            "include_addresses": {
                "type": "boolean",
                "description": "Include memory addresses for each line",
                "default": false
            }
        })
    }
    
    async fn execute(&self, arguments: HashMap<String, Value>, manager: &mut LldbManager) -> IncodeResult<ToolResponse> {
        match get_source_code(manager, arguments) {
            Ok(result) => Ok(ToolResponse::Success(result.to_string())),
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

pub struct ListFunctionsTool;

#[async_trait]
impl Tool for ListFunctionsTool {
    fn name(&self) -> &'static str {
        "list_functions"
    }
    
    fn description(&self) -> &'static str {
        "List all functions in target with addresses"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "module_filter": {
                "type": "string",
                "description": "Filter functions by module name"
            },
            "name_filter": {
                "type": "string",
                "description": "Filter functions by name (case-insensitive substring match)"
            },
            "include_addresses": {
                "type": "boolean",
                "description": "Include function start/end addresses and size",
                "default": true
            },
            "include_source_info": {
                "type": "boolean",
                "description": "Include source file path and line number",
                "default": false
            }
        })
    }
    
    async fn execute(&self, arguments: HashMap<String, Value>, manager: &mut LldbManager) -> IncodeResult<ToolResponse> {
        match list_functions(manager, arguments) {
            Ok(result) => Ok(ToolResponse::Success(result.to_string())),
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

pub struct GetLineInfoTool;

#[async_trait]
impl Tool for GetLineInfoTool {
    fn name(&self) -> &'static str {
        "get_line_info"
    }
    
    fn description(&self) -> &'static str {
        "Get source line information for address"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "address": {
                "type": ["number", "string"],
                "description": "Memory address to lookup (hex string or number)"
            }
        })
    }
    
    async fn execute(&self, arguments: HashMap<String, Value>, manager: &mut LldbManager) -> IncodeResult<ToolResponse> {
        match get_line_info(manager, arguments) {
            Ok(result) => Ok(ToolResponse::Success(result.to_string())),
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

pub struct GetDebugInfoTool;

#[async_trait]
impl Tool for GetDebugInfoTool {
    fn name(&self) -> &'static str {
        "get_debug_info"
    }
    
    fn description(&self) -> &'static str {
        "Get debug information summary"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "include_compilation_units": {
                "type": "boolean",
                "description": "Include detailed compilation unit information",
                "default": false
            },
            "include_detailed_stats": {
                "type": "boolean",
                "description": "Include detailed debug statistics and analysis",
                "default": false
            }
        })
    }
    
    async fn execute(&self, arguments: HashMap<String, Value>, manager: &mut LldbManager) -> IncodeResult<ToolResponse> {
        match get_debug_info(manager, arguments) {
            Ok(result) => Ok(ToolResponse::Success(result.to_string())),
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}