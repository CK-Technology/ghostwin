use anyhow::{Result, Context, bail};
use std::path::{Path, PathBuf};
use tracing::{info, warn, debug};
use crate::cli::{BuildArgs, GhostwinConfig};
use crate::wim::WimManager;
use crate::config::ConfigManager;
use crate::tools::ToolDetector;
use crate::drivers::DriverManager;
use crate::utils;
use crate::utils::recovery::RecoveryManager;

const STEP_EXTRACT: &str = "Step 1: Extracting source ISO";
const STEP_MOUNT: &str = "Step 2: Mounting WIM image";
const STEP_HELPERS: &str = "Step 3: Copying helper files";
const STEP_EXTRA: &str = "Step 4: Copying extra files";
const STEP_PACKAGES: &str = "Step 5: Adding WinPE packages";
const STEP_DRIVERS: &str = "Step 6: Detecting and injecting drivers";
const STEP_DPI_FIX: &str = "Step 7: Applying DPI fix";
const STEP_UNMOUNT: &str = "Step 8: Unmounting and committing WIM";
const STEP_CREATE_ISO: &str = "Step 9: Creating final ISO";
const STEP_VERIFY: &str = "Step 10: Verifying ISO integrity";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BuildProgressState {
    pub current_step: &'static str,
    pub completed_steps: usize,
    pub total_steps: usize,
}

type BuildProgressCallback = dyn Fn(&BuildProgressState) + Send + Sync;

pub async fn execute(args: BuildArgs) -> Result<()> {
    info!("Starting GhostWin ISO build process");
    
    // Load configuration
    let config = if let Some(config_path) = &args.config {
        ConfigManager::load_from_file(config_path)?
    } else {
        ConfigManager::load_default()?
    };
    
    // Validate inputs and host state before touching artifacts.
    validate_inputs(&args, &config)?;
    validate_build_prerequisites(&config)?;
    validate_helper_sources_for_config(&config)?;
    RecoveryManager::pre_build_check().await?;

    let output_dir = PathBuf::from(&args.output_dir);
    let build_result = execute_build(&args, &config).await;

    if let Err(ref error) = build_result {
        warn!("Build failed, starting cleanup: {}", error);
        if let Err(cleanup_error) = RecoveryManager::cleanup_failed_build(&output_dir).await {
            warn!("Cleanup after failed build also failed: {}", cleanup_error);
        }
    }

    build_result
}

async fn execute_build(args: &BuildArgs, config: &GhostwinConfig) -> Result<()> {
    execute_build_with_progress(args, config, None).await
}

pub(crate) async fn execute_build_with_progress(
    args: &BuildArgs,
    config: &GhostwinConfig,
    progress_callback: Option<&BuildProgressCallback>,
) -> Result<()> {
    // Initialize WIM manager
    let mut wim_manager = WimManager::new(config)?;
    let total_steps = total_build_steps(args, config);
    
    // Execute build steps
    let mut completed_steps = 0;

    let build_result: Result<()> = async {
        log_build_step(build_progress(STEP_EXTRACT, completed_steps, total_steps), progress_callback);
        extract_iso(&args.source_iso, &args.output_dir).await?;
        validate_extracted_media_layout(Path::new(&args.output_dir))?;
        completed_steps += 1;

        log_build_step(build_progress(STEP_MOUNT, completed_steps, total_steps), progress_callback);
        let wim_path = Path::new(&args.output_dir).join("sources/boot.wim");
        wim_manager.mount(&wim_path, &config.iso.wim_index).await?;
        completed_steps += 1;

        log_build_step(build_progress(STEP_HELPERS, completed_steps, total_steps), progress_callback);
        copy_helper_files(&wim_manager, &config).await?;
        completed_steps += 1;

        if let Some(extra_files) = &args.extra_files {
            log_build_step(build_progress(STEP_EXTRA, completed_steps, total_steps), progress_callback);
            copy_extra_files(&wim_manager, extra_files).await?;
            completed_steps += 1;
        }

        if !args.skip_packages {
            log_build_step(build_progress(STEP_PACKAGES, completed_steps, total_steps), progress_callback);
            add_winpe_packages(&wim_manager, &config).await?;
            completed_steps += 1;
        }

        log_build_step(build_progress(STEP_DRIVERS, completed_steps, total_steps), progress_callback);
        inject_drivers(&wim_manager, &args).await?;
        completed_steps += 1;

        if !args.skip_dpi_fix && config.winpe.disable_dpi_scaling {
            log_build_step(build_progress(STEP_DPI_FIX, completed_steps, total_steps), progress_callback);
            apply_dpi_fix(&wim_manager).await?;
            completed_steps += 1;
        }

        log_build_step(build_progress(STEP_UNMOUNT, completed_steps, total_steps), progress_callback);
        wim_manager.unmount_and_commit().await?;
        completed_steps += 1;

        log_build_step(build_progress(STEP_CREATE_ISO, completed_steps, total_steps), progress_callback);
        validate_iso_creation_layout(Path::new(&args.output_dir))?;
        create_iso(&args.output_dir, &args.output_iso).await?;
        completed_steps += 1;

        if args.verify {
            log_build_step(build_progress(STEP_VERIFY, completed_steps, total_steps), progress_callback);
            verify_iso(&args.output_iso).await?;
        }

        Ok(())
    }
    .await;

    if build_result.is_err() && wim_manager.is_mounted() {
        warn!("Build failed while WIM was mounted, attempting discard unmount");
        if let Err(unmount_error) = wim_manager.unmount_and_discard().await {
            warn!("Discard unmount failed after build error: {}", unmount_error);
        }
    }

    build_result?;

    info!("✅ GhostWin ISO build completed successfully!");
    info!("Output: {}", args.output_iso);

    Ok(())
}

