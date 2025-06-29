use anyhow::Result;
use tracing::{info, warn, error};
use std::path::Path;
use crate::tools::ToolManager;
use crate::executor::ScriptExecutor;
use crate::config::ConfigManager;

/// Execute system setup tasks before user logon
pub async fn execute() -> Result<()> {
    info!("Starting system setup task execution");
    
    let config = ConfigManager::load_config(None).await?;
    let tool_manager = ToolManager::new(&config.tools);
    let executor = ScriptExecutor::new(config.clone());
    
    // Scan for system setup scripts
    let system_tools = tool_manager.scan_tools().await?
        .into_iter()
        .filter(|tool| {
            tool.name.contains("[system]") ||
            tool.path.to_string_lossy().contains("system") ||
            tool.path.to_string_lossy().contains("System")
        })
        .collect::<Vec<_>>();
    
    info!("Found {} system setup script(s) to execute", system_tools.len());
    
    // Execute system setup scripts
    for tool in system_tools {
        info!("Executing system setup script: {}", tool.path.display());
        
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
    
    // Apply basic system configurations
    apply_system_configurations().await?;
    
    info!("System setup task execution completed");
    Ok(())
}

async fn apply_system_configurations() -> Result<()> {
    info!("Applying basic system configurations");
    
    // Check and configure Windows PE and ADK for Windows 11 24H2
    check_windows_pe_adk().await?;
    
    // Enable Administrator account
    let output = std::process::Command::new("net")
        .args(["user", "Administrator", "/active:Yes"])
        .output()?;
    
    if output.status.success() {
        info!("✅ Administrator account enabled");
    } else {
        warn!("⚠️ Failed to enable Administrator account: {}", 
              String::from_utf8_lossy(&output.stderr));
    }
    
    // Set up basic registry configurations
    apply_registry_fixes().await?;
    
    // Configure power settings for deployment
    configure_power_settings().await?;
    
    Ok(())
}

async fn apply_registry_fixes() -> Result<()> {
    info!("Applying registry fixes");
    
    let reg_script = r#"
Windows Registry Editor Version 5.00

; Disable Windows Defender real-time protection during deployment
[HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Windows Defender\Real-Time Protection]
"DisableRealtimeMonitoring"=dword:00000001

; Skip OOBE privacy settings
[HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Windows\OOBE]
"DisablePrivacyExperience"=dword:00000001

; Disable Windows Update during deployment
[HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU]
"NoAutoUpdate"=dword:00000001
"#;
    
    let temp_reg = Path::new("C:\\temp\\system_setup.reg");
    std::fs::create_dir_all("C:\\temp")?;
    std::fs::write(&temp_reg, reg_script)?;
    
    let output = std::process::Command::new("reg")
        .args(["import", temp_reg.to_str().unwrap()])
        .output()?;
    
    if output.status.success() {
        info!("✅ Registry fixes applied");
        let _ = std::fs::remove_file(temp_reg); // Clean up
    } else {
        warn!("⚠️ Failed to apply registry fixes: {}", 
              String::from_utf8_lossy(&output.stderr));
    }
    
    Ok(())
}

async fn configure_power_settings() -> Result<()> {
    info!("Configuring power settings for deployment");
    
    // Set high performance power plan
    let output = std::process::Command::new("powercfg")
        .args(["/setactive", "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c"])
        .output()?;
    
    if output.status.success() {
        info!("✅ High performance power plan activated");
    } else {
        warn!("⚠️ Failed to set power plan: {}", 
              String::from_utf8_lossy(&output.stderr));
    }
    
    // Disable sleep and hibernation during deployment
    let _ = std::process::Command::new("powercfg")
        .args(["/change", "standby-timeout-ac", "0"])
        .output();
    
    let _ = std::process::Command::new("powercfg")
        .args(["/change", "hibernate-timeout-ac", "0"])
        .output();
    
    info!("Power settings configured for deployment");
    Ok(())
}

async fn check_windows_pe_adk() -> Result<()> {
    info!("Checking Windows PE and ADK for Windows 11 24H2");
    
    // Check for Windows ADK installation
    let adk_path = Path::new(r"C:\Program Files (x86)\Windows Kits\10");
    let pe_addon_path = Path::new(r"C:\Program Files (x86)\Windows Kits\10\Assessment and Deployment Kit\Windows Preinstallation Environment");
    
    if adk_path.exists() {
        info!("✅ Windows ADK found at: {}", adk_path.display());
        
        // Check for specific Windows 11 24H2 components
        let winpe_path = adk_path.join("Assessment and Deployment Kit").join("Windows Preinstallation Environment");
        let deployment_tools_path = adk_path.join("Assessment and Deployment Kit").join("Deployment Tools");
        
        if pe_addon_path.exists() || winpe_path.exists() {
            info!("✅ Windows PE add-on found");
            
            // Check for DISM and other required tools
            check_deployment_tools(&deployment_tools_path).await?;
        } else {
            warn!("⚠️ Windows PE add-on not found!");
            warn!("   For Windows 11 24H2 deployment, please install:");
            warn!("   1. Windows ADK for Windows 11, version 24H2");
            warn!("   2. Windows PE add-on for the Windows ADK");
            warn!("   Download from: https://docs.microsoft.com/en-us/windows-hardware/get-started/adk-install");
        }
    } else {
        warn!("⚠️ Windows ADK not found!");
        warn!("   Windows ADK is required for Windows 11 24H2 deployment");
        warn!("   Please install both:");
        warn!("   1. Windows ADK for Windows 11, version 24H2");
        warn!("   2. Windows PE add-on for the Windows ADK");
        warn!("   Download from: https://docs.microsoft.com/en-us/windows-hardware/get-started/adk-install");
    }
    
    Ok(())
}

async fn check_deployment_tools(deployment_tools_path: &Path) -> Result<()> {
    info!("Checking deployment tools");
    
    // Check for DISM
    let dism_path = deployment_tools_path.join("amd64").join("DISM").join("dism.exe");
    if dism_path.exists() {
        info!("✅ DISM found at: {}", dism_path.display());
        
        // Test DISM functionality
        let output = std::process::Command::new(&dism_path)
            .args(["/Get-WimInfo", "/?"])
            .output()?;
            
        if output.status.success() {
            info!("✅ DISM is functional");
        } else {
            warn!("⚠️ DISM test failed: {}", String::from_utf8_lossy(&output.stderr));
        }
    } else {
        warn!("⚠️ DISM not found in deployment tools");
    }
    
    // Check for ImageX (legacy support)
    let imagex_path = deployment_tools_path.join("amd64").join("imagex.exe");
    if imagex_path.exists() {
        info!("✅ ImageX found for legacy support");
    }
    
    // Check for Windows PE tools
    let winpe_tools_path = deployment_tools_path.join("amd64").join("Oscdimg");
    if winpe_tools_path.exists() {
        info!("✅ Windows PE creation tools found");
    } else {
        warn!("⚠️ Windows PE creation tools not found");
    }
    
    Ok(())
}