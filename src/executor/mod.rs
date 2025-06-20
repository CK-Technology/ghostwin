use anyhow::Result;
use std::process::{Command, Stdio};
use std::path::Path;
use tracing::{info, error, debug};
use crate::tools::{DetectedTool, ToolCategory};
use crate::cli::GhostwinConfig;

pub struct ScriptExecutor {
    config: GhostwinConfig,
}

impl ScriptExecutor {
    pub fn new(config: GhostwinConfig) -> Self {
        Self { config }
    }
    
    pub fn execute_tool(&self, tool: &DetectedTool) -> Result<ExecutionResult> {
        info!("Executing tool: {} at {}", tool.name, tool.path.display());
        
        let path_str = tool.path.to_string_lossy();
        let extension = tool.path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        match extension.as_str() {
            "exe" | "com" => self.execute_executable(&path_str),
            "bat" | "cmd" => self.execute_batch_script(&path_str),
            "ps1" => self.execute_powershell_script(&path_str),
            "au3" => self.execute_autoit_script(&path_str),
            "reg" => self.execute_registry_file(&path_str),
            "vbs" => self.execute_vbscript(&path_str),
            _ => {
                error!("Unsupported file type: {}", extension);
                Err(anyhow::anyhow!("Unsupported file type: {}", extension))
            }
        }
    }
    
