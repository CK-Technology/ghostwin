use anyhow::Result;
use tracing::{info, warn, error};
use std::path::Path;
use crate::tools::ToolManager;
use crate::executor::ScriptExecutor;
use crate::config::ConfigManager;

/// Execute post-install logon scripts
pub async fn execute() -> Result<()> {
    info!("Starting post-install logon script execution");
    
    let config = ConfigManager::load_config(None).await?;
    let tool_manager = ToolManager::new(&config.tools);
    let executor = ScriptExecutor::new(config.clone());
    
    // Scan for logon scripts
    let logon_tools = tool_manager.scan_tools().await?
        .into_iter()
        .filter(|tool| {
            matches!(tool.category, crate::tools::ToolCategory::Logon) ||
            tool.path.to_string_lossy().contains("Logon") ||
            tool.path.to_string_lossy().contains("logon")
        })
        .collect::<Vec<_>>();
    
    if logon_tools.is_empty() {
        info!("No logon scripts found to execute");
        return Ok(());
    }
    
    info!("Found {} logon script(s) to execute", logon_tools.len());
    
    // Execute logon scripts in order
    for tool in logon_tools {
        info!("Executing logon script: {}", tool.path.display());
        
        match executor.execute_script(&tool.path).await {
            Ok(output) => {
                info!("✅ Successfully executed: {}", tool.name);
                if !output.trim().is_empty() {
                    info!("Output: {}", output);
                }
            }
            Err(e) => {
                error!("❌ Failed to execute {}: {}", tool.name, e);
                warn!("Continuing with next script...");
            }
        }
    }
    
    info!("Logon script execution completed");
    
    // Disable auto-logon after running scripts
    disable_auto_logon().await?;
    
    Ok(())
}

async fn disable_auto_logon() -> Result<()> {
    info!("Disabling automatic logon");
    
    let reg_script = r#"
Windows Registry Editor Version 5.00

; Disable automatic logon after first run
[HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon]
"AutoAdminLogon"=""
"DefaultUserName"=""
"AutoAdminLogon"="0"
"DefaultPassword"=""
"#;
    
    let temp_reg = Path::new("C:\\temp\\disable_autologon.reg");
    std::fs::write(&temp_reg, reg_script)?;
    
    let output = std::process::Command::new("reg")
        .args(["import", temp_reg.to_str().unwrap()])
        .output()?;
    
    if output.status.success() {
        info!("✅ Automatic logon disabled");
        let _ = std::fs::remove_file(temp_reg); // Clean up
    } else {
        warn!("⚠️ Failed to disable automatic logon: {}", 
              String::from_utf8_lossy(&output.stderr));
    }
    
    Ok(())
}