use anyhow::Result;
use tracing::{info, error};
use std::thread;
use std::process::Command;
use std::sync::{Arc, Mutex};
use crate::config::ConfigManager;
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
    ui.set_vnc_enabled(config.security.vnc_enabled);
    ui.set_vnc_status("Disconnected".into());
    
    // Set up callbacks
    let ui_weak = ui.as_weak();
    ui.on_start_normal_install(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_current_mode("install".into());
            
            // Start normal installation in background thread
            thread::spawn(move || {
                if let Err(e) = start_windows_installation(false, None, None) {
                    error!("Normal installation failed: {}", e);
                }
            });
        }
    });
    
    let ui_weak = ui.as_weak();
    let executor_clone3 = script_executor.clone();
    let tools_clone3 = detected_tools.clone();
    ui.on_start_automated_install(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_current_mode("install".into());
            
            // Start automated installation in background thread
            let executor = executor_clone3.clone();
            let tools = tools_clone3.clone();
            thread::spawn(move || {
                if let Err(e) = start_windows_installation(true, Some(executor), Some(tools)) {
                    error!("Automated installation failed: {}", e);
                }
            });
        }
    });
    
    let ui_weak = ui.as_weak();
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
    
    let ui_weak = ui.as_weak();
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
    tools: Option<Vec<crate::tools::DetectedTool>>
) -> Result<()> {
    info!("Starting Windows installation (automated: {})", automated);
    
    if automated {
        if let (Some(executor), Some(tools)) = (executor, tools) {
            // Run automated installation with PE autorun scripts
            info!("Executing PE autorun scripts");
            match executor.execute_pe_autorun_scripts(&tools) {
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

fn run_script_process(path: &str) -> Result<()> {
    // Same as launch_tool_process but with different logging
    info!("Running script: {}", path);
    launch_tool_process(path)
}

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

