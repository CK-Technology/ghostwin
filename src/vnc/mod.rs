use anyhow::Result;
use std::process::{Command, Child};
use std::path::Path;
use tracing::{info, error, debug};
use crate::cli::GhostwinConfig;

pub struct VncManager {
    config: GhostwinConfig,
    server_process: Option<Child>,
}

impl VncManager {
    pub fn new(config: GhostwinConfig) -> Self {
        Self {
            config,
            server_process: None,
        }
    }
    
    pub fn start_server(&mut self) -> Result<()> {
        if self.server_process.is_some() {
            info!("VNC server is already running");
            return Ok(());
        }
        
        info!("Starting VNC server on port {}", self.config.security.vnc_port);
        
        // Configure VNC password if provided
        if let Some(ref password) = self.config.security.vnc_password {
            if !password.is_empty() {
                self.set_vnc_password(password)?;
            }
        }
        
        // Start TightVNC server
        let vnc_server_path = self.find_vnc_server()?;
        
        #[cfg(target_os = "windows")]
        {
            let mut cmd = Command::new(&vnc_server_path);
            cmd.args([
                "-run",
                "-service",
                &format!("-rfbport={}", self.config.security.vnc_port)
            ]);
            
            // Add password authentication if configured
            if let Some(ref password) = self.config.security.vnc_password {
                if !password.is_empty() {
                    cmd.args(["-authentication", "1"]);
                }
            }
            
            let child = cmd.spawn()?;
            self.server_process = Some(child);
            info!("VNC server started successfully");
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            info!("VNC server would start: {} with port {}", 
                vnc_server_path.display(), self.config.security.vnc_port);
            // Simulate a running process for cross-platform compatibility
        }
        
        Ok(())
    }
    
    pub fn stop_server(&mut self) -> Result<()> {
        info!("Stopping VNC server");
        
        #[cfg(target_os = "windows")]
        {
            // Try to gracefully terminate first
            if let Some(mut child) = self.server_process.take() {
                match child.try_wait() {
                    Ok(Some(_)) => {
                        info!("VNC server process already terminated");
                    }
                    Ok(None) => {
                        // Process is still running, kill it
                        if let Err(e) = child.kill() {
                            error!("Failed to kill VNC server process: {}", e);
                        } else {
                            info!("VNC server process terminated");
                        }
                    }
                    Err(e) => {
                        error!("Failed to check VNC server process status: {}", e);
                    }
                }
            }
            
            // Also kill any remaining TightVNC processes
            let _ = Command::new("taskkill")
                .args(["/f", "/im", "tvnserver.exe"])
                .output();
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            info!("VNC server would be stopped");
        }
        
        self.server_process = None;
        Ok(())
    }
    
    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut child) = self.server_process {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited
                    self.server_process = None;
                    false
                }
                Ok(None) => {
                    // Process is still running
                    true
                }
                Err(_) => {
                    // Error checking status, assume not running
                    self.server_process = None;
                    false
                }
            }
        } else {
            false
        }
    }
    
    pub fn get_connection_info(&self) -> VncConnectionInfo {
        VncConnectionInfo {
            port: self.config.security.vnc_port,
            password: self.config.security.vnc_password.clone().unwrap_or_default(),
            ip_addresses: self.get_local_ip_addresses(),
        }
    }
    
    fn find_vnc_server(&self) -> Result<std::path::PathBuf> {
        // Look for TightVNC server in expected locations
        let possible_paths = [
            "tools/remote_access/vnc/tvnserver.exe",
            "pe_autorun/services/vnc_server/vncserver/tvnserver.exe",
            "C:\\Program Files\\TightVNC\\tvnserver.exe",
            "C:\\Program Files (x86)\\TightVNC\\tvnserver.exe",
        ];
        
        for path_str in &possible_paths {
            let path = Path::new(path_str);
            if path.exists() {
                debug!("Found VNC server at: {}", path.display());
                return Ok(path.to_path_buf());
            }
        }
        
        Err(anyhow::anyhow!("VNC server executable not found. Looked in: {:?}", possible_paths))
    }
    
    fn set_vnc_password(&self, password: &str) -> Result<()> {
        info!("Configuring VNC password");
        
        #[cfg(target_os = "windows")]
        {
            // Use VNC password utility if available
            let password_tool = Path::new("pe_autorun/services/vnc_server/vncserver/vncpassword/vncpassword.exe");
            if password_tool.exists() {
                let output = Command::new(password_tool)
                    .arg(password)
                    .output()?;
                
                if output.status.success() {
                    info!("VNC password configured successfully");
                } else {
                    error!("Failed to set VNC password: {}", String::from_utf8_lossy(&output.stderr));
                }
            } else {
                debug!("VNC password tool not found, password will be set via registry");
                // Could implement registry-based password setting here
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            info!("Would configure VNC password: {}", password);
        }
        
        Ok(())
    }
    
    fn get_local_ip_addresses(&self) -> Vec<String> {
        let mut addresses = Vec::new();
        
        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = Command::new("ipconfig").output() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.contains("IPv4 Address") && line.contains(":") {
                        if let Some(ip) = line.split(':').nth(1) {
                            let ip = ip.trim();
                            if !ip.starts_with("127.") && !ip.starts_with("169.254.") {
                                addresses.push(ip.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = Command::new("hostname").arg("-I").output() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for ip in output_str.split_whitespace() {
                    if !ip.starts_with("127.") && !ip.starts_with("169.254.") {
                        addresses.push(ip.to_string());
                    }
                }
            }
        }
        
        if addresses.is_empty() {
            addresses.push("localhost".to_string());
        }
        
        addresses
    }
    
    pub fn get_viewer_command(&self) -> Option<String> {
        // Look for TightVNC viewer
        let viewer_paths = [
            "tools/network/vnchelper/tvnviewer.exe",
            "C:\\Program Files\\TightVNC\\tvnviewer.exe",
            "C:\\Program Files (x86)\\TightVNC\\tvnviewer.exe",
        ];
        
        for path_str in &viewer_paths {
            let path = Path::new(path_str);
            if path.exists() {
                let connection_info = self.get_connection_info();
                if let Some(ip) = connection_info.ip_addresses.first() {
                    return Some(format!("{} {}:{}", path.display(), ip, connection_info.port));
                }
            }
        }
        
        None
    }
}

impl Drop for VncManager {
    fn drop(&mut self) {
        if let Err(e) = self.stop_server() {
            error!("Failed to stop VNC server during cleanup: {}", e);
        }
    }
}

#[derive(Debug, Clone)]
pub struct VncConnectionInfo {
    pub port: u16,
    pub password: String,
    pub ip_addresses: Vec<String>,
}

impl VncConnectionInfo {
    pub fn get_connection_string(&self) -> String {
        if let Some(ip) = self.ip_addresses.first() {
            format!("{}:{}", ip, self.port)
        } else {
            format!("localhost:{}", self.port)
        }
    }
}