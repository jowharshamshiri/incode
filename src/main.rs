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
        .get_matches();

    if matches.get_flag("debug") {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::new("debug"))
            .init();
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