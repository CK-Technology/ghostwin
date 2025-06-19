use anyhow::{Result, Context, bail};
use std::path::{Path, PathBuf};
use tracing::{info, debug, error};
use tempfile::TempDir;
use crate::cli::GhostwinConfig;

pub struct WimManager {
    mount_path: PathBuf,
    temp_dir: Option<TempDir>,
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
        
        self.is_mounted = true;
        Ok(())
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
                    "/Image:",
                    &self.mount_path.to_string_lossy(),
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
                    let reg_content = r#"Windows Registry Editor Version 5.00

[HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\SideBySide]
"PreferExternalManifest"=dword:00000001

[HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\GraphicsDrivers\Configuration]
"DisableScalingOptimizations"=dword:00000001
"#;
                    
                    let reg_file = self.mount_path.join("dpi_fix.reg");
                    tokio::fs::write(&reg_file, reg_content).await?;
                    
                    let status = tokio::process::Command::new("reg")
                        .args(&[
                            "load",
                            "HKLM\\WIM_SOFTWARE",
                            &format!("{}\\Windows\\System32\\config\\SOFTWARE", self.mount_path.display()),
                        ])
                        .status()
                        .await?;
                    
                    if status.success() {
                        let _ = tokio::process::Command::new("reg")
                            .args(&["import", &reg_file.to_string_lossy()])
                            .status()
                            .await;
                        
                        let _ = tokio::process::Command::new("reg")
                            .args(&["unload", "HKLM\\WIM_SOFTWARE"])
                            .status()
                            .await;
                    }
                    
                    tokio::fs::remove_file(&reg_file).await?;
                }
            }
            _ => bail!("Unknown registry fix type: {}", fix_type),
        }
        
        Ok(())
    }
    
    pub fn mount_path(&self) -> &Path {
        &self.mount_path
    }
    
    pub fn is_mounted(&self) -> bool {
        self.is_mounted
    }
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