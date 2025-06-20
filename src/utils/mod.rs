use anyhow::Result;
use std::path::Path;

pub mod recovery;

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
pub fn get_free_disk_space(drive: &str) -> Result<u64> {
    use winapi::um::fileapi::GetDiskFreeSpaceExW;
    use winapi::um::winnt::ULARGE_INTEGER;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    
    let wide_path: Vec<u16> = OsStr::new(drive).encode_wide().chain(std::iter::once(0)).collect();
    
    unsafe {
        let mut free_bytes = ULARGE_INTEGER { QuadPart: 0 };
        let result = GetDiskFreeSpaceExW(
            wide_path.as_ptr(),
            &mut free_bytes,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        
        if result != 0 {
            Ok(free_bytes.QuadPart as u64)
        } else {
            Err(anyhow::anyhow!("Failed to get disk space for {}", drive))
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_free_disk_space(_drive: &str) -> Result<u64> {
    // For non-Windows systems, return a large number to avoid blocking
    Ok(100 * 1024 * 1024 * 1024) // 100GB
}

pub fn check_dependencies() -> Result<Vec<String>> {
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
fn command_exists(cmd: &str) -> bool {
    std::process::Command::new("where")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(not(target_os = "windows"))]
fn command_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

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