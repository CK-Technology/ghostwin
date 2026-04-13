use anyhow::Result;
use std::path::Path;

pub mod recovery;

#[cfg(target_os = "windows")]
pub fn ensure_windows_host(_action: &str) -> Result<()> {
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn ensure_windows_host(action: &str) -> Result<()> {
    Err(anyhow::anyhow!("{} is only supported on Windows hosts", action))
}

pub fn write_temp_reg_script(file_name: &str, content: &str) -> Result<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    let base_dir = Path::new("C:\\temp").to_path_buf();

    #[cfg(not(target_os = "windows"))]
    let base_dir = std::env::temp_dir();

    std::fs::create_dir_all(&base_dir)?;
    let file_path = base_dir.join(file_name);
    std::fs::write(&file_path, content)?;
    Ok(file_path)
}

pub fn import_reg_file(reg_path: &Path) -> Result<std::process::Output> {
    ensure_windows_host("Registry import")?;
    Ok(std::process::Command::new("reg")
        .args(["import", reg_path.to_string_lossy().as_ref()])
        .output()?)
}

pub async fn execute_tools_with_dry_run(
    tools: &[crate::tools::DetectedTool],
    executor: &crate::executor::ScriptExecutor,
    dry_run: bool,
    action_label: &str,
) -> Result<()> {
    for tool in tools {
        tracing::info!("Executing {}: {}", action_label, tool.path.display());

        if dry_run {
            tracing::info!("Dry run: would execute {}", tool.path.display());
            continue;
        }

        match executor.execute_script(&tool.path).await {
            Ok(output) => {
                tracing::info!("✅ Successfully executed: {}", tool.name);
                if !output.trim().is_empty() {
                    tracing::info!("Output: {}", output);
                }
            }
            Err(e) => {
                tracing::error!("❌ Failed to execute {}: {}", tool.name, e);
                tracing::warn!("Continuing with next script...");
            }
        }
    }

    Ok(())
}

pub fn resolve_detected_tools(
    configured_paths: &[String],
    detected_tools: &[crate::tools::DetectedTool],
) -> Vec<crate::tools::DetectedTool> {
    configured_paths
        .iter()
        .filter_map(|configured_path| {
            let configured = Path::new(configured_path);
            detected_tools
                .iter()
                .find(|tool| tool.path.ends_with(configured))
                .cloned()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::resolve_detected_tools;
    use crate::tools::{DetectedTool, ToolCategory};
    use std::path::PathBuf;

    fn tool(path: &str, category: ToolCategory) -> DetectedTool {
        DetectedTool {
            name: PathBuf::from(path).file_name().unwrap().to_string_lossy().to_string(),
            path: PathBuf::from(path),
            category,
            executable: true,
            hidden: false,
            auto_run: true,
        }
    }

    #[test]
    fn resolves_phase_paths_against_detected_tools() {
        let detected = vec![
            tool("./pe_autorun/system_setup/fontfix.reg", ToolCategory::PEAutoRun),
            tool("./pe_autorun/drivers/Load-Drivers.ps1", ToolCategory::PEAutoRun),
            tool("./scripts/basic/registry/disable_auto_logon.reg", ToolCategory::Logon),
        ];

        let resolved = resolve_detected_tools(
            &[
                "pe_autorun/system_setup/fontfix.reg".to_string(),
                "scripts/basic/registry/disable_auto_logon.reg".to_string(),
            ],
            &detected,
        );

        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].name, "fontfix.reg");
        assert_eq!(resolved[1].name, "disable_auto_logon.reg");
    }
}

pub fn ensure_admin_privileges() -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        use winapi::um::processthreadsapi::GetCurrentProcess;
        use winapi::um::securitybaseapi::GetTokenInformation;
        use winapi::um::winnt::{TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
        use winapi::um::handleapi::CloseHandle;
        use std::mem;
        
        unsafe {
            let mut token = std::ptr::null_mut();
            let process = GetCurrentProcess();
            
            if winapi::um::processthreadsapi::OpenProcessToken(process, TOKEN_QUERY, &mut token) == 0 {
                return Err(anyhow::anyhow!("Failed to open process token"));
            }
            
            let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
            let mut return_length = 0;
            
            let result = GetTokenInformation(
                token,
                TokenElevation,
                &mut elevation as *mut _ as *mut _,
                mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut return_length,
            );
            
            CloseHandle(token);
            
            if result == 0 {
                return Err(anyhow::anyhow!("Failed to get token information"));
            }
            
            if elevation.TokenIsElevated == 0 {
                return Err(anyhow::anyhow!("Administrator privileges required"));
            }
        }
    }
    
    Ok(())
}

