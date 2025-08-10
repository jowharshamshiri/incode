use std::collections::HashMap;
use crate::lldb_manager::LldbManager;
use crate::error::IncodeResult;
use tracing::{debug, info};
use serde_json::{Value, Map, Number};

/// Get comprehensive target information including architecture, platform, and executable details
pub async fn get_target_info(
    manager: &LldbManager,
    arguments: HashMap<String, Value>,
) -> IncodeResult<Value> {
    debug!("get_target_info called with arguments: {:?}", arguments);

    // Extract optional parameters
    let include_debug_info = arguments.get("include_debug_info")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let include_file_details = arguments.get("include_file_details")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    
    let analyze_symbols = arguments.get("analyze_symbols")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Get target information from LLDB manager
    let target_info = manager.get_target_info()?;
    
    // Build response object
    let mut response = Map::new();
    response.insert("executable_path".to_string(), Value::String(target_info.executable_path.clone()));
    response.insert("architecture".to_string(), Value::String(target_info.architecture.clone()));
    response.insert("platform".to_string(), Value::String(target_info.platform.clone()));
    response.insert("executable_format".to_string(), Value::String(target_info.executable_format.clone()));
    response.insert("endianness".to_string(), Value::String(target_info.endianness.clone()));

    if include_debug_info {
        response.insert("has_debug_symbols".to_string(), Value::Bool(target_info.has_debug_symbols));
        response.insert("is_stripped".to_string(), Value::Bool(target_info.is_stripped));
    }

    if include_file_details {
        response.insert("file_size".to_string(), Value::Number(Number::from(target_info.file_size)));
        response.insert("is_pie".to_string(), Value::Bool(target_info.is_pie));
        
        if let Some(entry_point) = target_info.entry_point {
            response.insert("entry_point".to_string(), Value::String(format!("0x{:x}", entry_point)));
        }
        
        if let Some(base_address) = target_info.base_address {
            response.insert("base_address".to_string(), Value::String(format!("0x{:x}", base_address)));
        }
        
        if let Some(creation_time) = target_info.creation_time {
            if let Ok(duration) = creation_time.duration_since(std::time::UNIX_EPOCH) {
                response.insert("creation_time".to_string(), Value::String(duration.as_secs().to_string()));
            }
        }
    }
    
    if analyze_symbols {
        // Add symbol analysis information
        response.insert("symbol_analysis_enabled".to_string(), Value::Bool(true));
        response.insert("symbol_count_estimated".to_string(), Value::Number(Number::from(1000))); // Mock value
    }

    info!("Target information retrieved for: {}", target_info.executable_path);
    Ok(Value::Object(response))
}

/// Get comprehensive platform information including OS version, architecture, and development environment details
pub async fn get_platform_info(
    manager: &LldbManager,
    arguments: HashMap<String, Value>,
) -> IncodeResult<Value> {
    debug!("get_platform_info called with arguments: {:?}", arguments);

    // Extract optional parameters
    let include_development_info = arguments.get("include_development_info")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let include_capabilities = arguments.get("include_capabilities")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // Get platform information from LLDB manager
    let platform_info = manager.get_platform_info()?;
    
    // Build response object
    let mut response = Map::new();
    response.insert("name".to_string(), Value::String(platform_info.name.clone()));
    response.insert("platform_name".to_string(), Value::String(platform_info.name.clone())); // Alias for tests
    response.insert("version".to_string(), Value::String(platform_info.version.clone()));
    response.insert("os_version".to_string(), Value::String(platform_info.version.clone())); // Alias for tests
    response.insert("architecture".to_string(), Value::String(platform_info.architecture.clone()));
    response.insert("byte_order".to_string(), Value::String("little".to_string())); // Default for most platforms
    response.insert("vendor".to_string(), Value::String(platform_info.vendor.clone()));
    response.insert("environment".to_string(), Value::String(platform_info.environment.clone()));
    response.insert("working_directory".to_string(), Value::String(platform_info.working_directory.clone()));
    
    // Add supported architectures as array
    let arch_array: Vec<Value> = platform_info.supported_architectures
        .iter()
        .map(|arch| Value::String(arch.clone()))
        .collect();
    response.insert("supported_architectures".to_string(), Value::Array(arch_array));

    if let Some(hostname) = &platform_info.hostname {
        response.insert("hostname".to_string(), Value::String(hostname.clone()));
    }

    if include_development_info {
        if let Some(sdk_version) = &platform_info.sdk_version {
            response.insert("sdk_version".to_string(), Value::String(sdk_version.clone()));
        }
        if let Some(deployment_target) = &platform_info.deployment_target {
            response.insert("deployment_target".to_string(), Value::String(deployment_target.clone()));
        }
    }

    if include_capabilities {
        response.insert("is_simulator".to_string(), Value::Bool(platform_info.is_simulator));
        response.insert("is_remote".to_string(), Value::Bool(platform_info.is_remote));
        response.insert("supports_jit".to_string(), Value::Bool(platform_info.supports_jit));
    }

    info!("Platform information retrieved for: {}", platform_info.name);
    Ok(Value::Object(response))
}