fn log_build_step(progress: BuildProgressState, progress_callback: Option<&BuildProgressCallback>) {
    info!(
        "{} ({}/{})",
        progress.current_step,
        progress.completed_steps + 1,
        progress.total_steps,
    );

    if let Some(callback) = progress_callback {
        callback(&progress);
    }
}

fn build_progress(current_step: &'static str, completed_steps: usize, total_steps: usize) -> BuildProgressState {
    BuildProgressState {
        current_step,
        completed_steps,
        total_steps,
    }
}

fn total_build_steps(args: &BuildArgs, config: &GhostwinConfig) -> usize {
    let mut steps = 6;
    if args.extra_files.is_some() {
        steps += 1;
    }
    if !args.skip_packages {
        steps += 1;
    }
    if !args.skip_dpi_fix && config.winpe.disable_dpi_scaling {
        steps += 1;
    }
    if args.verify {
        steps += 1;
    }
    steps
}

fn validate_inputs(args: &BuildArgs, _config: &GhostwinConfig) -> Result<()> {
    utils::validate_iso_file(&args.source_iso)?;

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

    if !Path::new(&args.output_dir).exists() {
        std::fs::create_dir_all(&args.output_dir)
            .with_context(|| format!("Failed to create output directory {}", args.output_dir))?;
    }

    if let Some(extra_files) = &args.extra_files {
        let extra_path = Path::new(extra_files);
        if !extra_path.exists() {
            bail!("Extra files directory does not exist: {}", extra_files);
        }
        if !extra_path.is_dir() {
            bail!("Extra files path is not a directory: {}", extra_files);
        }
    }
    
    Ok(())
}

