use anyhow::Result;
use std::path::Path;
use tracing::{info, warn, error};

pub struct RecoveryManager;

impl RecoveryManager {
    /// Clean up failed build artifacts
    pub async fn cleanup_failed_build(build_dir: &Path) -> Result<()> {
        info!("🧹 Cleaning up failed build artifacts");
        
        // Unmount any mounted WIM files
        Self::force_unmount_all_wims().await?;
        
        // Clean up DISM operations
        Self::cleanup_dism_operations().await?;
        
        // Remove build directory if it exists
        if build_dir.exists() {
            info!("Removing build directory: {}", build_dir.display());
            match std::fs::remove_dir_all(build_dir) {
                Ok(_) => info!("✅ Build directory cleaned"),
                Err(e) => warn!("⚠️ Failed to remove build directory: {}", e),
            }
        }
        
        // Clean temporary files
        Self::cleanup_temp_files().await?;
        
        info!("✅ Cleanup completed");
        Ok(())
    }
    
    /// Force unmount all WIM files
    async fn force_unmount_all_wims() -> Result<()> {
        info!("Unmounting any active WIM images");
        
        // Get list of mounted images
        let output = std::process::Command::new("dism")
            .args(["/Get-MountedImageInfo"])
            .output()?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("No mounted images found") {
                info!("No mounted WIM images found");
                return Ok(());
            }
            
            // Force unmount with discard changes
            let unmount_output = std::process::Command::new("dism")
                .args(["/Unmount-Image", "/MountDir:*", "/Discard"])
                .output();
            
            match unmount_output {
                Ok(result) => {
                    if result.status.success() {
                        info!("✅ Force unmounted WIM images");
                    } else {
                        warn!("⚠️ Failed to unmount some WIM images");
                    }
                }
                Err(e) => warn!("⚠️ Error during WIM unmount: {}", e),
            }
        }
        
        Ok(())
    }
    
    /// Clean up DISM operations and mount points
    async fn cleanup_dism_operations() -> Result<()> {
        info!("Cleaning up DISM operations");
        
        // Clean up WIM files
        let _ = std::process::Command::new("dism")
            .args(["/Cleanup-Wim"])
            .output();
        
        // Clean up mount points
        let _ = std::process::Command::new("dism")
            .args(["/Cleanup-Mountpoints"])
            .output();
        
        info!("✅ DISM operations cleaned");
        Ok(())
    }
    
    /// Clean up temporary files
    async fn cleanup_temp_files() -> Result<()> {
        info!("Cleaning up temporary files");
        
        let temp_patterns = vec![
            "C:\\temp\\ghostwin_*",
            "C:\\temp\\WIMMount*",
            "C:\\temp\\*.reg",
            "C:\\temp\\*.wim",
        ];
        
        for pattern in temp_patterns {
            // Use PowerShell for glob pattern matching
            let _ = std::process::Command::new("powershell")
                .args(["-Command", &format!("Remove-Item '{}' -Force -Recurse -ErrorAction SilentlyContinue", pattern)])
                .output();
        }
        
        info!("✅ Temporary files cleaned");
        Ok(())
    }
    
    /// Create backup of important files before build
    pub async fn create_backup(source_iso: &Path, backup_dir: &Path) -> Result<()> {
        if !source_iso.exists() {
            return Err(anyhow::anyhow!("Source ISO does not exist: {}", source_iso.display()));
        }
        
        std::fs::create_dir_all(backup_dir)?;
        
        info!("📦 Creating backup of source ISO");
        
        let backup_path = backup_dir.join(format!("backup_{}", 
            source_iso.file_name().unwrap().to_string_lossy()));
        
        // Only create backup if it doesn't already exist
        if !backup_path.exists() {
            std::fs::copy(source_iso, &backup_path)?;
            info!("✅ Backup created: {}", backup_path.display());
        } else {
            info!("Backup already exists: {}", backup_path.display());
        }
        
        Ok(())
    }
    
    /// Check system state before build
    pub async fn pre_build_check() -> Result<()> {
        info!("🔍 Pre-build system check");
        
        // Check if any WIM files are mounted
        let output = std::process::Command::new("dism")
            .args(["/Get-MountedImageInfo"])
            .output()?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.contains("No mounted images found") {
                warn!("⚠️ Found mounted WIM images - will clean up");
                Self::force_unmount_all_wims().await?;
            }
        }
        
        // Check available disk space
        if let Ok(free_space) = crate::utils::get_free_disk_space("C:") {
            let gb_free = free_space / (1024 * 1024 * 1024);
            if gb_free < 10 {
                return Err(anyhow::anyhow!("Insufficient disk space: {}GB free, need 10GB+", gb_free));
            }
            info!("✅ Disk space check passed: {}GB available", gb_free);
        }
        
        // Check for running processes that might interfere
        let interferening_processes = vec!["7z.exe", "dism.exe"];
        for process in interferening_processes {
            let output = std::process::Command::new("tasklist")
                .args(["/FI", &format!("IMAGENAME eq {}", process)])
                .output();
            
            if let Ok(result) = output {
                let stdout = String::from_utf8_lossy(&result.stdout);
                if stdout.contains(process) {
                    warn!("⚠️ Found running process that might interfere: {}", process);
                }
            }
        }
        
        info!("✅ Pre-build check completed");
        Ok(())
    }
    
    /// Emergency stop function for build process
    pub async fn emergency_stop() -> Result<()> {
        error!("🚨 Emergency stop initiated");
        
        // Kill any running DISM processes
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/IM", "dism.exe"])
            .output();
        
        // Kill any running 7z processes
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/IM", "7z.exe"])
            .output();
        
        // Force unmount everything
        Self::force_unmount_all_wims().await?;
        
        // Clean up DISM
        Self::cleanup_dism_operations().await?;
        
        error!("🚨 Emergency stop completed");
        Ok(())
    }
}