use anyhow::Result;
use std::path::Path;
use tracing::{info, error};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use std::fs::OpenOptions;
use chrono::Utc;

pub struct LoggingManager;

impl LoggingManager {
    /// Initialize logging with both console and file output
    pub fn init_logging(verbose: bool, log_to_file: bool) -> Result<()> {
        let log_level = if verbose {
            "debug"
        } else {
            "info"
        };
        
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(format!("ghostwin={}", log_level)));
        
        if log_to_file {
            // Create logs directory if it doesn't exist
            std::fs::create_dir_all("logs")?;
            
            let log_file_path = format!("logs/ghostwin_{}.log", 
                Utc::now().format("%Y%m%d_%H%M%S"));
            
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file_path)?;
            
            let file_layer = fmt::layer()
                .with_writer(file)
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true);
            
            let console_layer = fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(true)
                .with_target(false);
            
            tracing_subscriber::registry()
                .with(env_filter)
                .with(console_layer)
                .with(file_layer)
                .init();
            
            info!("Logging initialized - Console + File: {}", log_file_path);
        } else {
            let console_layer = fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(true)
                .with_target(false);
            
            tracing_subscriber::registry()
                .with(env_filter)
                .with(console_layer)
                .init();
            
            info!("Logging initialized - Console only");
        }
        
        Ok(())
    }
    
    /// Log system information for debugging
    pub fn log_system_info() {
        info!("=== System Information ===");
        info!("GhostWin Version: {}", env!("CARGO_PKG_VERSION"));
        info!("Rust Version: {}", env!("VERGEN_RUSTC_SEMVER"));
        
        #[cfg(target_os = "windows")]
        {
            if let Ok(version) = std::process::Command::new("ver").output() {
                let version_str = String::from_utf8_lossy(&version.stdout);
                info!("Windows Version: {}", version_str.trim());
            }
            
            if let Ok(username) = std::env::var("USERNAME") {
                info!("Current User: {}", username);
            }
            
            if let Ok(computername) = std::env::var("COMPUTERNAME") {
                info!("Computer Name: {}", computername);
            }
        }
        
        info!("Working Directory: {}", std::env::current_dir().unwrap_or_default().display());
        info!("=============================");
    }
    
    /// Log build environment for diagnostics
    pub fn log_build_environment() {
        info!("=== Build Environment ===");
        
        // Check for required tools
        let tools = vec![
            ("DISM", "dism", "/English /Get-WimInfo /?"),
            ("7-Zip", "7z", "--help"),
            ("PowerShell", "powershell", "Get-Host | Select-Object Version"),
        ];
        
        for (name, cmd, test_arg) in tools {
            match std::process::Command::new(cmd)
                .args(test_arg.split_whitespace())
                .output() {
                Ok(output) => {
                    if output.status.success() {
                        info!("✅ {} - Available", name);
                    } else {
                        error!("❌ {} - Command failed", name);
                    }
                }
                Err(_) => {
                    error!("❌ {} - Not found in PATH", name);
                }
            }
        }
        
        // Check ADK installation
        let adk_paths = vec![
            "C:\\Program Files (x86)\\Windows Kits\\10\\Assessment and Deployment Kit",
            "C:\\Program Files\\Windows Kits\\10\\Assessment and Deployment Kit",
        ];
        
        let adk_found = adk_paths.iter().find(|path| Path::new(path).exists());
        match adk_found {
            Some(path) => info!("✅ Windows ADK - Found at {}", path),
            None => error!("❌ Windows ADK - Not found"),
        }
        
        info!("========================");
    }
    
    /// Create a detailed error report for failed builds
    pub fn create_error_report(error: &anyhow::Error, operation: &str) -> Result<String> {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        let mut report = String::new();
        
        report.push_str(&format!("=== GhostWin Error Report ===\n"));
        report.push_str(&format!("Timestamp: {}\n", timestamp));
        report.push_str(&format!("Operation: {}\n", operation));
        report.push_str(&format!("GhostWin Version: {}\n", env!("CARGO_PKG_VERSION")));
        report.push_str(&format!("\n=== Error Details ===\n"));
        report.push_str(&format!("Error: {}\n", error));
        
        // Add error chain
        let mut source = error.source();
        let mut level = 1;
        while let Some(err) = source {
            report.push_str(&format!("Caused by ({}): {}\n", level, err));
            source = err.source();
            level += 1;
        }
        
        report.push_str(&format!("\n=== Environment ===\n"));
        report.push_str(&format!("Working Directory: {}\n", 
            std::env::current_dir().unwrap_or_default().display()));
        
        #[cfg(target_os = "windows")]
        {
            if let Ok(username) = std::env::var("USERNAME") {
                report.push_str(&format!("User: {}\n", username));
            }
            if let Ok(computername) = std::env::var("COMPUTERNAME") {
                report.push_str(&format!("Computer: {}\n", computername));
            }
        }
        
        report.push_str(&format!("\n=== Suggestions ===\n"));
        
        // Add context-specific suggestions
        let error_msg = error.to_string().to_lowercase();
        if error_msg.contains("administrator") || error_msg.contains("privilege") {
            report.push_str("- Run GhostWin as Administrator\n");
            report.push_str("- Right-click Command Prompt → 'Run as administrator'\n");
        }
        if error_msg.contains("dism") {
            report.push_str("- Install Windows ADK from Microsoft\n");
            report.push_str("- Ensure DISM is available in PATH\n");
        }
        if error_msg.contains("7z") || error_msg.contains("extraction") {
            report.push_str("- Install 7-Zip from https://www.7-zip.org/\n");
            report.push_str("- Ensure 7z.exe is in PATH\n");
        }
        if error_msg.contains("space") || error_msg.contains("disk") {
            report.push_str("- Free up disk space (need 10GB+ available)\n");
            report.push_str("- Use a different output directory with more space\n");
        }
        
        report.push_str("- Run 'ghostwin validate' to check system requirements\n");
        report.push_str("- Check the documentation at https://github.com/CK-Technology/ghostwin\n");
        
        // Save error report to file
        let report_path = format!("error_report_{}.txt", 
            Utc::now().format("%Y%m%d_%H%M%S"));
        
        std::fs::write(&report_path, &report)?;
        info!("Error report saved to: {}", report_path);
        
        Ok(report)
    }
}