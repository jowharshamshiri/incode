use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Configure LLDB library linking based on platform
    if cfg!(target_os = "macos") {
        // macOS LLDB linking configuration
        configure_macos_lldb();
    } else if cfg!(target_os = "linux") {
        // Linux LLDB linking configuration
        configure_linux_lldb();
    } else if cfg!(target_os = "windows") {
        // Windows LLDB linking configuration
        configure_windows_lldb();
    }
}

fn configure_macos_lldb() {
    // Use the specific LLDB library we found earlier
    let lldb_lib_path = "/opt/homebrew/Cellar/llvm/20.1.7/lib";
    
    println!("cargo:rustc-link-search=native={}", lldb_lib_path);
    println!("cargo:rustc-link-lib=dylib=lldb");
    println!("cargo:rustc-env=LLDB_LIB_PATH={}", lldb_lib_path);
    
    // Add rpath for runtime linking
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lldb_lib_path);
    
    // Additional macOS framework dependencies
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=CoreFoundation");
    
    // Also need to include the headers path
    println!("cargo:rustc-env=LLDB_INCLUDE_PATH=/opt/homebrew/Cellar/llvm/20.1.7/include");
}

fn configure_linux_lldb() {
    // Linux LLDB library configuration
    let linux_paths = [
        "/usr/lib/x86_64-linux-gnu",
        "/usr/lib64",
        "/usr/local/lib",
        "/opt/llvm/lib",
    ];
    
    for path in &linux_paths {
        let lib_path = PathBuf::from(path);
        if lib_path.join("liblldb.so").exists() {
            println!("cargo:rustc-link-search=native={}", path);
            println!("cargo:rustc-link-lib=dylib=lldb");
            break;
        }
    }
    
    // System package manager installations
    println!("cargo:rustc-link-lib=dylib=lldb");
}

fn configure_windows_lldb() {
    // Windows LLDB library configuration
    if let Ok(llvm_sys_path) = env::var("LLVM_SYS_190_PREFIX") {
        let lib_path = PathBuf::from(llvm_sys_path).join("lib");
        println!("cargo:rustc-link-search=native={}", lib_path.display());
    }
    
    // Common Windows LLDB installation paths
    let windows_paths = [
        "C:\\Program Files\\LLVM\\lib",
        "C:\\Program Files (x86)\\LLVM\\lib",
        "C:\\llvm\\lib",
    ];
    
    for path in &windows_paths {
        let lib_path = PathBuf::from(path);
        if lib_path.join("lldb.lib").exists() {
            println!("cargo:rustc-link-search=native={}", path);
            break;
        }
    }
    
    println!("cargo:rustc-link-lib=static=lldb");
}