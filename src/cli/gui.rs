use anyhow::Result;
use tracing::{info, error};
#[cfg(target_os = "windows")]
use tracing::warn;
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
#[cfg(target_os = "windows")]
use std::process::Command;
use crate::config::ConfigManager;
use crate::cli::build::BuildProgressState;
use crate::cli::BuildArgs;
use crate::tools::{ToolDetector, ToolCategory};
use crate::vnc::VncManager;
use crate::executor::ScriptExecutor;
use slint::{ModelRc, VecModel};

slint::include_modules!();

/// Detect if running in WinPE environment
#[cfg(target_os = "windows")]
fn is_winpe_environment() -> bool {
    // WinPE typically runs from X:\Windows
    if let Ok(windir) = std::env::var("SYSTEMROOT") {
        if windir.to_uppercase().starts_with("X:") {
            return true;
        }
    }
    // Alternative: check for WinPE marker registry or missing explorer
    let explorer = std::path::Path::new("C:\\Windows\\explorer.exe");
    if !explorer.exists() {
        return true;
    }
    false
}

pub async fn execute() -> Result<()> {
    // Configure rendering backend for Windows
    #[cfg(target_os = "windows")]
    {
        if std::env::var("SLINT_BACKEND").is_ok() {
            info!("Using user-specified SLINT_BACKEND");
        } else if is_winpe_environment() {
            // SAFETY: Called at startup before any other threads are spawned
            unsafe { std::env::set_var("SLINT_BACKEND", "winit-software") };
            info!("WinPE detected - using software rendering (SLINT_BACKEND=winit-software)");
        }
        // Normal Windows: leave unset, use hardware rendering
    }

    info!("Launching GhostWin GUI");
    
    // Load configuration and detect tools
    let config = ConfigManager::load_default()?;
    let detector = ToolDetector::new(&config.tools);
    let detected_tools = detector.detect_tools(".")?;
    
    info!("Detected {} tools for GUI", detected_tools.len());
    
    // Convert detected tools to Slint format
    let slint_tools: Vec<ToolItem> = detected_tools.iter().map(|tool| {
        ToolItem {
            name: tool.name.clone().into(),
            category: match tool.category {
                ToolCategory::Tool => "Tool".into(),
                ToolCategory::PEAutoRun => "PEAutoRun".into(),
                ToolCategory::Logon => "Logon".into(),
            },
            executable: tool.executable,
            path: tool.path.to_string_lossy().to_string().into(),
            enabled: false,
        }
    }).collect();
    
    // Create VNC manager and script executor
    let vnc_manager = Arc::new(Mutex::new(VncManager::new(config.clone())));
    let script_executor = Arc::new(ScriptExecutor::new(config.clone()));
    
    // Create the main window
    let ui = match GhostWinApp::new() {
        Ok(app) => app,
        Err(e) => {
            error!("Failed to create GUI window: {}", e);
            #[cfg(target_os = "windows")]
            {
                if std::env::var("SLINT_BACKEND").is_err() {
                    warn!("Hint: If this is a graphics driver issue, retry with:");
                    warn!("  set SLINT_BACKEND=winit-software");
                    warn!("  ghostwin gui");
                }
            }
            return Err(e.into());
        }
    };
    
    // Set initial state
    let model = ModelRc::new(VecModel::from(slint_tools));
    ui.set_tools(model);
    ui.set_current_mode("home".into());
    ui.set_vnc_enabled(false);
    ui.set_vnc_status("Disconnected".into());
    ui.set_install_progress(InstallProgress {
        current_step: "Idle".into(),
        progress: 0.0,
        completed: false,
        error: "".into(),
    });
    
    // Build running guard - prevents concurrent builds
    let build_running = Arc::new(AtomicBool::new(false));
    // Build generation counter - prevents stale finalization from affecting new builds
    let build_generation = Arc::new(AtomicU64::new(0));

    // Install running guard - prevents concurrent install launches
    let install_running = Arc::new(AtomicBool::new(false));
    // Install generation counter - prevents stale finalization from affecting new installs
    let install_generation = Arc::new(AtomicU64::new(0));

    // Set up callbacks
    let ui_weak = ui.as_weak();
    let config_for_build = config.clone();
    let build_running_for_start = build_running.clone();
    let build_running_for_finalize = build_running.clone();
    let build_gen_for_start = build_generation.clone();
    let build_gen_for_finalize = build_generation.clone();
    ui.on_start_build(move |request| {
        // Prevent concurrent builds
        if build_running_for_start.load(Ordering::SeqCst) {
            info!("Build already in progress, ignoring request");
            return;
        }

        if let Some(ui) = ui_weak.upgrade() {
            // Increment generation and mark build as running
            let this_build_gen = build_gen_for_start.fetch_add(1, Ordering::SeqCst) + 1;
            build_running_for_start.store(true, Ordering::SeqCst);
            ui.set_build_running(true);

            // Stay in build view - show progress card (triggered by current_step being set)
            ui.set_build_progress(InstallProgress {
                current_step: "Initializing build...".into(),
                progress: 0.0,
                completed: false,
                error: "".into(),
            });

            let ui_weak = ui.as_weak();
            let config = config_for_build.clone();
            let build_running_flag = build_running_for_finalize.clone();
            let build_gen_flag = build_gen_for_finalize.clone();
            let build_args = BuildArgs {
                source_iso: request.source_iso.to_string(),
                output_dir: request.output_dir.to_string(),
                output_iso: request.output_iso.to_string(),
                extra_files: None,
                skip_packages: false,
                skip_dpi_fix: false,
                config: None,
                verify: request.verify,
            };

            thread::spawn(move || {
                let progress_ui = ui_weak.clone();
                let final_ui = ui_weak;
                let runtime = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
                let result = runtime.block_on(async {
                    crate::cli::build::execute_build_with_progress(
                        &build_args,
                        &config,
                        Some(&move |progress| update_build_progress_state(progress_ui.clone(), progress.clone())),
                    )
                    .await
                });
                finalize_build_progress(final_ui, result, build_running_flag, build_gen_flag, this_build_gen);
            });
        }
    });

    let ui_weak = ui.as_weak();
    let install_running_normal = install_running.clone();
    let install_gen_normal = install_generation.clone();
    let install_running_normal_finalize = install_running.clone();
    let install_gen_normal_finalize = install_generation.clone();
    ui.on_start_normal_install(move || {
        // Prevent concurrent install launches
        if install_running_normal.load(Ordering::SeqCst) {
            info!("Install launch already in progress, ignoring request");
            return;
        }

        if let Some(ui) = ui_weak.upgrade() {
            // Increment generation and mark install as running
            let this_install_gen = install_gen_normal.fetch_add(1, Ordering::SeqCst) + 1;
            install_running_normal.store(true, Ordering::SeqCst);
            ui.set_install_running(true);

            ui.set_current_mode("install".into());
            ui.set_install_progress(InstallProgress {
                current_step: "Launching Windows Setup...".into(),
                progress: 0.1,
                completed: false,
                error: "".into(),
            });

            // Start normal installation in background thread
            let ui_weak = ui.as_weak();
            let install_running_flag = install_running_normal_finalize.clone();
            let install_gen_flag = install_gen_normal_finalize.clone();
            thread::spawn(move || {
                let result = start_windows_installation(false, None, None);
                finalize_install_progress(ui_weak, result, install_running_flag, install_gen_flag, this_install_gen);
            });
        }
    });
    
    let ui_weak = ui.as_weak();
    let executor_clone3 = script_executor.clone();
    let config_clone3 = config.clone();
    let install_running_auto = install_running.clone();
    let install_gen_auto = install_generation.clone();
    let install_running_auto_finalize = install_running.clone();
    let install_gen_auto_finalize = install_generation.clone();
    ui.on_start_automated_install(move || {
        // Prevent concurrent install launches
        if install_running_auto.load(Ordering::SeqCst) {
            info!("Install launch already in progress, ignoring request");
            return;
        }

        if let Some(ui) = ui_weak.upgrade() {
            // Increment generation and mark install as running
            let this_install_gen = install_gen_auto.fetch_add(1, Ordering::SeqCst) + 1;
            install_running_auto.store(true, Ordering::SeqCst);
            ui.set_install_running(true);

            ui.set_current_mode("install".into());
            ui.set_install_progress(InstallProgress {
                current_step: "Preparing automated upgrade...".into(),
                progress: 0.1,
                completed: false,
                error: "".into(),
            });

            // Start automated installation in background thread
            let executor = executor_clone3.clone();
            let config = config_clone3.clone();
            let ui_weak = ui.as_weak();
            let install_running_flag = install_running_auto_finalize.clone();
            let install_gen_flag = install_gen_auto_finalize.clone();
            thread::spawn(move || {
                let result = start_windows_installation(true, Some(executor), Some(config));
                finalize_install_progress(ui_weak, result, install_running_flag, install_gen_flag, this_install_gen);
            });
        }
    });

    // Shared helper for tool/script execution
    fn execute_tool_async(
        path: &str,
        item_type: &'static str,
        tools: &[crate::tools::DetectedTool],
        executor: Arc<ScriptExecutor>,
        ui_weak: slint::Weak<GhostWinApp>,
    ) {
        let path_str = path.to_string();

        if let Some(tool) = tools.iter().find(|t| t.path.to_string_lossy() == path) {
            let tool = tool.clone();
            let executor = executor.clone();
            let ui_weak = ui_weak.clone();

            thread::spawn(move || {
                let result = executor.execute_tool(&tool);
                let tool_name = tool.name.clone();

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        match result {
                            Ok(exec_result) => {
                                info!("{}", exec_result.summary());
                                if !exec_result.success {
                                    // Execution failed - show error with detail if available
                                    let error_detail = if !exec_result.stderr.is_empty() {
                                        exec_result.stderr.clone()
                                    } else {
                                        format!("exited with code {}", exec_result.exit_code.unwrap_or(-1))
                                    };
                                    error!("{} failed: {}", item_type, error_detail);
                                    ui.invoke_show_notification(
                                        format!("{} failed: {}", item_type, truncate_error(&error_detail, 50)).into(),
                                        "error".into(),
                                    );
                                } else {
                                    ui.invoke_show_notification(
                                        format!("{}: {}", item_type, tool_name).into(),
                                        "success".into(),
                                    );
                                }
                            }
                            Err(e) => {
                                error!("Failed to execute {}: {}", item_type.to_lowercase(), e);
                                ui.invoke_show_notification(
                                    format!("Failed to run {}: {}", item_type.to_lowercase(), e).into(),
                                    "error".into(),
                                );
                            }
                        }
                    }
                });
            });
        } else {
            error!("{} not found: {}", item_type, path_str);
            if let Some(ui) = ui_weak.upgrade() {
                ui.invoke_show_notification(format!("{} not found", item_type).into(), "error".into());
            }
        }
    }

    let executor_clone = script_executor.clone();
    let tools_clone = detected_tools.clone();
    let ui_weak_tool = ui.as_weak();
    ui.on_launch_tool(move |path| {
        info!("Launching tool: {}", path);
        execute_tool_async(&path, "Tool", &tools_clone, executor_clone.clone(), ui_weak_tool.clone());
    });

    let executor_clone2 = script_executor.clone();
    let tools_clone2 = detected_tools.clone();
    let ui_weak_script = ui.as_weak();
    ui.on_run_script(move |path| {
        info!("Running script: {}", path);
        execute_tool_async(&path, "Script", &tools_clone2, executor_clone2.clone(), ui_weak_script.clone());
    });
    
    let ui_weak = ui.as_weak();
    let vnc_manager_clone = vnc_manager.clone();
    ui.on_toggle_vnc(move || {
        if let Some(ui) = ui_weak.upgrade() {
            if let Ok(mut vnc) = vnc_manager_clone.lock() {
                if vnc.is_running() {
                    info!("Stopping VNC server");
                    if let Err(e) = vnc.stop_server() {
                        error!("Failed to stop VNC server: {}", e);
                        let short_err = truncate_error(&e.to_string(), 20);
                        ui.set_vnc_status(format!("Error: {}", short_err).into());
                        ui.invoke_show_notification(format!("Failed to stop VNC: {}", e).into(), "error".into());
                    } else {
                        ui.set_vnc_enabled(false);
                        ui.set_vnc_status("Disconnected".into());
                        ui.invoke_show_notification("VNC server stopped".into(), "info".into());
                    }
                } else {
                    info!("Starting VNC server");
                    match vnc.start_server() {
                        Ok(_) => {
                            ui.set_vnc_enabled(true);
                            let connection_info = vnc.get_connection_info();
                            let conn_str = connection_info.get_connection_string();
                            ui.set_vnc_status(format!("Connected ({})", conn_str).into());
                            ui.invoke_show_notification(format!("VNC server started: {}", conn_str).into(), "success".into());
                        }
                        Err(e) => {
                            error!("Failed to start VNC server: {}", e);
                            let short_err = truncate_error(&e.to_string(), 20);
                            ui.set_vnc_status(format!("Error: {}", short_err).into());
                            ui.invoke_show_notification(format!("Failed to start VNC: {}", e).into(), "error".into());
                        }
                    }
                }
            }
        }
    });
    
    let ui_weak = ui.as_weak();
    ui.on_show_tools(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_current_mode("tools".into());
        }
    });
    
    let ui_weak = ui.as_weak();
    ui.on_show_menu(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_current_mode("home".into());
        }
    });

    // Notification dismiss callback - just logs for now
    ui.on_dismiss_notification(|| {
        info!("Notification dismissed");
    });
    
    info!("Starting GUI main loop");
    ui.run()?;

    Ok(())
}

