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
            Arg::new("debug-schemas")
                .long("debug-schemas")
                .help("Debug tool schemas")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    if matches.get_flag("debug") {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::new("debug"))
            .init();
    }

    // Debug schemas if requested
    if matches.get_flag("debug-schemas") {
        use crate::tools::ToolRegistry;
        let registry = ToolRegistry::new();
        let tools = registry.get_tool_list();
        
        println!("Total tools: {}", tools.len());
        
        for (i, tool) in tools.iter().enumerate() {
            if i >= 40 && i <= 50 {  // Focus on tools around 45
                println!("Tool {}: {} - {}", i, tool["name"], tool["description"]);
                if i == 45 {
                    println!("=== PROBLEMATIC TOOL 45 SCHEMA ===");
                    println!("{}", serde_json::to_string_pretty(&tool["inputSchema"]).unwrap());
                    
                    // Try to identify specific issues
                    let schema = &tool["inputSchema"];
                    if let Some(properties) = schema.get("properties") {
                        for (prop_name, prop_def) in properties.as_object().unwrap() {
                            if let Some(prop_enum) = prop_def.get("enum") {
                                if let Some(enum_array) = prop_enum.as_array() {
                                    for enum_val in enum_array {
                                        if enum_val.as_str() == Some("") {
                                            println!("  FOUND EMPTY STRING IN ENUM: {}", prop_name);
                                        }
                                    }
                                }
                            }
                        }
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