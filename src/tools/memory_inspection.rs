use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::error::{IncodeError, IncodeResult};
use crate::lldb_manager::LldbManager;
use super::{Tool, ToolResponse};

// Memory Inspection Tools (7 tools)
pub struct ReadMemoryTool;
pub struct WriteMemoryTool;
pub struct DisassembleTool;
pub struct SearchMemoryTool;
pub struct GetMemoryRegionsTool;
pub struct DumpMemoryTool;
pub struct MemoryMapTool;

// F0028: read_memory - Fully implemented
#[async_trait]
impl Tool for ReadMemoryTool {
    fn name(&self) -> &'static str {
        "read_memory"
    }

    fn description(&self) -> &'static str {
        "Read raw memory at address with size and format options"
    }

    fn parameters(&self) -> Value {
        json!({
            "address": {
                "type": "string",
                "description": "Memory address to read from (hexadecimal, e.g., '0x7fff12345678')"
            },
            "size": {
                "type": "integer",
                "description": "Number of bytes to read",
                "minimum": 1,
                "maximum": 1048576
            },
            "format": {
                "type": "string",
                "description": "Output format for memory data",
                "enum": ["hex", "ascii", "hex_ascii", "binary", "uint32", "uint64"],
                "default": "hex_ascii"
            },
            "bytes_per_line": {
                "type": "integer",
                "description": "Number of bytes to display per line (hex formats only)",
                "default": 16,
                "minimum": 4,
                "maximum": 64
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let address_str = arguments.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IncodeError::mcp("Missing address parameter"))?;

        let address = if address_str.starts_with("0x") {
            u64::from_str_radix(&address_str[2..], 16)
        } else {
            u64::from_str_radix(address_str, 16)
        }.map_err(|_| IncodeError::mcp(format!("Invalid address format: {}", address_str)))?;

        let size = arguments.get("size")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| IncodeError::mcp("Missing size parameter"))? as usize;

        let format = arguments.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("hex_ascii");

        let bytes_per_line = arguments.get("bytes_per_line")
            .and_then(|v| v.as_u64())
            .unwrap_or(16) as usize;

        match lldb_manager.read_memory(address, size) {
            Ok(memory_data) => {
                let formatted_output = Self::format_memory_data(&memory_data, address, format, bytes_per_line);
                
                Ok(ToolResponse::Json(json!({
                    "address": format!("0x{:x}", address),
                    "size": memory_data.len(),
                    "format": format,
                    "data": formatted_output,
                    "raw_bytes": memory_data.len(),
                    "message": format!("Read {} bytes from address 0x{:x}", memory_data.len(), address)
                })))
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

impl ReadMemoryTool {
    fn format_memory_data(data: &[u8], base_address: u64, format: &str, bytes_per_line: usize) -> Value {
        match format {
            "hex" => {
                let lines: Vec<String> = data.chunks(bytes_per_line)
                    .enumerate()
                    .map(|(i, chunk)| {
                        let addr = base_address + (i * bytes_per_line) as u64;
                        let hex: String = chunk.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
                        format!("0x{:08x}: {}", addr, hex)
                    })
                    .collect();
                json!(lines)
            },
            "ascii" => {
                let ascii: String = data.iter()
                    .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' })
                    .collect();
                json!(ascii)
            },
            "hex_ascii" => {
                let lines: Vec<String> = data.chunks(bytes_per_line)
                    .enumerate()
                    .map(|(i, chunk)| {
                        let addr = base_address + (i * bytes_per_line) as u64;
                        let hex: String = chunk.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
                        let ascii: String = chunk.iter()
                            .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' })
                            .collect();
                        format!("0x{:08x}: {:<48} |{}|", addr, hex, ascii)
                    })
                    .collect();
                json!(lines)
            },
            "binary" => {
                json!(data)
            },
            "uint32" => {
                let values: Vec<u32> = data.chunks_exact(4)
                    .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect();
                json!(values)
            },
            "uint64" => {
                let values: Vec<u64> = data.chunks_exact(8)
                    .map(|chunk| u64::from_le_bytes([
                        chunk[0], chunk[1], chunk[2], chunk[3],
                        chunk[4], chunk[5], chunk[6], chunk[7]
                    ]))
                    .collect();
                json!(values)
            },
            _ => json!(format!("Unknown format: {}", format))
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

// F0029: write_memory - Fully implemented
#[async_trait]
impl Tool for WriteMemoryTool {
    fn name(&self) -> &'static str {
        "write_memory"
    }

    fn description(&self) -> &'static str {
        "Write data to memory address"
    }

    fn parameters(&self) -> Value {
        json!({
            "address": {
                "type": "string",
                "description": "Memory address to write to (hexadecimal, e.g., '0x7fff12345678')"
            },
            "data": {
                "type": "string",
                "description": "Data to write - can be hex string (e.g., 'deadbeef') or ASCII string"
            },
            "format": {
                "type": "string",
                "description": "Input data format",
                "enum": ["hex", "ascii", "bytes"],
                "default": "hex"
            },
            "verify": {
                "type": "boolean",
                "description": "Read back and verify written data",
                "default": false
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let address_str = arguments.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IncodeError::mcp("Missing address parameter"))?;

        let address = if address_str.starts_with("0x") {
            u64::from_str_radix(&address_str[2..], 16)
        } else {
            u64::from_str_radix(address_str, 16)
        }.map_err(|_| IncodeError::mcp(format!("Invalid address format: {}", address_str)))?;

        let data_str = arguments.get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IncodeError::mcp("Missing data parameter"))?;

        let format = arguments.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("hex");

        let verify = arguments.get("verify")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Convert input data to bytes based on format
        let data_bytes = match format {
            "hex" => Self::parse_hex_data(data_str)?,
            "ascii" => data_str.as_bytes().to_vec(),
            "bytes" => {
                // Parse comma-separated byte values like "0xde,0xad,0xbe,0xef"
                Self::parse_byte_array(data_str)?
            }
            _ => return Ok(ToolResponse::Error(format!("Unknown format: {}", format)))
        };

        match lldb_manager.write_memory(address, &data_bytes) {
            Ok(bytes_written) => {
                let mut response = json!({
                    "address": format!("0x{:x}", address),
                    "bytes_written": bytes_written,
                    "format": format,
                    "success": true,
                    "message": format!("Wrote {} bytes to address 0x{:x}", bytes_written, address)
                });

                // Verify write if requested
                if verify {
                    match lldb_manager.read_memory(address, data_bytes.len()) {
                        Ok(read_data) => {
                            let verified = read_data == data_bytes;
                            response["verification"] = json!({
                                "requested": verified,
                                "success": verified,
                                "original_data": format!("{:02x?}", data_bytes),
                                "read_back_data": format!("{:02x?}", read_data)
                            });
                        }
                        Err(e) => {
                            response["verification"] = json!({
                                "requested": true,
                                "success": false,
                                "error": e.to_string()
                            });
                        }
                    }
                }

                Ok(ToolResponse::Json(response))
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

impl WriteMemoryTool {
    fn parse_hex_data(hex_str: &str) -> IncodeResult<Vec<u8>> {
        let cleaned = hex_str.replace("0x", "").replace(" ", "");
        if cleaned.len() % 2 != 0 {
            return Err(IncodeError::mcp("Hex data must have even number of characters"));
        }
        
        let mut bytes = Vec::new();
        for chunk in cleaned.as_bytes().chunks(2) {
            let hex_byte = std::str::from_utf8(chunk)
                .map_err(|_| IncodeError::mcp("Invalid hex characters"))?;
            let byte = u8::from_str_radix(hex_byte, 16)
                .map_err(|_| IncodeError::mcp(format!("Invalid hex byte: {}", hex_byte)))?;
            bytes.push(byte);
        }
        Ok(bytes)
    }

    fn parse_byte_array(byte_str: &str) -> IncodeResult<Vec<u8>> {
        let mut bytes = Vec::new();
        for part in byte_str.split(',') {
            let trimmed = part.trim();
            let byte = if trimmed.starts_with("0x") {
                u8::from_str_radix(&trimmed[2..], 16)
            } else {
                trimmed.parse::<u8>()
            }.map_err(|_| IncodeError::mcp(format!("Invalid byte value: {}", trimmed)))?;
            bytes.push(byte);
        }
        Ok(bytes)
    }
}
// F0030: disassemble - Fully implemented
#[async_trait]
impl Tool for DisassembleTool {
    fn name(&self) -> &'static str {
        "disassemble"
    }

    fn description(&self) -> &'static str {
        "Disassemble instructions at address or function name"
    }

    fn parameters(&self) -> Value {
        json!({
            "address": {
                "type": "string",
                "description": "Memory address to disassemble from (hexadecimal, e.g., '0x7fff12345678') or function name"
            },
            "count": {
                "type": "integer",
                "description": "Number of instructions to disassemble",
                "default": 10,
                "minimum": 1,
                "maximum": 1000
            },
            "format": {
                "type": "string",
                "description": "Disassembly output format",
                "enum": ["default", "verbose", "compact"],
                "default": "default"
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let address_str = arguments.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IncodeError::mcp("Missing address parameter"))?;

        let count = arguments.get("count")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as u32;

        let format = arguments.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        // Parse address - could be hex address or function name
        let address = if address_str.starts_with("0x") {
            u64::from_str_radix(&address_str[2..], 16)
                .map_err(|_| IncodeError::mcp(format!("Invalid address format: {}", address_str)))?
        } else if address_str.chars().all(|c| c.is_ascii_hexdigit()) {
            u64::from_str_radix(address_str, 16)
                .map_err(|_| IncodeError::mcp(format!("Invalid address format: {}", address_str)))?
        } else {
            // TODO: Implement function name to address resolution
            return Ok(ToolResponse::Error(format!("Function name resolution not yet implemented: {}", address_str)));
        };

        match lldb_manager.disassemble(address, count) {
            Ok(instructions) => {
                let formatted_instructions = Self::format_disassembly(&instructions, format);
                
                Ok(ToolResponse::Json(json!({
                    "address": format!("0x{:x}", address),
                    "count": instructions.len(),
                    "format": format,
                    "instructions": formatted_instructions,
                    "message": format!("Disassembled {} instructions from address 0x{:x}", instructions.len(), address)
                })))
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

impl DisassembleTool {
    fn format_disassembly(instructions: &[String], format: &str) -> Value {
        match format {
            "compact" => {
                let compact: Vec<String> = instructions.iter()
                    .map(|inst| inst.split(": ").nth(1).unwrap_or(inst).to_string())
                    .collect();
                json!(compact)
            },
            "verbose" => {
                let verbose: Vec<String> = instructions.iter()
                    .enumerate()
                    .map(|(i, inst)| format!("[{}] {}", i + 1, inst))
                    .collect();
                json!(verbose)
            },
            _ => json!(instructions) // default format
        }
    }
}
// F0031: search_memory - Fully implemented
#[async_trait]
impl Tool for SearchMemoryTool {
    fn name(&self) -> &'static str {
        "search_memory"
    }

    fn description(&self) -> &'static str {
        "Search for byte patterns in process memory"
    }

    fn parameters(&self) -> Value {
        json!({
            "pattern": {
                "type": "string",
                "description": "Byte pattern to search for - hex string (e.g., 'deadbeef') or ASCII string"
            },
            "pattern_format": {
                "type": "string",
                "description": "Format of the search pattern",
                "enum": ["hex", "ascii", "bytes"],
                "default": "hex"
            },
            "start_address": {
                "type": "string",
                "description": "Starting address for search (hexadecimal, optional)",
                "default": "0x100000000"
            },
            "search_size": {
                "type": "integer",
                "description": "Size of memory region to search in bytes",
                "default": 1048576,
                "maximum": 104857600
            },
            "max_results": {
                "type": "integer",
                "description": "Maximum number of matches to return",
                "default": 100,
                "maximum": 1000
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let pattern_str = arguments.get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IncodeError::mcp("Missing pattern parameter"))?;

        let pattern_format = arguments.get("pattern_format")
            .and_then(|v| v.as_str())
            .unwrap_or("hex");

        let start_address_str = arguments.get("start_address")
            .and_then(|v| v.as_str())
            .unwrap_or("0x100000000");

        let start_address = if start_address_str.starts_with("0x") {
            u64::from_str_radix(&start_address_str[2..], 16)
        } else {
            u64::from_str_radix(start_address_str, 16)
        }.map_err(|_| IncodeError::mcp(format!("Invalid start address: {}", start_address_str)))?;

        let search_size = arguments.get("search_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(1048576) as usize;

        let max_results = arguments.get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(100) as usize;

        // Convert pattern to bytes based on format
        let pattern_bytes = match pattern_format {
            "hex" => Self::parse_hex_pattern(pattern_str)?,
            "ascii" => pattern_str.as_bytes().to_vec(),
            "bytes" => Self::parse_byte_pattern(pattern_str)?,
            _ => return Ok(ToolResponse::Error(format!("Unknown pattern format: {}", pattern_format)))
        };

        match lldb_manager.search_memory(&pattern_bytes, Some(start_address), Some(search_size)) {
            Ok(matches) => {
                let limited_matches: Vec<u64> = matches.into_iter().take(max_results).collect();
                let match_addresses: Vec<String> = limited_matches.iter()
                    .map(|addr| format!("0x{:x}", addr))
                    .collect();

                Ok(ToolResponse::Json(json!({
                    "pattern": pattern_str,
                    "pattern_format": pattern_format,
                    "pattern_bytes": format!("{:02x?}", pattern_bytes),
                    "start_address": format!("0x{:x}", start_address),
                    "search_size": search_size,
                    "matches_found": limited_matches.len(),
                    "matches": match_addresses,
                    "truncated": limited_matches.len() == max_results,
                    "message": format!("Found {} matches for pattern in {} bytes of memory", 
                                     limited_matches.len(), search_size)
                })))
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

impl SearchMemoryTool {
    fn parse_hex_pattern(hex_str: &str) -> IncodeResult<Vec<u8>> {
        let cleaned = hex_str.replace("0x", "").replace(" ", "");
        if cleaned.len() % 2 != 0 {
            return Err(IncodeError::mcp("Hex pattern must have even number of characters"));
        }
        
        let mut bytes = Vec::new();
        for chunk in cleaned.as_bytes().chunks(2) {
            let hex_byte = std::str::from_utf8(chunk)
                .map_err(|_| IncodeError::mcp("Invalid hex characters"))?;
            let byte = u8::from_str_radix(hex_byte, 16)
                .map_err(|_| IncodeError::mcp(format!("Invalid hex byte: {}", hex_byte)))?;
            bytes.push(byte);
        }
        Ok(bytes)
    }

    fn parse_byte_pattern(byte_str: &str) -> IncodeResult<Vec<u8>> {
        let mut bytes = Vec::new();
        for part in byte_str.split(',') {
            let trimmed = part.trim();
            let byte = if trimmed.starts_with("0x") {
                u8::from_str_radix(&trimmed[2..], 16)
            } else {
                trimmed.parse::<u8>()
            }.map_err(|_| IncodeError::mcp(format!("Invalid byte value: {}", trimmed)))?;
            bytes.push(byte);
        }
        Ok(bytes)
    }
}
// F0032: get_memory_regions - Fully implemented
#[async_trait]
impl Tool for GetMemoryRegionsTool {
    fn name(&self) -> &'static str {
        "get_memory_regions"
    }

    fn description(&self) -> &'static str {
        "List memory mappings and permissions for the target process"
    }

    fn parameters(&self) -> Value {
        json!({
            "filter": {
                "type": "string",
                "description": "Filter regions by name or permission (optional)",
                "default": ""
            },
            "include_empty": {
                "type": "boolean",
                "description": "Include empty or unmapped regions",
                "default": false
            },
            "format": {
                "type": "string",
                "description": "Output format for region information",
                "enum": ["detailed", "compact", "addresses_only"],
                "default": "detailed"
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let filter = arguments.get("filter")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let include_empty = arguments.get("include_empty")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let format = arguments.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("detailed");

        match lldb_manager.get_memory_regions() {
            Ok(regions) => {
                // Apply filter if specified
                let filtered_regions: Vec<_> = if filter.is_empty() {
                    regions
                } else {
                    regions.into_iter().filter(|region| {
                        region.permissions.contains(filter) ||
                        region.name.as_ref().map_or(false, |name| name.contains(filter))
                    }).collect()
                };

                // Filter out empty regions if not requested
                let final_regions: Vec<_> = if include_empty {
                    filtered_regions
                } else {
                    filtered_regions.into_iter().filter(|region| region.size > 0).collect()
                };

                let formatted_regions = Self::format_regions(&final_regions, format);

                Ok(ToolResponse::Json(json!({
                    "regions": formatted_regions,
                    "total_regions": final_regions.len(),
                    "filter_applied": !filter.is_empty(),
                    "filter": filter,
                    "include_empty": include_empty,
                    "format": format,
                    "message": format!("Found {} memory regions", final_regions.len())
                })))
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

impl GetMemoryRegionsTool {
    fn format_regions(regions: &[crate::lldb_manager::MemoryRegion], format: &str) -> Value {
        match format {
            "compact" => {
                let compact: Vec<String> = regions.iter().map(|region| {
                    format!("0x{:x}-0x{:x} {} {}", 
                           region.start_address, 
                           region.end_address, 
                           region.permissions,
                           region.name.as_ref().unwrap_or(&"<unnamed>".to_string()))
                }).collect();
                json!(compact)
            },
            "addresses_only" => {
                let addresses: Vec<String> = regions.iter().map(|region| {
                    format!("0x{:x}", region.start_address)
                }).collect();
                json!(addresses)
            },
            _ => { // "detailed" format
                let detailed: Vec<Value> = regions.iter().map(|region| {
                    json!({
                        "start_address": format!("0x{:x}", region.start_address),
                        "end_address": format!("0x{:x}", region.end_address),
                        "size": region.size,
                        "size_kb": region.size / 1024,
                        "permissions": region.permissions,
                        "name": region.name.as_ref().unwrap_or(&"<unnamed>".to_string()),
                        "readable": region.permissions.contains('r'),
                        "writable": region.permissions.contains('w'),
                        "executable": region.permissions.contains('x')
                    })
                }).collect();
                json!(detailed)
            }
        }
    }
}
// F0033: dump_memory - Fully implemented
#[async_trait]
impl Tool for DumpMemoryTool {
    fn name(&self) -> &'static str {
        "dump_memory"
    }

    fn description(&self) -> &'static str {
        "Dump memory region to file for offline analysis"
    }

    fn parameters(&self) -> Value {
        json!({
            "address": {
                "type": "string",
                "description": "Memory address to start dump from (hexadecimal, e.g., '0x7fff12345678')"
            },
            "size": {
                "type": "integer",
                "description": "Number of bytes to dump",
                "minimum": 1,
                "maximum": 104857600
            },
            "file_path": {
                "type": "string",
                "description": "Path to output file for memory dump"
            },
            "format": {
                "type": "string",
                "description": "Output file format",
                "enum": ["raw", "hex", "hexdump"],
                "default": "raw"
            },
            "overwrite": {
                "type": "boolean",
                "description": "Overwrite existing file if it exists",
                "default": false
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let address_str = arguments.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IncodeError::mcp("Missing address parameter"))?;

        let address = if address_str.starts_with("0x") {
            u64::from_str_radix(&address_str[2..], 16)
        } else {
            u64::from_str_radix(address_str, 16)
        }.map_err(|_| IncodeError::mcp(format!("Invalid address format: {}", address_str)))?;

        let size = arguments.get("size")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| IncodeError::mcp("Missing size parameter"))? as usize;

        let file_path = arguments.get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IncodeError::mcp("Missing file_path parameter"))?;

        let format = arguments.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("raw");

        let overwrite = arguments.get("overwrite")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Check if file exists and overwrite is not allowed
        if std::path::Path::new(file_path).exists() && !overwrite {
            return Ok(ToolResponse::Error(format!("File {} already exists. Use overwrite=true to replace it.", file_path)));
        }

        match format {
            "raw" => {
                // Direct binary dump
                match lldb_manager.dump_memory_to_file(address, size, file_path) {
                    Ok(bytes_written) => {
                        Ok(ToolResponse::Json(json!({
                            "address": format!("0x{:x}", address),
                            "size_requested": size,
                            "bytes_written": bytes_written,
                            "file_path": file_path,
                            "format": format,
                            "success": true,
                            "message": format!("Dumped {} bytes from 0x{:x} to {}", bytes_written, address, file_path)
                        })))
                    }
                    Err(e) => Ok(ToolResponse::Error(e.to_string())),
                }
            },
            "hex" | "hexdump" => {
                // Read memory and format as hex
                match lldb_manager.read_memory(address, size) {
                    Ok(memory_data) => {
                        let formatted_content = if format == "hex" {
                            Self::format_as_hex(&memory_data)
                        } else {
                            Self::format_as_hexdump(&memory_data, address)
                        };

                        match std::fs::write(file_path, formatted_content) {
                            Ok(_) => {
                                Ok(ToolResponse::Json(json!({
                                    "address": format!("0x{:x}", address),
                                    "size_requested": size,
                                    "bytes_read": memory_data.len(),
                                    "file_path": file_path,
                                    "format": format,
                                    "success": true,
                                    "message": format!("Dumped {} bytes from 0x{:x} to {} in {} format", memory_data.len(), address, file_path, format)
                                })))
                            }
                            Err(e) => Ok(ToolResponse::Error(format!("Failed to write file: {}", e))),
                        }
                    }
                    Err(e) => Ok(ToolResponse::Error(e.to_string())),
                }
            },
            _ => Ok(ToolResponse::Error(format!("Unknown format: {}", format)))
        }
    }
}

impl DumpMemoryTool {
    fn format_as_hex(data: &[u8]) -> String {
        data.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join("")
    }

    fn format_as_hexdump(data: &[u8], base_address: u64) -> String {
        let mut result = String::new();
        for (i, chunk) in data.chunks(16).enumerate() {
            let addr = base_address + (i * 16) as u64;
            let hex: String = chunk.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
            let ascii: String = chunk.iter()
                .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' })
                .collect();
            result.push_str(&format!("{:08x}: {:<48} |{}|\n", addr, hex, ascii));
        }
        result
    }
}
// F0034: memory_map - Fully implemented
#[async_trait]
impl Tool for MemoryMapTool {
    fn name(&self) -> &'static str {
        "memory_map"
    }

    fn description(&self) -> &'static str {
        "Get detailed memory map with segments, load addresses, and ASLR information"
    }

    fn parameters(&self) -> Value {
        json!({
            "include_segments": {
                "type": "boolean",
                "description": "Include detailed segment information",
                "default": true
            },
            "filter_type": {
                "type": "string",
                "description": "Filter segments by type (optional)",
                "enum": ["TEXT", "DATA", "STACK", "HEAP", "PAGEZERO", ""],
                "default": ""
            },
            "format": {
                "type": "string",
                "description": "Output format for memory map",
                "enum": ["detailed", "summary", "addresses_only"],
                "default": "detailed"
            },
            "sort_by": {
                "type": "string",
                "description": "Sort segments by address, size, or name",
                "enum": ["address", "size", "name"],
                "default": "address"
            }
        })
    }

    async fn execute(
        &self,
        arguments: HashMap<String, Value>,
        lldb_manager: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        let include_segments = arguments.get("include_segments")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let filter_type = arguments.get("filter_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let format = arguments.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("detailed");

        let sort_by = arguments.get("sort_by")
            .and_then(|v| v.as_str())
            .unwrap_or("address");

        match lldb_manager.get_memory_map() {
            Ok(memory_map) => {
                let mut segments = memory_map.segments.clone();
                
                // Apply filter if specified
                if !filter_type.is_empty() {
                    segments.retain(|seg| seg.segment_type.contains(filter_type) || seg.name.contains(filter_type));
                }

                // Sort segments
                match sort_by {
                    "size" => segments.sort_by(|a, b| b.vm_size.cmp(&a.vm_size)),
                    "name" => segments.sort_by(|a, b| a.name.cmp(&b.name)),
                    _ => segments.sort_by(|a, b| a.vm_address.cmp(&b.vm_address)), // default: address
                }

                let formatted_output = if include_segments {
                    Self::format_memory_map(&segments, format, &memory_map)
                } else {
                    Self::format_summary_only(&memory_map)
                };

                Ok(ToolResponse::Json(json!({
                    "total_segments": memory_map.total_segments,
                    "total_vm_size": memory_map.total_vm_size,
                    "total_vm_size_mb": memory_map.total_vm_size / (1024 * 1024),
                    "load_address": format!("0x{:x}", memory_map.load_address),
                    "aslr_slide": memory_map.slide,
                    "filtered_segments": segments.len(),
                    "filter_applied": !filter_type.is_empty(),
                    "filter_type": filter_type,
                    "format": format,
                    "sort_by": sort_by,
                    "segments": formatted_output,
                    "message": format!("Memory map contains {} segments ({} after filtering)", memory_map.total_segments, segments.len())
                })))
            }
            Err(e) => Ok(ToolResponse::Error(e.to_string())),
        }
    }
}

impl MemoryMapTool {
    fn format_memory_map(segments: &[crate::lldb_manager::MemorySegment], format: &str, map: &crate::lldb_manager::MemoryMap) -> Value {
        match format {
            "summary" => {
                let summary: Vec<String> = segments.iter().map(|seg| {
                    format!("{}: 0x{:x}-0x{:x} ({}KB) {}", 
                           seg.name,
                           seg.vm_address, 
                           seg.vm_address + seg.vm_size,
                           seg.vm_size / 1024,
                           seg.max_protection)
                }).collect();
                json!(summary)
            },
            "addresses_only" => {
                let addresses: Vec<String> = segments.iter().map(|seg| {
                    format!("0x{:x}", seg.vm_address)
                }).collect();
                json!(addresses)
            },
            _ => { // "detailed" format
                let detailed: Vec<Value> = segments.iter().map(|seg| {
                    json!({
                        "name": seg.name,
                        "type": seg.segment_type,
                        "vm_address": format!("0x{:x}", seg.vm_address),
                        "vm_end": format!("0x{:x}", seg.vm_address + seg.vm_size),
                        "vm_size": seg.vm_size,
                        "vm_size_kb": seg.vm_size / 1024,
                        "vm_size_mb": seg.vm_size / (1024 * 1024),
                        "file_offset": format!("0x{:x}", seg.file_offset),
                        "file_size": seg.file_size,
                        "max_protection": seg.max_protection,
                        "initial_protection": seg.initial_protection,
                        "readable": seg.max_protection.contains('r'),
                        "writable": seg.max_protection.contains('w'),
                        "executable": seg.max_protection.contains('x')
                    })
                }).collect();
                json!({
                    "overview": {
                        "load_address": format!("0x{:x}", map.load_address),
                        "total_segments": map.total_segments,
                        "total_vm_size": map.total_vm_size,
                        "aslr_slide": map.slide
                    },
                    "segments": detailed
                })
            }
        }
    }

    fn format_summary_only(map: &crate::lldb_manager::MemoryMap) -> Value {
        json!({
            "load_address": format!("0x{:x}", map.load_address),
            "total_segments": map.total_segments,
            "total_vm_size": map.total_vm_size,
            "total_vm_size_mb": map.total_vm_size / (1024 * 1024),
            "aslr_slide": map.slide
        })
    }
}

// Keep the old PlaceholderTool for compatibility with tool registry
pub struct PlaceholderTool;

#[async_trait]
impl Tool for PlaceholderTool {
    fn name(&self) -> &'static str {
        "memory_placeholder"
    }
    
    fn description(&self) -> &'static str {
        "Memory inspection placeholder - use specific memory tools instead"
    }
    
    fn parameters(&self) -> Value {
        json!({})
    }
    
    async fn execute(
        &self,
        _: HashMap<String, Value>,
        _: &mut LldbManager,
    ) -> IncodeResult<ToolResponse> {
        Ok(ToolResponse::Error("Use specific memory inspection tools like read_memory instead".to_string()))
    }
}
