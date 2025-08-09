use clap::{Arg, Command};
use tracing::{info, error};
use tracing_subscriber::EnvFilter;

mod mcp_server;
mod lldb_manager;
mod tools;
mod error;

use crate::mcp_server::McpServer;
use crate::error::IncodeResult;

#[tokio::main]
async fn main() -> IncodeResult<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("incode=info"))
        )
        .init();

    let matches = Command::new("incode")
        .version(env!("CARGO_PKG_VERSION"))
        .about("InCode - Comprehensive MCP server for LLDB debugging automation")
        .arg(
            Arg::new("debug")
                .long("debug")
                .short('d')
                .help("Enable debug logging")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("lldb-path")
                .long("lldb-path")
                .help("Path to LLDB executable")
                .value_name("PATH")
        )
        .arg(
            Arg::new("debug-tool")
                .long("debug-tool")
                .help("Debug specific tool number")
                .value_name("NUM")
        )
        .get_matches();

    if matches.get_flag("debug") {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::new("debug"))
            .init();
    }

    // Debug specific tool if requested
    if let Some(tool_num_str) = matches.get_one::<String>("debug-tool") {
        use crate::tools::ToolRegistry;
        let tool_num: usize = tool_num_str.parse().unwrap_or(25);
        let registry = ToolRegistry::new();
        let tools = registry.get_tool_list();
        
        println!("Total tools: {}", tools.len());
        
        if tool_num < tools.len() {
            let tool = &tools[tool_num];
            println!("=== TOOL {} SCHEMA ===", tool_num);
            println!("Name: {}", tool["name"]);
            println!("Description: {}", tool["description"]);
            println!("{}", serde_json::to_string_pretty(&tool["inputSchema"]).unwrap());
            
            // Check for schema issues
            let schema = &tool["inputSchema"];
            if let Some(properties) = schema.get("properties") {
                for (prop_name, prop_def) in properties.as_object().unwrap() {
                    if let Some(enum_val) = prop_def.get("enum") {
                        if let Some(enum_array) = enum_val.as_array() {
                            for val in enum_array {
                                if val.as_str() == Some("") {
                                    println!("  ❌ Empty string in enum: {}", prop_name);
                                }
                            }
                        }
                    }
                    if prop_def.get("optional").is_some() {
                        println!("  ❌ Invalid 'optional' property: {}", prop_name);
                    }
                    if prop_def.get("required").is_some() {
                        println!("  ❌ Invalid 'required' property: {}", prop_name);
                    }
                }
            }
        }
        return Ok(());
    }

    info!("Starting InCode MCP Server v{}", env!("CARGO_PKG_VERSION"));
    info!("Comprehensive LLDB debugging automation with 65+ tools");

    // Initialize MCP server
    let lldb_path = matches.get_one::<String>("lldb-path").cloned();
    let mut server = McpServer::new(lldb_path)?;

    // Start the MCP server
    match server.run().await {
        Ok(_) => {
            info!("InCode MCP Server shutdown gracefully");
            Ok(())
        }
        Err(e) => {
            error!("InCode MCP Server error: {}", e);
            Err(e)
        }
    }
}