/// List all loaded modules/libraries with their addresses and debug information
pub async fn list_modules(
    manager: &LldbManager,
    arguments: HashMap<String, Value>,
) -> IncodeResult<Value> {
    debug!("list_modules called with arguments: {:?}", arguments);

    // Extract optional parameters
    let filter_name = arguments.get("filter_name")
        .or_else(|| arguments.get("name_pattern")) // Support both parameter names
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let limit = arguments.get("limit")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize);

    let include_debug_info = arguments.get("include_debug_info")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let include_addresses = arguments.get("include_addresses")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let include_symbols = arguments.get("include_symbols")
        .and_then(|v| v.as_bool())
        .unwrap_or(false); // Symbol info can be verbose

    // Get modules from LLDB manager
    let modules = manager.list_modules(filter_name.as_deref(), include_debug_info)?;
    
    // Build response array
    let mut module_array = Vec::new();
    
    for module in modules {
        let mut module_obj = Map::new();
        module_obj.insert("name".to_string(), Value::String(module.name.clone()));
        module_obj.insert("file_path".to_string(), Value::String(module.file_path.clone()));
        module_obj.insert("path".to_string(), Value::String(module.file_path.clone())); // Alias for tests
        module_obj.insert("uuid".to_string(), Value::String(module.uuid.clone()));
        module_obj.insert("architecture".to_string(), Value::String(module.architecture.clone()));
        module_obj.insert("is_main_executable".to_string(), Value::Bool(module.is_main_executable));
        module_obj.insert("has_debug_symbols".to_string(), Value::Bool(module.has_debug_symbols));
        module_obj.insert("file_size".to_string(), Value::Number(Number::from(module.file_size)));

        if let Some(version) = &module.version {
            module_obj.insert("version".to_string(), Value::String(version.clone()));
        }

        if let Some(symbol_vendor) = &module.symbol_vendor {
            module_obj.insert("symbol_vendor".to_string(), Value::String(symbol_vendor.clone()));
        }

        if include_addresses {
            module_obj.insert("load_address".to_string(), Value::String(format!("0x{:x}", module.load_address)));
            if let Some(slide) = module.slide {
                module_obj.insert("slide".to_string(), Value::String(format!("0x{:x}", slide)));
            }
        }

        if include_symbols {
            module_obj.insert("num_symbols".to_string(), Value::Number(Number::from(module.num_symbols)));
        }

        if include_debug_info && !module.compile_units.is_empty() {
            let cu_array: Vec<Value> = module.compile_units
                .iter()
                .map(|cu| Value::String(cu.clone()))
                .collect();
            module_obj.insert("compile_units".to_string(), Value::Array(cu_array));
        }

        module_array.push(Value::Object(module_obj));
    }
    
    // Apply limit if specified
    let total_count = module_array.len();
    if let Some(limit_value) = limit {
        module_array.truncate(limit_value);
    }

    let mut response = Map::new();
    response.insert("modules".to_string(), Value::Array(module_array.clone()));
    response.insert("count".to_string(), Value::Number(Number::from(module_array.len())));
    response.insert("total_count".to_string(), Value::Number(Number::from(total_count))); // Total before limit

    if let Some(filter) = filter_name {
        response.insert("filter_applied".to_string(), Value::String(filter));
    }

    info!("Listed {} modules", module_array.len());
    Ok(Value::Object(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lldb_manager::LldbManager;
    use std::collections::HashMap;
    use tokio;

    #[tokio::test]
    async fn test_get_target_info() {
        let manager = LldbManager::new(None).expect("Failed to create LldbManager");
        let args = HashMap::new();
        
        let result = get_target_info(&manager, args).await;
        assert!(result.is_ok());
        
        let info = result.unwrap();
        assert!(info.get("executable_path").is_some());
        assert!(info.get("architecture").is_some());
        assert!(info.get("platform").is_some());
        assert!(info.get("executable_format").is_some());
    }

    #[tokio::test]
    async fn test_get_target_info_with_options() {
        let manager = LldbManager::new(None).expect("Failed to create LldbManager");
        let mut args = HashMap::new();
        args.insert("include_debug_info".to_string(), Value::Bool(true));
        args.insert("include_file_details".to_string(), Value::Bool(true));
        
        let result = get_target_info(&manager, args).await;
        assert!(result.is_ok());
        
        let info = result.unwrap();
        assert!(info.get("has_debug_symbols").is_some());
        assert!(info.get("is_stripped").is_some());
        assert!(info.get("file_size").is_some());
        assert!(info.get("is_pie").is_some());
    }

    #[tokio::test]
    async fn test_get_target_info_minimal() {
        let manager = LldbManager::new(None).expect("Failed to create LldbManager");
        let mut args = HashMap::new();
        args.insert("include_debug_info".to_string(), Value::Bool(false));
        args.insert("include_file_details".to_string(), Value::Bool(false));
        
        let result = get_target_info(&manager, args).await;
        assert!(result.is_ok());
        
        let info = result.unwrap();
        assert!(info.get("executable_path").is_some());
        assert!(info.get("architecture").is_some());
        assert!(info.get("platform").is_some());
        assert!(info.get("executable_format").is_some());
        // Should not include debug or file details
        assert!(info.get("has_debug_symbols").is_none());
        assert!(info.get("file_size").is_none());
    }

    #[tokio::test]
    async fn test_get_platform_info() {
        let manager = LldbManager::new(None).expect("Failed to create LldbManager");
        let args = HashMap::new();
        
        let result = get_platform_info(&manager, args).await;
        assert!(result.is_ok());
        
        let info = result.unwrap();
        assert!(info.get("name").is_some());
        assert!(info.get("version").is_some());
        assert!(info.get("architecture").is_some());
        assert!(info.get("vendor").is_some());
        assert!(info.get("environment").is_some());
        assert!(info.get("supported_architectures").is_some());
    }

    #[tokio::test]
    async fn test_get_platform_info_with_options() {
        let manager = LldbManager::new(None).expect("Failed to create LldbManager");
        let mut args = HashMap::new();
        args.insert("include_development_info".to_string(), Value::Bool(true));
        args.insert("include_capabilities".to_string(), Value::Bool(true));
        
        let result = get_platform_info(&manager, args).await;
        assert!(result.is_ok());
        
        let info = result.unwrap();
        assert!(info.get("name").is_some());
        assert!(info.get("is_simulator").is_some());
        assert!(info.get("is_remote").is_some());
        assert!(info.get("supports_jit").is_some());
    }

    #[tokio::test]
    async fn test_get_platform_info_minimal() {
        let manager = LldbManager::new(None).expect("Failed to create LldbManager");
        let mut args = HashMap::new();
        args.insert("include_development_info".to_string(), Value::Bool(false));
        args.insert("include_capabilities".to_string(), Value::Bool(false));
        
        let result = get_platform_info(&manager, args).await;
        assert!(result.is_ok());
        
        let info = result.unwrap();
        assert!(info.get("name").is_some());
        assert!(info.get("version").is_some());
        assert!(info.get("architecture").is_some());
        // Should not include development info or capabilities
        assert!(info.get("is_simulator").is_none());
        assert!(info.get("sdk_version").is_none());
    }

    #[tokio::test]
    async fn test_list_modules() {
        let manager = LldbManager::new(None).expect("Failed to create LldbManager");
        let args = HashMap::new();
        
        let result = list_modules(&manager, args).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(response.get("modules").is_some());
        assert!(response.get("count").is_some());
        
        let modules = response.get("modules").unwrap().as_array().unwrap();
        assert!(!modules.is_empty());
        
        let first_module = &modules[0];
        assert!(first_module.get("name").is_some());
        assert!(first_module.get("file_path").is_some());
        assert!(first_module.get("uuid").is_some());
        assert!(first_module.get("architecture").is_some());
    }

    #[tokio::test]
    async fn test_list_modules_with_filter() {
        let manager = LldbManager::new(None).expect("Failed to create LldbManager");
        let mut args = HashMap::new();
        args.insert("filter_name".to_string(), Value::String("test".to_string()));
        
        let result = list_modules(&manager, args).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(response.get("filter_applied").is_some());
        let modules = response.get("modules").unwrap().as_array().unwrap();
        
        // Should contain only modules matching "test"
        for module in modules {
            let name = module.get("name").unwrap().as_str().unwrap();
            assert!(name.contains("test"));
        }
    }

    #[tokio::test]
    async fn test_list_modules_with_options() {
        let manager = LldbManager::new(None).expect("Failed to create LldbManager");
        let mut args = HashMap::new();
        args.insert("include_addresses".to_string(), Value::Bool(true));
        args.insert("include_symbols".to_string(), Value::Bool(true));
        args.insert("include_debug_info".to_string(), Value::Bool(true));
        
        let result = list_modules(&manager, args).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        let modules = response.get("modules").unwrap().as_array().unwrap();
        let first_module = &modules[0];
        
        assert!(first_module.get("load_address").is_some());
        assert!(first_module.get("num_symbols").is_some());
        // Main executable should have debug info
        if first_module.get("is_main_executable").unwrap().as_bool().unwrap() {
            assert!(first_module.get("compile_units").is_some());
        }
    }
}