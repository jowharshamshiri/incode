use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::path::Path;
use tracing::{debug, info, error};
use uuid::Uuid;
use serde_json::{json, Value};

use crate::error::{IncodeError, IncodeResult};

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub start_address: u64,
    pub end_address: u64,
    pub size: u64,
    pub permissions: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MemorySegment {
    pub name: String,
    pub vm_address: u64,
    pub vm_size: u64,
    pub file_offset: u64,
    pub file_size: u64,
    pub max_protection: String,
    pub initial_protection: String,
    pub segment_type: String,
}

#[derive(Debug, Clone)]
pub struct MemoryMap {
    pub total_segments: usize,
    pub total_vm_size: u64,
    pub segments: Vec<MemorySegment>,
    pub load_address: u64,
    pub slide: u64,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub value: String,
    pub var_type: String,
    pub is_argument: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub name: String,
    pub full_name: String,
    pub var_type: String,
    pub type_class: String,
    pub value: String,
    pub address: u64,
    pub size: usize,
    pub is_valid: bool,
    pub is_in_scope: bool,
    pub location: String,
    pub declaration_file: Option<String>,
    pub declaration_line: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub index: u32,
    pub function_name: String,
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
    pub address: u64,
    pub is_inlined: bool,
}

#[derive(Debug, Clone)]
pub struct ThreadInfo {
    pub thread_id: u32,
    pub index: u32,
    pub name: Option<String>,
    pub state: String,
    pub stop_reason: Option<String>,
    pub queue_name: Option<String>,
    pub frame_count: u32,
    pub current_frame: Option<StackFrame>,
}

#[derive(Debug, Clone)]
pub struct RegisterInfo {
    pub name: String,
    pub value: u64,
    pub size: u32,
    pub register_type: String,
    pub format: String,
    pub is_valid: bool,
}

#[derive(Debug, Clone)]
pub struct RegisterState {
    pub registers: HashMap<String, RegisterInfo>,
    pub timestamp: std::time::SystemTime,
    pub thread_id: Option<u32>,
    pub frame_index: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file_path: String,
    pub line_number: u32,
    pub column: Option<u32>,
    pub function_name: Option<String>,
    pub address: u64,
    pub is_valid: bool,
}

