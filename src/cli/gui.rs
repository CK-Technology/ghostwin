use anyhow::Result;
use tracing::{info, error};
use std::thread;
use std::sync::{Arc, Mutex};
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

pub async fn execute() -> Result<()> {
    info!("🖥️ Launching WinPE GUI interface");
    
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
    let ui = GhostWinApp::new()?;
    
    // Set initial state
    let model = ModelRc::new(VecModel::from(slint_tools));
    ui.set_tools(model);
    ui.set_current_mode("menu".into());
    ui.set_vnc_enabled(false);
    ui.set_vnc_status("Disconnected".into());
    ui.set_install_progress(InstallProgress {
        current_step: "Idle".into(),
        progress: 0.0,
        completed: false,
        error: "".into(),
    });
    
    // Set up callbacks
    let ui_weak = ui.as_weak();
    let config_for_build = config.clone();
    ui.on_start_build(move |request| {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_current_mode("install".into());
            ui.set_install_progress(InstallProgress {
                current_step: "Starting media build...".into(),
                progress: 0.0,
                completed: false,
                error: "".into(),
            });

            let ui_weak = ui.as_weak();
            let config = config_for_build.clone();
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
                        Some(&move |progress| update_build_progress(progress_ui.clone(), progress.clone())),
                    )
                    .await
                });
                update_install_progress(final_ui, result);
            });
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_start_normal_install(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_current_mode("install".into());
            ui.set_install_progress(InstallProgress {
                current_step: "Starting installation...".into(),
                progress: 0.1,
                completed: false,
                error: "".into(),
            });
            
            // Start normal installation in background thread
            let ui_weak = ui.as_weak();
            thread::spawn(move || {
                let result = start_windows_installation(false, None, None);
                update_install_progress(ui_weak, result);
            });
        }
    });
    
    let ui_weak = ui.as_weak();
    let executor_clone3 = script_executor.clone();
    let config_clone3 = config.clone();
    ui.on_start_automated_install(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_current_mode("install".into());
            ui.set_install_progress(InstallProgress {
                current_step: "Starting automated PE workflow...".into(),
                progress: 0.1,
                completed: false,
                error: "".into(),
            });
            
            // Start automated installation in background thread
            let executor = executor_clone3.clone();
            let config = config_clone3.clone();
            let ui_weak = ui.as_weak();
            thread::spawn(move || {
                let result = start_windows_installation(true, Some(executor), Some(config));
                update_install_progress(ui_weak, result);
            });
        }
    });

    let executor_clone = script_executor.clone();
    let tools_clone = detected_tools.clone();
    ui.on_launch_tool(move |path| {
        info!("Launching tool: {}", path);
        
        // Find the tool in our detected tools list
        if let Some(tool) = tools_clone.iter().find(|t| t.path.to_string_lossy() == path.as_str()) {
            match executor_clone.execute_tool(tool) {
                Ok(result) => {
                    info!("{}", result.summary());
                    if !result.stdout.is_empty() {
                        info!("Tool output: {}", result.stdout);
                    }
                    if !result.stderr.is_empty() && !result.success {
                        error!("Tool error: {}", result.stderr);
                    }
                }
                Err(e) => {
                    error!("Failed to execute tool {}: {}", path, e);
                }
            }
        } else {
            error!("Tool not found in detected tools list: {}", path);
        }
    });

    let executor_clone2 = script_executor.clone();
    let tools_clone2 = detected_tools.clone();
    ui.on_run_script(move |path| {
        info!("Running script: {}", path);
        
        // Find the script in our detected tools list
        if let Some(tool) = tools_clone2.iter().find(|t| t.path.to_string_lossy() == path.as_str()) {
            match executor_clone2.execute_tool(tool) {
                Ok(result) => {
                    info!("{}", result.summary());
                    if !result.stdout.is_empty() {
                        info!("Script output: {}", result.stdout);
                    }
                    if !result.stderr.is_empty() && !result.success {
                        error!("Script error: {}", result.stderr);
                    }
                }
                Err(e) => {
                    error!("Failed to execute script {}: {}", path, e);
                }
            }
        } else {
            error!("Script not found in detected tools list: {}", path);
        }
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
                        ui.set_vnc_status("Error".into());
                    } else {
                        ui.set_vnc_enabled(false);
                        ui.set_vnc_status("Disconnected".into());
                    }
                } else {
                    info!("Starting VNC server");
                    match vnc.start_server() {
                        Ok(_) => {
                            ui.set_vnc_enabled(true);
                            let connection_info = vnc.get_connection_info();
                            ui.set_vnc_status(format!("Connected ({})", connection_info.get_connection_string()).into());
                        }
                        Err(e) => {
                            error!("Failed to start VNC server: {}", e);
                            ui.set_vnc_status("Error".into());
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
            ui.set_current_mode("menu".into());
        }
    });
    
    info!("Starting GUI main loop");
    ui.run()?;
    
    Ok(())
}

