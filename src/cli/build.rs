use anyhow::{Result, Context, bail};
use std::path::Path;
use tracing::{info, warn, debug};
use crate::cli::{BuildArgs, GhostwinConfig};
use crate::wim::WimManager;
use crate::config::ConfigManager;
use crate::tools::ToolDetector;

pub async fn execute(args: BuildArgs) -> Result<()> {
    info!("Starting GhostWin ISO build process");
    
    // Load configuration
    let config = if let Some(config_path) = &args.config {
        ConfigManager::load_from_file(config_path)?
    } else {
        ConfigManager::load_default()?
    };
    
    // Validate inputs
    validate_inputs(&args, &config)?;
    
    // Initialize WIM manager
    let mut wim_manager = WimManager::new(&config)?;
    
    // Execute build steps
    info!("Step 1: Extracting source ISO");
    extract_iso(&args.source_iso, &args.output_dir).await?;
    
    info!("Step 2: Mounting WIM image");
    let wim_path = Path::new(&args.output_dir).join("sources/boot.wim");
    wim_manager.mount(&wim_path, &config.iso.wim_index).await?;
    
    info!("Step 3: Copying helper files");
    copy_helper_files(&wim_manager, &config).await?;
    
    if let Some(extra_files) = &args.extra_files {
        info!("Step 4: Copying extra files");
        copy_extra_files(&wim_manager, extra_files).await?;
    }
    
    if !args.skip_packages {
        info!("Step 5: Adding WinPE packages");
        add_winpe_packages(&wim_manager, &config).await?;
    }
    
    if !args.skip_dpi_fix && config.winpe.disable_dpi_scaling {
        info!("Step 6: Applying DPI fix");
        apply_dpi_fix(&wim_manager).await?;
    }
    
    info!("Step 7: Unmounting and committing WIM");
    wim_manager.unmount_and_commit().await?;
    
    info!("Step 8: Creating final ISO");
    create_iso(&args.output_dir, &args.output_iso).await?;
    
    info!("✅ GhostWin ISO build completed successfully!");
    info!("Output: {}", args.output_iso);
    
    Ok(())
}

fn validate_inputs(args: &BuildArgs, _config: &GhostwinConfig) -> Result<()> {
    if !Path::new(&args.source_iso).exists() {
        bail!("Source ISO not found: {}", args.source_iso);
    }
    
    if Path::new(&args.output_iso).exists() {
        warn!("Output ISO already exists and will be overwritten: {}", args.output_iso);
    }
    
    // Validate parent directory exists
    if let Some(parent) = Path::new(&args.output_iso).parent() {
        if !parent.exists() {
            bail!("Output directory does not exist: {}", parent.display());
        }
    }
    
    Ok(())
}

async fn extract_iso(source_iso: &str, output_dir: &str) -> Result<()> {
    debug!("Extracting ISO {} to {}", source_iso, output_dir);
    
    #[cfg(target_os = "windows")]
    {
        // Use 7-Zip on Windows
        let status = tokio::process::Command::new("7z")
            .args(&["x", source_iso, &format!("-o{}", output_dir), "-y"])
            .status()
            .await
            .context("Failed to run 7z command")?;
        
        if !status.success() {
            bail!("7-Zip extraction failed");
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // Use 7z on Linux/macOS
        let status = tokio::process::Command::new("mkdir")
            .args(&["-p", output_dir])
            .status()
            .await?;
        
        if !status.success() {
            bail!("Failed to create output directory");
        }
        
        let status = tokio::process::Command::new("7z")
            .args(&["x", source_iso, &format!("-o{}", output_dir), "-y"])
            .status()
            .await
            .context("Failed to run 7z command")?;
        
        if !status.success() {
            bail!("7-Zip extraction failed");
        }
    }
    
    Ok(())
}

async fn copy_helper_files(wim_manager: &WimManager, config: &GhostwinConfig) -> Result<()> {
    let helper_source = Path::new("concept/windows-setup-helper-master/Helper");
    let windows_source = Path::new("concept/windows-setup-helper-master/Windows");
    
    if helper_source.exists() {
        wim_manager.copy_to_mount(helper_source, "Helper").await?;
    }
    
    if windows_source.exists() {
        wim_manager.copy_to_mount(windows_source, "Windows").await?;
    }
    
    // Detect and copy tool folders
    let tool_detector = ToolDetector::new(&config.tools);
    let detected_tools = tool_detector.scan_tools()?;
    
    info!("Detected {} tool directories", detected_tools.len());
    for tool_dir in detected_tools {
        debug!("Copying tool directory: {}", tool_dir.display());
        wim_manager.copy_to_mount(&tool_dir, "Helper/Tools").await?;
    }
    
    Ok(())
}

async fn copy_extra_files(wim_manager: &WimManager, extra_files: &str) -> Result<()> {
    let extra_path = Path::new(extra_files);
    if !extra_path.exists() {
        warn!("Extra files directory not found: {}", extra_files);
        return Ok(());
    }
    
    wim_manager.copy_to_mount(extra_path, "").await?;
    Ok(())
}

async fn add_winpe_packages(wim_manager: &WimManager, config: &GhostwinConfig) -> Result<()> {
    for package in &config.winpe.packages {
        info!("Adding WinPE package: {}", package);
        wim_manager.add_package(package).await?;
    }
    Ok(())
}

async fn apply_dpi_fix(wim_manager: &WimManager) -> Result<()> {
    info!("Applying DPI scaling fix");
    wim_manager.apply_registry_fix("dpi_scaling").await?;
    Ok(())
}

async fn create_iso(media_path: &str, output_iso: &str) -> Result<()> {
    debug!("Creating ISO from {} to {}", media_path, output_iso);
    
    #[cfg(target_os = "windows")]
    {
        // Use oscdimg from Windows ADK
        let adk_path = std::env::var("ProgramFiles(x86)")
            .unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());
        let oscdimg_path = format!("{}\\Windows Kits\\10\\Assessment and Deployment Kit\\Deployment Tools\\amd64\\Oscdimg\\oscdimg.exe", adk_path);
        
        let status = tokio::process::Command::new(&oscdimg_path)
            .args(&[
                "-m", "-o", "-u2", "-udfver102",
                "-bootdata:2#p0,e,b\"boot\\etfsboot.com\"#pEF,e,b\"efi\\microsoft\\boot\\efisys.bin\"",
                media_path,
                output_iso
            ])
            .status()
            .await
            .context("Failed to run oscdimg command")?;
        
        if !status.success() {
            bail!("oscdimg ISO creation failed");
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        bail!("ISO creation not implemented for this platform");
    }
    
    Ok(())
}