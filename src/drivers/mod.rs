use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use tracing::{info, debug, warn};
use crate::wim::WimManager;

/// Driver injection manager for WinPE environments
/// Supports Intel VMD/RapidStorage, Dell Optiplex, and modern NVMe drivers
pub struct DriverManager {
    driver_paths: Vec<PathBuf>,
    priority_drivers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DetectedDriver {
    pub name: String,
    pub path: PathBuf,
    pub driver_type: DriverType,
    pub inf_file: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DriverType {
    Inf,         // .inf driver package
    Cab,         // .cab driver package
    Sys,         // .sys kernel driver
    #[allow(dead_code)]
    Unknown,
}

impl DriverManager {
    pub fn new() -> Self {
        // Priority drivers for Dell Optiplex and modern NVMe support
        // Includes support for 15th Gen Intel (Arrow Lake) and latest hardware
        let priority_drivers = vec![
            // Intel VMD/RapidStorage drivers (critical for Dell Optiplex)
            "iastorac".to_string(),        // Intel Rapid Storage Technology
            "iastorv".to_string(),         // Intel VMD Controller
            "iaStorAC".to_string(),        // Intel Rapid Storage Technology AHCI
            "iaStorAVC".to_string(),       // Intel Rapid Storage Technology VMD (15th gen support)
            "iastorav".to_string(),        // Intel Rapid Storage Technology AV
            "iastore".to_string(),         // Intel Storage Enhanced
            "vmd".to_string(),             // VMD Controller
            "vroc".to_string(),            // Intel VROC (Virtual RAID on CPU)

            // Modern NVMe drivers
            "stornvme".to_string(),        // Windows Standard NVMe Driver
            "nvme".to_string(),            // Generic NVMe

            // Micron NVMe specific (2200/2300/3400 series - Dell 15th gen common)
            "micron".to_string(),          // Micron NVMe drivers
            "mtfd".to_string(),            // Micron storage drivers
            "mtfd3400".to_string(),        // Micron 3400 NVMe (latest, common in Dell 15th gen)
            "mtfd2300".to_string(),        // Micron 2300 NVMe
            "mtfd7450".to_string(),        // Micron 7450 PRO NVMe

            // Samsung NVMe specific
            "samsung".to_string(),         // Samsung NVMe drivers
            "nvmexpresssam".to_string(),   // Samsung NVMe Express
            "samclass".to_string(),        // Samsung Class Driver

            // Dell specific storage drivers
            "dell".to_string(),            // Dell storage controllers
            "bossstornvme".to_string(),    // Dell BOSS-S1 Controller

            // Common storage controllers
            "storahci".to_string(),        // AHCI Storage Controller
            "msahci".to_string(),          // Microsoft AHCI
        ];

        Self {
            driver_paths: Vec::new(),
            priority_drivers,
        }
    }

    /// Check if driver is a priority driver (NVMe/VMD/RapidStorage)
    fn is_priority_driver(&self, driver_name: &str) -> bool {
        let name_lower = driver_name.to_lowercase();
        self.priority_drivers.iter().any(|priority| {
            name_lower.contains(&priority.to_lowercase())
        })
    }

    /// Scan for driver directories in standard locations
    pub fn scan_driver_directories(&mut self) -> Result<Vec<PathBuf>> {
        info!("🔍 Scanning for driver directories");

        let search_patterns = vec![
            "PEAutoRun/Drivers",
            "pe_autorun/drivers",
            "Tools/Drivers",
            "tools/drivers",
            "Drivers",
            "drivers",
        ];

        for pattern in search_patterns {
            let path = PathBuf::from(pattern);
            if path.exists() && path.is_dir() {
                info!("Found driver directory: {}", path.display());
                self.driver_paths.push(path);
            }
        }

        // Also scan all drives on Windows
        #[cfg(target_os = "windows")]
        {
            self.scan_all_drives_for_drivers()?;
        }

        Ok(self.driver_paths.clone())
    }

    /// Scan all drives for driver directories
    #[cfg(target_os = "windows")]
    fn scan_all_drives_for_drivers(&mut self) -> Result<()> {
        use winapi::um::fileapi::GetLogicalDrives;

        let drives = unsafe { GetLogicalDrives() };

        for i in 0..26 {
            if (drives >> i) & 1 != 0 {
                let drive_letter = (b'A' + i) as char;
                let patterns = vec![
                    format!("{}:\\PEAutoRun\\Drivers", drive_letter),
                    format!("{}:\\Helper\\Drivers", drive_letter),
                    format!("{}:\\Tools\\Drivers", drive_letter),
                ];

                for pattern in patterns {
                    let path = PathBuf::from(&pattern);
                    if path.exists() && path.is_dir() {
                        if !self.driver_paths.contains(&path) {
                            info!("Found driver directory on drive: {}", path.display());
                            self.driver_paths.push(path);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Detect all drivers in configured directories
    /// Prioritizes Intel VMD/RapidStorage and NVMe drivers
    pub fn detect_drivers(&self) -> Result<Vec<DetectedDriver>> {
        info!("🔍 Detecting drivers in {} directories", self.driver_paths.len());
        let mut drivers = Vec::new();
        let mut priority_drivers = Vec::new();

        for driver_dir in &self.driver_paths {
            debug!("Scanning directory: {}", driver_dir.display());

            for entry in WalkDir::new(driver_dir).max_depth(5) {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    if let Some(driver) = self.classify_driver(path)? {
                        let is_priority = self.is_priority_driver(&driver.name);

                        if is_priority {
                            info!("🔥 PRIORITY driver detected: {} ({})", driver.name, driver.path.display());
                            priority_drivers.push(driver);
                        } else {
                            info!("Detected driver: {} ({})", driver.name, driver.path.display());
                            drivers.push(driver);
                        }
                    }
                }
            }
        }

        // Priority drivers go first (critical for Dell Optiplex boot)
        let priority_count = priority_drivers.len();
        let total_count = priority_count + drivers.len();
        priority_drivers.extend(drivers);

        info!("✅ Detected {} drivers ({} priority storage drivers)",
              total_count,
              priority_count);

        Ok(priority_drivers)
    }

    /// Classify a file as a driver
    fn classify_driver(&self, path: &Path) -> Result<Option<DetectedDriver>> {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let driver_type = match extension.as_str() {
            "inf" => DriverType::Inf,
            "cab" => DriverType::Cab,
            "sys" => DriverType::Sys,
            _ => return Ok(None),
        };

        let name = path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // For .sys files, look for associated .inf file
        let inf_file = if driver_type == DriverType::Sys {
            self.find_inf_for_sys(path)?
        } else if driver_type == DriverType::Inf {
            Some(path.to_path_buf())
        } else {
            None
        };

        Ok(Some(DetectedDriver {
            name,
            path: path.to_path_buf(),
            driver_type,
            inf_file,
        }))
    }

    /// Find .inf file associated with a .sys driver
    fn find_inf_for_sys(&self, sys_path: &Path) -> Result<Option<PathBuf>> {
        if let Some(parent) = sys_path.parent() {
            for entry in std::fs::read_dir(parent)? {
                let entry = entry?;
                let path = entry.path();

                if path.extension().and_then(|ext| ext.to_str()) == Some("inf") {
                    return Ok(Some(path));
                }
            }
        }

        Ok(None)
    }

    /// Inject drivers into mounted WIM image
    pub async fn inject_drivers_to_wim(
        &self,
        wim_manager: &WimManager,
        drivers: &[DetectedDriver],
    ) -> Result<()> {
        if drivers.is_empty() {
            info!("No drivers to inject");
            return Ok(());
        }

        info!("💉 Injecting {} drivers into WIM", drivers.len());

        for driver in drivers {
            match driver.driver_type {
                DriverType::Inf => {
                    self.inject_inf_driver(wim_manager, driver).await?;
                }
                DriverType::Cab => {
                    self.inject_cab_driver(wim_manager, driver).await?;
                }
                DriverType::Sys => {
                    if let Some(ref inf_file) = driver.inf_file {
                        info!("Injecting .sys driver via INF: {}", inf_file.display());
                        let inf_driver = DetectedDriver {
                            name: driver.name.clone(),
                            path: inf_file.clone(),
                            driver_type: DriverType::Inf,
                            inf_file: Some(inf_file.clone()),
                        };
                        self.inject_inf_driver(wim_manager, &inf_driver).await?;
                    } else {
                        warn!("Skipping .sys driver without .inf file: {}", driver.path.display());
                    }
                }
                DriverType::Unknown => {
                    warn!("Unknown driver type, skipping: {}", driver.path.display());
                }
            }
        }

        info!("✅ Driver injection completed");
        Ok(())
    }

    /// Inject a .inf driver package
    async fn inject_inf_driver(
        &self,
        #[allow(unused_variables)] wim_manager: &WimManager,
        driver: &DetectedDriver,
    ) -> Result<()> {
        info!("Injecting INF driver: {}", driver.name);

        #[cfg(target_os = "windows")]
        {
            let mount_path = wim_manager.mount_path();

            let output = tokio::process::Command::new("dism")
                .args([
                    &format!("/Image:{}", mount_path.display()),
                    "/Add-Driver",
                    &format!("/Driver:{}", driver.path.display()),
                    "/ForceUnsigned",
                ])
                .output()
                .await
                .context("Failed to run DISM add-driver command")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("Driver injection warning for {}: {}", driver.name, stderr);
                // Don't fail the whole process for one driver
            } else {
                info!("✅ Injected INF driver: {}", driver.name);
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            info!("Would inject INF driver: {} (not on Windows)", driver.name);
        }

        Ok(())
    }

    /// Inject a .cab driver package
    async fn inject_cab_driver(
        &self,
        #[allow(unused_variables)] wim_manager: &WimManager,
        driver: &DetectedDriver,
    ) -> Result<()> {
        info!("Injecting CAB driver: {}", driver.name);

        #[cfg(target_os = "windows")]
        {
            // First extract the CAB to a temp location
            let temp_dir = std::env::temp_dir().join(format!("ghostwin_driver_{}", driver.name));
            std::fs::create_dir_all(&temp_dir)?;

            // Extract CAB
            let extract_output = tokio::process::Command::new("expand")
                .args([
                    driver.path.to_str().unwrap(),
                    "-F:*",
                    temp_dir.to_str().unwrap(),
                ])
                .output()
                .await?;

            if !extract_output.status.success() {
                bail!("Failed to extract CAB driver: {}", driver.name);
            }

            // Find .inf files in extracted directory
            let mut found_inf = false;
            for entry in WalkDir::new(&temp_dir).max_depth(2) {
                let entry = entry?;
                if entry.path().extension().and_then(|ext| ext.to_str()) == Some("inf") {
                    found_inf = true;
                    let inf_driver = DetectedDriver {
                        name: entry.file_name().to_string_lossy().to_string(),
                        path: entry.path().to_path_buf(),
                        driver_type: DriverType::Inf,
                        inf_file: Some(entry.path().to_path_buf()),
                    };
                    self.inject_inf_driver(wim_manager, &inf_driver).await?;
                }
            }

            if !found_inf {
                warn!("No .inf files found in CAB: {}", driver.name);
            }

            // Cleanup
            let _ = std::fs::remove_dir_all(&temp_dir);
        }

        #[cfg(not(target_os = "windows"))]
        {
            info!("Would inject CAB driver: {} (not on Windows)", driver.name);
        }

        Ok(())
    }

    /// Copy drivers to WIM for manual installation during WinPE boot
    pub async fn copy_drivers_to_wim(
        &self,
        wim_manager: &WimManager,
        drivers: &[DetectedDriver],
    ) -> Result<()> {
        info!("📋 Copying {} drivers to WIM for manual installation", drivers.len());

        // Group drivers by their parent directory
        use std::collections::HashMap;
        let mut drivers_by_dir: HashMap<PathBuf, Vec<&DetectedDriver>> = HashMap::new();

        for driver in drivers {
            if let Some(parent) = driver.path.parent() {
                drivers_by_dir.entry(parent.to_path_buf())
                    .or_default()
                    .push(driver);
            }
        }

        // Copy each driver directory to the WIM
        for (driver_dir, dir_drivers) in drivers_by_dir {
            info!("Copying driver directory: {}", driver_dir.display());
            wim_manager.copy_to_mount(&driver_dir, "Windows/System32/Drivers").await?;
            info!("✅ Copied {} drivers from {}", dir_drivers.len(), driver_dir.display());
        }

        Ok(())
    }
}

impl Default for DriverManager {
    fn default() -> Self {
        Self::new()
    }
}