/// Truncate error message safely for UI display (UTF-8 aware)
fn truncate_error(msg: &str, max_len: usize) -> String {
    // Take first line only
    let first_line = msg.lines().next().unwrap_or(msg);
    let char_count = first_line.chars().count();
    if char_count <= max_len {
        first_line.to_string()
    } else {
        // Use chars() to avoid slicing in the middle of multi-byte UTF-8 characters
        let truncated: String = first_line.chars().take(max_len.saturating_sub(3)).collect();
        format!("{}...", truncated)
    }
}

/// Resolve setup.exe from known locations instead of relying on PATH/cwd
/// Returns the absolute path to setup.exe if found, or an error with actionable message
///
/// Search strategy (deterministic, media-aware, no cwd dependency):
/// 1. Skip X: (WinPE runtime volume) and C: in early passes (likely system drive)
/// 2. First pass: D:-Z: with sources\setup.exe AND sources\install.wim (confirmed install media)
/// 3. Second pass: D:-Z: with sources\setup.exe AND sources\boot.wim (likely install media)
/// 4. Third pass: D:-Z: with sources\setup.exe (any media layout)
/// 5. Fourth pass: C: with full media validation (system drive fallback)
/// 6. Fifth pass: Any drive with root setup.exe (last resort)
#[cfg(target_os = "windows")]
fn resolve_setup_exe() -> Result<std::path::PathBuf> {
    use std::path::PathBuf;

    // Helper: check if a drive looks like Windows install media
    let has_install_wim = |drive: char| -> bool {
        PathBuf::from(format!("{}:\\sources\\install.wim", drive)).exists()
    };
    let has_boot_wim = |drive: char| -> bool {
        PathBuf::from(format!("{}:\\sources\\boot.wim", drive)).exists()
    };
    let has_sources_setup = |drive: char| -> bool {
        PathBuf::from(format!("{}:\\sources\\setup.exe", drive)).exists()
    };
    let has_root_setup = |drive: char| -> bool {
        PathBuf::from(format!("{}:\\setup.exe", drive)).exists()
    };

    // Pass 1: D:-Z: with sources\setup.exe AND install.wim (confirmed Windows install media)
    for drive in 'D'..='Z' {
        if drive == 'X' {
            continue;
        }
        if has_sources_setup(drive) && has_install_wim(drive) {
            let path = PathBuf::from(format!("{}:\\sources\\setup.exe", drive));
            info!("Found Windows Setup with install.wim at: {}", path.display());
            return Ok(path.canonicalize().unwrap_or(path));
        }
    }

    // Pass 2: D:-Z: with sources\setup.exe AND boot.wim (likely install media)
    for drive in 'D'..='Z' {
        if drive == 'X' {
            continue;
        }
        if has_sources_setup(drive) && has_boot_wim(drive) {
            let path = PathBuf::from(format!("{}:\\sources\\setup.exe", drive));
            info!("Found Windows Setup with boot.wim at: {}", path.display());
            return Ok(path.canonicalize().unwrap_or(path));
        }
    }

    // Pass 3: D:-Z: with any sources\setup.exe
    for drive in 'D'..='Z' {
        if drive == 'X' {
            continue;
        }
        if has_sources_setup(drive) {
            let path = PathBuf::from(format!("{}:\\sources\\setup.exe", drive));
            info!("Found Windows Setup at: {}", path.display());
            return Ok(path.canonicalize().unwrap_or(path));
        }
    }

    // Pass 4: C: with full media validation (system drive, only if it looks like real media)
    if has_sources_setup('C') && (has_install_wim('C') || has_boot_wim('C')) {
        let path = PathBuf::from("C:\\sources\\setup.exe");
        info!("Found Windows Setup on C: with media validation: {}", path.display());
        return Ok(path.canonicalize().unwrap_or(path));
    }

    // Pass 5: Any drive with root-level setup.exe (last resort, less common layout)
    for drive in 'D'..='Z' {
        if drive == 'X' {
            continue;
        }
        if has_root_setup(drive) {
            let path = PathBuf::from(format!("{}:\\setup.exe", drive));
            info!("Found Windows Setup (root) at: {}", path.display());
            return Ok(path.canonicalize().unwrap_or(path));
        }
    }

    Err(anyhow::anyhow!(
        "Windows Setup not found. Mount the Windows installation media first."
    ))
}