#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub fn get_free_disk_space(drive: &str) -> Result<u64> {
    use winapi::um::fileapi::GetDiskFreeSpaceExW;
    use winapi::um::winnt::ULARGE_INTEGER;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    
    let wide_path: Vec<u16> = OsStr::new(drive).encode_wide().chain(std::iter::once(0)).collect();
    
    unsafe {
        let mut free_bytes: ULARGE_INTEGER = std::mem::zeroed();
        let result = GetDiskFreeSpaceExW(
            wide_path.as_ptr(),
            &mut free_bytes,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        
        if result != 0 {
            Ok(*free_bytes.QuadPart())
        } else {
            Err(anyhow::anyhow!("Failed to get disk space for {}", drive))
        }
    }
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn get_free_disk_space(_drive: &str) -> Result<u64> {
    // For non-Windows systems, return a large number to avoid blocking
    Ok(100 * 1024 * 1024 * 1024) // 100GB
}

pub fn check_dependencies() -> Result<Vec<String>> {
    #[cfg_attr(not(target_os = "windows"), allow(unused_mut))]
    let mut missing = Vec::new();

    #[cfg(target_os = "windows")]
    {
        // Check for DISM
        if !command_exists("dism") {
            missing.push("DISM (Windows ADK) - Required for WIM mounting".to_string());
        }
        
        // Check for 7-Zip
        if !command_exists("7z") {
            missing.push("7-Zip - Required for ISO extraction. Install from https://www.7-zip.org/".to_string());
        }
        
        // Check for oscdimg
        let adk_paths = vec![
            "C:\\Program Files (x86)\\Windows Kits\\10\\Assessment and Deployment Kit\\Deployment Tools\\amd64\\Oscdimg\\oscdimg.exe",
            "C:\\Program Files\\Windows Kits\\10\\Assessment and Deployment Kit\\Deployment Tools\\amd64\\Oscdimg\\oscdimg.exe",
        ];
        
        let oscdimg_found = adk_paths.iter().any(|path| Path::new(path).exists());
        if !oscdimg_found {
            missing.push("oscdimg (Windows ADK) - Required for ISO creation. Install Windows ADK from Microsoft".to_string());
        }
        
        // Check for Windows ADK WinPE packages
        let winpe_paths = vec![
            "C:\\Program Files (x86)\\Windows Kits\\10\\Assessment and Deployment Kit\\Windows Preinstallation Environment",
            "C:\\Program Files\\Windows Kits\\10\\Assessment and Deployment Kit\\Windows Preinstallation Environment",
        ];
        
        let winpe_found = winpe_paths.iter().any(|path| Path::new(path).exists());
        if !winpe_found {
            missing.push("Windows ADK WinPE Add-on - Required for WinPE package injection".to_string());
        }
        
        // Check for PowerShell (should be available on all modern Windows)
        if !command_exists("powershell") {
            missing.push("PowerShell - Required for system configuration".to_string());
        }
        
        // Check available disk space (need at least 10GB)
        if let Ok(free_space) = get_free_disk_space("C:") {
            if free_space < 10 * 1024 * 1024 * 1024 { // 10GB in bytes
                missing.push(format!("Insufficient disk space - Need 10GB+, have {}", format_file_size(free_space)));
            }
        }
    }
    
    Ok(missing)
}

#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub fn command_exists(cmd: &str) -> bool {
    std::process::Command::new("where")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn command_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[allow(dead_code)]
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[0])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

#[allow(dead_code)]
pub fn validate_iso_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    
    if !path.exists() {
        return Err(anyhow::anyhow!("ISO file does not exist: {}", path.display()));
    }
    
    if !path.is_file() {
        return Err(anyhow::anyhow!("Path is not a file: {}", path.display()));
    }
    
    // Check file extension
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("iso") => {}
        Some(ext) => return Err(anyhow::anyhow!("Expected .iso file, got .{}", ext)),
        None => return Err(anyhow::anyhow!("File has no extension")),
    }
    
    // Check minimum file size (Windows ISOs are typically > 100MB)
    let metadata = std::fs::metadata(path)?;
    if metadata.len() < 100 * 1024 * 1024 {
        return Err(anyhow::anyhow!("ISO file seems too small: {}", format_file_size(metadata.len())));
    }
    
    Ok(())
}
