# InLLDB - Intelligent LLDB Debugging Automation

**Version**: 0.1.0  
**Type**: MCP Server for AI-Powered LLDB Debugging  
**Scope**: 65+ debugging tools across 13 categories

## Overview

InLLDB is a comprehensive Model Context Protocol (MCP) server that provides AI agents with complete LLDB debugging capabilities. Inspired by [insite's](https://github.com/jowharshamshiri/insite) comprehensive browser automation (52 tools across 13 categories), InLLDB delivers the same breadth of coverage for LLDB debugging automation.

Just as insite allows agents to see into browsers - taking screenshots, monitoring network activity, executing JavaScript - InLLDB allows agents to see into native binary execution via LLDB debugging with comprehensive process control, memory inspection, and program analysis.

## Architecture

- **Language**: Rust (performance, safety, memory management)
- **LLDB Integration**: lldb-sys crate for direct C++ API access
- **Protocol**: Model Context Protocol (MCP) for AI agent communication
- **Design**: Feature-centric development with 65+ tools organized by category

## Features Overview

### 🔧 Process Control & Lifecycle (6 tools)
- Launch/attach to processes with full environment control
- Process discovery and debugging target management
- Graceful detachment and resource cleanup

### ⚡ Execution Control (7 tools)
- Continue, step over, step into, step out operations
- Instruction-level stepping and conditional execution
- Process interruption and execution flow control

### 🎯 Breakpoint Management (8 tools)
- Comprehensive breakpoint system (address, function, file:line)
- Watchpoints for memory access monitoring
- Conditional breakpoints with automated actions

### 📚 Stack & Frame Analysis (6 tools)
- Complete call stack inspection and navigation
- Frame-scoped variable access and expression evaluation
- Function argument and local variable analysis

### 🧠 Memory Inspection (7 tools)
- Raw memory read/write with multiple formats
- Assembly disassembly and pattern searching  
- Memory mapping and region analysis

### 🔍 Variable & Symbol Inspection (6 tools)
- Local, global, and scoped variable access
- Runtime expression evaluation in debugging context
- Symbol table lookup and introspection

### 🧵 Thread Management (5 tools)
- Multi-threaded debugging with thread enumeration
- Thread selection and individual thread control
- Thread state management (suspend/resume)

### 💾 Register Inspection (4 tools)
- CPU register access and modification
- Register state management and introspection
- Cross-architecture register handling

### 📖 Debug Information (4 tools)
- Source code integration and display
- Function discovery and address-to-source mapping
- Debug symbol analysis and metadata

### 🎯 Target Information (3 tools)
- Executable analysis (architecture, format, symbols)
- Platform and environment information
- Loaded module and library enumeration

### ⚙️ LLDB Control & Configuration (3 tools)
- Direct LLDB command execution
- LLDB settings and configuration management
- Version information and capability detection

### 💾 Session Management (4 tools)
- Debugging session persistence and restoration
- State management across debugging workflows
- Resource cleanup and session lifecycle

### 🔬 Advanced Analysis (2 tools)
- Automated crash analysis and root cause identification
- Core dump generation for offline analysis

## Installation

### Requirements

- Rust 1.70+ 
- LLDB development libraries
- Compatible with macOS, Linux, Windows

### Quick Start

```bash
# Install directly from source
git clone <repository-url>
cd inlldb
cargo build --release

# Run the MCP server
./target/release/inlldb
```

### MCP Client Configuration

Add to your MCP client configuration (e.g., Claude Desktop):

```json
{
  "mcpServers": {
    "inlldb": {
      "command": "/path/to/inlldb/target/release/inlldb",
      "args": ["--debug"],
      "env": {
        "LLDB_PATH": "/usr/bin/lldb"
      }
    }
  }
}
```

## Usage Examples

### Basic Process Debugging

```json
[
  {
    "name": "launch_process",
    "arguments": {
      "executable": "./my_program",
      "args": ["arg1", "arg2"],
      "env": {"DEBUG": "1"}
    }
  },
  {
    "name": "set_breakpoint", 
    "arguments": {
      "location": "main.cpp:42"
    }
  },
  {
    "name": "continue_execution"
  }
]
```

### Memory Analysis Workflow

```json
[
  {
    "name": "read_memory",
    "arguments": {
      "address": "0x7fff12345000",
      "size": 256,
      "format": "hex"
    }
  },
  {
    "name": "search_memory",
    "arguments": {
      "pattern": "deadbeef",
      "start_address": "0x7fff00000000",
      "size": "0x100000"
    }
  }
]
```

### Advanced Analysis

```json
[
  {
    "name": "get_backtrace"
  },
  {
    "name": "evaluate_expression",
    "arguments": {
      "expression": "*(struct my_data*)0x12345"
    }
  },
  {
    "name": "analyze_crash",
    "arguments": {
      "generate_report": true
    }
  }
]
```

## Development Status

**Current Phase**: Foundation Complete  
**Implementation**: Core architecture and MCP server framework operational  
**Next Phase**: Tool implementation (starting with Process Control category)

### Feature Implementation Roadmap

- **Phase 1**: Core Debugging (21 tools) - Process Control, Execution Control, Breakpoints  
- **Phase 2**: Inspection & Analysis (19 tools) - Stack, Memory, Variables
- **Phase 3**: Advanced Control (9 tools) - Threads, Registers
- **Phase 4**: Information & Management (10 tools) - Debug Info, Target Info, LLDB Control  
- **Phase 5**: Enterprise Features (6 tools) - Session Management, Advanced Analysis

## Project Goals

- **Comprehensive Coverage**: Match insite's breadth for LLDB debugging automation
- **AI Agent Integration**: Seamless MCP protocol for natural language debugging
- **Performance**: Sub-second response times for inspection operations  
- **Reliability**: Robust error handling and session management
- **Extensibility**: Modular design for additional debugging capabilities

## Contributing

InLLDB follows feature-centric development methodology:

1. All work organized around feature codes (F0001-F0065)
2. Feature status tracked in `internal/features.md`
3. Implementation guided by `internal/architectural_decisions.md`
4. Test coverage required for all features

## License

MIT License - See LICENSE file for details.

---

**InLLDB: Bringing the power of expert-level LLDB debugging to AI agents**