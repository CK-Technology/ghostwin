use anyhow::Result;
use tracing::{info, warn};
use crate::tools::ToolManager;
use crate::executor::ScriptExecutor;
use crate::config::ConfigManager;
use crate::cli::LogonArgs;

/// Execute post-install logon scripts
pub async fn execute(args: LogonArgs) -> Result<()> {
    info!("Starting post-install logon script execution");
    crate::cli::validate_host_change_mode("logon", args.dry_run, args.force)?;
    
    let config = ConfigManager::load_config(None).await?;
    let tool_manager = ToolManager::new(&config.tools);
    let executor = ScriptExecutor::new(config.clone());
    let detected_tools = tool_manager.scan_tools().await?;
    let logon_tools = crate::utils::resolve_detected_tools(&config.phases.post_install_logon_paths, &detected_tools);
    
    if logon_tools.is_empty() {
        info!("No logon scripts found to execute");
        return Ok(());
    }
    
    info!("Found {} logon script(s) to execute", logon_tools.len());
    crate::utils::execute_tools_with_dry_run(&logon_tools, &executor, args.dry_run, "logon script").await?;
    
    info!("Logon script execution completed");
    
    // Disable auto-logon after running scripts
    disable_auto_logon(args.dry_run).await?;
    
    Ok(())
}

async fn disable_auto_logon(dry_run: bool) -> Result<()> {
    info!("Disabling automatic logon");

    if dry_run {
        info!("Dry run: would disable automatic logon via Winlogon registry values");
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        let reg_script = r#"
Windows Registry Editor Version 5.00

; Disable automatic logon after first run
[HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon]
"AutoAdminLogon"=""
"DefaultUserName"=""
"AutoAdminLogon"="0"
"DefaultPassword"=""
"#;

        let temp_reg = crate::utils::write_temp_reg_script("disable_autologon.reg", reg_script)?;
        let output = crate::utils::import_reg_file(&temp_reg)?;

        if output.status.success() {
            info!("✅ Automatic logon disabled");
            let _ = std::fs::remove_file(temp_reg); // Clean up
        } else {
            warn!("⚠️ Failed to disable automatic logon: {}", 
                  String::from_utf8_lossy(&output.stderr));
        }

        return Ok(());
    }

    #[cfg(not(target_os = "windows"))]
    {
        warn!("Automatic logon disable is only supported on Windows hosts");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::validate_host_change_mode;

    #[test]
    fn rejects_missing_host_change_mode() {
        let error = validate_host_change_mode("logon", false, false).unwrap_err();
        assert!(error.to_string().contains("requires --dry-run to preview or --force"));
    }

    #[test]
    fn rejects_conflicting_host_change_mode() {
        let error = validate_host_change_mode("logon", true, true).unwrap_err();
        assert!(error.to_string().contains("either --dry-run or --force, but not both"));
    }

    #[test]
    fn accepts_single_host_change_mode() {
        assert!(validate_host_change_mode("logon", true, false).is_ok());
        assert!(validate_host_change_mode("logon", false, true).is_ok());
    }
}
