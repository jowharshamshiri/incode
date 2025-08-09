# InCode - LLDB Debugging Automation

**Version**: 0.1.0  
**Type**: MCP Server for LLDB Debugging  
**Scope**: 65 debugging tools across 13 categories

## Overview

InCode is a Model Context Protocol (MCP) server that provides AI agents with LLDB debugging capabilities. Following the pattern established by [insite](https://github.com/jowharshamshiri/insite) for browser automation (52 tools across 13 categories), InCode provides similar coverage for LLDB debugging automation.

Where insite enables agents to interact with browsers through screenshots, network monitoring, and JavaScript execution, InCode enables agents to debug native binaries through LLDB with process control, memory inspection, and program analysis.

## Architecture

- **Language**: Rust (performance, safety, memory management)
- **LLDB Integration**: lldb-sys crate for direct C++ API access
- **Protocol**: Model Context Protocol (MCP) for AI agent communication
- **Design**: Feature-centric development with 65 tools organized by category

## Features Overview

### Process Control & Lifecycle (6 tools)

- Launch/attach to processes with full environment control
- Process discovery and debugging target management
- Graceful detachment and resource cleanup

### Execution Control (7 tools)

- Continue, step over, step into, step out operations
- Instruction-level stepping and conditional execution
- Process interruption and execution flow control

### Breakpoint Management (8 tools)

- Breakpoint system (address, function, file:line)
- Watchpoints for memory access monitoring
- Conditional breakpoints with automated actions

### Stack & Frame Analysis (6 tools)

- Call stack inspection and navigation
- Frame-scoped variable access and expression evaluation
- Function argument and local variable analysis

### Memory Inspection (7 tools)

- Raw memory read/write with multiple formats
- Assembly disassembly and pattern searching  
- Memory mapping and region analysis

### Variable & Symbol Inspection (6 tools)

- Local, global, and scoped variable access
- Runtime expression evaluation in debugging context
- Symbol table lookup and introspection

### Thread Management (5 tools)

- Multi-threaded debugging with thread enumeration
- Thread selection and individual thread control
- Thread state management (suspend/resume)

### Register Inspection (4 tools)

- CPU register access and modification
- Register state management and introspection
- Cross-architecture register handling

### Debug Information (4 tools)

- Source code integration and display
- Function discovery and address-to-source mapping
- Debug symbol analysis and metadata

### Target Information (3 tools)

- Executable analysis (architecture, format, symbols)
- Platform and environment information
- Loaded module and library enumeration

### LLDB Control & Configuration (3 tools)

- Direct LLDB command execution
- LLDB settings and configuration management
- Version information and capability detection

### Session Management (4 tools)

- Debugging session persistence and restoration
- State management across debugging workflows
- Resource cleanup and session lifecycle

### Advanced Analysis (2 tools)

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
git clone https://github.com/jowharshamshiri/incode.git
cd incode
cargo build --release

# Run the MCP server
./target/release/incode
```

### MCP Client Configuration

Add to your MCP client configuration (e.g., Claude Desktop):

```json
{
  "mcpServers": {
    "incode": {
      "command": "/path/to/incode/target/release/incode",
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

**Current Status**: All 65 tools implemented and validated  
**Implementation**: Complete LLDB debugging platform operational  
**Test Coverage**: Real LLDB integration with comprehensive test suites

### Implementation Status

All 65 debugging tools across 13 categories are implemented with real LLDB C++ API integration. The platform includes comprehensive test infrastructure using actual LLDB debugging sessions.

## Project Goals

- **Coverage**: Match insite's approach for LLDB debugging automation
- **AI Agent Integration**: MCP protocol for programmatic debugging workflows
- **Performance**: Reasonable response times for debugging operations  
- **Reliability**: Error handling and session management
- **Modularity**: Organized design for debugging capabilities

## License

MIT License - See LICENSE file for details.