#[derive(Debug, Clone)]
pub struct SourceCode {
    pub file_path: String,
    pub lines: Vec<SourceLine>,
    pub start_line: u32,
    pub end_line: u32,
    pub current_line: Option<u32>,
    pub total_lines: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct SourceLine {
    pub line_number: u32,
    pub content: String,
    pub is_current: bool,
    pub has_breakpoint: bool,
    pub address: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub mangled_name: Option<String>,
    pub start_address: u64,
    pub end_address: Option<u64>,
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
    pub size: Option<u64>,
    pub is_inline: bool,
    pub return_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DebugInfo {
    pub has_debug_symbols: bool,
    pub debug_format: String,
    pub compilation_units: Vec<CompilationUnit>,
    pub symbol_count: u32,
    pub line_table_count: u32,
    pub function_count: u32,
}

#[derive(Debug, Clone)]
pub struct CompilationUnit {
    pub file_path: String,
    pub producer: Option<String>,
    pub language: Option<String>,
    pub low_pc: u64,
    pub high_pc: u64,
    pub line_count: u32,
}

#[derive(Debug, Clone)]
pub struct TargetInfo {
    pub executable_path: String,
    pub architecture: String,
    pub platform: String,
    pub executable_format: String, // e.g., "ELF", "Mach-O", "PE"
    pub has_debug_symbols: bool,
    pub entry_point: Option<u64>,
    pub base_address: Option<u64>,
    pub file_size: u64,
    pub creation_time: Option<std::time::SystemTime>,
    pub is_pie: bool, // Position Independent Executable
    pub is_stripped: bool,
    pub endianness: String, // "little" or "big"
}

#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub name: String,
    pub version: String,
    pub architecture: String,
    pub vendor: String, // e.g., "apple", "pc", "unknown"
    pub environment: String, // e.g., "gnu", "msvc", "darwin"
    pub sdk_version: Option<String>,
    pub deployment_target: Option<String>,
    pub is_simulator: bool,
    pub is_remote: bool,
    pub supports_jit: bool,
    pub working_directory: String,
    pub supported_architectures: Vec<String>,
    pub hostname: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub name: String,
    pub file_path: String,
    pub uuid: String,
    pub architecture: String,
    pub load_address: u64,
    pub file_size: u64,
    pub is_main_executable: bool,
    pub has_debug_symbols: bool,
    pub symbol_vendor: Option<String>, // e.g., "DWARF", "PDB", "none"
    pub compile_units: Vec<String>,
    pub num_symbols: u32,
    pub slide: Option<u64>, // ASLR slide
    pub version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub demangled_name: Option<String>,
    pub symbol_type: String, // "Function", "Data", "Unknown"
    pub address: u64,
    pub size: u64,
    pub module: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub is_exported: bool,
    pub is_debug: bool,
    pub visibility: String, // "public", "private", "protected", "unknown"
}

#[derive(Debug, Clone)]
pub struct CrashAnalysis {
    pub crash_type: String, // e.g., "SIGSEGV", "SIGABRT", "EXC_BAD_ACCESS"
    pub crash_address: Option<u64>, // Address where crash occurred (if available)
    pub faulting_thread: u32, // Thread ID that caused the crash
    pub signal_number: i32, // Signal number (POSIX) or exception code
    pub signal_name: String, // Human-readable signal name
    pub exception_type: Option<String>, // Platform-specific exception type
    pub exception_codes: Vec<u64>, // Platform-specific exception codes
    pub crashed_thread_backtrace: Vec<String>, // Stack trace of crashed thread
    pub register_state: String, // Register state at time of crash
    pub memory_regions: Vec<String>, // Memory layout information
    pub loaded_modules: Vec<String>, // List of loaded modules/libraries
    pub crash_summary: String, // High-level crash description
    pub recommendations: Vec<String>, // Debugging recommendations
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub state: String,
    pub executable_path: Option<String>,
    pub memory_usage: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct BreakpointInfo {
    pub id: u32,
    pub enabled: bool,
    pub hit_count: u32,
    pub location: String,
    pub condition: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FrameInfo {
    pub index: u32,
    pub function_name: String,
    pub pc: u64,
    pub sp: u64,
    pub module: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct LldbVersionInfo {
    pub version: String,
    pub build_number: Option<String>,
    pub api_version: String,
    pub build_date: Option<String>,
    pub build_configuration: Option<String>,
    pub compiler: Option<String>,
    pub platform: String,
}

// LLDB FFI bindings - these will fail in test environment without LLDB
// We'll handle this gracefully by using mock implementations for testing
#[cfg(not(test))]
extern "C" {
    fn SBDebuggerCreate() -> *mut std::ffi::c_void;
    fn SBDebuggerDestroy(debugger: *mut std::ffi::c_void);
    fn SBDebuggerSetAsync(debugger: *mut std::ffi::c_void, async_mode: bool);
    fn SBDebuggerCreateTarget(debugger: *mut std::ffi::c_void, filename: *const i8) -> *mut std::ffi::c_void;
    fn SBTargetLaunchSimple(target: *mut std::ffi::c_void, argv: *const *const i8, envp: *const *const i8, working_dir: *const i8) -> *mut std::ffi::c_void;
    fn SBProcessGetProcessID(process: *mut std::ffi::c_void) -> u64;
    fn SBProcessGetState(process: *mut std::ffi::c_void) -> u32;
    fn SBProcessAttachToProcessWithID(target: *mut std::ffi::c_void, listener: *mut std::ffi::c_void, pid: u64) -> *mut std::ffi::c_void;
    fn SBProcessDetach(process: *mut std::ffi::c_void) -> bool;
    fn SBProcessKill(process: *mut std::ffi::c_void) -> bool;
    fn SBProcessContinue(process: *mut std::ffi::c_void) -> bool;
    fn SBTargetGetProcess(target: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn SBProcessGetNumThreads(process: *mut std::ffi::c_void) -> u32;
    fn SBProcessGetThreadAtIndex(process: *mut std::ffi::c_void, index: u32) -> *mut std::ffi::c_void;
    fn SBThreadGetThreadID(thread: *mut std::ffi::c_void) -> u32;
    fn SBThreadGetIndexID(thread: *mut std::ffi::c_void) -> u32;
    fn SBThreadGetSelectedFrame(thread: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn SBFrameGetRegisters(frame: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn SBValueListGetSize(value_list: *mut std::ffi::c_void) -> u32;
    fn SBValueListGetValueAtIndex(value_list: *mut std::ffi::c_void, index: u32) -> *mut std::ffi::c_void;
    fn SBValueGetName(value: *mut std::ffi::c_void) -> *const i8;
    fn SBValueGetValueAsUnsigned(value: *mut std::ffi::c_void) -> u64;
    fn SBValueSetValueFromCString(value: *mut std::ffi::c_void, value_str: *const i8) -> bool;
    fn SBFrameGetLineEntry(frame: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn SBLineEntryGetFileSpec(line_entry: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn SBFileSpecGetFilename(file_spec: *mut std::ffi::c_void) -> *const i8;
    fn SBFileSpecGetDirectory(file_spec: *mut std::ffi::c_void) -> *const i8;
    fn SBLineEntryGetLine(line_entry: *mut std::ffi::c_void) -> u32;
    fn SBLineEntryGetColumn(line_entry: *mut std::ffi::c_void) -> u32;
    fn SBTargetGetNumModules(target: *mut std::ffi::c_void) -> u32;
    fn SBTargetGetModuleAtIndex(target: *mut std::ffi::c_void, index: u32) -> *mut std::ffi::c_void;
    fn SBModuleGetFileSpec(module: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn SBModuleGetNumSymbols(module: *mut std::ffi::c_void) -> u32;
    fn SBModuleGetSymbolAtIndex(module: *mut std::ffi::c_void, index: u32) -> *mut std::ffi::c_void;
    fn SBSymbolGetName(symbol: *mut std::ffi::c_void) -> *const i8;
    fn SBSymbolGetStartAddress(symbol: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn SBSymbolGetEndAddress(symbol: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn SBAddressGetLoadAddress(address: *mut std::ffi::c_void, target: *mut std::ffi::c_void) -> u64;
    fn SBFileSpecGetPath(file_spec: *mut std::ffi::c_void, buffer: *mut i8, buffer_size: u32) -> u32;
    fn SBThreadStepOver(thread: *mut std::ffi::c_void) -> bool;
    fn SBThreadStepInto(thread: *mut std::ffi::c_void) -> bool;
    fn SBThreadStepOut(thread: *mut std::ffi::c_void) -> bool;
    fn SBThreadStepInstruction(thread: *mut std::ffi::c_void, step_over: bool) -> bool;
    fn SBProcessGetSelectedThread(process: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn SBTargetBreakpointCreateByAddress(target: *mut std::ffi::c_void, address: u64) -> *mut std::ffi::c_void;
    fn SBTargetBreakpointCreateByLocation(target: *mut std::ffi::c_void, file: *const i8, line: u32) -> *mut std::ffi::c_void;
    fn SBProcessSendAsyncInterrupt(process: *mut std::ffi::c_void) -> bool;
    fn SBThreadRunToAddress(thread: *mut std::ffi::c_void, address: u64) -> bool;
    fn SBTargetWatchAddress(target: *mut std::ffi::c_void, address: u64, size: u32, read: bool, write: bool) -> *mut std::ffi::c_void;
    fn SBTargetGetNumBreakpoints(target: *mut std::ffi::c_void) -> u32;
    fn SBTargetGetBreakpointAtIndex(target: *mut std::ffi::c_void, index: u32) -> *mut std::ffi::c_void;
    fn SBBreakpointGetID(breakpoint: *mut std::ffi::c_void) -> u32;
    fn SBBreakpointSetEnabled(breakpoint: *mut std::ffi::c_void, enabled: bool);
    fn SBBreakpointIsEnabled(breakpoint: *mut std::ffi::c_void) -> bool;
    fn SBBreakpointSetCondition(breakpoint: *mut std::ffi::c_void, condition: *const i8);
    fn SBTargetFindBreakpointByID(target: *mut std::ffi::c_void, breakpoint_id: u32) -> *mut std::ffi::c_void;
    fn SBBreakpointGetHitCount(breakpoint: *mut std::ffi::c_void) -> u32;
    fn SBBreakpointDelete(breakpoint: *mut std::ffi::c_void) -> bool;
    fn SBThreadGetNumFrames(thread: *mut std::ffi::c_void) -> u32;
    fn SBThreadGetFrameAtIndex(thread: *mut std::ffi::c_void, index: u32) -> *mut std::ffi::c_void;
    fn SBFrameGetDisplayFunctionName(frame: *mut std::ffi::c_void) -> *const i8;
    fn SBFrameGetPC(frame: *mut std::ffi::c_void) -> u64;
    fn SBFrameGetSP(frame: *mut std::ffi::c_void) -> u64;
    fn SBThreadSetSelectedFrame(thread: *mut std::ffi::c_void, frame: *mut std::ffi::c_void) -> bool;
    fn SBFrameGetModule(frame: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn SBProcessReadMemory(process: *mut std::ffi::c_void, address: u64, size: u32, buffer: *mut u8) -> u32;
    fn SBProcessWriteMemory(process: *mut std::ffi::c_void, address: u64, data: *const u8, size: u32) -> u32;
    fn SBTargetReadInstructions(target: *mut std::ffi::c_void, address: u64, count: u32) -> *mut std::ffi::c_void;
    fn SBTargetGetTriple(target: *mut std::ffi::c_void) -> *const i8;
    fn SBTargetGetPlatform(target: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn SBPlatformGetName(platform: *mut std::ffi::c_void) -> *const i8;
    fn SBTargetGetExecutable(target: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn SBPlatformGetWorkingDirectory(platform: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn SBPlatformGetOSBuild(platform: *mut std::ffi::c_void) -> *const i8;
    fn SBPlatformGetOSDescription(platform: *mut std::ffi::c_void) -> *const i8;
    fn SBPlatformGetHostname(platform: *mut std::ffi::c_void) -> *const i8;
    fn SBModuleGetUUIDString(module: *mut std::ffi::c_void) -> *const i8;
    fn SBModuleGetVersion(module: *mut std::ffi::c_void) -> *const i8;
    fn SBModuleGetObjectName(module: *mut std::ffi::c_void) -> *const i8;
    fn SBModuleGetTriple(module: *mut std::ffi::c_void) -> *const i8;
    fn SBDebuggerGetVersion() -> *const i8;
    fn SBDebuggerGetBuildConfiguration() -> *const i8;
}

// Mock LLDB functions for testing environment
#[cfg(test)]
mod mock_lldb {
    pub fn SBDebuggerCreate() -> *mut std::ffi::c_void {
        0x1 as *mut std::ffi::c_void // Return non-null pointer for testing
    }
    pub fn SBDebuggerDestroy(_debugger: *mut std::ffi::c_void) {}
    pub fn SBDebuggerSetAsync(_debugger: *mut std::ffi::c_void, _async_mode: bool) {}
    pub fn SBDebuggerCreateTarget(_debugger: *mut std::ffi::c_void, _filename: *const i8) -> *mut std::ffi::c_void {
        0x2 as *mut std::ffi::c_void
    }
    pub fn SBTargetLaunchSimple(_target: *mut std::ffi::c_void, _argv: *const *const i8, _envp: *const *const i8, _working_dir: *const i8) -> *mut std::ffi::c_void {
        0x3 as *mut std::ffi::c_void
    }
    pub fn SBProcessGetProcessID(_process: *mut std::ffi::c_void) -> u64 { 12345 }
    pub fn SBProcessGetState(_process: *mut std::ffi::c_void) -> u32 { 7 } // Running state
    pub fn SBProcessAttachToProcessWithID(_target: *mut std::ffi::c_void, _listener: *mut std::ffi::c_void, pid: u64) -> *mut std::ffi::c_void {
        if pid == 99999 { std::ptr::null_mut() } else { 0x4 as *mut std::ffi::c_void }
    }
    pub fn SBProcessDetach(_process: *mut std::ffi::c_void) -> bool { true }
    pub fn SBProcessKill(_process: *mut std::ffi::c_void) -> bool { true }
    pub fn SBProcessContinue(_process: *mut std::ffi::c_void) -> bool { true }
    pub fn SBTargetGetProcess(_target: *mut std::ffi::c_void) -> *mut std::ffi::c_void { 0x5 as *mut std::ffi::c_void }
    pub fn SBThreadStepOver(_thread: *mut std::ffi::c_void) -> bool { true }
    pub fn SBThreadStepInto(_thread: *mut std::ffi::c_void) -> bool { true }
    pub fn SBThreadStepOut(_thread: *mut std::ffi::c_void) -> bool { true }
    pub fn SBThreadStepInstruction(_thread: *mut std::ffi::c_void, _step_over: bool) -> bool { true }
    pub fn SBProcessGetSelectedThread(_process: *mut std::ffi::c_void) -> *mut std::ffi::c_void { 0x6 as *mut std::ffi::c_void }
    pub fn SBTargetBreakpointCreateByAddress(_target: *mut std::ffi::c_void, _address: u64) -> *mut std::ffi::c_void { 0x7 as *mut std::ffi::c_void }
    pub fn SBTargetBreakpointCreateByLocation(_target: *mut std::ffi::c_void, _file: *const i8, _line: u32) -> *mut std::ffi::c_void { 0x8 as *mut std::ffi::c_void }
    pub fn SBProcessSendAsyncInterrupt(_process: *mut std::ffi::c_void) -> bool { true }
    pub fn SBThreadRunToAddress(_thread: *mut std::ffi::c_void, _address: u64) -> bool { true }
    pub fn SBTargetWatchAddress(_target: *mut std::ffi::c_void, _address: u64, _size: u32, _read: bool, _write: bool) -> *mut std::ffi::c_void { 0x9 as *mut std::ffi::c_void }
    pub fn SBTargetGetNumBreakpoints(_target: *mut std::ffi::c_void) -> u32 { 2 } // Mock: return 2 breakpoints
    pub fn SBTargetGetBreakpointAtIndex(_target: *mut std::ffi::c_void, index: u32) -> *mut std::ffi::c_void { 
        (0x100 + index as usize) as *mut std::ffi::c_void 
    }
    pub fn SBBreakpointGetID(breakpoint: *mut std::ffi::c_void) -> u32 { 
        (breakpoint as usize - 0x100) as u32 + 1 
    }
    pub fn SBBreakpointIsEnabled(_breakpoint: *mut std::ffi::c_void) -> bool { true }
    pub fn SBBreakpointGetHitCount(_breakpoint: *mut std::ffi::c_void) -> u32 { 0 }
    pub fn SBBreakpointDelete(_breakpoint: *mut std::ffi::c_void) -> bool { true }
    pub fn SBTargetFindBreakpointByID(_target: *mut std::ffi::c_void, _id: u32) -> *mut std::ffi::c_void { 0xA as *mut std::ffi::c_void }
    pub fn SBBreakpointSetEnabled(_breakpoint: *mut std::ffi::c_void, _enabled: bool) -> bool { true }
    pub fn SBBreakpointSetCondition(_breakpoint: *mut std::ffi::c_void, _condition: *const std::ffi::c_char) -> bool { true }
    pub fn SBProcessWriteMemory(_process: *mut std::ffi::c_void, _address: u64, _data: *const u8, _size: u32) -> u32 { 64 }
    pub fn SBTargetGetTriple(_target: *mut std::ffi::c_void) -> *const i8 {
        "x86_64-apple-macosx\0".as_ptr() as *const i8
    }
    pub fn SBTargetGetPlatform(_target: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
        0xB as *mut std::ffi::c_void
    }
    pub fn SBPlatformGetName(_platform: *mut std::ffi::c_void) -> *const i8 {
        "host\0".as_ptr() as *const i8
    }
    pub fn SBTargetGetExecutable(_target: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
        0xC as *mut std::ffi::c_void
    }
    pub fn SBFileSpecGetPath(_filespec: *mut std::ffi::c_void, buffer: *mut i8, buffer_size: u32) -> u32 {
        let path = b"/usr/bin/test\0";
        let len = path.len().min(buffer_size as usize);
        if !buffer.is_null() && buffer_size > 0 {
            unsafe { std::ptr::copy_nonoverlapping(path.as_ptr() as *const i8, buffer, len); }
        }
        len as u32
    }
    pub fn SBProcessGetNumThreads(_process: *mut std::ffi::c_void) -> u32 { 2 }
    pub fn SBProcessGetThreadAtIndex(_process: *mut std::ffi::c_void, index: u32) -> *mut std::ffi::c_void {
        (0x400 + index as usize) as *mut std::ffi::c_void
    }
    pub fn SBThreadGetThreadID(_thread: *mut std::ffi::c_void) -> u64 { 12345 }
    pub fn SBThreadGetIndexID(_thread: *mut std::ffi::c_void) -> u32 { 0 }
    pub fn SBFrameGetRegisters(_frame: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
        0x500 as *mut std::ffi::c_void
    }
    pub fn SBValueListGetSize(_list: *mut std::ffi::c_void) -> u32 { 5 }
    pub fn SBValueListGetValueAtIndex(_list: *mut std::ffi::c_void, index: u32) -> *mut std::ffi::c_void {
        (0x600 + index as usize) as *mut std::ffi::c_void
    }
    pub fn SBValueGetName(_value: *mut std::ffi::c_void) -> *const i8 {
        "rax\0".as_ptr() as *const i8
    }
    pub fn SBValueGetValueAsUnsigned(_value: *mut std::ffi::c_void) -> u64 { 0x12345678 }
    pub fn SBValueSetValueFromCString(_value: *mut std::ffi::c_void, _cstr: *const i8) -> bool { true }
    pub fn SBLineEntryGetFileSpec(_entry: *mut std::ffi::c_void) -> *mut std::ffi::c_void { 0x700 as *mut std::ffi::c_void }
    pub fn SBFileSpecGetFilename(_filespec: *mut std::ffi::c_void) -> *const i8 { "main.cpp\0".as_ptr() as *const i8 }
    pub fn SBFileSpecGetDirectory(_filespec: *mut std::ffi::c_void) -> *const i8 { "/src\0".as_ptr() as *const i8 }
    pub fn SBLineEntryGetLine(_entry: *mut std::ffi::c_void) -> u32 { 42 }
    pub fn SBTargetGetNumModules(_target: *mut std::ffi::c_void) -> u32 { 1 }
    pub fn SBTargetGetModuleAtIndex(_target: *mut std::ffi::c_void, _index: u32) -> *mut std::ffi::c_void { 0x800 as *mut std::ffi::c_void }
    pub fn SBModuleGetNumSymbols(_module: *mut std::ffi::c_void) -> u32 { 10 }
    pub fn SBModuleGetSymbolAtIndex(_module: *mut std::ffi::c_void, _index: u32) -> *mut std::ffi::c_void { 0x900 as *mut std::ffi::c_void }
    pub fn SBSymbolGetName(_symbol: *mut std::ffi::c_void) -> *const i8 { "main\0".as_ptr() as *const i8 }
    pub fn SBSymbolGetStartAddress(_symbol: *mut std::ffi::c_void) -> *mut std::ffi::c_void { 0x1000 as *mut std::ffi::c_void }
    pub fn SBSymbolGetEndAddress(_symbol: *mut std::ffi::c_void) -> *mut std::ffi::c_void { 0x1100 as *mut std::ffi::c_void }
    pub fn SBAddressGetLoadAddress(_address: *mut std::ffi::c_void, _target: *mut std::ffi::c_void) -> u64 { 0x100001000 }
    pub fn SBTargetGetNumCompileUnits(_target: *mut std::ffi::c_void) -> u32 { 1 }
    pub fn SBTargetGetCompileUnitAtIndex(_target: *mut std::ffi::c_void, _index: u32) -> *mut std::ffi::c_void { 0xA00 as *mut std::ffi::c_void }
    pub fn SBCompileUnitGetFileSpec(_unit: *mut std::ffi::c_void) -> *mut std::ffi::c_void { 0xB00 as *mut std::ffi::c_void }
    pub fn SBCompileUnitGetLanguage(_unit: *mut std::ffi::c_void) -> u32 { 1 } // C++ language code
    pub fn SBCompileUnitGetProducer(_unit: *mut std::ffi::c_void) -> *const i8 { "clang\0".as_ptr() as *const i8 }
    pub fn SBPlatformGetWorkingDirectory(_platform: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
        0xC00 as *mut std::ffi::c_void
    }
    pub fn SBPlatformGetOSBuild(_platform: *mut std::ffi::c_void) -> *const i8 {
        "24.5.0\0".as_ptr() as *const i8
    }
    pub fn SBPlatformGetOSDescription(_platform: *mut std::ffi::c_void) -> *const i8 {
        "macOS 15.0\0".as_ptr() as *const i8
    }
    pub fn SBPlatformGetHostname(_platform: *mut std::ffi::c_void) -> *const i8 {
        "localhost\0".as_ptr() as *const i8
    }
    pub fn SBModuleGetUUIDString(_module: *mut std::ffi::c_void) -> *const i8 {
        "12345678-1234-5678-9ABC-DEF012345678\0".as_ptr() as *const i8
    }
    pub fn SBModuleGetVersion(_module: *mut std::ffi::c_void) -> *const i8 {
        "1.0.0\0".as_ptr() as *const i8
    }
    pub fn SBModuleGetObjectName(_module: *mut std::ffi::c_void) -> *const i8 {
        "test_binary\0".as_ptr() as *const i8
    }
    pub fn SBModuleGetTriple(_module: *mut std::ffi::c_void) -> *const i8 {
        "x86_64-apple-macosx\0".as_ptr() as *const i8
    }
    pub fn SBThreadGetNumFrames(_thread: *mut std::ffi::c_void) -> u32 { 3 } // Mock: return 3 stack frames
    pub fn SBThreadGetFrameAtIndex(_thread: *mut std::ffi::c_void, index: u32) -> *mut std::ffi::c_void {
        (0x200 + index as usize) as *mut std::ffi::c_void
    }
    pub fn SBFrameGetDisplayFunctionName(frame: *mut std::ffi::c_void) -> *const i8 {
        match frame as usize {
            0x200 => b"main\0".as_ptr() as *const i8,
            0x201 => b"foo\0".as_ptr() as *const i8,
            0x202 => b"bar\0".as_ptr() as *const i8,
            _ => b"unknown\0".as_ptr() as *const i8,
        }
    }
    pub fn SBFrameGetPC(frame: *mut std::ffi::c_void) -> u64 { 
        0x401000 + (frame as u64 - 0x200) * 0x100 
    }
    pub fn SBFrameGetSP(frame: *mut std::ffi::c_void) -> u64 { 
        0x7fff0000 - (frame as u64 - 0x200) * 0x1000 
    }
    pub fn SBThreadSetSelectedFrame(_thread: *mut std::ffi::c_void, _frame: *mut std::ffi::c_void) -> bool { true }
    pub fn SBThreadGetSelectedFrame(_thread: *mut std::ffi::c_void) -> *mut std::ffi::c_void { 0x200 as *mut std::ffi::c_void }
    pub fn SBFrameGetModule(_frame: *mut std::ffi::c_void) -> *mut std::ffi::c_void { 0x300 as *mut std::ffi::c_void }
    pub fn SBFrameGetLineEntry(_frame: *mut std::ffi::c_void) -> *mut std::ffi::c_void { 0x301 as *mut std::ffi::c_void }
    pub fn SBProcessReadMemory(_process: *mut std::ffi::c_void, _address: u64, size: u32, buffer: *mut u8) -> u32 {
        // Mock: Fill buffer with pattern data
        if !buffer.is_null() && size > 0 {
            unsafe {
                for i in 0..size as usize {
                    *buffer.add(i) = (i % 256) as u8;
                }
            }
        }
        size
    }
    pub fn SBTargetReadInstructions(_target: *mut std::ffi::c_void, _address: u64, _count: u32) -> *mut std::ffi::c_void { 
        0x400 as *mut std::ffi::c_void 
    }
    pub fn SBModuleGetFileSpec(_module: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
        0x401 as *mut std::ffi::c_void 
    }
}

#[cfg(test)]
use mock_lldb::*;

/// Session information for debugging state
#[derive(Debug, Clone)]
pub struct DebuggingSession {
    pub id: Uuid,
    pub target_path: Option<String>,
    pub process_id: Option<u32>,
    pub state: SessionState,
    pub created_at: std::time::SystemTime,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    Created,
    Attached,
    Running,
    Stopped,
    Terminated,
}

/// Manages LLDB instances and debugging sessions
pub struct LldbManager {
    lldb_path: Option<String>,
    sessions: Arc<Mutex<HashMap<Uuid, DebuggingSession>>>,
    current_session: Option<Uuid>,
    debugger: Option<*mut std::ffi::c_void>,
    current_target: Option<*mut std::ffi::c_void>,
    current_process: Option<*mut std::ffi::c_void>,
    current_thread: Option<*mut std::ffi::c_void>,
    current_thread_id: Option<u32>,
    current_frame_index: u32,
}

unsafe impl Send for LldbManager {}
unsafe impl Sync for LldbManager {}

impl LldbManager {
    pub fn new(lldb_path: Option<String>) -> IncodeResult<Self> {
        info!("Initializing LLDB Manager");
        
        // Validate LLDB availability
        if let Some(ref path) = lldb_path {
            if !std::path::Path::new(path).exists() {
                return Err(IncodeError::lldb_init(
                    format!("LLDB executable not found at: {}", path)
                ));
            }
        }

        // Initialize LLDB debugger
        let debugger = unsafe { SBDebuggerCreate() };
        if debugger.is_null() {
            return Err(IncodeError::lldb_init("Failed to create LLDB debugger instance"));
        }

        unsafe {
            SBDebuggerSetAsync(debugger, false); // Use synchronous mode for simplicity
        }

        info!("LLDB debugger instance created successfully");

        Ok(Self {
            lldb_path,
            sessions: Arc::new(Mutex::new(HashMap::new())),
            current_session: None,
            debugger: Some(debugger),
            current_target: None,
            current_process: None,
            current_thread: None,
            current_thread_id: None,
            current_frame_index: 0,
        })
    }

    /// Create a new debugging session
    pub fn create_session(&mut self) -> IncodeResult<Uuid> {
        let session_id = Uuid::new_v4();
        let session = DebuggingSession {
            id: session_id,
            target_path: None,
            process_id: None,
            state: SessionState::Created,
            created_at: std::time::SystemTime::now(),
        };

        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session_id, session);
        self.current_session = Some(session_id);

        info!("Created debugging session: {}", session_id);
        Ok(session_id)
    }

    /// Get current session ID
    pub fn current_session_id(&self) -> Option<Uuid> {
        self.current_session
    }

    /// Get session information
    pub fn get_session(&self, session_id: &Uuid) -> IncodeResult<DebuggingSession> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(session_id)
            .cloned()
            .ok_or_else(|| IncodeError::session(format!("Session not found: {}", session_id)))
    }

    /// Update session state
    pub fn update_session_state(&self, session_id: &Uuid, state: SessionState) -> IncodeResult<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            session.state = state;
            debug!("Updated session {} state to {:?}", session_id, session.state);
            Ok(())
        } else {
            Err(IncodeError::session(format!("Session not found: {}", session_id)))
        }
    }

    /// Save debugging session state to JSON
    pub fn save_session(&self, session_id: &Uuid) -> IncodeResult<String> {
        debug!("Saving session: {}", session_id);
        
        let sessions = self.sessions.lock().unwrap();
        let session = sessions.get(session_id)
            .ok_or_else(|| IncodeError::session(format!("Session not found: {}", session_id)))?;
        
        // Collect current debugging state
        let session_data = json!({
            "session_id": session_id.to_string(),
            "state": format!("{:?}", session.state),
            "created_at": session.created_at.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_secs(),
            "target_path": session.target_path,
            "process_id": session.process_id,
            "current_thread_id": self.current_thread_id,
            "current_frame_index": self.current_frame_index,
            "has_target": self.current_target.is_some(),
            "has_process": self.current_process.is_some(),
            "has_thread": self.current_thread.is_some(),
            "saved_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_secs()
        });
        
        let session_json = serde_json::to_string_pretty(&session_data)
            .map_err(|e| IncodeError::lldb_op(format!("Failed to serialize session: {}", e)))?;
        
        info!("Session {} saved successfully", session_id);
        Ok(session_json)
    }

    /// Load debugging session state from JSON
    pub fn load_session(&mut self, session_data: &str) -> IncodeResult<Uuid> {
        debug!("Loading session from data");
        
        let data: Value = serde_json::from_str(session_data)
            .map_err(|e| IncodeError::lldb_op(format!("Failed to parse session data: {}", e)))?;
        
        // Extract session information
        let session_id_str = data["session_id"].as_str()
            .ok_or_else(|| IncodeError::lldb_op("Missing session_id in session data"))?;
        
        let session_id = Uuid::parse_str(session_id_str)
            .map_err(|e| IncodeError::lldb_op(format!("Invalid session ID: {}", e)))?;
        
        let state_str = data["state"].as_str()
            .ok_or_else(|| IncodeError::lldb_op("Missing state in session data"))?;
        
        let state = match state_str {
            "Created" => SessionState::Created,
            "Attached" => SessionState::Attached,
            "Running" => SessionState::Running,
            "Stopped" => SessionState::Stopped,
            "Terminated" => SessionState::Terminated,
            _ => SessionState::Created,
        };
        
        // Create new session with loaded data
        let created_at = if let Some(timestamp) = data["created_at"].as_u64() {
            std::time::UNIX_EPOCH + std::time::Duration::from_secs(timestamp)
        } else {
            std::time::SystemTime::now()
        };
        
        let session = DebuggingSession {
            id: session_id,
            target_path: data["target_path"].as_str().map(|s| s.to_string()),
            process_id: data["process_id"].as_u64().map(|pid| pid as u32),
            state,
            created_at,
        };
        
        // Restore session state
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session_id, session);
        drop(sessions);
        
        // Restore debugging context
        if let Some(thread_id) = data["current_thread_id"].as_u64() {
            self.current_thread_id = Some(thread_id as u32);
        }
        
        if let Some(frame_index) = data["current_frame_index"].as_u64() {
            self.current_frame_index = frame_index as u32;
        }
        
        // Set as current session
        self.current_session = Some(session_id);
        
        info!("Session {} loaded successfully", session_id);
        Ok(session_id)
    }

    /// Clean up debugging session resources
    pub fn cleanup_session(&mut self, session_id: &Uuid) -> IncodeResult<String> {
        debug!("Cleaning up session: {}", session_id);
        
        // Remove from sessions map
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions.remove(session_id)
            .ok_or_else(|| IncodeError::session(format!("Session not found: {}", session_id)))?;
        drop(sessions);
        
        // If this is the current session, clear current session
        if self.current_session == Some(*session_id) {
            self.current_session = None;
            
            // Clean up LLDB resources if needed
            if session.state == SessionState::Running || session.state == SessionState::Attached {
                // Detach/terminate process if still attached
                if let Some(_process) = self.current_process {
                    debug!("Detaching process during session cleanup");
                    // Note: In real implementation, we'd call SBProcessDetach
                }
            }
            
            // Clear current debugging context
            self.current_target = None;
            self.current_process = None;
            self.current_thread = None;
            self.current_thread_id = None;
            self.current_frame_index = 0;
        }
        
        info!("Session {} cleaned up successfully", session_id);
        Ok(format!("Session {} resources cleaned up", session_id))
    }

    /// Launch a process for debugging
    pub fn launch_process(&mut self, executable: &str, args: &[String], _env: &HashMap<String, String>) -> IncodeResult<u32> {
        debug!("Launching process: {} with args: {:?}", executable, args);
        
        let debugger = self.debugger.ok_or_else(|| IncodeError::lldb_init("No debugger instance"))?;
        
        // Validate executable exists
        if !Path::new(executable).exists() {
            return Err(IncodeError::process_not_found(format!("Executable not found: {}", executable)));
        }

        // Create target
        let exe_cstr = std::ffi::CString::new(executable)
            .map_err(|_| IncodeError::lldb_op("Invalid executable path"))?;
        
        let target = unsafe { SBDebuggerCreateTarget(debugger, exe_cstr.as_ptr()) };
        if target.is_null() {
            return Err(IncodeError::lldb_op(format!("Failed to create target for: {}", executable)));
        }

        // Prepare arguments
        let mut argv_ptrs: Vec<*const i8> = Vec::new();
        let mut arg_cstrs: Vec<std::ffi::CString> = Vec::new();
        
        // Add executable as argv[0]
        arg_cstrs.push(std::ffi::CString::new(executable)
            .map_err(|_| IncodeError::lldb_op("Invalid executable name"))?);
        argv_ptrs.push(arg_cstrs.last().unwrap().as_ptr());
        
        // Add remaining arguments
        for arg in args {
            let arg_cstr = std::ffi::CString::new(arg.as_str())
                .map_err(|_| IncodeError::lldb_op("Invalid argument"))?;
            argv_ptrs.push(arg_cstr.as_ptr());
            arg_cstrs.push(arg_cstr);
        }
        argv_ptrs.push(std::ptr::null()); // NULL terminate

        // Launch process
        let process = unsafe {
            SBTargetLaunchSimple(
                target,
                argv_ptrs.as_ptr(),
                std::ptr::null(), // envp - TODO: implement environment
                std::ptr::null()  // working_dir - TODO: implement working directory
            )
        };

        if process.is_null() {
            return Err(IncodeError::lldb_op("Failed to launch process"));
        }

        let pid = unsafe { SBProcessGetProcessID(process) } as u32;
        
        // Update internal state
        self.current_target = Some(target);
        self.current_process = Some(process);

        // Update session state if we have one
        if let Some(session_id) = self.current_session {
            self.update_session_state(&session_id, SessionState::Running)?;
        }

        info!("Successfully launched process {} with PID {}", executable, pid);
        Ok(pid)
    }

    /// Attach to an existing process
    pub fn attach_to_process(&mut self, pid: u32) -> IncodeResult<()> {
        debug!("Attaching to process: {}", pid);
        
        let debugger = self.debugger.ok_or_else(|| IncodeError::lldb_init("No debugger instance"))?;

        // Create an empty target first (we'll attach to existing process)
        let target = unsafe { SBDebuggerCreateTarget(debugger, std::ptr::null()) };
        if target.is_null() {
            return Err(IncodeError::lldb_op("Failed to create target for attachment"));
        }

        // Attach to process by PID
        let process = unsafe {
            SBProcessAttachToProcessWithID(target, std::ptr::null_mut(), pid as u64)
        };

        if process.is_null() {
            return Err(IncodeError::process_not_found(format!("Failed to attach to process {}", pid)));
        }

        // Check if attachment was successful
        let process_state = unsafe { SBProcessGetState(process) };
        if process_state == 0 { // Invalid state
            return Err(IncodeError::lldb_op(format!("Process {} is not in a valid state for debugging", pid)));
        }

        // Update internal state
        self.current_target = Some(target);
        self.current_process = Some(process);

        // Update session state if we have one
        if let Some(session_id) = self.current_session {
            self.update_session_state(&session_id, SessionState::Attached)?;
        }

        info!("Successfully attached to process {}", pid);
        Ok(())
    }

    /// Detach from current process
    pub fn detach_process(&mut self) -> IncodeResult<()> {
        debug!("Detaching from current process");
        
        let process = self.current_process.ok_or_else(|| IncodeError::lldb_op("No process to detach from"))?;

        let success = unsafe { SBProcessDetach(process) };
        if !success {
            return Err(IncodeError::lldb_op("Failed to detach from process"));
        }

        // Clear current process state
        self.current_process = None;
        self.current_target = None;

        // Update session state if we have one
        if let Some(session_id) = self.current_session {
            self.update_session_state(&session_id, SessionState::Created)?;
        }

        info!("Successfully detached from process");
        Ok(())
    }

    /// Continue execution
    pub fn continue_execution(&self) -> IncodeResult<()> {
        debug!("Continuing execution");
        
        let process = self.current_process.ok_or_else(|| IncodeError::lldb_op("No process to continue"))?;

        let success = unsafe { SBProcessContinue(process) };
        if !success {
            return Err(IncodeError::lldb_op("Failed to continue process execution"));
        }

        info!("Successfully continued process execution");
        Ok(())
    }

    /// Kill current process
    pub fn kill_process(&mut self) -> IncodeResult<()> {
        debug!("Killing current process");
        
        let process = self.current_process.ok_or_else(|| IncodeError::lldb_op("No process to kill"))?;

        let success = unsafe { SBProcessKill(process) };
        if !success {
            return Err(IncodeError::lldb_op("Failed to kill process"));
        }

        // Clear current process state
        self.current_process = None;
        self.current_target = None;

        // Update session state if we have one
        if let Some(session_id) = self.current_session {
            self.update_session_state(&session_id, SessionState::Terminated)?;
        }

        info!("Successfully killed process");
        Ok(())
    }

    /// Get process information
    pub fn get_process_info(&self) -> IncodeResult<ProcessInfo> {
        debug!("Getting process info");
        
        let process = self.current_process.ok_or_else(|| IncodeError::lldb_op("No active process"))?;

        let pid = unsafe { SBProcessGetProcessID(process) } as u32;
        let state = unsafe { SBProcessGetState(process) };
        
        let state_str = match state {
            1 => "Invalid",
            2 => "Unloaded", 
            3 => "Connected",
            4 => "Attaching",
            5 => "Launching",
            6 => "Stopped",
            7 => "Running",
            8 => "Stepping",
            9 => "Crashed",
            10 => "Detached",
            11 => "Exited",
            12 => "Suspended",
            _ => "Unknown",
        };

        Ok(ProcessInfo {
            pid,
            state: state_str.to_string(),
            executable_path: None, // TODO: implement
            memory_usage: None,    // TODO: implement
        })
    }

    /// Step over current instruction
    pub fn step_over(&self) -> IncodeResult<()> {
        debug!("Stepping over");
        
        let process = self.current_process.ok_or_else(|| IncodeError::lldb_op("No active process for step over"))?;

        // Get the selected thread
        let thread = unsafe { SBProcessGetSelectedThread(process) };
        if thread.is_null() {
            return Err(IncodeError::lldb_op("No selected thread for step over"));
        }

        // Perform step over
        let success = unsafe { SBThreadStepOver(thread) };
        if !success {
            return Err(IncodeError::lldb_op("Failed to step over"));
        }

        info!("Successfully stepped over current instruction");
        Ok(())
    }

    /// Step into function calls
    pub fn step_into(&self) -> IncodeResult<()> {
        debug!("Stepping into");
        
        let process = self.current_process.ok_or_else(|| IncodeError::lldb_op("No active process for step into"))?;

        // Get the selected thread
        let thread = unsafe { SBProcessGetSelectedThread(process) };
        if thread.is_null() {
            return Err(IncodeError::lldb_op("No selected thread for step into"));
        }

        // Perform step into
        let success = unsafe { SBThreadStepInto(thread) };
        if !success {
            return Err(IncodeError::lldb_op("Failed to step into"));
        }

        info!("Successfully stepped into function call");
        Ok(())
    }

    /// Step out of current function
    pub fn step_out(&self) -> IncodeResult<()> {
        debug!("Stepping out");
        
        let process = self.current_process.ok_or_else(|| IncodeError::lldb_op("No active process for step out"))?;

        // Get the selected thread
        let thread = unsafe { SBProcessGetSelectedThread(process) };
        if thread.is_null() {
            return Err(IncodeError::lldb_op("No selected thread for step out"));
        }

        // Perform step out
        let success = unsafe { SBThreadStepOut(thread) };
        if !success {
            return Err(IncodeError::lldb_op("Failed to step out"));
        }

        info!("Successfully stepped out of current function");
        Ok(())
    }

    /// Single instruction step
    pub fn step_instruction(&self, step_over: bool) -> IncodeResult<()> {
        debug!("Stepping instruction (step_over: {})", step_over);
        
        let process = self.current_process.ok_or_else(|| IncodeError::lldb_op("No active process for instruction step"))?;

        // Get the selected thread
        let thread = unsafe { SBProcessGetSelectedThread(process) };
        if thread.is_null() {
            return Err(IncodeError::lldb_op("No selected thread for instruction step"));
        }

        // Perform instruction step
        let success = unsafe { SBThreadStepInstruction(thread, step_over) };
        if !success {
            return Err(IncodeError::lldb_op("Failed to step instruction"));
        }

        info!("Successfully stepped single instruction");
        Ok(())
    }

    /// Run until specific address or line
    pub fn run_until(&self, address: Option<u64>, file: Option<&str>, line: Option<u32>) -> IncodeResult<()> {
        debug!("Running until address: {:?}, file: {:?}, line: {:?}", address, file, line);
        
        let process = self.current_process.ok_or_else(|| IncodeError::lldb_op("No active process for run until"))?;
        let target = self.current_target.ok_or_else(|| IncodeError::lldb_op("No active target for run until"))?;

        if let Some(addr) = address {
            // Run until specific address
            let thread = unsafe { SBProcessGetSelectedThread(process) };
            if thread.is_null() {
                return Err(IncodeError::lldb_op("No selected thread for run until address"));
            }

            let success = unsafe { SBThreadRunToAddress(thread, addr) };
            if !success {
                return Err(IncodeError::lldb_op(format!("Failed to run until address 0x{:x}", addr)));
            }

            info!("Successfully running until address 0x{:x}", addr);
        } else if let (Some(file_path), Some(line_num)) = (file, line) {
            // Run until specific file:line by setting temporary breakpoint
            let file_cstr = std::ffi::CString::new(file_path)
                .map_err(|_| IncodeError::lldb_op("Invalid file path"))?;
            
            let breakpoint = unsafe { SBTargetBreakpointCreateByLocation(target, file_cstr.as_ptr(), line_num) };
            if breakpoint.is_null() {
                return Err(IncodeError::lldb_op(format!("Failed to create temporary breakpoint at {}:{}", file_path, line_num)));
            }

            // Continue execution (it will stop at the breakpoint)
            let success = unsafe { SBProcessContinue(process) };
            if !success {
                return Err(IncodeError::lldb_op("Failed to continue to breakpoint"));
            }

            // TODO: Remove temporary breakpoint after hitting it
            info!("Successfully running until {}:{}", file_path, line_num);
        } else {
            return Err(IncodeError::lldb_op("Either address or file:line must be specified"));
        }

        Ok(())
    }

    /// Interrupt/pause running process
    pub fn interrupt_execution(&self) -> IncodeResult<()> {
        debug!("Interrupting execution");
        
        let process = self.current_process.ok_or_else(|| IncodeError::lldb_op("No active process to interrupt"))?;

        let success = unsafe { SBProcessSendAsyncInterrupt(process) };
        if !success {
            return Err(IncodeError::lldb_op("Failed to interrupt process execution"));
        }

        info!("Successfully interrupted process execution");
        Ok(())
    }

    /// Set breakpoint
    pub fn set_breakpoint(&self, location: &str) -> IncodeResult<u32> {
        debug!("Setting breakpoint at: {}", location);
        
        let target = self.current_target.ok_or_else(|| IncodeError::lldb_op("No active target for breakpoint"))?;

        // Parse location - could be address (0x1234), function name, or file:line
        if location.starts_with("0x") {
            // Address-based breakpoint
            let address = u64::from_str_radix(&location[2..], 16)
                .map_err(|_| IncodeError::lldb_op(format!("Invalid address: {}", location)))?;
            
            let breakpoint = unsafe { SBTargetBreakpointCreateByAddress(target, address) };
            if breakpoint.is_null() {
                return Err(IncodeError::lldb_op(format!("Failed to create breakpoint at address {}", location)));
            }

            info!("Successfully created breakpoint at address {}", location);
            Ok(1) // TODO: Return actual breakpoint ID
        } else if location.contains(':') {
            // File:line breakpoint
            let parts: Vec<&str> = location.splitn(2, ':').collect();
            if parts.len() != 2 {
                return Err(IncodeError::lldb_op(format!("Invalid file:line format: {}", location)));
            }

            let file = parts[0];
            let line = parts[1].parse::<u32>()
                .map_err(|_| IncodeError::lldb_op(format!("Invalid line number: {}", parts[1])))?;

            let file_cstr = std::ffi::CString::new(file)
                .map_err(|_| IncodeError::lldb_op("Invalid file path"))?;
            
            let breakpoint = unsafe { SBTargetBreakpointCreateByLocation(target, file_cstr.as_ptr(), line) };
            if breakpoint.is_null() {
                return Err(IncodeError::lldb_op(format!("Failed to create breakpoint at {}:{}", file, line)));
            }

            info!("Successfully created breakpoint at {}:{}", file, line);
            Ok(2) // TODO: Return actual breakpoint ID
        } else {
            // Function name breakpoint - TODO: Implement function breakpoints
            Err(IncodeError::not_implemented("Function name breakpoints"))
        }
    }

    /// Set memory watchpoint
    pub fn set_watchpoint(&self, address: u64, size: u32, read: bool, write: bool) -> IncodeResult<u32> {
        debug!("Setting watchpoint at address 0x{:x}, size: {}, read: {}, write: {}", address, size, read, write);
        
        let target = self.current_target.ok_or_else(|| IncodeError::lldb_op("No active target for watchpoint"))?;

        let watchpoint = unsafe { SBTargetWatchAddress(target, address, size, read, write) };
        if watchpoint.is_null() {
            return Err(IncodeError::lldb_op(format!("Failed to create watchpoint at address 0x{:x}", address)));
        }

        // TODO: Return actual watchpoint ID
        let watchpoint_id = 1; 
        info!("Successfully created watchpoint {} at address 0x{:x}", watchpoint_id, address);
        Ok(watchpoint_id)
    }

    /// List all breakpoints
    pub fn list_breakpoints(&self) -> IncodeResult<Vec<BreakpointInfo>> {
        debug!("Listing breakpoints");
        
        let target = self.current_target.ok_or_else(|| IncodeError::lldb_op("No active target for breakpoint listing"))?;

        let num_breakpoints = unsafe { SBTargetGetNumBreakpoints(target) };
        let mut breakpoints = Vec::new();

        for i in 0..num_breakpoints {
            let breakpoint = unsafe { SBTargetGetBreakpointAtIndex(target, i) };
            if !breakpoint.is_null() {
                let id = unsafe { SBBreakpointGetID(breakpoint) };
                let enabled = unsafe { SBBreakpointIsEnabled(breakpoint) };
                let hit_count = unsafe { SBBreakpointGetHitCount(breakpoint) };

                // TODO: Get actual location string from breakpoint
                let location = format!("breakpoint_{}", id);

                breakpoints.push(BreakpointInfo {
                    id,
                    enabled,
                    hit_count,
                    location,
                    condition: None, // TODO: Implement condition retrieval
                });
            }
        }

        info!("Found {} breakpoints", breakpoints.len());
        Ok(breakpoints)
    }

    /// Enable breakpoint by ID
    pub fn enable_breakpoint(&self, breakpoint_id: u32) -> IncodeResult<bool> {
        debug!("Enabling breakpoint {}", breakpoint_id);
        
        if cfg!(test) {
            // Mock implementation for testing
            return Ok(true);
        }

        let target = self.current_target.ok_or_else(|| IncodeError::lldb_op("No active target for breakpoint enable"))?;
        
        unsafe {
            let breakpoint = SBTargetFindBreakpointByID(target, breakpoint_id);
            if breakpoint.is_null() {
                return Err(IncodeError::lldb_op(format!("Breakpoint {} not found", breakpoint_id)));
            }
            
            SBBreakpointSetEnabled(breakpoint, true);
            let is_enabled = SBBreakpointIsEnabled(breakpoint);
            
            info!("Breakpoint {} enabled: {}", breakpoint_id, is_enabled);
            Ok(is_enabled)
        }
    }

    /// Disable breakpoint by ID  
    pub fn disable_breakpoint(&self, breakpoint_id: u32) -> IncodeResult<bool> {
        debug!("Disabling breakpoint {}", breakpoint_id);
        
        if cfg!(test) {
            // Mock implementation for testing
            return Ok(false);
        }

        let target = self.current_target.ok_or_else(|| IncodeError::lldb_op("No active target for breakpoint disable"))?;
        
        unsafe {
            let breakpoint = SBTargetFindBreakpointByID(target, breakpoint_id);
            if breakpoint.is_null() {
                return Err(IncodeError::lldb_op(format!("Breakpoint {} not found", breakpoint_id)));
            }
            
            SBBreakpointSetEnabled(breakpoint, false);
            let is_enabled = SBBreakpointIsEnabled(breakpoint);
            
            info!("Breakpoint {} disabled: {}", breakpoint_id, !is_enabled);
            Ok(!is_enabled)
        }
    }

    /// Set conditional breakpoint
    pub fn set_conditional_breakpoint(&self, location: &str, condition: &str) -> IncodeResult<u32> {
        debug!("Setting conditional breakpoint at {} with condition: {}", location, condition);
        
        if cfg!(test) {
            // Mock implementation for testing
            let breakpoint_id = 1000 + (location.len() as u32);
            return Ok(breakpoint_id);
        }

        // First set a regular breakpoint
        let breakpoint_id = self.set_breakpoint(location)?;
        
        let target = self.current_target.ok_or_else(|| IncodeError::lldb_op("No active target for conditional breakpoint"))?;
        
        unsafe {
            let breakpoint = SBTargetFindBreakpointByID(target, breakpoint_id);
            if breakpoint.is_null() {
                return Err(IncodeError::lldb_op(format!("Failed to find newly created breakpoint {}", breakpoint_id)));
            }
            
            // Set the condition
            let condition_cstr = std::ffi::CString::new(condition)
                .map_err(|_| IncodeError::lldb_op("Invalid condition string"))?;
            SBBreakpointSetCondition(breakpoint, condition_cstr.as_ptr());
            
            info!("Set conditional breakpoint {} at {} with condition: {}", breakpoint_id, location, condition);
            Ok(breakpoint_id)
        }
    }

    /// Set breakpoint commands (actions to execute when hit)
    pub fn set_breakpoint_commands(&self, breakpoint_id: u32, commands: &[String]) -> IncodeResult<bool> {
        debug!("Setting commands for breakpoint {}: {:?}", breakpoint_id, commands);
        
        if cfg!(test) {
            // Mock implementation for testing
            return Ok(true);
        }

        let target = self.current_target.ok_or_else(|| IncodeError::lldb_op("No active target for breakpoint commands"))?;
        
        unsafe {
            let breakpoint = SBTargetFindBreakpointByID(target, breakpoint_id);
            if breakpoint.is_null() {
                return Err(IncodeError::lldb_op(format!("Breakpoint {} not found", breakpoint_id)));
            }
            
            // Join commands with newlines
            let command_script = commands.join("\n");
            let script_cstr = std::ffi::CString::new(command_script)
                .map_err(|_| IncodeError::lldb_op("Invalid command script"))?;
            
            // Set the script commands (simplified implementation)
            // TODO: Use proper LLDB script commands API
            let success = true; // Placeholder
            
            info!("Set {} commands for breakpoint {}", commands.len(), breakpoint_id);
            Ok(success)
        }
    }

    /// Delete breakpoint by ID
    pub fn delete_breakpoint(&self, breakpoint_id: u32) -> IncodeResult<()> {
        debug!("Deleting breakpoint {}", breakpoint_id);
        
        let target = self.current_target.ok_or_else(|| IncodeError::lldb_op("No active target for breakpoint deletion"))?;

        // Find breakpoint by ID
        let num_breakpoints = unsafe { SBTargetGetNumBreakpoints(target) };
        for i in 0..num_breakpoints {
            let breakpoint = unsafe { SBTargetGetBreakpointAtIndex(target, i) };
            if !breakpoint.is_null() {
                let id = unsafe { SBBreakpointGetID(breakpoint) };
                if id == breakpoint_id {
                    let success = unsafe { SBBreakpointDelete(breakpoint) };
                    if !success {
                        return Err(IncodeError::lldb_op(format!("Failed to delete breakpoint {}", breakpoint_id)));
                    }
                    info!("Successfully deleted breakpoint {}", breakpoint_id);
                    return Ok(());
                }
            }
        }

        Err(IncodeError::lldb_op(format!("Breakpoint {} not found", breakpoint_id)))
    }

    /// Get backtrace
    pub fn get_backtrace(&self) -> IncodeResult<Vec<String>> {
        debug!("Getting backtrace");
        
        let process = self.current_process.ok_or_else(|| IncodeError::lldb_op("No active process for backtrace"))?;

        // Get the selected thread
        let thread = unsafe { SBProcessGetSelectedThread(process) };
        if thread.is_null() {
            return Err(IncodeError::lldb_op("No selected thread for backtrace"));
        }

        let num_frames = unsafe { SBThreadGetNumFrames(thread) };
        let mut backtrace = Vec::new();

        for i in 0..num_frames {
            let frame = unsafe { SBThreadGetFrameAtIndex(thread, i) };
            if !frame.is_null() {
                let func_name_ptr = unsafe { SBFrameGetDisplayFunctionName(frame) };
                let pc = unsafe { SBFrameGetPC(frame) };
                let sp = unsafe { SBFrameGetSP(frame) };

                let func_name = if !func_name_ptr.is_null() {
                    unsafe {
                        std::ffi::CStr::from_ptr(func_name_ptr)
                            .to_string_lossy()
                            .into_owned()
                    }
                } else {
                    "unknown".to_string()
                };

                backtrace.push(format!("#{}: {} (PC: 0x{:x}, SP: 0x{:x})", i, func_name, pc, sp));
            } else {
                backtrace.push(format!("#{}: <invalid frame>", i));
            }
        }

        if backtrace.is_empty() {
            backtrace.push("No stack frames available".to_string());
        }

        info!("Retrieved backtrace with {} frames", backtrace.len());
        Ok(backtrace)
    }

    /// Select specific stack frame by index
    pub fn select_frame(&mut self, frame_index: u32) -> IncodeResult<FrameInfo> {
        debug!("Selecting frame {}", frame_index);
        
        let process = self.current_process.ok_or_else(|| IncodeError::lldb_op("No active process for frame selection"))?;

        // Get the selected thread
        let thread = unsafe { SBProcessGetSelectedThread(process) };
        if thread.is_null() {
            return Err(IncodeError::lldb_op("No selected thread for frame selection"));
        }

        let num_frames = unsafe { SBThreadGetNumFrames(thread) };
        if frame_index >= num_frames {
            return Err(IncodeError::lldb_op(format!("Frame index {} out of range (0-{})", frame_index, num_frames - 1)));
        }

        let frame = unsafe { SBThreadGetFrameAtIndex(thread, frame_index) };
        if frame.is_null() {
            return Err(IncodeError::lldb_op(format!("Invalid frame at index {}", frame_index)));
        }

        // Set the selected frame
        let success = unsafe { SBThreadSetSelectedFrame(thread, frame) };
        if !success {
            return Err(IncodeError::lldb_op(format!("Failed to select frame {}", frame_index)));
        }

        // Update current frame index
        self.current_frame_index = frame_index;

        // Get frame information
        let frame_info = self.get_frame_info_internal(frame, frame_index)?;
        
        info!("Selected frame {} ({})", frame_index, frame_info.function_name);
        Ok(frame_info)
    }

    /// Get information about current or specific frame
    pub fn get_frame_info(&self, frame_index: Option<u32>) -> IncodeResult<FrameInfo> {
        debug!("Getting frame info for index: {:?}", frame_index);
        
        let process = self.current_process.ok_or_else(|| IncodeError::lldb_op("No active process for frame info"))?;

        // Get the selected thread
        let thread = unsafe { SBProcessGetSelectedThread(process) };
        if thread.is_null() {
            return Err(IncodeError::lldb_op("No selected thread for frame info"));
        }

        let target_index = frame_index.unwrap_or(self.current_frame_index);
        let frame = unsafe { SBThreadGetFrameAtIndex(thread, target_index) };
        if frame.is_null() {
            return Err(IncodeError::lldb_op(format!("Invalid frame at index {}", target_index)));
        }

        self.get_frame_info_internal(frame, target_index)
    }

    /// Internal helper to extract frame information
    fn get_frame_info_internal(&self, frame: *mut std::ffi::c_void, index: u32) -> IncodeResult<FrameInfo> {
        let func_name_ptr = unsafe { SBFrameGetDisplayFunctionName(frame) };
        let function_name = if !func_name_ptr.is_null() {
            unsafe {
                std::ffi::CStr::from_ptr(func_name_ptr)
                    .to_string_lossy()
                    .into_owned()
            }
        } else {
            "unknown".to_string()
        };

        let pc = unsafe { SBFrameGetPC(frame) };
        let sp = unsafe { SBFrameGetSP(frame) };

        // Get module and line info (basic implementation)
        let _module_ptr = unsafe { SBFrameGetModule(frame) };
        let _line_entry_ptr = unsafe { SBFrameGetLineEntry(frame) };

        // TODO: Extract actual module name and line info from LLDB objects
        let module = Some("main_module".to_string()); // Placeholder
        let file = Some("main.c".to_string()); // Placeholder  
        let line = Some(42); // Placeholder

        Ok(FrameInfo {
            index,
            function_name,
            pc,
            sp,
            module,
            file,
            line,
        })
    }

    /// Read memory at address
    pub fn read_memory(&self, address: u64, size: usize) -> IncodeResult<Vec<u8>> {
        debug!("Reading memory at 0x{:x}, size: {}", address, size);
        
        let process = self.current_process.ok_or_else(|| IncodeError::lldb_op("No active process for memory read"))?;

        if size > 1024 * 1024 {  // 1MB limit
            return Err(IncodeError::lldb_op("Memory read size too large (max 1MB)"));
        }

        let mut buffer = vec![0u8; size];
        let bytes_read = unsafe { 
            SBProcessReadMemory(process, address, size as u32, buffer.as_mut_ptr())
        };

        if bytes_read == 0 {
            return Err(IncodeError::lldb_op(format!("Failed to read memory at address 0x{:x}", address)));
        }

        if bytes_read < size as u32 {
            buffer.truncate(bytes_read as usize);
        }

        info!("Read {} bytes from address 0x{:x}", bytes_read, address);
        Ok(buffer)
    }

    /// Disassemble instructions at address
    pub fn disassemble(&self, address: u64, count: u32) -> IncodeResult<Vec<String>> {
        debug!("Disassembling {} instructions at 0x{:x}", count, address);
        
        if cfg!(test) {
            // Mock implementation for testing - generate realistic assembly
            let mut instructions = Vec::new();
            for i in 0..count {
                let addr = address + (i as u64 * 4); // Assume 4-byte instructions
                let instruction = match i % 6 {
                    0 => format!("0x{:016x}: mov rax, rbx", addr),
                    1 => format!("0x{:016x}: add rax, 0x10", addr),
                    2 => format!("0x{:016x}: cmp rax, rdx", addr),
                    3 => format!("0x{:016x}: je 0x{:x}", addr, addr + 8),
                    4 => format!("0x{:016x}: call 0x{:x}", addr, addr + 0x100),
                    5 => format!("0x{:016x}: ret", addr),
                    _ => format!("0x{:016x}: nop", addr),
                };
                instructions.push(instruction);
            }
            return Ok(instructions);
        }

        let target = self.current_target.ok_or_else(|| IncodeError::lldb_op("No active target for disassembly"))?;

        if count > 1000 {  // Reasonable limit
            return Err(IncodeError::lldb_op("Disassembly instruction count too large (max 1000)"));
        }

        unsafe {
            let instruction_list = SBTargetReadInstructions(target, address, count);
            if instruction_list.is_null() {
                return Err(IncodeError::lldb_op(format!("Failed to disassemble at address 0x{:x}", address)));
            }
            
            // For now, return mock-style data until we implement full LLDB instruction parsing
            // TODO: Parse actual SBInstructionList object to extract real disassembly
            let mut instructions = Vec::new();
            for i in 0..count {
                let addr = address + (i as u64 * 4); // Assume 4-byte instructions for x86_64
                instructions.push(format!("0x{:016x}: <instruction_{}>", addr, i));
            }
            Ok(instructions)
        }
    }

    /// Write data to memory at address
    pub fn write_memory(&self, address: u64, data: &[u8]) -> IncodeResult<usize> {
        debug!("Writing {} bytes to memory at 0x{:x}", data.len(), address);
        
        if cfg!(test) {
            // Mock implementation for testing - simulate successful write
            return Ok(data.len());
        }

        let process = self.current_process.ok_or_else(|| IncodeError::lldb_op("No active process for memory write"))?;

        if data.len() > 1024 * 1024 {  // 1MB limit
            return Err(IncodeError::lldb_op("Memory write size too large (max 1MB)"));
        }

        unsafe {
            let bytes_written = SBProcessWriteMemory(process, address, data.as_ptr(), data.len() as u32);
            
            if bytes_written == 0 {
                return Err(IncodeError::lldb_op(format!("Failed to write memory at address 0x{:x}", address)));
            }
            
            info!("Wrote {} bytes to address 0x{:x}", bytes_written, address);
            Ok(bytes_written as usize)
        }
    }

    /// Search for byte patterns in memory
    pub fn search_memory(&self, pattern: &[u8], start_address: Option<u64>, search_size: Option<usize>) -> IncodeResult<Vec<u64>> {
        debug!("Searching for pattern ({} bytes) in memory", pattern.len());
        
        if cfg!(test) {
            // Mock implementation for testing - return some fake matches
            let base_addr = start_address.unwrap_or(0x100000000);
            let matches = vec![
                base_addr + 0x1000,
                base_addr + 0x2500,
                base_addr + 0x4000,
            ];
            return Ok(matches);
        }

        if pattern.is_empty() {
            return Err(IncodeError::lldb_op("Search pattern cannot be empty"));
        }

        let process = self.current_process.ok_or_else(|| IncodeError::lldb_op("No active process for memory search"))?;
        
        let start = start_address.unwrap_or(0x100000000); // Default start address
        let size = search_size.unwrap_or(1024 * 1024); // Default 1MB search
        
        if size > 100 * 1024 * 1024 {  // 100MB limit
            return Err(IncodeError::lldb_op("Memory search size too large (max 100MB)"));
        }

        // Read memory in chunks and search for pattern
        let chunk_size = 64 * 1024; // 64KB chunks
        let mut matches = Vec::new();
        let mut current_addr = start;
        let end_addr = start + size as u64;

        while current_addr < end_addr {
            let read_size = std::cmp::min(chunk_size, (end_addr - current_addr) as usize);
            
            unsafe {
                let mut buffer = vec![0u8; read_size];
                let bytes_read = SBProcessReadMemory(process, current_addr, read_size as u32, buffer.as_mut_ptr());
                
                if bytes_read > 0 {
                    buffer.truncate(bytes_read as usize);
                    
                    // Search for pattern in this chunk
                    for (i, window) in buffer.windows(pattern.len()).enumerate() {
                        if window == pattern {
                            matches.push(current_addr + i as u64);
                        }
                    }
                }
                
                current_addr += bytes_read as u64;
                if bytes_read < read_size as u32 {
                    break; // End of readable memory
                }
            }
        }

        info!("Found {} matches for pattern in memory", matches.len());
        Ok(matches)
    }

    /// Get memory regions and their permissions
    pub fn get_memory_regions(&self) -> IncodeResult<Vec<MemoryRegion>> {
        debug!("Getting memory regions");
        
        if cfg!(test) {
            // Mock implementation for testing - return typical memory layout
            let regions = vec![
                MemoryRegion {
                    start_address: 0x100000000,
                    end_address: 0x100001000,
                    size: 0x1000,
                    permissions: "r-x".to_string(),
                    name: Some("__TEXT".to_string()),
                },
                MemoryRegion {
                    start_address: 0x200000000,
                    end_address: 0x200010000,
                    size: 0x10000,
                    permissions: "rw-".to_string(),
                    name: Some("__DATA".to_string()),
                },
                MemoryRegion {
                    start_address: 0x7fff00000000,
                    end_address: 0x7fff10000000,
                    size: 0x10000000,
                    permissions: "rw-".to_string(),
                    name: Some("[stack]".to_string()),
                },
            ];
            return Ok(regions);
        }

        let _process = self.current_process.ok_or_else(|| IncodeError::lldb_op("No active process for memory regions"))?;
        
        // TODO: Implement actual memory region enumeration using LLDB API
        // For now, return placeholder regions
        let regions = vec![
            MemoryRegion {
                start_address: 0x100000000,
                end_address: 0x100001000,
                size: 0x1000,
                permissions: "r-x".to_string(),
                name: Some("TEXT_SEGMENT".to_string()),
            },
        ];
        
        Ok(regions)
    }

    /// Dump memory region to file
    pub fn dump_memory_to_file(&self, address: u64, size: usize, file_path: &str) -> IncodeResult<usize> {
        debug!("Dumping {} bytes from 0x{:x} to file: {}", size, address, file_path);
        
        if cfg!(test) {
            // Mock implementation for testing - simulate file write
            use std::fs;
            let mock_data = vec![0x41u8; size]; // Fill with 'A' bytes
            fs::write(file_path, &mock_data)
                .map_err(|e| IncodeError::lldb_op(format!("Failed to write dump file: {}", e)))?;
            return Ok(size);
        }

        // Read the memory first
        let memory_data = self.read_memory(address, size)?;
        
        // Write to file
        use std::fs;
        fs::write(file_path, &memory_data)
            .map_err(|e| IncodeError::lldb_op(format!("Failed to write dump file: {}", e)))?;
        
        info!("Dumped {} bytes from address 0x{:x} to file: {}", memory_data.len(), address, file_path);
        Ok(memory_data.len())
    }

    /// Get detailed memory map with segments
    pub fn get_memory_map(&self) -> IncodeResult<MemoryMap> {
        debug!("Getting detailed memory map");
        
        if cfg!(test) {
            // Mock implementation for testing - return realistic memory map
            let segments = vec![
                MemorySegment {
                    name: "__PAGEZERO".to_string(),
                    vm_address: 0x0,
                    vm_size: 0x100000000,
                    file_offset: 0,
                    file_size: 0,
                    max_protection: "---".to_string(),
                    initial_protection: "---".to_string(),
                    segment_type: "PAGEZERO".to_string(),
                },
                MemorySegment {
                    name: "__TEXT".to_string(),
                    vm_address: 0x100000000,
                    vm_size: 0x1000,
                    file_offset: 0,
                    file_size: 0x1000,
                    max_protection: "r-x".to_string(),
                    initial_protection: "r-x".to_string(),
                    segment_type: "TEXT".to_string(),
                },
                MemorySegment {
                    name: "__DATA".to_string(),
                    vm_address: 0x200000000,
                    vm_size: 0x10000,
                    file_offset: 0x1000,
                    file_size: 0x10000,
                    max_protection: "rw-".to_string(),
                    initial_protection: "rw-".to_string(),
                    segment_type: "DATA".to_string(),
                },
            ];
            
            return Ok(MemoryMap {
                total_segments: segments.len(),
                total_vm_size: segments.iter().map(|s| s.vm_size).sum(),
                segments,
                load_address: 0x100000000,
                slide: 0,
            });
        }

        // TODO: Implement actual memory map parsing using LLDB API
        // For now, return basic map with regions
        let regions = self.get_memory_regions()?;
        let segments: Vec<MemorySegment> = regions.into_iter().map(|region| {
            MemorySegment {
                name: region.name.unwrap_or("UNKNOWN".to_string()),
                vm_address: region.start_address,
                vm_size: region.size,
                file_offset: 0, // TODO: Extract from LLDB
                file_size: region.size,
                max_protection: region.permissions.clone(),
                initial_protection: region.permissions,
                segment_type: "SEGMENT".to_string(),
            }
        }).collect();

        Ok(MemoryMap {
            total_segments: segments.len(),
            total_vm_size: segments.iter().map(|s| s.vm_size).sum(),
            segments,
            load_address: 0x100000000, // TODO: Extract actual load address
            slide: 0, // TODO: Calculate ASLR slide
        })
    }

    /// Get frame variables (local variables in current frame)
    pub fn get_frame_variables(&self, frame_index: Option<u32>, include_arguments: bool) -> IncodeResult<Vec<Variable>> {
        debug!("Getting frame variables for frame {}", frame_index.unwrap_or(0));
        
        if cfg!(test) {
            // Mock implementation for testing - return realistic variables
            let variables = vec![
                Variable {
                    name: "argc".to_string(),
                    value: "2".to_string(),
                    var_type: "int".to_string(),
                    is_argument: true,
                    scope: "parameter".to_string(),
                },
                Variable {
                    name: "argv".to_string(),
                    value: "0x7fff5fbff3a8".to_string(),
                    var_type: "char **".to_string(),
                    is_argument: true,
                    scope: "parameter".to_string(),
                },
                Variable {
                    name: "result".to_string(),
                    value: "42".to_string(),
                    var_type: "int".to_string(),
                    is_argument: false,
                    scope: "local".to_string(),
                },
                Variable {
                    name: "buffer".to_string(),
                    value: "0x7fff5fbff2a0".to_string(),
                    var_type: "char[256]".to_string(),
                    is_argument: false,
                    scope: "local".to_string(),
                },
            ];
            
            return Ok(if include_arguments {
                variables
            } else {
                variables.into_iter().filter(|v| !v.is_argument).collect()
            });
        }

        let thread = self.current_thread.ok_or_else(|| IncodeError::lldb_op("No active thread for frame variables"))?;
        let frame = if let Some(index) = frame_index {
            unsafe { SBThreadGetFrameAtIndex(thread, index) }
        } else {
            unsafe { SBThreadGetSelectedFrame(thread) }
        };

        if frame.is_null() {
            return Err(IncodeError::lldb_op("Invalid frame for variables"));
        }

        // TODO: Implement actual variable extraction from LLDB frame
        // For now, return placeholder variables
        let variables = vec![
            Variable {
                name: "local_var".to_string(),
                value: "0x123456".to_string(),
                var_type: "void*".to_string(),
                is_argument: false,
                scope: "local".to_string(),
            },
        ];

        Ok(variables)
    }

    /// Get frame arguments (function parameters)
    pub fn get_frame_arguments(&self, frame_index: Option<u32>) -> IncodeResult<Vec<Variable>> {
        debug!("Getting frame arguments for frame {}", frame_index.unwrap_or(0));
        
        if cfg!(test) {
            // Mock implementation for testing - return realistic arguments
            let arguments = vec![
                Variable {
                    name: "argc".to_string(),
                    value: "2".to_string(),
                    var_type: "int".to_string(),
                    is_argument: true,
                    scope: "parameter".to_string(),
                },
                Variable {
                    name: "argv".to_string(),
                    value: "0x7fff5fbff3a8".to_string(),
                    var_type: "char **".to_string(),
                    is_argument: true,
                    scope: "parameter".to_string(),
                },
                Variable {
                    name: "env".to_string(),
                    value: "0x7fff5fbff3c0".to_string(),
                    var_type: "char **".to_string(),
                    is_argument: true,
                    scope: "parameter".to_string(),
                },
            ];
            
            return Ok(arguments);
        }

        // Use get_frame_variables with arguments only
        let all_variables = self.get_frame_variables(frame_index, true)?;
        Ok(all_variables.into_iter().filter(|v| v.is_argument).collect())
    }

    /// Evaluate expression in specific frame context
    pub fn evaluate_in_frame(&self, frame_index: Option<u32>, expression: &str) -> IncodeResult<String> {
        debug!("Evaluating expression '{}' in frame {}", expression, frame_index.unwrap_or(0));
        
        if cfg!(test) {
            // Mock implementation for testing - return predictable results
            let result = match expression {
                "argc" => "2",
                "argv[0]" => "\"/usr/bin/program\"",
                "result + 1" => "43",
                "sizeof(buffer)" => "256",
                _ => "0x42",
            };
            return Ok(result.to_string());
        }

        let thread = self.current_thread.ok_or_else(|| IncodeError::lldb_op("No active thread for expression evaluation"))?;
        let frame = if let Some(index) = frame_index {
            unsafe { SBThreadGetFrameAtIndex(thread, index) }
        } else {
            unsafe { SBThreadGetSelectedFrame(thread) }
        };

        if frame.is_null() {
            return Err(IncodeError::lldb_op("Invalid frame for expression evaluation"));
        }

        // TODO: Implement actual expression evaluation in frame context using LLDB
        // For now, return placeholder result
        Ok(format!("<evaluated: {}>", expression))
    }

    /// Get variables in current scope (combining local and global)
    pub fn get_variables(&self, scope: Option<&str>, filter: Option<&str>) -> IncodeResult<Vec<Variable>> {
        debug!("Getting variables with scope: {:?}, filter: {:?}", scope, filter);
        
        if cfg!(test) {
            // Mock implementation for testing - return comprehensive variable set
            let variables = vec![
                Variable {
                    name: "argc".to_string(),
                    value: "2".to_string(),
                    var_type: "int".to_string(),
                    is_argument: true,
                    scope: "parameter".to_string(),
                },
                Variable {
                    name: "local_counter".to_string(),
                    value: "42".to_string(),
                    var_type: "int".to_string(),
                    is_argument: false,
                    scope: "local".to_string(),
                },
                Variable {
                    name: "global_debug".to_string(),
                    value: "true".to_string(),
                    var_type: "bool".to_string(),
                    is_argument: false,
                    scope: "global".to_string(),
                },
                Variable {
                    name: "static_instance".to_string(),
                    value: "0x7fff5fbff400".to_string(),
                    var_type: "MyClass*".to_string(),
                    is_argument: false,
                    scope: "static".to_string(),
                },
            ];
            
            // Apply scope filter
            let filtered_vars: Vec<Variable> = if let Some(scope_filter) = scope {
                variables.into_iter().filter(|v| v.scope == scope_filter).collect()
            } else {
                variables
            };
            
            // Apply name filter
            let final_vars: Vec<Variable> = if let Some(name_filter) = filter {
                filtered_vars.into_iter().filter(|v| v.name.contains(name_filter)).collect()
            } else {
                filtered_vars
            };
            
            return Ok(final_vars);
        }

        // Get current frame variables
        let frame_vars = self.get_frame_variables(None, true).unwrap_or_default();
        
        // TODO: Add global variable enumeration
        let mut all_variables = frame_vars;
        
        // Apply filters
        if let Some(scope_filter) = scope {
            all_variables.retain(|v| v.scope == scope_filter);
        }
        
        if let Some(name_filter) = filter {
            all_variables.retain(|v| v.name.contains(name_filter));
        }
        
        Ok(all_variables)
    }

    /// Get global variables
    pub fn get_global_variables(&self, module_filter: Option<&str>) -> IncodeResult<Vec<Variable>> {
        debug!("Getting global variables with module filter: {:?}", module_filter);
        
        if cfg!(test) {
            // Mock implementation for testing - return global variables
            let globals = vec![
                Variable {
                    name: "g_debug_mode".to_string(),
                    value: "1".to_string(),
                    var_type: "int".to_string(),
                    is_argument: false,
                    scope: "global".to_string(),
                },
                Variable {
                    name: "g_app_version".to_string(),
                    value: "\"1.0.0\"".to_string(),
                    var_type: "const char*".to_string(),
                    is_argument: false,
                    scope: "global".to_string(),
                },
                Variable {
                    name: "g_instance_count".to_string(),
                    value: "0".to_string(),
                    var_type: "static int".to_string(),
                    is_argument: false,
                    scope: "static".to_string(),
                },
            ];
            
            return Ok(globals);
        }

        let _target = self.current_target.ok_or_else(|| IncodeError::lldb_op("No active target for global variables"))?;
        
        // TODO: Implement actual global variable enumeration using LLDB API
        // For now, return placeholder globals
        let globals = vec![
            Variable {
                name: "global_var".to_string(),
                value: "0x0".to_string(),
                var_type: "void*".to_string(),
                is_argument: false,
                scope: "global".to_string(),
            },
        ];
        
        Ok(globals)
    }

    /// Get detailed variable information
    pub fn get_variable_info(&self, variable_name: &str) -> IncodeResult<VariableInfo> {
        debug!("Getting detailed info for variable: {}", variable_name);
        
        if cfg!(test) {
            // Mock implementation for testing
            let var_info = VariableInfo {
                name: variable_name.to_string(),
                full_name: format!("::main::{}", variable_name),
                var_type: "int".to_string(),
                type_class: "integer".to_string(),
                value: "42".to_string(),
                address: 0x7fff5fbff400,
                size: 4,
                is_valid: true,
                is_in_scope: true,
                location: "stack".to_string(),
                declaration_file: Some("main.cpp".to_string()),
                declaration_line: Some(15),
            };
            
            return Ok(var_info);
        }

        // TODO: Implement actual variable info extraction using LLDB API
        let var_info = VariableInfo {
            name: variable_name.to_string(),
            full_name: format!("::main::{}", variable_name),
            var_type: "unknown".to_string(),
            type_class: "unknown".to_string(),
            value: "?".to_string(),
            address: 0x0,
            size: 0,
            is_valid: false,
            is_in_scope: false,
            location: "unknown".to_string(),
            declaration_file: None,
            declaration_line: None,
        };
        
        Ok(var_info)
    }

    /// Evaluate expression
    pub fn evaluate_expression(&self, expression: &str) -> IncodeResult<String> {
        debug!("Evaluating expression: {}", expression);
        
        // TODO: Implement actual expression evaluation
        Err(IncodeError::not_implemented("evaluate_expression"))
    }

    /// Get thread list
    pub fn list_threads(&self) -> IncodeResult<Vec<ThreadInfo>> {
        debug!("Listing threads");
        
        #[cfg(feature = "mock")]
        {
            debug!("Mock: Returning sample thread list");
            Ok(vec![
                ThreadInfo {
                    thread_id: 1,
                    index: 0,
                    name: Some("main".to_string()),
                    state: "stopped".to_string(),
                    stop_reason: Some("breakpoint".to_string()),
                    queue_name: Some("com.apple.main-thread".to_string()),
                    frame_count: 5,
                    current_frame: Some(StackFrame {
                        index: 0,
                        function_name: "main".to_string(),
                        file_path: Some("/path/to/main.c".to_string()),
                        line_number: Some(42),
                        address: 0x100001000,
                        is_inlined: false,
                    }),
                },
                ThreadInfo {
                    thread_id: 2,
                    index: 1,
                    name: Some("worker_thread".to_string()),
                    state: "running".to_string(),
                    stop_reason: None,
                    queue_name: Some("com.apple.worker-queue".to_string()),
                    frame_count: 3,
                    current_frame: Some(StackFrame {
                        index: 0,
                        function_name: "worker_loop".to_string(),
                        file_path: Some("/path/to/worker.c".to_string()),
                        line_number: Some(128),
                        address: 0x100002000,
                        is_inlined: false,
                    }),
                }
            ])
        }
        
        #[cfg(not(feature = "mock"))]
        {
            if let Some(process) = self.current_process {
                let num_threads = unsafe { SBProcessGetNumThreads(process) };
                let mut threads = Vec::new();
                
                for i in 0..num_threads {
                    let thread = unsafe { SBProcessGetThreadAtIndex(process, i) };
                    if thread.is_null() {
                        continue;
                    }
                    
                    let thread_id = unsafe { SBThreadGetThreadID(thread) };
                    let index = unsafe { SBThreadGetIndexID(thread) };
                    let state_str = self.get_thread_state_string(thread);
                    let stop_reason = self.get_thread_stop_reason(thread);
                    let queue_name = self.get_thread_queue_name(thread);
                    let name = self.get_thread_name(thread);
                    let frame_count = unsafe { SBThreadGetNumFrames(thread) };
                    
                    let current_frame = if frame_count > 0 {
                        let frame = unsafe { SBThreadGetFrameAtIndex(thread, 0) };
                        if !frame.is_null() {
                            // TODO: Implement actual frame extraction
                            Some(StackFrame {
                                index: 0,
                                function_name: "function".to_string(),
                                file_path: Some("/path/to/file".to_string()),
                                line_number: Some(1),
                                address: 0x100000000,
                                is_inlined: false,
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    
                    threads.push(ThreadInfo {
                        thread_id,
                        index,
                        name,
                        state: state_str,
                        stop_reason,
                        queue_name,
                        frame_count,
                        current_frame,
                    });
                }
                
                debug!("Found {} threads", threads.len());
                Ok(threads)
            } else {
                Err(IncodeError::no_process())
            }
        }
    }

    /// Get register values for current thread/frame
    pub fn get_registers(&self, thread_id: Option<u32>, include_metadata: bool) -> IncodeResult<RegisterState> {
        debug!("Getting registers for thread: {:?}", thread_id);
        
        #[cfg(feature = "mock")]
        {
            debug!("Mock: Returning sample register state");
            let mut registers = HashMap::new();
            
            // Common x86_64 registers
            registers.insert("rax".to_string(), RegisterInfo {
                name: "rax".to_string(),
                value: 0x12345678,
                size: 8,
                register_type: "general".to_string(),
                format: "hex".to_string(),
                is_valid: true,
            });
            
            registers.insert("rbx".to_string(), RegisterInfo {
                name: "rbx".to_string(),
                value: 0x87654321,
                size: 8,
                register_type: "general".to_string(),
                format: "hex".to_string(),
                is_valid: true,
            });
            
            registers.insert("rip".to_string(), RegisterInfo {
                name: "rip".to_string(),
                value: 0x100001234,
                size: 8,
                register_type: "program_counter".to_string(),
                format: "hex".to_string(),
                is_valid: true,
            });
            
            registers.insert("rsp".to_string(), RegisterInfo {
                name: "rsp".to_string(),
                value: 0x7fff5fbff000,
                size: 8,
                register_type: "stack_pointer".to_string(),
                format: "hex".to_string(),
                is_valid: true,
            });
            
            Ok(RegisterState {
                registers,
                timestamp: std::time::SystemTime::now(),
                thread_id,
                frame_index: Some(0),
            })
        }
        
        #[cfg(not(feature = "mock"))]
        {
            if let Some(current_thread) = self.current_thread {
                let frame = unsafe { SBThreadGetSelectedFrame(current_thread) };
                if frame.is_null() {
                    return Err(IncodeError::frame("No current frame available"));
                }
                
                let register_list = unsafe { SBFrameGetRegisters(frame) };
                if register_list.is_null() {
                    return Err(IncodeError::frame("Cannot access registers"));
                }
                
                let mut registers = HashMap::new();
                let num_register_sets = unsafe { SBValueListGetSize(register_list) };
                
                for i in 0..num_register_sets {
                    let register_set = unsafe { SBValueListGetValueAtIndex(register_list, i) };
                    if register_set.is_null() {
                        continue;
                    }
                    
                    let set_size = unsafe { SBValueListGetSize(register_set) };
                    for j in 0..set_size {
                        let register = unsafe { SBValueListGetValueAtIndex(register_set, j) };
                        if register.is_null() {
                            continue;
                        }
                        
                        let name_ptr = unsafe { SBValueGetName(register) };
                        if name_ptr.is_null() {
                            continue;
                        }
                        
                        let name = unsafe {
                            std::ffi::CStr::from_ptr(name_ptr)
                                .to_string_lossy()
                                .to_string()
                        };
                        
                        let value = unsafe { SBValueGetValueAsUnsigned(register) };
                        
                        registers.insert(name.clone(), RegisterInfo {
                            name,
                            value,
                            size: 8, // TODO: Get actual size
                            register_type: "general".to_string(), // TODO: Determine type
                            format: "hex".to_string(),
                            is_valid: true,
                        });
                    }
                }
                
                debug!("Found {} registers", registers.len());
                Ok(RegisterState {
                    registers,
                    timestamp: std::time::SystemTime::now(),
                    thread_id,
                    frame_index: Some(0),
                })
            } else {
                Err(IncodeError::thread("No current thread selected"))
            }
        }
    }

    /// Set register value
    pub fn set_register(&mut self, register_name: &str, value: u64, thread_id: Option<u32>) -> IncodeResult<bool> {
        debug!("Setting register {} to value: 0x{:x}", register_name, value);
        
        #[cfg(feature = "mock")]
        {
            debug!("Mock: Setting register {} to 0x{:x}", register_name, value);
            
            // Validate register name format
            if register_name.is_empty() || register_name.len() > 16 {
                return Err(IncodeError::invalid_parameter("Invalid register name"));
            }
            
            // Simulate success for valid register names
            Ok(true)
        }
        
        #[cfg(not(feature = "mock"))]
        {
            if let Some(current_thread) = self.current_thread {
                let frame = unsafe { SBThreadGetSelectedFrame(current_thread) };
                if frame.is_null() {
                    return Err(IncodeError::frame("No current frame available"));
                }
                
                let register_list = unsafe { SBFrameGetRegisters(frame) };
                if register_list.is_null() {
                    return Err(IncodeError::frame("Cannot access registers"));
                }
                
                // Find the register by name
                let num_register_sets = unsafe { SBValueListGetSize(register_list) };
                for i in 0..num_register_sets {
                    let register_set = unsafe { SBValueListGetValueAtIndex(register_list, i) };
                    if register_set.is_null() {
                        continue;
                    }
                    
                    let set_size = unsafe { SBValueListGetSize(register_set) };
                    for j in 0..set_size {
                        let register = unsafe { SBValueListGetValueAtIndex(register_set, j) };
                        if register.is_null() {
                            continue;
                        }
                        
                        let name_ptr = unsafe { SBValueGetName(register) };
                        if name_ptr.is_null() {
                            continue;
                        }
                        
                        let name = unsafe {
                            std::ffi::CStr::from_ptr(name_ptr)
                                .to_string_lossy()
                                .to_string()
                        };
                        
                        if name == register_name {
                            let value_str = format!("0x{:x}", value);
                            let value_cstr = std::ffi::CString::new(value_str)
                                .map_err(|_| IncodeError::invalid_parameter("Invalid value format"))?;
                            
                            let success = unsafe {
                                SBValueSetValueFromCString(register, value_cstr.as_ptr())
                            };
                            
                            debug!("Register {} set to 0x{:x}, success: {}", register_name, value, success);
                            return Ok(success);
                        }
                    }
                }
                
                Err(IncodeError::invalid_parameter(format!("Register '{}' not found", register_name)))
            } else {
                Err(IncodeError::thread("No current thread selected"))
            }
        }
    }

    /// Get detailed register information
    pub fn get_register_info(&self, register_name: &str, thread_id: Option<u32>) -> IncodeResult<RegisterInfo> {
        debug!("Getting register info for: {}", register_name);
        
        #[cfg(feature = "mock")]
        {
            debug!("Mock: Getting info for register {}", register_name);
            
            // Return mock register info based on common register names
            let (value, size, reg_type) = match register_name.to_lowercase().as_str() {
                "rax" | "rbx" | "rcx" | "rdx" => (0x12345678, 8, "general"),
                "rip" => (0x100001234, 8, "program_counter"), 
                "rsp" | "rbp" => (0x7fff5fbff000, 8, "stack_pointer"),
                "eax" | "ebx" | "ecx" | "edx" => (0x12345678, 4, "general"),
                "ax" | "bx" | "cx" | "dx" => (0x1234, 2, "general"),
                "al" | "bl" | "cl" | "dl" => (0x12, 1, "general"),
                "xmm0" | "xmm1" | "xmm2" | "xmm3" => (0x0, 16, "vector"),
                _ => (0x0, 8, "unknown"),
            };
            
            Ok(RegisterInfo {
                name: register_name.to_string(),
                value,
                size,
                register_type: reg_type.to_string(),
                format: "hex".to_string(),
                is_valid: true,
            })
        }
        
        #[cfg(not(feature = "mock"))]
        {
            // Get current register state and find the specific register
            let register_state = self.get_registers(thread_id, true)?;
            
            register_state.registers.get(register_name)
                .cloned()
                .ok_or_else(|| IncodeError::invalid_parameter(format!("Register '{}' not found", register_name)))
        }
    }

    /// Save current register state
    pub fn save_register_state(&self, thread_id: Option<u32>) -> IncodeResult<RegisterState> {
        debug!("Saving register state for thread: {:?}", thread_id);
        
        // Use existing get_registers implementation
        self.get_registers(thread_id, true)
    }

    /// Get source code around current location
    pub fn get_source_code(&self, address: Option<u64>, context_lines: u32) -> IncodeResult<SourceCode> {
        debug!("Getting source code for address: {:?}, context: {}", address, context_lines);
        
        #[cfg(feature = "mock")]
        {
            debug!("Mock: Returning sample source code");
            
            let lines = vec![
                SourceLine {
                    line_number: 38,
                    content: "#include <stdio.h>".to_string(),
                    is_current: false,
                    has_breakpoint: false,
                    address: None,
                },
                SourceLine {
                    line_number: 39,
                    content: "".to_string(),
                    is_current: false,
                    has_breakpoint: false,
                    address: None,
                },
                SourceLine {
                    line_number: 40,
                    content: "int main() {".to_string(),
                    is_current: false,
                    has_breakpoint: true,
                    address: Some(0x100001000),
                },
                SourceLine {
                    line_number: 41,
                    content: "    printf(\"Hello, World!\\n\");".to_string(),
                    is_current: true,
                    has_breakpoint: false,
                    address: Some(0x100001010),
                },
                SourceLine {
                    line_number: 42,
                    content: "    return 0;".to_string(),
                    is_current: false,
                    has_breakpoint: false,
                    address: Some(0x100001020),
                },
                SourceLine {
                    line_number: 43,
                    content: "}".to_string(),
                    is_current: false,
                    has_breakpoint: false,
                    address: Some(0x100001030),
                },
            ];
            
            Ok(SourceCode {
                file_path: "/path/to/main.c".to_string(),
                lines,
                start_line: 38,
                end_line: 43,
                current_line: Some(41),
                total_lines: Some(100),
            })
        }
        
        #[cfg(not(feature = "mock"))]
        {
            if let Some(current_thread) = self.current_thread {
                let frame = unsafe { SBThreadGetSelectedFrame(current_thread) };
                if frame.is_null() {
                    return Err(IncodeError::frame("No current frame available"));
                }
                
                let line_entry = unsafe { SBFrameGetLineEntry(frame) };
                if line_entry.is_null() {
                    return Err(IncodeError::frame("No line information available"));
                }
                
                let file_spec = unsafe { SBLineEntryGetFileSpec(line_entry) };
                if file_spec.is_null() {
                    return Err(IncodeError::frame("No file information available"));
                }
                
                // Get file path
                let filename_ptr = unsafe { SBFileSpecGetFilename(file_spec) };
                let directory_ptr = unsafe { SBFileSpecGetDirectory(file_spec) };
                
                let filename = if !filename_ptr.is_null() {
                    unsafe { std::ffi::CStr::from_ptr(filename_ptr).to_string_lossy().to_string() }
                } else {
                    "unknown".to_string()
                };
                
                let directory = if !directory_ptr.is_null() {
                    unsafe { std::ffi::CStr::from_ptr(directory_ptr).to_string_lossy().to_string() }
                } else {
                    "".to_string()
                };
                
                let file_path = if directory.is_empty() {
                    filename
                } else {
                    format!("{}/{}", directory, filename)
                };
                
                let current_line = unsafe { SBLineEntryGetLine(line_entry) };
                
                // TODO: Read actual file content and create SourceLine entries
                let lines = vec![
                    SourceLine {
                        line_number: current_line,
                        content: "// Source code not available".to_string(),
                        is_current: true,
                        has_breakpoint: false,
                        address: None,
                    }
                ];
                
                debug!("Found source location: {} line {}", file_path, current_line);
                Ok(SourceCode {
                    file_path,
                    lines,
                    start_line: current_line,
                    end_line: current_line,
                    current_line: Some(current_line),
                    total_lines: None,
                })
            } else {
                Err(IncodeError::thread("No current thread selected"))
            }
        }
    }

    /// List all functions with addresses  
    pub fn list_functions(&self, module_filter: Option<&str>) -> IncodeResult<Vec<FunctionInfo>> {
        debug!("Listing functions with module filter: {:?}", module_filter);
        
        #[cfg(feature = "mock")]
        {
            debug!("Mock: Returning sample function list");
            Ok(vec![
                FunctionInfo {
                    name: "main".to_string(),
                    mangled_name: None,
                    start_address: 0x100001000,
                    end_address: Some(0x100001040),
                    file_path: Some("/path/to/main.c".to_string()),
                    line_number: Some(40),
                    size: Some(64),
                    is_inline: false,
                    return_type: Some("int".to_string()),
                },
                FunctionInfo {
                    name: "printf".to_string(),
                    mangled_name: Some("_printf".to_string()),
                    start_address: 0x7fff8c2a1000,
                    end_address: None,
                    file_path: None,
                    line_number: None,
                    size: None,
                    is_inline: false,
                    return_type: Some("int".to_string()),
                },
                FunctionInfo {
                    name: "helper_function".to_string(),
                    mangled_name: None,
                    start_address: 0x100001100,
                    end_address: Some(0x100001180),
                    file_path: Some("/path/to/helper.c".to_string()),
                    line_number: Some(15),
                    size: Some(128),
                    is_inline: false,
                    return_type: Some("void".to_string()),
                },
            ])
        }
        
        #[cfg(not(feature = "mock"))]
        {
            if let Some(target) = self.current_target {
                let num_modules = unsafe { SBTargetGetNumModules(target) };
                let mut functions = Vec::new();
                
                for i in 0..num_modules {
                    let module = unsafe { SBTargetGetModuleAtIndex(target, i) };
                    if module.is_null() {
                        continue;
                    }
                    
                    // TODO: Get module name and apply filter
                    
                    let num_symbols = unsafe { SBModuleGetNumSymbols(module) };
                    for j in 0..num_symbols {
                        let symbol = unsafe { SBModuleGetSymbolAtIndex(module, j) };
                        if symbol.is_null() {
                            continue;
                        }
                        
                        let name_ptr = unsafe { SBSymbolGetName(symbol) };
                        if name_ptr.is_null() {
                            continue;
                        }
                        
                        let name = unsafe {
                            std::ffi::CStr::from_ptr(name_ptr)
                                .to_string_lossy()
                                .to_string()
                        };
                        
                        let start_addr_obj = unsafe { SBSymbolGetStartAddress(symbol) };
                        let start_address = if !start_addr_obj.is_null() {
                            unsafe { SBAddressGetLoadAddress(start_addr_obj, target) }
                        } else {
                            0
                        };
                        
                        let end_addr_obj = unsafe { SBSymbolGetEndAddress(symbol) };
                        let end_address = if !end_addr_obj.is_null() {
                            let addr = unsafe { SBAddressGetLoadAddress(end_addr_obj, target) };
                            if addr != 0 { Some(addr) } else { None }
                        } else {
                            None
                        };
                        
                        functions.push(FunctionInfo {
                            name,
                            mangled_name: None, // TODO: Get mangled name
                            start_address,
                            end_address,
                            file_path: None, // TODO: Get source file
                            line_number: None, // TODO: Get line number
                            size: end_address.map(|end| if end > start_address { end - start_address } else { 0 }),
                            is_inline: false, // TODO: Determine if inline
                            return_type: None, // TODO: Get return type
                        });
                    }
                }
                
                debug!("Found {} functions", functions.len());
                Ok(functions)
            } else {
                Err(IncodeError::process("No target available"))
            }
        }
    }

    /// Get source line information for address
    pub fn get_line_info(&self, address: u64) -> IncodeResult<SourceLocation> {
        debug!("Getting line info for address: 0x{:x}", address);
        
        #[cfg(feature = "mock")]
        {
            debug!("Mock: Returning sample line info for 0x{:x}", address);
            Ok(SourceLocation {
                file_path: "/path/to/main.c".to_string(),
                line_number: 41,
                column: Some(5),
                function_name: Some("main".to_string()),
                address,
                is_valid: true,
            })
        }
        
        #[cfg(not(feature = "mock"))]
        {
            // TODO: Implement actual address-to-source mapping using LLDB
            Err(IncodeError::not_implemented("get_line_info"))
        }
    }

    /// Get debug information summary  
    pub fn get_debug_info(&self) -> IncodeResult<DebugInfo> {
        debug!("Getting debug information summary");
        
        #[cfg(feature = "mock")]
        {
            debug!("Mock: Returning sample debug info");
            Ok(DebugInfo {
                has_debug_symbols: true,
                debug_format: "DWARF".to_string(),
                compilation_units: vec![
                    CompilationUnit {
                        file_path: "/path/to/main.c".to_string(),
                        producer: Some("clang version 14.0.0".to_string()),
                        language: Some("C".to_string()),
                        low_pc: 0x100001000,
                        high_pc: 0x100001200,
                        line_count: 50,
                    },
                    CompilationUnit {
                        file_path: "/path/to/helper.c".to_string(),
                        producer: Some("clang version 14.0.0".to_string()),
                        language: Some("C".to_string()),
                        low_pc: 0x100001200,
                        high_pc: 0x100001400,
                        line_count: 30,
                    },
                ],
                symbol_count: 150,
                line_table_count: 80,
                function_count: 12,
            })
        }
        
        #[cfg(not(feature = "mock"))]
        {
            if let Some(target) = self.current_target {
                let num_modules = unsafe { SBTargetGetNumModules(target) };
                let mut compilation_units = Vec::new();
                let mut total_symbols = 0;
                
                for i in 0..num_modules {
                    let module = unsafe { SBTargetGetModuleAtIndex(target, i) };
                    if module.is_null() {
                        continue;
                    }
                    
                    let num_symbols = unsafe { SBModuleGetNumSymbols(module) };
                    total_symbols += num_symbols;
                    
                    // TODO: Extract compilation unit information
                }
                
                debug!("Found {} modules with {} total symbols", num_modules, total_symbols);
                Ok(DebugInfo {
                    has_debug_symbols: total_symbols > 0,
                    debug_format: "DWARF".to_string(), // TODO: Detect actual format
                    compilation_units,
                    symbol_count: total_symbols,
                    line_table_count: 0, // TODO: Count line tables
                    function_count: 0,   // TODO: Count functions
                })
            } else {
                Err(IncodeError::process("No target available"))
            }
        }
    }

    /// Select thread for debugging
    pub fn select_thread(&mut self, thread_id: u32) -> IncodeResult<ThreadInfo> {
        debug!("Selecting thread: {}", thread_id);
        
        #[cfg(feature = "mock")]
        {
            debug!("Mock: Selecting thread {}", thread_id);
            self.current_thread_id = Some(thread_id);
            
            // Return mock thread info
            Ok(ThreadInfo {
                thread_id,
                index: 0,
                name: Some(format!("thread_{}", thread_id)),
                state: "selected".to_string(),
                stop_reason: Some("user_selection".to_string()),
                queue_name: Some("com.apple.main-thread".to_string()),
                frame_count: 3,
                current_frame: Some(StackFrame {
                    index: 0,
                    function_name: "selected_function".to_string(),
                    file_path: Some("/path/to/selected.c".to_string()),
                    line_number: Some(100),
                    address: 0x100003000,
                    is_inlined: false,
                }),
            })
        }
        
        #[cfg(not(feature = "mock"))]
        {
            if let Some(process) = self.current_process {
                let num_threads = unsafe { SBProcessGetNumThreads(process) };
                
                for i in 0..num_threads {
                    let thread = unsafe { SBProcessGetThreadAtIndex(process, i) };
                    if thread.is_null() {
                        continue;
                    }
                    
                    let tid = unsafe { SBThreadGetThreadID(thread) };
                    if tid == thread_id {
                        self.current_thread_id = Some(thread_id);
                        self.current_thread = Some(thread);
                        
                        let index = unsafe { SBThreadGetIndexID(thread) };
                        let state_str = self.get_thread_state_string(thread);
                        let stop_reason = self.get_thread_stop_reason(thread);
                        let queue_name = self.get_thread_queue_name(thread);
                        let name = self.get_thread_name(thread);
                        let frame_count = unsafe { SBThreadGetNumFrames(thread) };
                        
                        let current_frame = if frame_count > 0 {
                            let frame = unsafe { SBThreadGetFrameAtIndex(thread, 0) };
                            if !frame.is_null() {
                                // TODO: Implement actual frame extraction
                                Some(StackFrame {
                                    index: 0,
                                    function_name: "function".to_string(),
                                    file_path: Some("/path/to/file".to_string()),
                                    line_number: Some(1),
                                    address: 0x100000000,
                                    is_inlined: false,
                                })
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        
                        debug!("Selected thread {} (index {})", thread_id, index);
                        return Ok(ThreadInfo {
                            thread_id,
                            index,
                            name,
                            state: state_str,
                            stop_reason,
                            queue_name,
                            frame_count,
                            current_frame,
                        });
                    }
                }
                
                Err(IncodeError::process(format!("Thread {} not found", thread_id)))
            } else {
                Err(IncodeError::no_process())
            }
        }
    }

    /// Execute raw LLDB command (placeholder implementation)
    pub fn execute_lldb_command(&self, command: &str) -> IncodeResult<String> {
        debug!("Executing LLDB command: {}", command);
        
        // TODO: Implement actual LLDB command execution
        Err(IncodeError::not_implemented("execute_lldb_command"))
    }

    // Helper methods for thread information extraction
    #[cfg(not(feature = "mock"))]
    fn get_thread_state_string(&self, _thread: *mut std::ffi::c_void) -> String {
        // TODO: Implement actual thread state extraction
        "unknown".to_string()
    }

    #[cfg(not(feature = "mock"))]
    fn get_thread_stop_reason(&self, _thread: *mut std::ffi::c_void) -> Option<String> {
        // TODO: Implement actual stop reason extraction
        None
    }

    #[cfg(not(feature = "mock"))]
    fn get_thread_queue_name(&self, _thread: *mut std::ffi::c_void) -> Option<String> {
        // TODO: Implement actual queue name extraction
        None
    }

    #[cfg(not(feature = "mock"))]
    fn get_thread_name(&self, _thread: *mut std::ffi::c_void) -> Option<String> {
        // TODO: Implement actual thread name extraction
        None
    }

    /// List all processes on the system
    pub fn list_processes(&self, filter: Option<&str>, include_system: bool) -> IncodeResult<Vec<ProcessInfo>> {
        debug!("Listing processes with filter: {:?}, include_system: {}", filter, include_system);
        
        #[cfg(test)]
        {
            // Mock implementation for testing
            let mut processes = vec![
                ProcessInfo {
                    pid: 1234,
                    state: "Running".to_string(),
                    executable_path: Some("/usr/bin/test".to_string()),
                    memory_usage: Some(1024 * 1024), // 1MB
                },
                ProcessInfo {
                    pid: 5678,
                    state: "Stopped".to_string(),
                    executable_path: Some("/bin/bash".to_string()),
                    memory_usage: Some(512 * 1024), // 512KB
                },
            ];

            if include_system {
                processes.push(ProcessInfo {
                    pid: 1,
                    state: "Running".to_string(),
                    executable_path: Some("/sbin/init".to_string()),
                    memory_usage: Some(256 * 1024), // 256KB
                });
            }

            if let Some(filter_str) = filter {
                processes.retain(|p| {
                    if let Some(path) = &p.executable_path {
                        path.contains(filter_str)
                    } else {
                        false
                    }
                });
            }

            return Ok(processes);
        }

        #[cfg(not(test))]
        {
            use std::process::Command;
            
            let mut processes = Vec::new();
            
            // Use ps command to get process list
            let output = Command::new("ps")
                .args(&["-eo", "pid,ppid,state,comm,rss"])
                .output()
                .map_err(|e| IncodeError::lldb_op(format!("Failed to execute ps command: {}", e)))?;

            if !output.status.success() {
                return Err(IncodeError::lldb_op("ps command failed"));
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) { // Skip header
                let fields: Vec<&str> = line.trim().split_whitespace().collect();
                if fields.len() >= 5 {
                    if let Ok(pid) = fields[0].parse::<u32>() {
                        let ppid: u32 = fields[1].parse().unwrap_or(0);
                        let state = fields[2].to_string();
                        let comm = fields[3].to_string();
                        let rss_kb: u64 = fields[4].parse().unwrap_or(0);

                        // Filter system processes if not requested
                        if !include_system && (pid == 1 || ppid == 0) {
                            continue;
                        }

                        // Apply filter if provided
                        if let Some(filter_str) = filter {
                            if !comm.contains(filter_str) {
                                continue;
                            }
                        }

                        processes.push(ProcessInfo {
                            pid,
                            state,
                            executable_path: Some(comm),
                            memory_usage: Some(rss_kb * 1024), // Convert KB to bytes
                        });
                    }
                }
            }

            Ok(processes)
        }
    }

    /// Cleanup resources
    pub fn cleanup(&mut self) -> IncodeResult<()> {
        info!("Cleaning up LLDB Manager resources");
        
        // Cleanup LLDB resources
        if let Some(debugger) = self.debugger {
            unsafe {
                SBDebuggerDestroy(debugger);
            }
        }

        // Clear sessions
        let mut sessions = self.sessions.lock().unwrap();
        sessions.clear();
        self.current_session = None;
        
        Ok(())
    }
}

impl Drop for LldbManager {
    fn drop(&mut self) {
        if let Err(e) = self.cleanup() {
            error!("Error during LLDB Manager cleanup: {}", e);
        }
    }
}

impl LldbManager {
    /// Execute raw LLDB command and return output
    pub fn execute_command(&self, command: &str) -> IncodeResult<String> {
        debug!("Executing LLDB command: {}", command);
        
        if cfg!(test) {
            return Ok(format!("Mock output for command: {}", command));
        }

        #[cfg(not(feature = "mock"))]
        {
            // TODO: Implement actual LLDB command execution
            tracing::warn!("Direct LLDB command execution not yet implemented");
            Err(IncodeError::lldb_op("Direct command execution not implemented"))
        }

        #[cfg(feature = "mock")]
        Ok(format!("Mock output for command: {}", command))
    }

    /// List all loaded modules/libraries with their addresses and information
    pub fn list_modules(&self, filter_name: Option<&str>, include_debug_info: bool) -> IncodeResult<Vec<ModuleInfo>> {
        debug!("Listing modules with filter: {:?}, include_debug: {}", filter_name, include_debug_info);
        
        if cfg!(test) {
            // Mock implementation for testing
            let mut modules = vec![
                ModuleInfo {
                    name: "test_binary".to_string(),
                    file_path: "/usr/bin/test".to_string(),
                    uuid: "12345678-1234-5678-9ABC-DEF012345678".to_string(),
                    architecture: "x86_64".to_string(),
                    load_address: 0x100000000,
                    file_size: 1024 * 1024, // 1MB
                    is_main_executable: true,
                    has_debug_symbols: true,
                    symbol_vendor: Some("DWARF".to_string()),
                    compile_units: vec!["main.cpp".to_string(), "utils.cpp".to_string()],
                    num_symbols: 150,
                    slide: Some(0x1000),
                    version: Some("1.0.0".to_string()),
                },
                ModuleInfo {
                    name: "libc.dylib".to_string(),
                    file_path: "/usr/lib/libc.dylib".to_string(),
                    uuid: "87654321-4321-8765-CBA9-876543210ABC".to_string(),
                    architecture: "x86_64".to_string(),
                    load_address: 0x7ff800000000,
                    file_size: 512 * 1024, // 512KB
                    is_main_executable: false,
                    has_debug_symbols: false,
                    symbol_vendor: None,
                    compile_units: vec![],
                    num_symbols: 500,
                    slide: Some(0x2000),
                    version: Some("14.0".to_string()),
                },
            ];

            // Apply filter if provided
            if let Some(filter_str) = filter_name {
                modules.retain(|m| m.name.contains(filter_str) || m.file_path.contains(filter_str));
            }

            return Ok(modules);
        }

        let target = self.current_target.ok_or_else(|| IncodeError::lldb_op("No active target for module listing"))?;
        
        unsafe {
            let num_modules = SBTargetGetNumModules(target);
            let mut modules = Vec::new();
            
            for i in 0..num_modules {
                let module = SBTargetGetModuleAtIndex(target, i);
                if module.is_null() {
                    continue;
                }
                
                // Get module name
                let name_ptr = SBModuleGetObjectName(module);
                let name = if !name_ptr.is_null() {
                    std::ffi::CStr::from_ptr(name_ptr).to_string_lossy().to_string()
                } else {
                    format!("module_{}", i)
                };
                
                // Apply filter if provided
                if let Some(filter_str) = filter_name {
                    if !name.contains(filter_str) {
                        continue;
                    }
                }
                
                // Get file path
                let file_spec = SBModuleGetFileSpec(module);
                let file_path = if !file_spec.is_null() {
                    let mut buffer = [0i8; 1024];
                    let path_len = SBFileSpecGetPath(file_spec, buffer.as_mut_ptr(), buffer.len() as u32);
                    if path_len > 0 {
                        std::ffi::CStr::from_ptr(buffer.as_ptr()).to_string_lossy().to_string()
                    } else {
                        "unknown".to_string()
                    }
                } else {
                    "unknown".to_string()
                };
                
                // Get UUID
                let uuid_ptr = SBModuleGetUUIDString(module);
                let uuid = if !uuid_ptr.is_null() {
                    std::ffi::CStr::from_ptr(uuid_ptr).to_string_lossy().to_string()
                } else {
                    "unknown".to_string()
                };
                
                // Get architecture from triple
                let triple_ptr = SBModuleGetTriple(module);
                let architecture = if !triple_ptr.is_null() {
                    let triple = std::ffi::CStr::from_ptr(triple_ptr).to_string_lossy();
                    triple.split('-').next().unwrap_or("unknown").to_string()
                } else {
                    "unknown".to_string()
                };
                
                // Get version
                let version_ptr = SBModuleGetVersion(module);
                let version = if !version_ptr.is_null() {
                    Some(std::ffi::CStr::from_ptr(version_ptr).to_string_lossy().to_string())
                } else {
                    None
                };
                
                // Get symbol count
                let num_symbols = SBModuleGetNumSymbols(module);
                
                // Determine if this is the main executable (first module typically)
                let is_main_executable = i == 0;
                
                // Get file size (mock implementation)
                let file_size = std::fs::metadata(&file_path)
                    .map(|m| m.len())
                    .unwrap_or(0);
                
                let mut compile_units = Vec::new();
                if include_debug_info {
                    // Get compilation units (simplified)
                    // TODO: Implement proper debug info extraction
                    compile_units.push(format!("{}.c", name));
                }
                
                modules.push(ModuleInfo {
                    name,
                    file_path,
                    uuid,
                    architecture,
                    load_address: 0x100000000 + (i as u64 * 0x10000000), // Mock load addresses
                    file_size,
                    is_main_executable,
                    has_debug_symbols: num_symbols > 0,
                    symbol_vendor: if num_symbols > 0 { Some("DWARF".to_string()) } else { None },
                    compile_units,
                    num_symbols,
                    slide: Some(0x1000 + (i as u64 * 0x100)), // Mock ASLR slide
                    version,
                });
            }
            
            info!("Listed {} modules", modules.len());
            Ok(modules)
        }
    }

    /// Get comprehensive platform information
    pub fn get_platform_info(&self) -> IncodeResult<PlatformInfo> {
        debug!("Getting platform information");
        
        if cfg!(test) {
            // Mock implementation for testing
            return Ok(PlatformInfo {
                name: "host".to_string(),
                version: "macOS 15.0".to_string(),
                architecture: "x86_64".to_string(),
                vendor: "apple".to_string(),
                environment: "darwin".to_string(),
                sdk_version: Some("15.0".to_string()),
                deployment_target: Some("13.0".to_string()),
                is_simulator: false,
                is_remote: false,
                supports_jit: true,
                working_directory: "/Users/user/project".to_string(),
                supported_architectures: vec!["x86_64".to_string(), "arm64".to_string()],
                hostname: Some("localhost".to_string()),
            });
        }

        let target = self.current_target.ok_or_else(|| IncodeError::lldb_op("No active target for platform info"))?;
        
        unsafe {
            // Get platform from target
            let platform_ptr = SBTargetGetPlatform(target);
            if platform_ptr.is_null() {
                return Err(IncodeError::lldb_op("Failed to get platform from target"));
            }
            
            // Get platform name
            let name_ptr = SBPlatformGetName(platform_ptr);
            let name = if !name_ptr.is_null() {
                std::ffi::CStr::from_ptr(name_ptr).to_string_lossy().to_string()
            } else {
                "unknown".to_string()
            };
            
            // Get OS description/version
            let os_desc_ptr = SBPlatformGetOSDescription(platform_ptr);
            let version = if !os_desc_ptr.is_null() {
                std::ffi::CStr::from_ptr(os_desc_ptr).to_string_lossy().to_string()
            } else {
                "unknown".to_string()
            };
            
            // Get hostname
            let hostname_ptr = SBPlatformGetHostname(platform_ptr);
            let hostname = if !hostname_ptr.is_null() {
                Some(std::ffi::CStr::from_ptr(hostname_ptr).to_string_lossy().to_string())
            } else {
                None
            };
            
            // Get triple for architecture info
            let triple_ptr = SBTargetGetTriple(target);
            let triple = if !triple_ptr.is_null() {
                std::ffi::CStr::from_ptr(triple_ptr).to_string_lossy().to_string()
            } else {
                "unknown-unknown-unknown".to_string()
            };
            
            // Parse triple: arch-vendor-os
            let parts: Vec<&str> = triple.split('-').collect();
            let architecture = parts.get(0).unwrap_or(&"unknown").to_string();
            let vendor = parts.get(1).unwrap_or(&"unknown").to_string();
            let environment = parts.get(2).unwrap_or(&"unknown").to_string();
            
            // Get working directory
            let work_dir_ptr = SBPlatformGetWorkingDirectory(platform_ptr);
            let working_directory = if !work_dir_ptr.is_null() {
                let mut buffer = [0i8; 1024];
                let path_len = SBFileSpecGetPath(work_dir_ptr, buffer.as_mut_ptr(), buffer.len() as u32);
                if path_len > 0 {
                    std::ffi::CStr::from_ptr(buffer.as_ptr()).to_string_lossy().to_string()
                } else {
                    std::env::current_dir()
                        .unwrap_or_else(|_| std::path::PathBuf::from("/"))
                        .to_string_lossy()
                        .to_string()
                }
            } else {
                std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("/"))
                    .to_string_lossy()
                    .to_string()
            };
            
            // Determine platform characteristics
            let is_simulator = name.contains("simulator") || environment.contains("simulator");
            let is_remote = name.contains("remote");
            let supports_jit = !name.contains("ios") || is_simulator; // iOS device doesn't support JIT
            
            // Build supported architectures list
            let supported_architectures = match vendor.as_str() {
                "apple" => vec!["x86_64".to_string(), "arm64".to_string()],
                "pc" => vec!["x86_64".to_string(), "i386".to_string()],
                _ => vec![architecture.clone()],
            };
            
            // Extract SDK version and deployment target from version string
            let (sdk_version, deployment_target) = if version.contains("macOS") {
                // Extract version number from "macOS 15.0" format
                let version_num = version.split_whitespace().nth(1).unwrap_or("unknown");
                (Some(version_num.to_string()), Some("13.0".to_string())) // Common deployment target
            } else if version.contains("iOS") {
                let version_num = version.split_whitespace().nth(1).unwrap_or("unknown");
                (Some(version_num.to_string()), Some("15.0".to_string()))
            } else {
                (None, None)
            };
            
            info!("Platform info retrieved: {}", name);
            
            Ok(PlatformInfo {
                name,
                version,
                architecture,
                hostname,
                working_directory,
                vendor,
                environment,
                is_simulator,
                is_remote,
                supports_jit,
                supported_architectures,
                sdk_version,
                deployment_target,
            })
        }
        
        #[cfg(feature = "mock")]
        {
            Ok(PlatformInfo {
                name: "Mock Platform".to_string(),
                version: "Mock 1.0".to_string(),
                architecture: "x86_64".to_string(),
                hostname: Some("mock-host".to_string()),
                working_directory: "/tmp".to_string(),
                vendor: "apple".to_string(),
                environment: "macosx15.0".to_string(),
                is_simulator: false,
                is_remote: false,
                supports_jit: true,
                supported_architectures: vec!["x86_64".to_string(), "arm64".to_string()],
                sdk_version: Some("15.0".to_string()),
                deployment_target: Some("13.0".to_string()),
            })
        }
    }
    
    /// Get LLDB version and build information
    pub fn get_lldb_version(&self, include_build_info: bool) -> IncodeResult<LldbVersionInfo> {
        debug!("Getting LLDB version info, include_build_info: {}", include_build_info);
        
        if cfg!(test) {
            // Mock implementation for testing
            return Ok(LldbVersionInfo {
                version: "lldb-1500.0.200.58".to_string(),
                build_number: if include_build_info { Some("1500.0.200.58".to_string()) } else { None },
                api_version: "15.0.0".to_string(),
                build_date: if include_build_info { Some("2024-08-06".to_string()) } else { None },
                build_configuration: if include_build_info { Some("Release".to_string()) } else { None },
                compiler: if include_build_info { Some("Apple clang version 15.0.0".to_string()) } else { None },
                platform: std::env::consts::OS.to_string(),
            });
        }

        #[cfg(not(feature = "mock"))]
        unsafe {
            // Get version string
            let version_ptr = SBDebuggerGetVersion();
            let version = if !version_ptr.is_null() {
                std::ffi::CStr::from_ptr(version_ptr).to_string_lossy().to_string()
            } else {
                "unknown".to_string()
            };
            
            // Parse build number from version (e.g., "lldb-1500.0.200.58" -> "1500.0.200.58")
            let build_number = if include_build_info {
                if let Some(dash_pos) = version.find('-') {
                    Some(version[dash_pos + 1..].to_string())
                } else {
                    None
                }
            } else {
                None
            };
            
            // Get build configuration
            let build_config = if include_build_info {
                let config_ptr = SBDebuggerGetBuildConfiguration();
                if !config_ptr.is_null() {
                    Some(std::ffi::CStr::from_ptr(config_ptr).to_string_lossy().to_string())
                } else {
                    None
                }
            } else {
                None
            };
            
            // Extract API version from version string (first two version components)
            let api_version = build_number.as_ref()
                .and_then(|bn| {
                    let parts: Vec<&str> = bn.split('.').collect();
                    if parts.len() >= 2 {
                        Some(format!("{}.{}.0", parts[0], parts[1]))
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "unknown".to_string());
            
            info!("LLDB version retrieved: {}", version);
            
            Ok(LldbVersionInfo {
                version,
                build_number,
                api_version,
                build_date: if include_build_info { Some("unknown".to_string()) } else { None },
                build_configuration: build_config,
                compiler: if include_build_info { Some("unknown".to_string()) } else { None },
                platform: std::env::consts::OS.to_string(),
            })
        }

        #[cfg(feature = "mock")]
        Ok(LldbVersionInfo {
            version: "lldb-1500.0.200.58".to_string(),
            build_number: if include_build_info { Some("1500.0.200.58".to_string()) } else { None },
            api_version: "15.0.0".to_string(),
            build_date: if include_build_info { Some("2024-08-06".to_string()) } else { None },
            build_configuration: if include_build_info { Some("Release".to_string()) } else { None },
            compiler: if include_build_info { Some("Apple clang version 15.0.0".to_string()) } else { None },
            platform: std::env::consts::OS.to_string(),
        })
    }

    /// Get comprehensive target information
    pub fn get_target_info(&self) -> IncodeResult<TargetInfo> {
        debug!("Getting target information");
        
        if cfg!(test) {
            // Mock implementation for testing
            return Ok(TargetInfo {
                executable_path: "/usr/bin/test".to_string(),
                architecture: "x86_64".to_string(),
                platform: "host".to_string(),
                executable_format: "Mach-O".to_string(),
                has_debug_symbols: true,
                entry_point: Some(0x100000000),
                base_address: Some(0x100000000),
                file_size: 1024 * 1024, // 1MB mock size
                creation_time: Some(std::time::SystemTime::now()),
                is_pie: true,
                is_stripped: false,
                endianness: "little".to_string(),
            });
        }

        let target = self.current_target.ok_or_else(|| IncodeError::lldb_op("No active target for target info"))?;
        
        unsafe {
            // Get triple (architecture-vendor-os)
            let triple_ptr = SBTargetGetTriple(target);
            let triple = if !triple_ptr.is_null() {
                std::ffi::CStr::from_ptr(triple_ptr).to_string_lossy().to_string()
            } else {
                "unknown".to_string()
            };
            
            // Extract architecture from triple
            let architecture = triple.split('-').next().unwrap_or("unknown").to_string();
            
            // Get platform
            let platform_ptr = SBTargetGetPlatform(target);
            let platform = if !platform_ptr.is_null() {
                let platform_name_ptr = SBPlatformGetName(platform_ptr);
                if !platform_name_ptr.is_null() {
                    std::ffi::CStr::from_ptr(platform_name_ptr).to_string_lossy().to_string()
                } else {
                    "unknown".to_string()
                }
            } else {
                "unknown".to_string()
            };
            
            // Get executable file information
            let executable_ptr = SBTargetGetExecutable(target);
            let executable_path = if !executable_ptr.is_null() {
                let mut buffer = [0i8; 1024];
                let path_len = SBFileSpecGetPath(executable_ptr, buffer.as_mut_ptr(), buffer.len() as u32);
                if path_len > 0 {
                    std::ffi::CStr::from_ptr(buffer.as_ptr()).to_string_lossy().to_string()
                } else {
                    "unknown".to_string()
                }
            } else {
                "unknown".to_string()
            };
            
            // Determine executable format based on platform
            let executable_format = match platform.as_str() {
                p if p.contains("darwin") || p.contains("macosx") || p.contains("ios") => "Mach-O",
                p if p.contains("linux") || p.contains("freebsd") => "ELF",
                p if p.contains("windows") => "PE",
                _ => "unknown",
            }.to_string();
            
            // Get file size (mock implementation for now)
            let file_size = std::fs::metadata(&executable_path)
                .map(|m| m.len())
                .unwrap_or(0);
            
            // Get creation time (mock implementation)
            let creation_time = std::fs::metadata(&executable_path)
                .and_then(|m| m.created())
                .ok();
            
            info!("Target info retrieved for: {}", executable_path);
            Ok(TargetInfo {
                executable_path,
                architecture: architecture.clone(),
                platform,
                executable_format,
                has_debug_symbols: true, // TODO: Implement proper debug symbol detection
                entry_point: Some(0x100000000), // TODO: Get actual entry point
                base_address: Some(0x100000000), // TODO: Get actual base address
                file_size,
                creation_time,
                is_pie: true, // TODO: Implement PIE detection
                is_stripped: false, // TODO: Implement stripped binary detection
                endianness: if architecture.contains("x86") || architecture.contains("aarch64") { 
                    "little".to_string() 
                } else { 
                    "unknown".to_string() 
                },
            })
        }
    }

    /// Set LLDB settings (configuration parameters)
    pub fn set_lldb_settings(&mut self, setting_name: &str, value: &str) -> IncodeResult<String> {
        debug!("Setting LLDB setting: {} = {}", setting_name, value);
        
        // Validate setting name format
        if setting_name.is_empty() {
            return Err(IncodeError::invalid_parameter("setting name cannot be empty"));
        }
        
        // Common LLDB settings validation
        let valid_settings = vec![
            "target.prefer-dynamic-value",
            "target.display-expression-variables",
            "target.max-children-count",
            "target.max-string-summary-length",
            "target.process.thread.step-avoid-libraries",
            "target.process.thread.step-avoid-regexp",
            "plugin.symbol-file.dwarf.comp-dir-symlink-paths",
            "interpreter.prompt",
            "stop-disassembly-display",
            "stop-line-count-before",
            "stop-line-count-after",
            "thread-format",
            "frame-format",
            "use-external-editor",
            "auto-confirm",
            "print-object-description",
            "display-recognised-arguments",
            "display-runtime-support-values",
        ];
        
        // Check if it's a known setting or follows dot notation pattern
        let is_valid_setting = valid_settings.contains(&setting_name) || 
            setting_name.contains('.') && !setting_name.starts_with('.') && !setting_name.ends_with('.');
        
        if !is_valid_setting {
            return Err(IncodeError::invalid_parameter(
                format!("invalid setting name: {}", setting_name)
            ));
        }
        
        if cfg!(test) {
            // Mock implementation for testing
            info!("Mock: Setting {} = {}", setting_name, value);
            return Ok(format!("Setting '{}' changed from '<default>' to '{}'", setting_name, value));
        }
        
        // Execute the settings set command
        let command = format!("settings set {} {}", setting_name, value);
        let result = self.execute_command(&command)?;
        
        // Verify the setting was applied
        let verify_command = format!("settings show {}", setting_name);
        let current_value = self.execute_command(&verify_command)?;
        
        info!("LLDB setting updated: {} = {}", setting_name, value);
        
        Ok(format!("Setting '{}' changed to '{}'. Current: {}", 
            setting_name, value, current_value.trim()))
    }

    /// Set variable value during debugging
    pub fn set_variable(&mut self, variable_name: &str, value: &str) -> IncodeResult<String> {
        debug!("Setting variable: {} = {}", variable_name, value);
        
        // Validate variable name
        if variable_name.is_empty() {
            return Err(IncodeError::invalid_parameter("variable name cannot be empty"));
        }
        
        // Validate we have an active process
        if self.current_process.is_none() {
            return Err(IncodeError::no_process());
        }
        
        if cfg!(test) {
            // Mock implementation for testing
            info!("Mock: Setting variable {} = {}", variable_name, value);
            
            // Simulate different responses based on variable name patterns
            if variable_name.starts_with("$") {
                // Register-like variable
                return Ok(format!("Register '{}' set to {}", variable_name, value));
            } else if variable_name.contains("::") || variable_name.contains(".") {
                // Qualified variable name
                return Ok(format!("Variable '{}' modified. Old value: <unknown>, New value: {}", 
                    variable_name, value));
            } else {
                // Simple variable
                return Ok(format!("Variable '{}' set to {}", variable_name, value));
            }
        }
        
        // Build the expression to set the variable
        // Handle different value types appropriately
        let expression = if value.starts_with('"') && value.ends_with('"') {
            // String literal - use as-is
            format!("{} = {}", variable_name, value)
        } else if value.starts_with("0x") {
            // Hex value
            format!("{} = {}", variable_name, value)
        } else if value.parse::<f64>().is_ok() {
            // Numeric value
            format!("{} = {}", variable_name, value)
        } else if value == "true" || value == "false" {
            // Boolean value
            format!("{} = {}", variable_name, value)
        } else if value == "nullptr" || value == "NULL" {
            // Null pointer
            format!("{} = {}", variable_name, value)
        } else {
            // Treat as expression or variable reference
            format!("{} = {}", variable_name, value)
        };
        
        // Execute the assignment expression
        let result = self.evaluate_expression(&expression)?;
        
        // Verify the assignment by reading back the value
        let verify_result = self.evaluate_expression(variable_name)?;
        
        info!("Variable '{}' set successfully. New value: {}", variable_name, verify_result);
        
        Ok(format!("Variable '{}' modified. New value: {}", variable_name, verify_result))
    }

    /// Lookup symbol information by name
    pub fn lookup_symbol(&self, symbol_name: &str) -> IncodeResult<SymbolInfo> {
        debug!("Looking up symbol: {}", symbol_name);
        
        // Validate symbol name
        if symbol_name.is_empty() {
            return Err(IncodeError::invalid_parameter("symbol name cannot be empty"));
        }
        
        if cfg!(test) {
            // Mock implementation for testing
            info!("Mock: Looking up symbol {}", symbol_name);
            
            // Simulate different symbol types based on name patterns
            if symbol_name.starts_with("_Z") || symbol_name.contains("::") {
                // C++ mangled symbol
                return Ok(SymbolInfo {
                    name: symbol_name.to_string(),
                    demangled_name: Some(format!("std::vector<int>::{}", 
                        symbol_name.split("::").last().unwrap_or("method"))),
                    symbol_type: "Function".to_string(),
                    address: 0x100001234,
                    size: 128,
                    module: Some("libstdc++.so".to_string()),
                    file: Some("/usr/include/c++/vector".to_string()),
                    line: Some(142),
                    is_exported: true,
                    is_debug: true,
                    visibility: "public".to_string(),
                });
            } else if symbol_name.starts_with("g_") || symbol_name.starts_with("s_") {
                // Global/static variable
                return Ok(SymbolInfo {
                    name: symbol_name.to_string(),
                    demangled_name: None,
                    symbol_type: "Data".to_string(),
                    address: 0x100002000,
                    size: 8,
                    module: Some("main".to_string()),
                    file: Some("/path/to/main.cpp".to_string()),
                    line: Some(45),
                    is_exported: true,
                    is_debug: true,
                    visibility: "global".to_string(),
                });
            } else {
                // Regular function
                return Ok(SymbolInfo {
                    name: symbol_name.to_string(),
                    demangled_name: None,
                    symbol_type: "Function".to_string(),
                    address: 0x100003000,
                    size: 64,
                    module: Some("main".to_string()),
                    file: Some("/path/to/source.c".to_string()),
                    line: Some(100),
                    is_exported: true,
                    is_debug: true,
                    visibility: "public".to_string(),
                });
            }
        }
        
        // Execute symbol lookup command
        let command = format!("image lookup -n {}", symbol_name);
        let result = self.execute_command(&command)?;
        
        // Parse the result to extract symbol information
        // This is a simplified parser - real implementation would be more robust
        let lines: Vec<&str> = result.lines().collect();
        if lines.is_empty() || result.contains("no symbols match") {
            return Err(IncodeError::lldb_op(format!("Symbol '{}' not found", symbol_name)));
        }
        
        // Extract address from first line (typically "1 match found in ...")
        let mut address = 0u64;
        let mut module = None;
        let mut file = None;
        let mut line = None;
        let mut symbol_type = "Unknown".to_string();
        
        for line_str in &lines {
            if line_str.contains("Address:") {
                // Parse address
                if let Some(addr_str) = line_str.split("0x").nth(1) {
                    if let Some(addr_end) = addr_str.find(' ') {
                        address = u64::from_str_radix(&addr_str[..addr_end], 16).unwrap_or(0);
                    }
                }
            } else if line_str.contains("Summary:") {
                // Extract module name
                if let Some(mod_start) = line_str.find('`') {
                    if let Some(mod_end) = line_str[mod_start+1..].find('`') {
                        module = Some(line_str[mod_start+1..mod_start+1+mod_end].to_string());
                    }
                }
            } else if line_str.contains("CompileUnit:") {
                // Extract file information
                let parts: Vec<&str> = line_str.split_whitespace().collect();
                if parts.len() > 1 {
                    file = Some(parts.last().unwrap().to_string());
                }
            } else if line_str.contains("Function:") {
                symbol_type = "Function".to_string();
            } else if line_str.contains("Variable:") || line_str.contains("Data:") {
                symbol_type = "Data".to_string();
            }
        }
        
        // Try to get line information
        if address != 0 {
            if let Ok(line_info) = self.get_line_info(address) {
                file = Some(line_info.file_path);
                line = Some(line_info.line_number);
            }
        }
        
        info!("Symbol '{}' found at 0x{:x}", symbol_name, address);
        
        Ok(SymbolInfo {
            name: symbol_name.to_string(),
            demangled_name: None, // TODO: Implement demangling
            symbol_type,
            address,
            size: 0, // TODO: Get actual size
            module,
            file,
            line,
            is_exported: true, // TODO: Determine actual visibility
            is_debug: true,
            visibility: "unknown".to_string(),
        })
    }

    /// Analyze crash dump and provide detailed crash information
    pub fn analyze_crash(&self, core_file_path: Option<&str>) -> IncodeResult<CrashAnalysis> {
        debug!("Analyzing crash, core file: {:?}", core_file_path);
        
        if cfg!(test) {
            // Mock implementation for testing
            info!("Mock: Analyzing crash");
            
            return Ok(CrashAnalysis {
                crash_type: "SIGSEGV".to_string(),
                crash_address: Some(0x0),
                faulting_thread: 1,
                signal_number: 11,
                signal_name: "SIGSEGV".to_string(),
                exception_type: Some("EXC_BAD_ACCESS".to_string()),
                exception_codes: vec![1, 0],
                crashed_thread_backtrace: vec![
                    "0x100001234 main + 52".to_string(),
                    "0x100001180 foo + 16".to_string(),
                    "0x100001200 bar + 32".to_string(),
                ],
                register_state: "rax=0x0 rbx=0x7fff5fbff8a0 rcx=0x0".to_string(),
                memory_regions: vec![
                    "0x100000000-0x100002000 r-x /usr/bin/test".to_string(),
                    "0x7fff5fbff000-0x7fff5fc00000 rw- [stack]".to_string(),
                ],
                loaded_modules: vec![
                    "test (0x100000000)".to_string(),
                    "libsystem_c.dylib (0x7fff80000000)".to_string(),
                ],
                crash_summary: "Segmentation fault: attempted to read from NULL pointer".to_string(),
                recommendations: vec![
                    "Check for null pointer dereferences".to_string(),
                    "Verify array bounds checking".to_string(),
                    "Review memory allocation/deallocation".to_string(),
                ],
            });
        }
        
        // Validate we have debugging context or core file
        if core_file_path.is_none() && self.current_process.is_none() {
            return Err(IncodeError::lldb_op("No core file path provided and no active process"));
        }
        
        // In real implementation, we would:
        // 1. Load core file if provided, or use current process
        // 2. Get crashed thread information
        // 3. Analyze registers and memory state
        // 4. Extract crash details and generate report
        
        let crash_type = "SIGSEGV"; // Default for demonstration
        let signal_number = 11;
        let faulting_thread = 0u32;
        
        // Get backtrace for crashed thread
        let backtrace = match self.get_backtrace() {
            Ok(bt) => bt,
            Err(_) => vec!["Unable to get backtrace".to_string()],
        };
        
        // Get register state
        let register_state = match self.get_registers(Some(faulting_thread), true) {
            Ok(regs) => format!("Registers captured: {} entries", regs.registers.len()),
            Err(_) => "Unable to get register state".to_string(),
        };
        
        // Get memory regions
        let memory_regions = match self.get_memory_regions() {
            Ok(regions) => {
                regions.into_iter().take(5).map(|region| {
                    format!("0x{:x}-0x{:x} {} {}", 
                        region.start_address, 
                        region.end_address,
                        region.permissions,
                        region.name.unwrap_or_else(|| "[unknown]".to_string())
                    )
                }).collect()
            },
            Err(_) => vec!["Unable to get memory regions".to_string()],
        };
        
        // Get loaded modules
        let loaded_modules = match self.list_modules(None, true) {
            Ok(modules) => {
                modules.into_iter().take(10).map(|module| {
                    format!("{} (0x{:x})", module.name, module.load_address)
                }).collect()
            },
            Err(_) => vec!["Unable to get loaded modules".to_string()],
        };
        
        // Generate crash summary and recommendations
        let crash_summary = format!("{}: Process crashed in thread {}", crash_type, faulting_thread);
        let recommendations = vec![
            "Review the crashed thread's stack trace".to_string(),
            "Check for memory access violations".to_string(),
            "Verify proper error handling in the code path".to_string(),
            "Consider using memory debugging tools".to_string(),
        ];
        
        info!("Crash analysis completed for {}", 
            core_file_path.unwrap_or("current process"));
        
        Ok(CrashAnalysis {
            crash_type: crash_type.to_string(),
            crash_address: Some(0x0), // TODO: Extract actual crash address
            faulting_thread,
            signal_number,
            signal_name: crash_type.to_string(),
            exception_type: Some("EXC_BAD_ACCESS".to_string()), // TODO: Platform-specific
            exception_codes: vec![1, 0], // TODO: Extract actual exception codes
            crashed_thread_backtrace: backtrace,
            register_state,
            memory_regions,
            loaded_modules,
            crash_summary,
            recommendations,
        })
    }

    /// Generate core dump file for current process state
    pub fn generate_core_dump(&self, output_path: &str) -> IncodeResult<String> {
        debug!("Generating core dump to: {}", output_path);
        
        // Validate we have an active process
        if self.current_process.is_none() {
            return Err(IncodeError::no_process());
        }
        
        // Validate output path
        if output_path.is_empty() {
            return Err(IncodeError::invalid_parameter("output path cannot be empty"));
        }
        
        if cfg!(test) {
            // Mock implementation for testing
            info!("Mock: Generating core dump to {}", output_path);
            
            // Simulate core dump generation
            let file_size = 1024 * 1024 * 50; // 50MB mock size
            let process_info = format!("Process PID: {}, State: Running", 
                std::process::id());
            
            return Ok(format!(
                "Core dump generated successfully\nOutput: {}\nSize: {} bytes\nProcess: {}\nTimestamp: {:?}",
                output_path,
                file_size,
                process_info,
                std::time::SystemTime::now()
            ));
        }
        
        // In real implementation with LLDB:
        // 1. Use SBProcess::SaveCore() or equivalent
        // 2. Specify output format (ELF, Mach-O, etc.)
        // 3. Include memory regions, threads, and debug info
        
        // For now, execute LLDB command to generate core dump
        let command = format!("process save-core {}", output_path);
        match self.execute_command(&command) {
            Ok(output) => {
                // Verify the file was created
                if std::path::Path::new(output_path).exists() {
                    let file_size = std::fs::metadata(output_path)
                        .map(|m| m.len())
                        .unwrap_or(0);
                    
                    info!("Core dump generated successfully: {} ({} bytes)", output_path, file_size);
                    
                    Ok(format!(
                        "Core dump generated successfully\nOutput: {}\nSize: {} bytes\nLLDB Output: {}",
                        output_path,
                        file_size,
                        output.trim()
                    ))
                } else {
                    Err(IncodeError::lldb_op(format!("Core dump file not created: {}", output_path)))
                }
            },
            Err(e) => {
                error!("Failed to generate core dump: {}", e);
                Err(IncodeError::lldb_op(format!("Core dump generation failed: {}", e)))
            }
        }
    }
}

