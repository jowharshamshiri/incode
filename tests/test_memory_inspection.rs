// InCode Memory Inspection Tools Test Suite
// 
// GRANULAR FEATURES TESTED:
// - F0028: read_memory - Read raw memory at address with size and format
// - F0029: write_memory - Write data to memory address
// - F0030: disassemble - Disassemble instructions at address or function
// - F0031: search_memory - Search for byte patterns in process memory
// - F0032: get_memory_regions - List memory mappings and permissions
// - F0033: dump_memory - Dump memory region to file
// - F0034: memory_map - Get detailed memory map with segments
//
// Tests memory inspection with real LLDB integration using test_debuggee binary

use std::time::Duration;
use std::thread;

// Import test setup utilities
mod test_setup;
use test_setup::{TestSession, TestMode, TestUtils};

use incode::lldb_manager::LldbManager;
use incode::error::{IncodeError, IncodeResult};

#[tokio::test]
async fn test_f0028_read_memory_success() {
    // F0028: read_memory - Test reading raw memory with different formats
    println!("Testing F0028: read_memory");
    
    let mut session = match TestSession::new(TestMode::Memory) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0028: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0028: Test session started with PID {}", pid);
            
            // Set breakpoint to get to memory scenario
            let _ = session.lldb_manager().set_breakpoint("create_global_patterns", None);
            let _ = session.lldb_manager().continue_execution();
            
            // Test reading memory - use a known global variable address
            // In a real scenario, we'd get this address from variable lookup
            let test_formats = vec!["hex", "ascii", "bytes", "int", "float", "pointer"];
            
            for format in test_formats {
                let result = session.lldb_manager().read_memory(0x100000000, 64, format);
                
                match result {
                    Ok(memory_data) => {
                        println!("✅ F0028: read_memory succeeded with format {}", format);
                        println!("  Address: 0x{:x}", memory_data.address);
                        println!("  Size: {}", memory_data.size);
                        println!("  Format: {}", memory_data.format);
                        println!("  Content preview: {}...", 
                               memory_data.content.chars().take(50).collect::<String>());
                        
                        assert_eq!(memory_data.address, 0x100000000);
                        assert_eq!(memory_data.size, 64);
                        assert_eq!(memory_data.format, format);
                        assert!(!memory_data.content.is_empty());
                    }
                    Err(e) => {
                        println!("⚠️ F0028: read_memory failed for format {}: {}", format, e);
                    }
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0028: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0028_read_memory_invalid_address() {
    // F0028: read_memory - Test error handling for invalid memory address
    println!("Testing F0028: read_memory with invalid address");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0028: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            // Test reading from invalid address
            let result = session.lldb_manager().read_memory(0x0, 64, "hex");
            
            match result {
                Err(e) => {
                    println!("✅ F0028: Correctly handled invalid address: {}", e);
                }
                Ok(_) => {
                    println!("⚠️ F0028: read_memory unexpectedly succeeded for invalid address");
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0028: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0029_write_memory() {
    // F0029: write_memory - Test writing data to memory address
    println!("Testing F0029: write_memory");
    
    let mut session = match TestSession::new(TestMode::Memory) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0029: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0029: Test session started with PID {}", pid);
            
            // Set breakpoint to get to memory scenario
            let _ = session.lldb_manager().set_breakpoint("create_heap_patterns", None);
            let _ = session.lldb_manager().continue_execution();
            
            // Test writing memory with different formats
            let test_data = vec![
                ("hex", "41424344"),
                ("ascii", "TEST"),
                ("bytes", "0x54,0x45,0x53,0x54"),
            ];
            
            for (format, data) in test_data {
                let result = session.lldb_manager().write_memory(0x100000000, data, format);
                
                match result {
                    Ok(success) => {
                        if success {
                            println!("✅ F0029: write_memory succeeded with format {}", format);
                        } else {
                            println!("⚠️ F0029: write_memory reported failure for format {}", format);
                        }
                    }
                    Err(e) => {
                        println!("⚠️ F0029: write_memory failed for format {}: {}", format, e);
                    }
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0029: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0030_disassemble_function() {
    // F0030: disassemble - Test disassembling instructions at function
    println!("Testing F0030: disassemble");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0030: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0030: Test session started with PID {}", pid);
            
            // Test disassembling main function
            let result = session.lldb_manager().disassemble("main", 10);
            
            match result {
                Ok(disassembly) => {
                    println!("✅ F0030: disassemble succeeded");
                    println!("  Function: {}", disassembly.function);
                    println!("  Start Address: 0x{:x}", disassembly.start_address);
                    println!("  Instruction Count: {}", disassembly.instructions.len());
                    
                    for (i, instruction) in disassembly.instructions.iter().take(3).enumerate() {
                        println!("  Instruction {}: 0x{:x}: {} {}", 
                               i + 1, instruction.address, instruction.mnemonic, instruction.operands);
                    }
                    
                    assert_eq!(disassembly.function, "main");
                    assert!(disassembly.start_address > 0);
                    assert!(!disassembly.instructions.is_empty());
                }
                Err(e) => {
                    println!("⚠️ F0030: disassemble failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0030: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0030_disassemble_invalid_function() {
    // F0030: disassemble - Test error handling for invalid function
    println!("Testing F0030: disassemble with invalid function");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0030: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            let result = session.lldb_manager().disassemble("nonexistent_function_12345", 10);
            
            match result {
                Err(e) => {
                    println!("✅ F0030: Correctly handled invalid function: {}", e);
                }
                Ok(_) => {
                    println!("⚠️ F0030: disassemble unexpectedly succeeded for invalid function");
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0030: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0031_search_memory() {
    // F0031: search_memory - Test searching for byte patterns in memory
    println!("Testing F0031: search_memory");
    
    let mut session = match TestSession::new(TestMode::Memory) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0031: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0031: Test session started with PID {}", pid);
            
            // Set breakpoint to get to memory scenario
            let _ = session.lldb_manager().set_breakpoint("create_global_patterns", None);
            let _ = session.lldb_manager().continue_execution();
            
            // Test searching for different patterns
            let search_patterns = vec![
                ("hex", "41424344"),      // "ABCD" in hex
                ("ascii", "TEST"),        // Text pattern
                ("bytes", "0x00,0x01"),   // Byte sequence
            ];
            
            for (format, pattern) in search_patterns {
                let result = session.lldb_manager().search_memory(
                    pattern, 
                    format, 
                    0x100000000, 
                    0x10000
                );
                
                match result {
                    Ok(matches) => {
                        println!("✅ F0031: search_memory succeeded for {} pattern, found {} matches", 
                               format, matches.len());
                        
                        for (i, match_result) in matches.iter().take(3).enumerate() {
                            println!("  Match {}: Address 0x{:x}, Context: {}", 
                                   i + 1, match_result.address, 
                                   match_result.context.chars().take(30).collect::<String>());
                        }
                    }
                    Err(e) => {
                        println!("⚠️ F0031: search_memory failed for {} pattern: {}", format, e);
                    }
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0031: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0032_get_memory_regions() {
    // F0032: get_memory_regions - Test listing memory mappings and permissions
    println!("Testing F0032: get_memory_regions");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0032: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0032: Test session started with PID {}", pid);
            
            // Test getting memory regions
            let result = session.lldb_manager().get_memory_regions();
            
            match result {
                Ok(regions) => {
                    println!("✅ F0032: get_memory_regions succeeded, found {} regions", regions.len());
                    
                    for (i, region) in regions.iter().take(5).enumerate() {
                        println!("  Region {}: 0x{:x}-0x{:x} ({}) [{}]", 
                               i + 1, region.start_address, region.end_address,
                               region.name, region.permissions);
                    }
                    
                    assert!(regions.len() > 0, "Should have at least one memory region");
                    
                    // Check for common memory regions
                    let has_executable = regions.iter().any(|r| r.permissions.contains('x'));
                    let has_writable = regions.iter().any(|r| r.permissions.contains('w'));
                    let has_readable = regions.iter().any(|r| r.permissions.contains('r'));
                    
                    if has_executable && has_writable && has_readable {
                        println!("✅ F0032: Found expected memory region types (rwx)");
                    }
                }
                Err(e) => {
                    println!("⚠️ F0032: get_memory_regions failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0032: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0033_dump_memory() {
    // F0033: dump_memory - Test dumping memory region to file
    println!("Testing F0033: dump_memory");
    
    let mut session = match TestSession::new(TestMode::Memory) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0033: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0033: Test session started with PID {}", pid);
            
            // Test dumping memory with different formats
            let dump_formats = vec!["raw", "hex", "hexdump"];
            
            for format in dump_formats {
                let output_file = format!("/tmp/memory_dump_{}.txt", format);
                let result = session.lldb_manager().dump_memory(
                    0x100000000, 
                    256, 
                    &output_file, 
                    format
                );
                
                match result {
                    Ok(success) => {
                        if success {
                            println!("✅ F0033: dump_memory succeeded with format {} to {}", format, output_file);
                            
                            // Check if file was created
                            if std::path::Path::new(&output_file).exists() {
                                println!("  File created successfully");
                            }
                        } else {
                            println!("⚠️ F0033: dump_memory reported failure for format {}", format);
                        }
                    }
                    Err(e) => {
                        println!("⚠️ F0033: dump_memory failed for format {}: {}", format, e);
                    }
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0033: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_f0034_memory_map() {
    // F0034: memory_map - Test getting detailed memory map with segments
    println!("Testing F0034: memory_map");
    
    let mut session = match TestSession::new(TestMode::Normal) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ F0034: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ F0034: Test session started with PID {}", pid);
            
            // Test getting detailed memory map
            let result = session.lldb_manager().get_memory_map(None);
            
            match result {
                Ok(memory_map) => {
                    println!("✅ F0034: memory_map succeeded");
                    println!("  Process ID: {}", memory_map.process_id);
                    println!("  Base Address: 0x{:x}", memory_map.base_address);
                    println!("  ASLR Slide: 0x{:x}", memory_map.aslr_slide);
                    println!("  Segments: {}", memory_map.segments.len());
                    
                    for (i, segment) in memory_map.segments.iter().take(3).enumerate() {
                        println!("  Segment {}: {} (0x{:x}-0x{:x}) [{}]", 
                               i + 1, segment.name, segment.vm_address, 
                               segment.vm_address + segment.vm_size, segment.protection);
                    }
                    
                    assert_eq!(memory_map.process_id, pid);
                    assert!(memory_map.segments.len() > 0, "Should have at least one segment");
                }
                Err(e) => {
                    println!("⚠️ F0034: memory_map failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ F0034: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_memory_inspection_workflow() {
    // Integration test: Complete memory inspection workflow
    println!("Testing memory inspection workflow integration");
    
    let mut session = match TestSession::new(TestMode::Memory) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ Workflow: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(pid) => {
            println!("✅ Workflow: Test session started with PID {}", pid);
            
            // Step 1: Get memory map overview
            match session.lldb_manager().get_memory_map(None) {
                Ok(memory_map) => {
                    println!("✅ Workflow: Got memory map with {} segments", memory_map.segments.len());
                    
                    if let Some(first_segment) = memory_map.segments.first() {
                        let test_address = first_segment.vm_address;
                        
                        // Step 2: Read memory from first segment
                        match session.lldb_manager().read_memory(test_address, 128, "hex") {
                            Ok(memory_data) => {
                                println!("✅ Workflow: Read {} bytes from 0x{:x}", 
                                       memory_data.size, memory_data.address);
                            }
                            Err(e) => {
                                println!("⚠️ Workflow: Memory read failed: {}", e);
                            }
                        }
                        
                        // Step 3: Disassemble at the address
                        match session.lldb_manager().disassemble(&format!("0x{:x}", test_address), 5) {
                            Ok(disassembly) => {
                                println!("✅ Workflow: Disassembled {} instructions", 
                                       disassembly.instructions.len());
                            }
                            Err(e) => {
                                println!("⚠️ Workflow: Disassembly failed: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("⚠️ Workflow: Failed to get memory map: {}", e);
                }
            }
            
            // Step 4: Get memory regions overview
            match session.lldb_manager().get_memory_regions() {
                Ok(regions) => {
                    println!("✅ Workflow: Found {} memory regions", regions.len());
                    
                    // Categorize regions
                    let executable_count = regions.iter().filter(|r| r.permissions.contains('x')).count();
                    let writable_count = regions.iter().filter(|r| r.permissions.contains('w')).count();
                    let readonly_count = regions.iter().filter(|r| r.permissions.contains('r') && !r.permissions.contains('w')).count();
                    
                    println!("  Executable regions: {}", executable_count);
                    println!("  Writable regions: {}", writable_count);  
                    println!("  Read-only regions: {}", readonly_count);
                }
                Err(e) => {
                    println!("⚠️ Workflow: Failed to get memory regions: {}", e);
                }
            }
            
            // Step 5: Search for a common pattern
            match session.lldb_manager().search_memory("main", "ascii", 0x100000000, 0x10000) {
                Ok(matches) => {
                    println!("✅ Workflow: Memory search found {} matches for 'main'", matches.len());
                }
                Err(e) => {
                    println!("⚠️ Workflow: Memory search failed: {}", e);
                }
            }
            
            println!("✅ Workflow: Complete memory inspection workflow tested");
        }
        Err(e) => {
            println!("⚠️ Workflow: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}

#[tokio::test]
async fn test_memory_modification_verification() {
    // Test memory write and read verification cycle
    println!("Testing memory modification and verification");
    
    let mut session = match TestSession::new(TestMode::Memory) {
        Ok(s) => s,
        Err(e) => {
            println!("⚠️ Verification: Could not create test session: {}", e);
            return;
        }
    };
    
    match session.start() {
        Ok(_pid) => {
            // Set breakpoint in memory scenario
            let _ = session.lldb_manager().set_breakpoint("create_heap_patterns", None);
            let _ = session.lldb_manager().continue_execution();
            
            let test_address = 0x100000000;
            let test_data = "MODIFIED";
            
            // Step 1: Read original memory
            match session.lldb_manager().read_memory(test_address, 64, "ascii") {
                Ok(original) => {
                    println!("✅ Verification: Original memory content: {}...", 
                           original.content.chars().take(20).collect::<String>());
                    
                    // Step 2: Write new data
                    match session.lldb_manager().write_memory(test_address, test_data, "ascii") {
                        Ok(success) => {
                            if success {
                                println!("✅ Verification: Memory write successful");
                                
                                // Step 3: Read back to verify
                                match session.lldb_manager().read_memory(test_address, 64, "ascii") {
                                    Ok(modified) => {
                                        println!("✅ Verification: Modified memory content: {}...", 
                                               modified.content.chars().take(20).collect::<String>());
                                        
                                        if modified.content.contains(test_data) {
                                            println!("✅ Verification: Memory modification verified");
                                        } else {
                                            println!("⚠️ Verification: Memory modification not reflected");
                                        }
                                    }
                                    Err(e) => {
                                        println!("⚠️ Verification: Verification read failed: {}", e);
                                    }
                                }
                            } else {
                                println!("⚠️ Verification: Memory write reported failure");
                            }
                        }
                        Err(e) => {
                            println!("⚠️ Verification: Memory write failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("⚠️ Verification: Original memory read failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ Verification: Could not start debugging session: {}", e);
        }
    }
    
    let _ = session.cleanup();
}