use anyhow::{Result, Context, bail};
use std::path::{Path, PathBuf};
use tracing::{info, debug, error};
use tempfile::TempDir;
use crate::cli::GhostwinConfig;

pub struct WimManager {
    mount_path: PathBuf,
    #[allow(dead_code)]
    temp_dir: Option<TempDir>,
    #[allow(dead_code)]
    config: GhostwinConfig,
    is_mounted: bool,
}

impl WimManager {
    pub fn new(config: &GhostwinConfig) -> Result<Self> {
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
        let mount_path = if let Some(ref path) = config.iso.mount_path {
            PathBuf::from(path)
        } else {
            temp_dir.path().join("WIMMount")
        };
        
        // Create mount directory
        std::fs::create_dir_all(&mount_path)?;
        
        Ok(Self {
            mount_path,
            temp_dir: Some(temp_dir),
            config: config.clone(),
            is_mounted: false,
        })
    }
    
    pub async fn mount(&mut self, wim_path: &Path, index: &str) -> Result<()> {
        if self.is_mounted {
            bail!("WIM is already mounted");
        }
        
        info!("Mounting WIM: {} (index: {})", wim_path.display(), index);
        
        #[cfg(target_os = "windows")]
        {
            let status = tokio::process::Command::new("dism")
                .args(&[
                    "/Mount-Wim",
                    &format!("/WimFile:{}", wim_path.display()),
                    &format!("/Index:{}", index),
                    &format!("/MountDir:{}", self.mount_path.display()),
                ])
                .status()
                .await
                .context("Failed to run DISM mount command")?;
            
            if !status.success() {
                bail!("DISM mount failed");
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            bail!("WIM mounting is only supported on Windows");
        }

        #[cfg(target_os = "windows")]
        {
            self.is_mounted = true;
            Ok(())
        }
    }
    
    pub async fn unmount_and_commit(&mut self) -> Result<()> {
        if !self.is_mounted {
            return Ok(());
        }
        
        info!("Unmounting and committing WIM changes");
        
        #[cfg(target_os = "windows")]
        {
            let status = tokio::process::Command::new("dism")
                .args(&[
                    "/Unmount-Wim",
                    &format!("/MountDir:{}", self.mount_path.display()),
                    "/Commit",
                ])
                .status()
                .await
                .context("Failed to run DISM unmount command")?;
            
            if !status.success() {
                bail!("DISM unmount failed");
            }
        }
        
        self.is_mounted = false;
        Ok(())
    }

    pub async fn unmount_and_discard(&mut self) -> Result<()> {
        if !self.is_mounted {
            return Ok(());
        }

        info!("Unmounting and discarding WIM changes");

        #[cfg(target_os = "windows")]
        {
            let status = tokio::process::Command::new("dism")
                .args(&[
                    "/Unmount-Wim",
                    &format!("/MountDir:{}", self.mount_path.display()),
                    "/Discard",
                ])
                .status()
                .await
                .context("Failed to run DISM discard unmount command")?;

            if !status.success() {
                bail!("DISM discard unmount failed");
            }
        }

        self.is_mounted = false;
        Ok(())
    }
    
    pub async fn copy_to_mount(&self, source: &Path, destination: &str) -> Result<()> {
        if !self.is_mounted {
            bail!("WIM is not mounted");
        }
        
        let dest_path = if destination.is_empty() {
            self.mount_path.clone()
        } else {
            self.mount_path.join(destination)
        };
        
        debug!("Copying {} to {}", source.display(), dest_path.display());
        
        if source.is_dir() {
            self.copy_dir_recursive(source, &dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(source, &dest_path)?;
        }
        
        Ok(())
    }
    
    fn copy_dir_recursive(&self, source: &Path, destination: &Path) -> Result<()> {
        if !destination.exists() {
            std::fs::create_dir_all(destination)?;
        }
        
        for entry in std::fs::read_dir(source)? {
            let entry = entry?;
            let source_path = entry.path();
            let dest_path = destination.join(entry.file_name());
            
            if source_path.is_dir() {
                self.copy_dir_recursive(&source_path, &dest_path)?;
            } else {
                std::fs::copy(&source_path, &dest_path)?;
            }
        }
        
        Ok(())
    }
    
    pub async fn add_package(&self, package: &str) -> Result<()> {
        if !self.is_mounted {
            bail!("WIM is not mounted");
        }
        
        debug!("Adding WinPE package: {}", package);
        
        #[cfg(target_os = "windows")]
        {
            let adk_path = if let Some(ref path) = self.config.iso.adk_path {
                PathBuf::from(path)
            } else {
                let program_files = std::env::var("ProgramFiles(x86)")
                    .unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());
                PathBuf::from(format!("{}\\Windows Kits\\10\\Assessment and Deployment Kit", program_files))
            };
            
            let package_path = adk_path
                .join("Windows Preinstallation Environment")
                .join("amd64")
                .join("WinPE_OCs")
                .join(format!("{}.cab", package));
            
            if !package_path.exists() {
                bail!("WinPE package not found: {}", package_path.display());
            }
            
            let status = tokio::process::Command::new("dism")
                .args(&[
                    &format!("/Image:{}", self.mount_path.display()),
                    "/Add-Package",
                    &format!("/PackagePath:{}", package_path.display()),
                ])
                .status()
                .await
                .context("Failed to add WinPE package")?;
            
            if !status.success() {
                bail!("Failed to add WinPE package: {}", package);
            }
        }
        
        Ok(())
    }
    
    pub async fn apply_registry_fix(&self, fix_type: &str) -> Result<()> {
        if !self.is_mounted {
            bail!("WIM is not mounted");
        }
        
        match fix_type {
            "dpi_scaling" => {
                debug!("Applying DPI scaling fix");
                
                #[cfg(target_os = "windows")]
                {
                    let hives = Self::offline_hive_paths(&self.mount_path);
                    let mut loaded_hives = Vec::new();

                    Self::load_hive(Self::WIM_SOFTWARE_HIVE, &hives.software).await?;
                    loaded_hives.push(Self::WIM_SOFTWARE_HIVE);

                    Self::load_hive(Self::WIM_SYSTEM_HIVE, &hives.system).await?;
                    loaded_hives.push(Self::WIM_SYSTEM_HIVE);

                    let result: Result<()> = async {
                        Self::add_registry_dword(
                            &Self::side_by_side_key(),
                            "PreferExternalManifest",
                            1,
                        )
                        .await?;

                        let current_control_set = Self::query_current_control_set().await?;
                        let graphics_key = Self::graphics_drivers_key(&current_control_set);

                        Self::add_registry_dword(
                            &graphics_key,
                            "DisableScalingOptimizations",
                            1,
                        )
                        .await?;

                        Ok(())
                    }
                    .await;

                    let unload_result = Self::unload_loaded_hives(&loaded_hives).await;

                    result?;
                    unload_result?;
                }
            }
            _ => bail!("Unknown registry fix type: {}", fix_type),
        }
        
        Ok(())
    }
    
    #[allow(dead_code)]
    pub fn mount_path(&self) -> &Path {
        &self.mount_path
    }

    #[allow(dead_code)]
    pub fn is_mounted(&self) -> bool {
        self.is_mounted
    }

    #[cfg_attr(not(test), allow(dead_code))]
    const WIM_SOFTWARE_HIVE: &'static str = r"HKLM\WIM_SOFTWARE";
    #[cfg_attr(not(test), allow(dead_code))]
    const WIM_SYSTEM_HIVE: &'static str = r"HKLM\WIM_SYSTEM";

    #[cfg_attr(not(test), allow(dead_code))]
    fn offline_hive_paths(mount_path: &Path) -> OfflineHives {
        OfflineHives {
            software: mount_path.join("Windows/System32/config/SOFTWARE"),
            system: mount_path.join("Windows/System32/config/SYSTEM"),
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn side_by_side_key() -> String {
        format!(
            r"{}\Microsoft\Windows\CurrentVersion\SideBySide",
            Self::WIM_SOFTWARE_HIVE
        )
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn graphics_drivers_key(control_set: &str) -> String {
        format!(
            r"{}\{}\Control\GraphicsDrivers\Configuration",
            Self::WIM_SYSTEM_HIVE,
            control_set
        )
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn format_control_set(value: u32) -> String {
        format!("ControlSet{:03}", value)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn parse_current_control_set_output(stdout: &str) -> Result<String> {
        let current_value = stdout
            .lines()
            .find_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 && parts[0] == "Current" && parts[1] == "REG_DWORD" {
                    parts.last().copied()
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow::anyhow!("Unable to parse offline SYSTEM control set"))?;

        let value = if let Some(hex) = current_value.strip_prefix("0x") {
            u32::from_str_radix(hex, 16)
        } else {
            current_value.parse::<u32>()
        }
        .context("Failed to parse offline SYSTEM control set value")?;

        Ok(Self::format_control_set(value))
    }

    #[cfg(target_os = "windows")]
    async fn unload_loaded_hives(hives: &[&str]) -> Result<()> {
        let mut unload_error = None;

        for hive_name in hives.iter().rev() {
            if let Err(error) = Self::unload_hive(hive_name).await {
                error!("Failed to unload offline hive {}: {}", hive_name, error);
                if unload_error.is_none() {
                    unload_error = Some(error);
                }
            }
        }

        if let Some(error) = unload_error {
            return Err(error);
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn load_hive(hive_name: &str, hive_path: &Path) -> Result<()> {
        Self::run_reg_command(Self::load_hive_args(hive_name, hive_path)).await
    }

    #[cfg(target_os = "windows")]
    async fn unload_hive(hive_name: &str) -> Result<()> {
        Self::run_reg_command(Self::unload_hive_args(hive_name)).await
    }

    #[cfg(target_os = "windows")]
    async fn add_registry_dword(key: &str, value_name: &str, value: u32) -> Result<()> {
        Self::run_reg_command(Self::add_registry_dword_args(key, value_name, value)).await
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn load_hive_args(hive_name: &str, hive_path: &Path) -> Vec<String> {
        vec![
            "load".to_string(),
            hive_name.to_string(),
            hive_path.to_string_lossy().into_owned(),
        ]
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn unload_hive_args(hive_name: &str) -> Vec<String> {
        vec!["unload".to_string(), hive_name.to_string()]
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn add_registry_dword_args(key: &str, value_name: &str, value: u32) -> Vec<String> {
        vec![
            "add".to_string(),
            key.to_string(),
            "/v".to_string(),
            value_name.to_string(),
            "/t".to_string(),
            "REG_DWORD".to_string(),
            "/d".to_string(),
            value.to_string(),
            "/f".to_string(),
        ]
    }

    #[cfg(target_os = "windows")]
    async fn query_current_control_set() -> Result<String> {
        let output = tokio::process::Command::new("reg")
            .args(["query", r"HKLM\WIM_SYSTEM\Select", "/v", "Current"])
            .output()
            .await
            .context("Failed to query offline SYSTEM control set")?;

        if !output.status.success() {
            bail!(
                "Failed to query offline SYSTEM control set: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Self::parse_current_control_set_output(&stdout)
    }

    #[cfg(target_os = "windows")]
    async fn run_reg_command(args: Vec<String>) -> Result<()> {
        let output = tokio::process::Command::new("reg")
            .args(args.iter().map(String::as_str))
            .output()
            .await
            .context("Failed to run reg command")?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let details = stderr.trim();
        let details = if details.is_empty() { stdout.trim() } else { details };

        bail!("reg command failed: {}", details)
    }
}

#[cfg_attr(not(test), allow(dead_code))]
struct OfflineHives {
    software: PathBuf,
    system: PathBuf,
}

impl Drop for WimManager {
    fn drop(&mut self) {
        if self.is_mounted {
            error!("WIM was not properly unmounted!");
            // Try to unmount synchronously
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("dism")
                    .args(&[
                        "/Unmount-Wim",
                        &format!("/MountDir:{}", self.mount_path.display()),
                        "/Discard",
                    ])
                    .status();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WimManager;
    use std::path::Path;

    #[test]
    fn builds_offline_hive_paths_from_mount_root() {
        let hives = WimManager::offline_hive_paths(Path::new("/mnt/wim"));
        assert_eq!(hives.software, Path::new("/mnt/wim/Windows/System32/config/SOFTWARE"));
        assert_eq!(hives.system, Path::new("/mnt/wim/Windows/System32/config/SYSTEM"));
    }

    #[test]
    fn builds_registry_command_arguments() {
        let load_args = WimManager::load_hive_args("HKLM\\WIM_SOFTWARE", Path::new("C:/mount/SOFTWARE"));
        assert_eq!(load_args, vec!["load", "HKLM\\WIM_SOFTWARE", "C:/mount/SOFTWARE"]);

        let unload_args = WimManager::unload_hive_args("HKLM\\WIM_SYSTEM");
        assert_eq!(unload_args, vec!["unload", "HKLM\\WIM_SYSTEM"]);

        let add_args = WimManager::add_registry_dword_args(
            r"HKLM\WIM_SYSTEM\ControlSet001\Control\GraphicsDrivers\Configuration",
            "DisableScalingOptimizations",
            1,
        );
        assert_eq!(
            add_args,
            vec![
                "add",
                r"HKLM\WIM_SYSTEM\ControlSet001\Control\GraphicsDrivers\Configuration",
                "/v",
                "DisableScalingOptimizations",
                "/t",
                "REG_DWORD",
                "/d",
                "1",
                "/f",
            ]
        );
    }

    #[test]
    fn formats_registry_keys_consistently() {
        assert_eq!(
            WimManager::side_by_side_key(),
            r"HKLM\WIM_SOFTWARE\Microsoft\Windows\CurrentVersion\SideBySide"
        );
        assert_eq!(
            WimManager::graphics_drivers_key("ControlSet007"),
            r"HKLM\WIM_SYSTEM\ControlSet007\Control\GraphicsDrivers\Configuration"
        );
    }

    #[test]
    fn parses_current_control_set_from_decimal_and_hex() {
        let decimal = "HKEY_LOCAL_MACHINE\\WIM_SYSTEM\\Select\n    Current    REG_DWORD    0x1\n";
        let hex = "HKEY_LOCAL_MACHINE\\WIM_SYSTEM\\Select\n    Current    REG_DWORD    2\n";

        assert_eq!(
            WimManager::parse_current_control_set_output(decimal).unwrap(),
            "ControlSet001"
        );
        assert_eq!(
            WimManager::parse_current_control_set_output(hex).unwrap(),
            "ControlSet002"
        );
    }

    #[test]
    fn rejects_invalid_control_set_output() {
        let error = WimManager::parse_current_control_set_output("missing Current value").unwrap_err();
        assert!(error.to_string().contains("Unable to parse offline SYSTEM control set"));
    }
}
