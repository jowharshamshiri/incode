// InCode Library - Export modules for testing

pub mod error;
pub mod lldb_manager;
pub mod mcp_server;
pub mod tools;

// Re-export commonly used types
pub use error::{IncodeError, IncodeResult};
pub use lldb_manager::LldbManager;
pub use mcp_server::McpServer;
pub use tools::{ToolRegistry, ToolResponse};