use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, info, warn, error};

use crate::error::{IncodeError, IncodeResult};
use crate::lldb_manager::LldbManager;
use crate::tools::{ToolRegistry, ToolResponse};

pub struct McpServer {
    lldb_manager: LldbManager,
    tool_registry: ToolRegistry,
}

impl McpServer {
    pub fn new(lldb_path: Option<String>) -> IncodeResult<Self> {
        info!("Initializing InCode MCP Server");
        
        let lldb_manager = LldbManager::new(lldb_path)?;
        let tool_registry = ToolRegistry::new();
        
        info!("Registered {} debugging tools across 13 categories", tool_registry.tool_count());
        
        Ok(Self {
            lldb_manager,
            tool_registry,
        })
    }

    pub async fn run(&mut self) -> IncodeResult<()> {
        info!("Starting MCP protocol communication");
        
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        // Send initialization response
        self.send_initialize_response(&mut stdout).await?;

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    debug!("EOF reached, shutting down");
                    break;
                }
                Ok(_) => {
                    if let Err(e) = self.process_request(&line, &mut stdout).await {
                        error!("Error processing request: {}", e);
                        self.send_error_response(&mut stdout, &e).await?;
                    }
                }
                Err(e) => {
                    error!("Failed to read from stdin: {}", e);
                    break;
                }
            }
        }

        info!("MCP Server shutting down");
        Ok(())
    }

    async fn send_initialize_response(&self, stdout: &mut tokio::io::Stdout) -> IncodeResult<()> {
        let response = json!({
            "jsonrpc": "2.0",
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {
                        "listChanged": true
                    }
                },
                "serverInfo": {
                    "name": "incode",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "instructions": "InCode provides comprehensive LLDB debugging automation with 65+ tools across 13 categories for AI agents."
            }
        });

        self.send_response(stdout, response).await
    }

    async fn process_request(&mut self, request: &str, stdout: &mut tokio::io::Stdout) -> IncodeResult<()> {
        let request: Value = serde_json::from_str(request.trim())?;
        debug!("Processing request: {}", request);

        let method = request["method"].as_str()
            .ok_or_else(|| IncodeError::mcp("Missing method in request"))?;

        let response = match method {
            "tools/list" => self.handle_tools_list()?,
            "tools/call" => self.handle_tool_call(&request).await?,
            "initialize" => json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {
                        "listChanged": true
                    }
                },
                "serverInfo": {
                    "name": "incode", 
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
            _ => {
                warn!("Unknown method: {}", method);
                return Err(IncodeError::mcp(format!("Unknown method: {}", method)));
            }
        };

        let full_response = json!({
            "jsonrpc": "2.0",
            "id": request["id"],
            "result": response
        });

        self.send_response(stdout, full_response).await
    }

    fn handle_tools_list(&self) -> IncodeResult<Value> {
        Ok(json!({
            "tools": self.tool_registry.get_tool_list()
        }))
    }

    async fn handle_tool_call(&mut self, request: &Value) -> IncodeResult<Value> {
        let params = &request["params"];
        let tool_name = params["name"].as_str()
            .ok_or_else(|| IncodeError::mcp("Missing tool name"))?;
        
        let arguments = params["arguments"].as_object()
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();

        debug!("Calling tool: {} with arguments: {:?}", tool_name, arguments);

        let response = self.tool_registry
            .execute_tool(tool_name, arguments, &mut self.lldb_manager)
            .await?;

        Ok(json!({
            "content": [
                {
                    "type": "text",
                    "text": match response {
                        ToolResponse::Success(text) => text,
                        ToolResponse::Error(error) => format!("Error: {}", error),
                        ToolResponse::Json(data) => serde_json::to_string_pretty(&data)?,
                    }
                }
            ]
        }))
    }

    async fn send_response(&self, stdout: &mut tokio::io::Stdout, response: Value) -> IncodeResult<()> {
        let response_str = serde_json::to_string(&response)?;
        debug!("Sending response: {}", response_str);
        
        stdout.write_all(response_str.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
        
        Ok(())
    }

    async fn send_error_response(&self, stdout: &mut tokio::io::Stdout, error: &IncodeError) -> IncodeResult<()> {
        let response = json!({
            "jsonrpc": "2.0",
            "error": {
                "code": -1,
                "message": error.to_string()
            }
        });

        self.send_response(stdout, response).await
    }
}