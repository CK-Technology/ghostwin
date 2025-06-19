use anyhow::Result;
use tracing::{info, warn, error};
use crate::config::ConfigManager;
use crate::utils;

pub async fn execute() -> Result<()> {
    info!("🔍 Validating GhostWin configuration and dependencies");
    
    let mut errors = 0;
    let mut warnings = 0;
    
    // Check admin privileges
    match utils::ensure_admin_privileges() {
        Ok(_) => info!("✅ Administrator privileges confirmed"),
        Err(e) => {
            error!("❌ Administrator privileges required: {}", e);
            errors += 1;
        }
    }
    
    // Check dependencies
    match utils::check_dependencies() {
        Ok(missing) => {
            if missing.is_empty() {
                info!("✅ All required dependencies found");
            } else {
                for dep in &missing {
                    error!("❌ Missing dependency: {}", dep);
                    errors += 1;
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to check dependencies: {}", e);
            errors += 1;
        }
    }
    
    // Load and validate configuration
    match ConfigManager::load_default() {
        Ok(config) => {
            info!("✅ Configuration loaded successfully");
            
            // Validate ADK path
            if let Some(ref adk_path) = config.iso.adk_path {
                if !std::path::Path::new(adk_path).exists() {
                    warn!("⚠️  Custom ADK path does not exist: {}", adk_path);
                    warnings += 1;
                }
            }
            
            // Validate tool folders
            for folder in &config.tools.folders {
                if !std::path::Path::new(folder).exists() {
                    warn!("⚠️  Tool folder not found: {} (will be created during build)", folder);
                    warnings += 1;
                }
            }
            
            // Check WinPE packages
            info!("📦 Configured WinPE packages: {}", config.winpe.packages.len());
            for package in &config.winpe.packages {
                info!("   - {}", package);
            }
            
            // Security settings
            if config.security.password_hash.is_none() && config.security.access_secret.is_none() {
                warn!("⚠️  No access protection configured");
                warnings += 1;
            }
            
            if config.security.vnc_enabled {
                info!("🔗 VNC server enabled on port {}", config.security.vnc_port);
                if config.security.vnc_password.is_none() {
                    warn!("⚠️  VNC enabled but no password set");
                    warnings += 1;
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to load configuration: {}", e);
            errors += 1;
        }
    }
    
    // Summary
    println!("\n📊 Validation Summary:");
    if errors > 0 {
        error!("❌ {} error(s) found", errors);
    }
    if warnings > 0 {
        warn!("⚠️  {} warning(s) found", warnings);
    }
    if errors == 0 && warnings == 0 {
        info!("✅ All validation checks passed!");
    }
    
    if errors > 0 {
        return Err(anyhow::anyhow!("Validation failed with {} error(s)", errors));
    }
    
    Ok(())
}