    pub fn execute_pe_autorun_scripts(&self, tools: &[DetectedTool]) -> Result<Vec<ExecutionResult>> {
        info!("Executing PE autorun scripts");
        let mut results = Vec::new();
        
        for tool in tools {
            if matches!(tool.category, ToolCategory::PEAutoRun) && tool.auto_run {
                info!("Auto-running: {}", tool.path.display());
                match self.execute_tool(tool) {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        error!("Failed to execute autorun tool {}: {}", tool.name, e);
                        results.push(ExecutionResult {
                            tool_name: tool.name.clone(),
                            success: false,
                            exit_code: Some(-1),
                            stdout: String::new(),
                            stderr: format!("Execution failed: {}", e),
                            execution_time_ms: 0,
                        });
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    fn execute_executable(&self, path: &str) -> Result<ExecutionResult> {
        debug!("Executing executable: {}", path);
        
        #[cfg(target_os = "windows")]
        {
            let start_time = std::time::Instant::now();
            let output = Command::new(path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?;
            
            let execution_time = start_time.elapsed().as_millis() as u64;
            
            Ok(ExecutionResult {
                tool_name: Path::new(path).file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                success: output.status.success(),
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                execution_time_ms: execution_time,
            })
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            info!("Would execute Windows executable: {}", path);
            Ok(ExecutionResult {
                tool_name: Path::new(path).file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                success: true,
                exit_code: Some(0),
                stdout: "Simulated execution (not on Windows)".to_string(),
                stderr: String::new(),
                execution_time_ms: 0,
            })
        }
    }
    
    fn execute_batch_script(&self, path: &str) -> Result<ExecutionResult> {
        debug!("Executing batch script: {}", path);
        
        #[cfg(target_os = "windows")]
        {
            let start_time = std::time::Instant::now();
            let output = Command::new("cmd")
                .args(["/c", path])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?;
            
            let execution_time = start_time.elapsed().as_millis() as u64;
            
            Ok(ExecutionResult {
                tool_name: Path::new(path).file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                success: output.status.success(),
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                execution_time_ms: execution_time,
            })
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            info!("Would execute batch script: {}", path);
            Ok(ExecutionResult {
                tool_name: Path::new(path).file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                success: true,
                exit_code: Some(0),
                stdout: "Simulated batch execution (not on Windows)".to_string(),
                stderr: String::new(),
                execution_time_ms: 0,
            })
        }
    }
    
    fn execute_powershell_script(&self, path: &str) -> Result<ExecutionResult> {
        debug!("Executing PowerShell script: {}", path);
        
        #[cfg(target_os = "windows")]
        {
            let start_time = std::time::Instant::now();
            let output = Command::new("powershell")
                .args(["-ExecutionPolicy", "Bypass", "-File", path])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?;
            
            let execution_time = start_time.elapsed().as_millis() as u64;
            
            Ok(ExecutionResult {
                tool_name: Path::new(path).file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                success: output.status.success(),
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                execution_time_ms: execution_time,
            })
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            info!("Would execute PowerShell script: {}", path);
            Ok(ExecutionResult {
                tool_name: Path::new(path).file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                success: true,
                exit_code: Some(0),
                stdout: "Simulated PowerShell execution (not on Windows)".to_string(),
                stderr: String::new(),
                execution_time_ms: 0,
            })
        }
    }
    
    fn execute_autoit_script(&self, path: &str) -> Result<ExecutionResult> {
        debug!("Executing AutoIt script: {}", path);
        
        // Look for AutoIt executable
        let autoit_paths = [
            "C:\\Program Files (x86)\\AutoIt3\\AutoIt3.exe",
            "C:\\Program Files\\AutoIt3\\AutoIt3.exe",
            "autoit3.exe",
        ];
        
        #[cfg(target_os = "windows")]
        {
            for autoit_path in &autoit_paths {
                if Path::new(autoit_path).exists() {
                    let start_time = std::time::Instant::now();
                    let output = Command::new(autoit_path)
                        .arg(path)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .output()?;
                    
                    let execution_time = start_time.elapsed().as_millis() as u64;
                    
                    return Ok(ExecutionResult {
                        tool_name: Path::new(path).file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        success: output.status.success(),
                        exit_code: output.status.code(),
                        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                        execution_time_ms: execution_time,
                    });
                }
            }
            
            Err(anyhow::anyhow!("AutoIt3.exe not found in standard locations"))
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            info!("Would execute AutoIt script: {}", path);
            Ok(ExecutionResult {
                tool_name: Path::new(path).file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                success: true,
                exit_code: Some(0),
                stdout: "Simulated AutoIt execution (not on Windows)".to_string(),
                stderr: String::new(),
                execution_time_ms: 0,
            })
        }
    }
    
    fn execute_registry_file(&self, path: &str) -> Result<ExecutionResult> {
        debug!("Importing registry file: {}", path);
        
        #[cfg(target_os = "windows")]
        {
            let start_time = std::time::Instant::now();
            let output = Command::new("reg")
                .args(["import", path])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?;
            
            let execution_time = start_time.elapsed().as_millis() as u64;
            
            Ok(ExecutionResult {
                tool_name: Path::new(path).file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                success: output.status.success(),
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                execution_time_ms: execution_time,
            })
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            info!("Would import registry file: {}", path);
            Ok(ExecutionResult {
                tool_name: Path::new(path).file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                success: true,
                exit_code: Some(0),
                stdout: "Simulated registry import (not on Windows)".to_string(),
                stderr: String::new(),
                execution_time_ms: 0,
            })
        }
    }
    
    fn execute_vbscript(&self, path: &str) -> Result<ExecutionResult> {
        debug!("Executing VBScript: {}", path);
        
        #[cfg(target_os = "windows")]
        {
            let start_time = std::time::Instant::now();
            let output = Command::new("cscript")
                .args(["/nologo", path])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?;
            
            let execution_time = start_time.elapsed().as_millis() as u64;
            
            Ok(ExecutionResult {
                tool_name: Path::new(path).file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                success: output.status.success(),
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                execution_time_ms: execution_time,
            })
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            info!("Would execute VBScript: {}", path);
            Ok(ExecutionResult {
                tool_name: Path::new(path).file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                success: true,
                exit_code: Some(0),
                stdout: "Simulated VBScript execution (not on Windows)".to_string(),
                stderr: String::new(),
                execution_time_ms: 0,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub tool_name: String,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
}

impl ExecutionResult {
    pub fn summary(&self) -> String {
        if self.success {
            format!("✅ {} completed successfully ({}ms)", self.tool_name, self.execution_time_ms)
        } else {
            format!("❌ {} failed with exit code {:?} ({}ms)", 
                self.tool_name, self.exit_code, self.execution_time_ms)
        }
    }
}