/// Generate the list of setup.exe candidate paths for testing
/// Returns paths in conceptual search order (deterministic, media-aware, no cwd dependency)
///
/// The actual resolver uses multiple passes with media validation:
/// 1. D:-Z: sources\setup.exe with install.wim
/// 2. D:-Z: sources\setup.exe with boot.wim
/// 3. D:-Z: sources\setup.exe (any)
/// 4. C: sources\setup.exe with media validation
/// 5. D:-Z: root setup.exe
#[cfg(test)]
fn setup_exe_search_candidates() -> Vec<String> {
    let mut candidates = Vec::new();
    // Primary candidates: D:-Z: sources paths (media drives, not system drive)
    for drive in 'D'..='Z' {
        if drive == 'X' {
            continue;
        }
        candidates.push(format!("{}:\\sources\\setup.exe", drive));
    }
    // C: sources path (system drive, requires media validation in actual resolver)
    candidates.push("C:\\sources\\setup.exe".to_string());
    // Root-level fallbacks (less common layout)
    for drive in 'D'..='Z' {
        if drive == 'X' {
            continue;
        }
        candidates.push(format!("{}:\\setup.exe", drive));
    }
    candidates
}

/// Returns Ok(true) if Windows Setup was actually launched, Ok(false) if simulated (non-Windows)
///
/// IMPORTANT: setup.exe is resolved BEFORE any PE scripts run.
/// This ensures missing install media fails fast with no side effects.
fn start_windows_installation(
    automated: bool,
    executor: Option<Arc<ScriptExecutor>>,
    config: Option<crate::cli::GhostwinConfig>
) -> Result<bool> {
    info!("Starting Windows installation (automated: {})", automated);

    // FIRST: Resolve setup.exe BEFORE running any PE scripts
    // This ensures missing install media fails fast with no side effects
    #[cfg(target_os = "windows")]
    let setup_path = resolve_setup_exe()?;

    #[cfg(target_os = "windows")]
    info!("Validated Windows Setup at: {}", setup_path.display());

    // NOW safe to run PE scripts - we know setup.exe exists
    if automated {
        if let (Some(executor), Some(config)) = (executor, config) {
            let detector = ToolDetector::new(&config.tools);
            let detected_tools = detector.detect_tools(".")?;
            let phase_tools = crate::utils::resolve_detected_tools(&config.phases.pe_system_setup_paths, &detected_tools);

            info!("Executing configured PE system-setup scripts");
            let results = executor.execute_pe_autorun_scripts(&phase_tools)?;

            // Check if any scripts failed, collect failure details
            let mut failed_scripts: Vec<String> = Vec::new();
            for result in &results {
                info!("{}", result.summary());
                if !result.success {
                    let detail = if result.stderr.is_empty() {
                        result.tool_name.clone()
                    } else {
                        format!("{}: {}", result.tool_name, result.stderr.lines().next().unwrap_or(""))
                    };
                    error!("Script failed: {}", detail);
                    failed_scripts.push(detail);
                }
            }

            if !failed_scripts.is_empty() {
                let summary = if failed_scripts.len() == 1 {
                    format!("PE script failed: {}", failed_scripts[0])
                } else {
                    format!("PE scripts failed: {}", failed_scripts.join(", "))
                };
                return Err(anyhow::anyhow!("{}", summary));
            }
        }
    }

    // Launch Windows setup using already-validated path
    #[cfg(target_os = "windows")]
    {
        info!("Launching Windows Setup from: {}", setup_path.display());

        if automated {
            // Automated upgrade: use /auto upgrade for upgrade-style automation
            Command::new(&setup_path)
                .arg("/auto")
                .arg("upgrade")
                .spawn()?;
        } else {
            // Normal: launch interactive Windows Setup (no automation flags)
            Command::new(&setup_path)
                .spawn()?;
        }
        Ok(true)  // Actually launched
    }

    #[cfg(not(target_os = "windows"))]
    {
        info!("Windows installation simulation (not on Windows host, automated={})", automated);
        Ok(false)  // Simulated only
    }
}