fn start_windows_installation(
    automated: bool, 
    executor: Option<Arc<ScriptExecutor>>, 
    config: Option<crate::cli::GhostwinConfig>
) -> Result<()> {
    info!("Starting Windows installation (automated: {})", automated);
    
    if automated {
        if let (Some(executor), Some(config)) = (executor, config) {
            let detector = ToolDetector::new(&config.tools);
            let detected_tools = detector.detect_tools(".")?;
            let phase_tools = crate::utils::resolve_detected_tools(&config.phases.pe_system_setup_paths, &detected_tools);

            info!("Executing configured PE system-setup scripts");
            match executor.execute_pe_autorun_scripts(&phase_tools) {
                Ok(results) => {
                    for result in results {
                        info!("{}", result.summary());
                    }
                }
                Err(e) => {
                    error!("Failed to execute PE autorun scripts: {}", e);
                }
            }
        }
    }
    
    // Launch Windows setup
    #[cfg(target_os = "windows")]
    {
        Command::new("setup.exe")
            .arg("/auto")
            .arg("upgrade")
            .spawn()?;
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        info!("Windows installation simulation (not on Windows)");
    }
    
    Ok(())
}

fn update_install_progress(ui_weak: slint::Weak<GhostWinApp>, result: Result<()>) {
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            match result {
                Ok(()) => {
                    ui.set_install_progress(InstallProgress {
                        current_step: "Installation command launched".into(),
                        progress: 1.0,
                        completed: true,
                        error: "".into(),
                    });
                }
                Err(e) => {
                    error!("Installation failed: {}", e);
                    ui.set_install_progress(InstallProgress {
                        current_step: "Installation failed".into(),
                        progress: 0.0,
                        completed: false,
                        error: e.to_string().into(),
                    });
                }
            }
        }
    });
}

fn update_build_progress(ui_weak: slint::Weak<GhostWinApp>, progress: BuildProgressState) {
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_install_progress(install_progress_from_build_progress(&progress));
        }
    });
}

#[allow(dead_code)]
fn install_progress_from_build_progress(progress: &BuildProgressState) -> InstallProgress {
    let fraction = if progress.total_steps == 0 {
        0.0
    } else {
        (progress.completed_steps + 1) as f32 / progress.total_steps as f32
    };
    let completed = progress.completed_steps + 1 >= progress.total_steps;

    InstallProgress {
        current_step: progress.current_step.into(),
        progress: fraction,
        completed,
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
        assert!(!install.completed);
        assert_eq!(install.error.as_str(), "");
    }

    #[test]
    fn marks_build_progress_complete_on_final_step() {
        let progress = BuildProgressState {
            current_step: "Step 10: Verifying ISO integrity",
            completed_steps: 9,
            total_steps: 10,
        };

        let install = install_progress_from_build_progress(&progress);
        assert_eq!(install.progress, 1.0);
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
