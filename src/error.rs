use thiserror::Error;

pub type IncodeResult<T> = Result<T, IncodeError>;

#[derive(Error, Debug)]
pub enum IncodeError {
    #[error("LLDB initialization failed: {0}")]
    LldbInitialization(String),

    #[error("LLDB operation failed: {0}")]
    LldbOperation(String),

    #[error("Process not found or not debuggable: {0}")]
    ProcessNotFound(String),

    #[error("Invalid memory address: 0x{address:x}")]
    InvalidMemoryAddress { address: u64 },

    #[error("Breakpoint operation failed: {0}")]
    BreakpointError(String),

    #[error("Thread operation failed: {0}")]
    ThreadError(String),

    #[error("Frame operation failed: {0}")]
    FrameError(String),

    #[error("Expression evaluation failed: {0}")]
    ExpressionError(String),

    #[error("MCP protocol error: {0}")]
    McpProtocol(String),

    #[error("Session management error: {0}")]
    SessionError(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid configuration: {0}")]
    Configuration(String),

    #[error("Timeout error: operation took too long")]
    Timeout,

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

impl IncodeError {
    pub fn lldb_init<S: Into<String>>(msg: S) -> Self {
        Self::LldbInitialization(msg.into())
    }

    pub fn lldb_op<S: Into<String>>(msg: S) -> Self {
        Self::LldbOperation(msg.into())
    }

    pub fn process_not_found<S: Into<String>>(msg: S) -> Self {
        Self::ProcessNotFound(msg.into())
    }

    pub fn invalid_address(address: u64) -> Self {
        Self::InvalidMemoryAddress { address }
    }

    pub fn breakpoint<S: Into<String>>(msg: S) -> Self {
        Self::BreakpointError(msg.into())
    }

    pub fn thread<S: Into<String>>(msg: S) -> Self {
        Self::ThreadError(msg.into())
    }

    pub fn frame<S: Into<String>>(msg: S) -> Self {
        Self::FrameError(msg.into())
    }

    pub fn expression<S: Into<String>>(msg: S) -> Self {
        Self::ExpressionError(msg.into())
    }

    pub fn mcp<S: Into<String>>(msg: S) -> Self {
        Self::McpProtocol(msg.into())
    }

    pub fn session<S: Into<String>>(msg: S) -> Self {
        Self::SessionError(msg.into())
    }

    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Configuration(msg.into())
    }

    pub fn not_implemented<S: Into<String>>(feature: S) -> Self {
        Self::NotImplemented(feature.into())
    }
}