fn finalize_install_progress(
    ui_weak: slint::Weak<GhostWinApp>,
    result: Result<bool>,
    install_running: Arc<AtomicBool>,
    install_generation: Arc<AtomicU64>,
    expected_generation: u64,
) {
    let _ = slint::invoke_from_event_loop(move || {
        // Check if this finalization is for the current install (race safety)
        let current_gen = install_generation.load(Ordering::SeqCst);
        if current_gen != expected_generation {
            // A newer install has started - don't modify state
            info!("Ignoring stale install finalization (gen {} vs current {})", expected_generation, current_gen);
            return;
        }

        // Clear the running flag only if we're still the current install
        install_running.store(false, Ordering::SeqCst);

        if let Some(ui) = ui_weak.upgrade() {
            // Clear UI running flag
            ui.set_install_running(false);

            match result {
                Ok(true) => {
                    // Handed off to Windows Setup - NOT complete, just launched
                    ui.set_install_progress(InstallProgress {
                        current_step: "Handed off to Windows Setup".into(),
                        progress: 0.0,  // No progress bar - handoff is not completion
                        completed: false,
                        error: "".into(),
                    });
                    ui.invoke_show_notification("Control handed off to Windows Setup".into(), "info".into());
                }
                Ok(false) => {
                    // Simulated on non-Windows host
                    ui.set_install_progress(InstallProgress {
                        current_step: "Simulated - requires Windows host".into(),
                        progress: 0.0,  // No progress bar - simulation is not completion
                        completed: false,
                        error: "".into(),
                    });
                    ui.invoke_show_notification("Install simulated (not on Windows)".into(), "info".into());
                }
                Err(e) => {
                    error!("Failed to launch installation: {}", e);
                    ui.set_install_progress(InstallProgress {
                        current_step: "Failed to launch".into(),
                        progress: 0.0,
                        completed: false,
                        error: e.to_string().into(),
                    });
                    ui.invoke_show_notification(format!("Failed: {}", e).into(), "error".into());
                }
            }
        }
    });
}