fn validate_build_prerequisites(config: &GhostwinConfig) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        validate_windows_build_prerequisites(config)?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = config;
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn validate_windows_build_prerequisites(config: &GhostwinConfig) -> Result<()> {
    if !utils::command_exists("7z") {
        bail!("7-Zip is required for ISO extraction on Windows hosts");
    }

    if !utils::command_exists("dism") {
        bail!("DISM is required for WIM mounting and package injection");
    }

    let oscdimg_path = resolve_oscdimg_path(config)
        .ok_or_else(|| anyhow::anyhow!("oscdimg.exe not found in Windows ADK deployment tools"))?;
    if !oscdimg_path.exists() {
        bail!("Resolved oscdimg path does not exist: {}", oscdimg_path.display());
    }

    let winpe_root = resolve_winpe_root(config)
        .ok_or_else(|| anyhow::anyhow!("Windows PE add-on not found in configured or default ADK paths"))?;
    if !winpe_root.exists() {
        bail!("Resolved WinPE path does not exist: {}", winpe_root.display());
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
fn validate_windows_build_prerequisites(_config: &GhostwinConfig) -> Result<()> {
    Ok(())
}

#[cfg_attr(not(test), allow(dead_code))]
fn validate_helper_sources() -> Result<()> {
    validate_optional_directory(helper_source_path(&GhostwinConfig::default()).as_deref(), "Helper source")?;
    validate_optional_directory(windows_overlay_source_path(&GhostwinConfig::default()).as_deref(), "Windows overlay source")?;

    Ok(())
}

fn validate_helper_sources_for_config(config: &GhostwinConfig) -> Result<()> {
    validate_optional_directory(helper_source_path(config).as_deref(), "Helper source")?;
    validate_optional_directory(windows_overlay_source_path(config).as_deref(), "Windows overlay source")?;

    Ok(())
}

fn validate_optional_directory(path: Option<&Path>, label: &str) -> Result<()> {
    let Some(path) = path else {
        return Ok(());
    };

    if path.exists() && !path.is_dir() {
        bail!("{} exists but is not a directory: {}", label, path.display());
    }

    Ok(())
}

fn helper_source_path(config: &GhostwinConfig) -> Option<PathBuf> {
    config.iso.helper_source.as_ref().map(PathBuf::from)
}

fn windows_overlay_source_path(config: &GhostwinConfig) -> Option<PathBuf> {
    config.iso.windows_overlay_source.as_ref().map(PathBuf::from)
}

#[cfg_attr(not(test), allow(dead_code))]
fn resolve_adk_root(config: &GhostwinConfig) -> Option<PathBuf> {
    if let Some(path) = &config.iso.adk_path {
        return Some(PathBuf::from(path));
    }

    [
        r"C:\Program Files (x86)\Windows Kits\10",
        r"C:\Program Files\Windows Kits\10",
    ]
    .iter()
    .map(PathBuf::from)
    .find(|path| path.exists())
}

#[cfg_attr(not(test), allow(dead_code))]
fn resolve_oscdimg_path(config: &GhostwinConfig) -> Option<PathBuf> {
    resolve_adk_root(config).map(|adk_root| {
        adk_root
            .join("Assessment and Deployment Kit")
            .join("Deployment Tools")
            .join("amd64")
            .join("Oscdimg")
            .join("oscdimg.exe")
    })
}

#[cfg_attr(not(test), allow(dead_code))]
fn resolve_winpe_root(config: &GhostwinConfig) -> Option<PathBuf> {
    resolve_adk_root(config).map(|adk_root| {
        adk_root
            .join("Assessment and Deployment Kit")
            .join("Windows Preinstallation Environment")
    })
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
    let helper_source = helper_source_path(config);
    let windows_source = windows_overlay_source_path(config);
    
    if let Some(helper_source) = helper_source.as_deref().filter(|path| path.exists()) {
        wim_manager.copy_to_mount(helper_source, "Helper").await?;
    }
    
    if let Some(windows_source) = windows_source.as_deref().filter(|path| path.exists()) {
        wim_manager.copy_to_mount(windows_source, "Windows").await?;
    }
    
    // Detect and copy tool folders
    let tool_detector = ToolDetector::new(&config.tools);
    let detected_tools = tool_detector.scan_tools()?;
    
    info!("Detected {} tool directories", detected_tools.len());
    for tool_dir in detected_tools {
        if helper_source.as_deref().is_some_and(|path| path.exists() && tool_dir.starts_with(path)) {
            debug!("Skipping helper-managed tool directory already included: {}", tool_dir.display());
            continue;
        }

        let destination = tool_destination_for_dir(&tool_dir);
        debug!("Copying tool directory: {}", tool_dir.display());
        wim_manager.copy_to_mount(&tool_dir, destination).await?;
    }
    
    Ok(())
}

fn tool_destination_for_dir(tool_dir: &Path) -> &'static str {
    let folder_name = tool_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Tools");
    ToolDetector::helper_destination_for_folder(folder_name)
}

fn validate_extracted_media_layout(media_root: &Path) -> Result<()> {
    validate_required_paths(
        media_root,
        &[
            "bootmgr",
            "sources/boot.wim",
            "boot/bcd",
        ],
        "Extracted Windows media is incomplete",
    )
}

fn validate_iso_creation_layout(media_root: &Path) -> Result<()> {
    validate_required_paths(
        media_root,
        &[
            "boot/etfsboot.com",
            "efi/microsoft/boot/efisys.bin",
            "sources/boot.wim",
        ],
        "ISO creation layout is incomplete",
    )
}

fn validate_required_paths(root: &Path, required_paths: &[&str], context: &str) -> Result<()> {
    let missing_paths: Vec<String> = required_paths
        .iter()
        .filter_map(|relative| {
            let path = root.join(relative);
            if path.exists() {
                None
            } else {
                Some((*relative).to_string())
            }
        })
        .collect();

    if missing_paths.is_empty() {
        return Ok(());
    }

    bail!("{}: missing {}", context, missing_paths.join(", "))
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

async fn inject_drivers(wim_manager: &WimManager, _args: &BuildArgs) -> Result<()> {
    info!("🔍 Scanning for drivers");

    let mut driver_manager = DriverManager::new();
    driver_manager.scan_driver_directories()?;

    let drivers = driver_manager.detect_drivers()?;

    if drivers.is_empty() {
        info!("No drivers found to inject");
        return Ok(());
    }

    info!("Found {} drivers, beginning injection", drivers.len());
    driver_manager.warn_about_driver_risks(&drivers);

    // Inject drivers into WIM
    driver_manager.inject_drivers_to_wim(wim_manager, &drivers).await?;

    // Also copy drivers to WIM for manual installation
    driver_manager.copy_drivers_to_wim(wim_manager, &drivers).await?;

    info!("✅ Driver injection completed");
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

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        bail!("ISO creation not implemented for this platform");
    }
}

async fn verify_iso(iso_path: &str) -> Result<()> {
    debug!("Verifying ISO: {}", iso_path);

    verify_iso_sync(Path::new(iso_path))?;

    // Check 5: List key files using 7z (if available)
    #[cfg(target_os = "windows")]
    {
        let output = tokio::process::Command::new("7z")
            .args(&["l", "-ba", iso_path])
            .output()
            .await;

        if let Ok(output) = output {
            if output.status.success() {
                let listing = String::from_utf8_lossy(&output.stdout);

                // Check for critical Windows boot files
                let required_files = [
                    "bootmgr",
                    "boot\\bcd",
                    "sources\\boot.wim",
                ];

                let mut missing_files = Vec::new();
                for required_file in &required_files {
                    if !listing.contains(required_file) {
                        missing_files.push(*required_file);
                    }
                }

                if missing_files.is_empty() {
                    info!("✅ All critical boot files present");
                } else {
                    warn!("⚠️ Missing boot files: {:?}", missing_files);
                    warn!("ISO may not be bootable");
                }

                // Count WIM files
                let wim_count = listing.matches(".wim").count();
                info!("Found {} WIM files in ISO", wim_count);

            } else {
                warn!("Could not verify file contents (7z failed)");
            }
        } else {
            warn!("Could not verify file contents (7z not available)");
        }
    }

    info!("✅ ISO verification completed successfully");
    Ok(())
}

fn verify_iso_sync(iso_file: &Path) -> Result<()> {
    debug!("Verifying ISO synchronously: {}", iso_file.display());

    utils::validate_iso_file(iso_file)?;

    // Check 2: File size (should be reasonable for Windows ISO)
    let metadata = std::fs::metadata(iso_file)
        .context("Failed to read ISO file metadata")?;
    let size_mb = metadata.len() / (1024 * 1024);

    info!("ISO size: {} MB", size_mb);

    if size_mb < 100 {
        bail!("ISO file suspiciously small ({} MB). Build may have failed.", size_mb);
    }

    if size_mb > 20000 {
        warn!("ISO file very large ({} MB). This is unusual.", size_mb);
    }

    // Check 3: Boot signature verification (ISO 9660 format)
    use std::io::{Read, Seek, SeekFrom};
    let mut file = std::fs::File::open(iso_file)
        .context("Failed to open ISO for verification")?;

    // ISO 9660 signature is at offset 0x8001 (32768 + 1)
    file.seek(SeekFrom::Start(32769))
        .context("Failed to seek to ISO signature")?;

    let mut signature = [0u8; 5];
    file.read_exact(&mut signature)
        .context("Failed to read ISO signature")?;

    if &signature != b"CD001" {
        bail!("Invalid ISO 9660 signature. File may be corrupted.");
    }

    info!("✅ ISO signature valid (ISO 9660)");

    // Check 4: El Torito boot record (bootable ISO)
    file.seek(SeekFrom::Start(32768))
        .context("Failed to seek to boot record")?;

    let mut boot_record = [0u8; 32];
    file.read_exact(&mut boot_record)
        .context("Failed to read boot record")?;

    // First byte should be 0 (boot record volume descriptor)
    if boot_record[0] == 0 && &boot_record[1..6] == b"CD001" {
        // Check for El Torito boot indicator
        if &boot_record[7..30] == b"EL TORITO SPECIFICATION" {
            info!("✅ Bootable ISO detected (El Torito)");
        } else {
            info!("✅ ISO 9660 volume descriptor found");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        build_progress,
        helper_source_path,
        STEP_CREATE_ISO,
        STEP_DPI_FIX,
        STEP_DRIVERS,
        STEP_EXTRACT,
        STEP_HELPERS,
        STEP_MOUNT,
        STEP_PACKAGES,
        STEP_UNMOUNT,
        STEP_VERIFY,
        resolve_adk_root,
        resolve_oscdimg_path,
        resolve_winpe_root,
        total_build_steps,
        tool_destination_for_dir,
        validate_helper_sources,
        validate_helper_sources_for_config,
        validate_extracted_media_layout,
        validate_inputs,
        validate_iso_creation_layout,
        verify_iso_sync,
    };
    use crate::cli::{BuildArgs, GhostwinConfig};
    use tempfile::tempdir;
    use std::io::{Seek, SeekFrom, Write};

    fn build_args(source_iso: &std::path::Path, output_root: &std::path::Path) -> BuildArgs {
        BuildArgs {
            source_iso: source_iso.display().to_string(),
            output_dir: output_root.join("build").display().to_string(),
            output_iso: output_root.join("ghostwin.iso").display().to_string(),
            extra_files: None,
            skip_packages: false,
            skip_dpi_fix: false,
            config: None,
            verify: false,
        }
    }

    #[test]
    fn validate_inputs_creates_output_dir() {
        let temp = tempdir().unwrap();
        let source_iso = temp.path().join("windows.iso");
        std::fs::write(&source_iso, vec![0_u8; 101 * 1024 * 1024]).unwrap();

        let args = build_args(&source_iso, temp.path());
        let config = GhostwinConfig::default();

        validate_inputs(&args, &config).unwrap();
        assert!(temp.path().join("build").exists());
    }

    #[test]
    fn validate_inputs_rejects_missing_extra_files_dir() {
        let temp = tempdir().unwrap();
        let source_iso = temp.path().join("windows.iso");
        std::fs::write(&source_iso, vec![0_u8; 101 * 1024 * 1024]).unwrap();

        let mut args = build_args(&source_iso, temp.path());
        args.extra_files = Some(temp.path().join("missing-extra").display().to_string());

        let error = validate_inputs(&args, &GhostwinConfig::default()).unwrap_err();
        assert!(error.to_string().contains("Extra files directory does not exist"));
    }

    #[test]
    fn verify_iso_sync_rejects_invalid_signature() {
        let temp = tempdir().unwrap();
        let iso_path = temp.path().join("broken.iso");
        let mut file = std::fs::File::create(&iso_path).unwrap();
        file.set_len(101 * 1024 * 1024).unwrap();
        file.seek(SeekFrom::Start(32769)).unwrap();
        file.write_all(b"WRONG").unwrap();

        let error = verify_iso_sync(&iso_path).unwrap_err();
        assert!(error.to_string().contains("Invalid ISO 9660 signature"));
    }

    #[test]
    fn verify_iso_sync_accepts_minimal_iso_signature() {
        let temp = tempdir().unwrap();
        let iso_path = temp.path().join("valid.iso");
        let mut file = std::fs::File::create(&iso_path).unwrap();
        file.set_len(101 * 1024 * 1024).unwrap();
        file.seek(SeekFrom::Start(32768)).unwrap();

        let mut boot_record = [0_u8; 32];
        boot_record[0] = 0;
        boot_record[1..6].copy_from_slice(b"CD001");
        file.write_all(&boot_record).unwrap();

        file.seek(SeekFrom::Start(32769)).unwrap();
        file.write_all(b"CD001").unwrap();

        verify_iso_sync(&iso_path).unwrap();
    }

    #[test]
    fn validate_extracted_media_layout_accepts_required_files() {
        let temp = tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("sources")).unwrap();
        std::fs::create_dir_all(temp.path().join("boot")).unwrap();
        std::fs::write(temp.path().join("bootmgr"), "bootmgr").unwrap();
        std::fs::write(temp.path().join("sources/boot.wim"), "wim").unwrap();
        std::fs::write(temp.path().join("boot/bcd"), "bcd").unwrap();

        validate_extracted_media_layout(temp.path()).unwrap();
    }

    #[test]
    fn validate_iso_creation_layout_rejects_missing_boot_assets() {
        let temp = tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("sources")).unwrap();
        std::fs::write(temp.path().join("sources/boot.wim"), "wim").unwrap();

        let error = validate_iso_creation_layout(temp.path()).unwrap_err();
        assert!(error.to_string().contains("ISO creation layout is incomplete"));
        assert!(error.to_string().contains("boot/etfsboot.com"));
        assert!(error.to_string().contains("efi/microsoft/boot/efisys.bin"));
    }

    #[test]
    fn tool_destination_matches_folder_role() {
        assert_eq!(tool_destination_for_dir(std::path::Path::new("Tools")), "Helper/Tools");
        assert_eq!(tool_destination_for_dir(std::path::Path::new("PEAutoRun")), "Helper/PEAutoRun");
        assert_eq!(tool_destination_for_dir(std::path::Path::new("Logon")), "Helper/Logon");
    }

    #[test]
    fn validate_required_paths_reports_all_missing_entries() {
        let temp = tempdir().unwrap();
        let error = super::validate_required_paths(
            temp.path(),
            &["bootmgr", "sources/boot.wim", "boot/bcd"],
            "layout invalid",
        )
        .unwrap_err();

        let message = error.to_string();
        assert!(message.contains("layout invalid"));
        assert!(message.contains("bootmgr"));
        assert!(message.contains("sources/boot.wim"));
        assert!(message.contains("boot/bcd"));
    }

    #[test]
    fn build_step_labels_stay_stable() {
        assert_eq!(STEP_EXTRACT, "Step 1: Extracting source ISO");
        assert_eq!(STEP_MOUNT, "Step 2: Mounting WIM image");
        assert_eq!(STEP_HELPERS, "Step 3: Copying helper files");
        assert_eq!(STEP_PACKAGES, "Step 5: Adding WinPE packages");
        assert_eq!(STEP_DRIVERS, "Step 6: Detecting and injecting drivers");
        assert_eq!(STEP_DPI_FIX, "Step 7: Applying DPI fix");
        assert_eq!(STEP_UNMOUNT, "Step 8: Unmounting and committing WIM");
        assert_eq!(STEP_CREATE_ISO, "Step 9: Creating final ISO");
        assert_eq!(STEP_VERIFY, "Step 10: Verifying ISO integrity");
    }

    #[test]
    fn resolve_adk_related_paths_from_custom_root() {
        let mut config = GhostwinConfig::default();
        config.iso.adk_path = Some(r"C:\ADK".to_string());

        assert_eq!(resolve_adk_root(&config).unwrap(), std::path::PathBuf::from(r"C:\ADK"));
        let oscdimg = resolve_oscdimg_path(&config).unwrap();
        let winpe = resolve_winpe_root(&config).unwrap();

        assert!(oscdimg.ends_with("Assessment and Deployment Kit/Deployment Tools/amd64/Oscdimg/oscdimg.exe"));
        assert!(winpe.ends_with("Assessment and Deployment Kit/Windows Preinstallation Environment"));
    }

    #[test]
    fn validate_helper_sources_accepts_missing_optional_dirs() {
        validate_helper_sources().unwrap();
    }

    #[test]
    fn helper_source_paths_come_from_config() {
        let mut config = GhostwinConfig::default();
        config.iso.helper_source = Some("/tmp/helper".to_string());
        assert_eq!(helper_source_path(&config).unwrap(), std::path::PathBuf::from("/tmp/helper"));
    }

    #[test]
    fn total_build_steps_reflects_optional_flags() {
        let temp = tempdir().unwrap();
        let mut args = build_args(&temp.path().join("input.iso"), temp.path());
        let config = GhostwinConfig::default();

        assert_eq!(total_build_steps(&args, &config), 8);

        args.extra_files = Some(temp.path().join("extra").display().to_string());
        args.verify = true;
        assert_eq!(total_build_steps(&args, &config), 10);
    }

    #[test]
    fn build_progress_tracks_step_metadata() {
        let progress = build_progress(STEP_DRIVERS, 4, 9);
        assert_eq!(progress.current_step, STEP_DRIVERS);
        assert_eq!(progress.completed_steps, 4);
        assert_eq!(progress.total_steps, 9);
    }

    #[test]
    fn validate_helper_sources_rejects_file_path() {
        let temp = tempdir().unwrap();
        let helper_file = temp.path().join("helper.txt");
        std::fs::write(&helper_file, "not a dir").unwrap();

        let mut config = GhostwinConfig::default();
        config.iso.helper_source = Some(helper_file.display().to_string());

        let error = validate_helper_sources_for_config(&config).unwrap_err();
        assert!(error.to_string().contains("Helper source exists but is not a directory"));
    }
}