fn update_build_progress_state(ui_weak: slint::Weak<GhostWinApp>, progress: BuildProgressState) {
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_build_progress(install_progress_from_build_progress(&progress));
        }
    });
}

fn finalize_build_progress(
    ui_weak: slint::Weak<GhostWinApp>,
    result: Result<()>,
    build_running: Arc<AtomicBool>,
    build_generation: Arc<AtomicU64>,
    expected_generation: u64,
) {
    let _ = slint::invoke_from_event_loop(move || {
        // Check if this finalization is for the current build (race safety)
        let current_gen = build_generation.load(Ordering::SeqCst);
        if current_gen != expected_generation {
            // A newer build has started - don't modify state
            info!("Ignoring stale build finalization (gen {} vs current {})", expected_generation, current_gen);
            return;
        }

        // Clear the running flag only if we're still the current build
        build_running.store(false, Ordering::SeqCst);

        if let Some(ui) = ui_weak.upgrade() {
            // Clear UI running flag
            ui.set_build_running(false);

            match result {
                Ok(()) => {
                    ui.set_build_progress(InstallProgress {
                        current_step: "Build complete".into(),
                        progress: 1.0,
                        completed: true,
                        error: "".into(),
                    });
                    ui.invoke_show_notification("Build completed successfully!".into(), "success".into());
                }
                Err(e) => {
                    error!("Build failed: {}", e);
                    ui.set_build_progress(InstallProgress {
                        current_step: "Build failed".into(),
                        progress: 0.0,
                        completed: false,
                        error: e.to_string().into(),
                    });
                    ui.invoke_show_notification(format!("Build failed: {}", e).into(), "error".into());
                }
            }
        }
    });
}

fn install_progress_from_build_progress(progress: &BuildProgressState) -> InstallProgress {
    let fraction = if progress.total_steps == 0 {
        0.0
    } else {
        // Cap at 0.95 during progress - finalize_build_progress sets 1.0 on completion
        let raw = (progress.completed_steps + 1) as f32 / progress.total_steps as f32;
        raw.min(0.95)
    };

    // Never set completed=true during progress updates
    // Only finalize_build_progress should mark the build as complete
    InstallProgress {
        current_step: progress.current_step.into(),
        progress: fraction,
        completed: false,
        error: "".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::install_progress_from_build_progress;
    use crate::cli::build::BuildProgressState;

    #[test]
    fn maps_build_progress_into_install_progress() {
        let progress = BuildProgressState {
            current_step: "Step 3: Copying helper files",
            completed_steps: 2,
            total_steps: 8,
        };

        let install = install_progress_from_build_progress(&progress);
        assert_eq!(install.current_step.as_str(), "Step 3: Copying helper files");
        assert_eq!(install.progress, 3.0 / 8.0);
        assert!(!install.completed); // Never completed during progress updates
        assert_eq!(install.error.as_str(), "");
    }

    #[test]
    fn caps_progress_at_95_percent_during_final_step() {
        // Progress updates should never show 100% - that's reserved for finalize_build_progress
        let progress = BuildProgressState {
            current_step: "Step 10: Verifying ISO integrity",
            completed_steps: 9,
            total_steps: 10,
        };

        let install = install_progress_from_build_progress(&progress);
        assert_eq!(install.progress, 0.95); // Capped, not 1.0
        assert!(!install.completed); // Never completed during progress updates
    }

    #[test]
    fn truncate_error_handles_ascii() {
        use super::truncate_error;

        // Short string - no truncation
        assert_eq!(truncate_error("short", 10), "short");

        // Exact length
        assert_eq!(truncate_error("exactly10!", 10), "exactly10!");

        // Needs truncation
        assert_eq!(truncate_error("this is a long error message", 15), "this is a lo...");
    }

    #[test]
    fn truncate_error_handles_unicode() {
        use super::truncate_error;

        // Chinese characters (3 bytes each in UTF-8)
        let chinese = "错误信息很长";
        assert_eq!(truncate_error(chinese, 10), chinese); // 6 chars, fits in 10

        // Truncate multi-byte chars safely
        let long_chinese = "这是一个很长的错误信息需要截断";
        let truncated = truncate_error(long_chinese, 10);
        assert_eq!(truncated, "这是一个很长的..."); // 7 chars + "..."
        assert!(truncated.is_char_boundary(truncated.len())); // Valid UTF-8

        // Emoji (4 bytes each in UTF-8, but 1 char each)
        let emoji = "🔥🚀💥error";  // 8 chars total
        let truncated = truncate_error(emoji, 6);  // Force truncation
        assert_eq!(truncated, "🔥🚀💥..."); // 3 chars + "..."
    }

    #[test]
    fn truncate_error_handles_multiline() {
        use super::truncate_error;

        // Only takes first line
        let multiline = "first line\nsecond line\nthird line";
        assert_eq!(truncate_error(multiline, 20), "first line");
    }

    #[test]
    fn setup_exe_resolver_has_no_relative_paths() {
        use super::setup_exe_search_candidates;

        let candidates = setup_exe_search_candidates();

        // Verify no relative paths (cwd-dependent)
        for candidate in &candidates {
            assert!(
                !candidate.starts_with(".\\") && !candidate.starts_with("./"),
                "Found relative path: {}", candidate
            );
            assert!(
                !candidate.starts_with("sources"),
                "Found cwd-relative path: {}", candidate
            );
            // All paths should be absolute (start with drive letter)
            assert!(
                candidate.chars().nth(1) == Some(':'),
                "Path should be absolute: {}", candidate
            );
        }
    }

    #[test]
    fn setup_exe_resolver_skips_x_drive() {
        use super::setup_exe_search_candidates;

        let candidates = setup_exe_search_candidates();

        // X: is WinPE runtime volume, should never be searched
        for candidate in &candidates {
            assert!(
                !candidate.starts_with("X:"),
                "Resolver should skip X: drive: {}", candidate
            );
        }
    }

    #[test]
    fn setup_exe_resolver_prefers_sources_over_root() {
        use super::setup_exe_search_candidates;

        let candidates = setup_exe_search_candidates();

        // Find where sources paths end and root paths begin
        let first_root_idx = candidates.iter().position(|c| !c.contains("sources"));
        let last_sources_idx = candidates.iter().rposition(|c| c.contains("sources"));

        // All sources paths should come before all root paths
        if let (Some(first_root), Some(last_sources)) = (first_root_idx, last_sources_idx) {
            assert!(
                last_sources < first_root,
                "All sources paths should come before root paths. Last sources at {}, first root at {}",
                last_sources, first_root
            );
        }
    }

    #[test]
    fn setup_exe_resolver_prefers_media_drives_over_system() {
        use super::setup_exe_search_candidates;

        let candidates = setup_exe_search_candidates();

        // Find first C: path and first D: path
        let first_c_idx = candidates.iter().position(|c| c.starts_with("C:"));
        let first_d_idx = candidates.iter().position(|c| c.starts_with("D:"));

        // D: (media drive) should come before C: (system drive) in sources paths
        if let (Some(c_idx), Some(d_idx)) = (first_c_idx, first_d_idx) {
            assert!(
                d_idx < c_idx,
                "Media drives (D:) should be checked before system drive (C:). D: at {}, C: at {}",
                d_idx, c_idx
            );
        }
    }
}

#[allow(dead_code)]
fn launch_tool_process(path: &str) -> Result<()> {
    info!("Launching tool at: {}", path);
    
    let path_buf = std::path::PathBuf::from(path);
    let extension = path_buf.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    
    match extension {
        "exe" | "com" => {
            #[cfg(target_os = "windows")]
            {
                Command::new(path).spawn()?;
            }
            #[cfg(not(target_os = "windows"))]
            {
                info!("Would launch Windows executable: {}", path);
            }
        }
        "bat" | "cmd" => {
            #[cfg(target_os = "windows")]
            {
                Command::new("cmd")
                    .args(&["/c", path])
                    .spawn()?;
            }
            #[cfg(not(target_os = "windows"))]
            {
                info!("Would run batch script: {}", path);
            }
        }
        "ps1" => {
            #[cfg(target_os = "windows")]
            {
                Command::new("powershell")
                    .args(&["-ExecutionPolicy", "Bypass", "-File", path])
                    .spawn()?;
            }
            #[cfg(not(target_os = "windows"))]
            {
                info!("Would run PowerShell script: {}", path);
            }
        }
        "au3" => {
            // AutoIt script - would need AutoIt3.exe
            info!("AutoIt script detected: {}", path);
        }
        "reg" => {
            #[cfg(target_os = "windows")]
            {
                Command::new("reg")
                    .args(&["import", path])
                    .spawn()?;
            }
            #[cfg(not(target_os = "windows"))]
            {
                info!("Would import registry file: {}", path);
            }
        }
        _ => {
            info!("Unknown file type for: {}", path);
        }
    }
    
    Ok(())
}

#[allow(dead_code)]
fn run_script_process(path: &str) -> Result<()> {
    // Same as launch_tool_process but with different logging
    info!("Running script: {}", path);
    launch_tool_process(path)
}

#[allow(dead_code)]
fn run_pe_autorun_scripts() -> Result<()> {
    info!("Running PE autorun scripts");
    
    let config = ConfigManager::load_default()?;
    let detector = ToolDetector::new(&config.tools);
    let tools = detector.detect_tools(".")?;
    
    // Find and run all PEAutoRun tools
    for tool in tools {
        if matches!(tool.category, ToolCategory::PEAutoRun) && tool.auto_run {
            info!("Auto-running: {}", tool.path.display());
            launch_tool_process(&tool.path.to_string_lossy())?;
        }
    }
    
    Ok(())
}
