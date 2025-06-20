use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use tracing::debug;
use serde::{Deserialize, Serialize};
use crate::cli::ToolsConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedTool {
    pub name: String,
    pub path: PathBuf,
    pub category: ToolCategory,
    pub executable: bool,
    pub hidden: bool,
    pub auto_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolCategory {
    Tool,       // Tools folder - manually runnable
    PEAutoRun,  // PEAutoRun folder - runs automatically on PE start
    Logon,      // Logon folder - runs after Windows installation
}

pub struct ToolDetector {
    config: ToolsConfig,
}

impl ToolDetector {
    pub fn new(config: &ToolsConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
    
    pub fn scan_tools(&self) -> Result<Vec<PathBuf>> {
        let mut tool_dirs = Vec::new();
        
        if self.config.auto_detect {
            // Scan for folder-based tools
            tool_dirs.extend(self.scan_folder_structure()?);
        }
        
        // Add explicitly configured folders
        for folder in &self.config.folders {
            let path = PathBuf::from(folder);
            if path.exists() && path.is_dir() {
                tool_dirs.push(path);
            }
        }
        
        Ok(tool_dirs)
    }
    
    pub fn detect_tools<P: AsRef<Path>>(&self, base_path: P) -> Result<Vec<DetectedTool>> {
        let mut tools = Vec::new();
        
        for folder_name in &self.config.folders {
            let folder_path = if folder_name.starts_with('/') || (folder_name.len() > 1 && folder_name.chars().nth(1) == Some(':')) {
                // Absolute path
                PathBuf::from(folder_name)
            } else {
                // Relative path - join with base_path
                base_path.as_ref().join(folder_name)
            };
            
            if !folder_path.exists() {
                debug!("Folder does not exist: {}", folder_path.display());
                continue;
            }
            
            let category = match folder_name.as_str() {
                name if name.contains("tools") => ToolCategory::Tool,
                name if name.contains("pe_autorun") => ToolCategory::PEAutoRun,
                name if name.contains("scripts") => ToolCategory::Logon,
                _ => ToolCategory::Tool,
            };
            
            debug!("Scanning folder: {} as {:?}", folder_path.display(), category);
            tools.extend(self.scan_folder(&folder_path, category)?);
        }
        
        // Also scan all drives for matching folder patterns
        if self.config.auto_detect {
            tools.extend(self.scan_all_drives()?);
        }
        
        Ok(tools)
    }
    
    fn scan_folder_structure(&self) -> Result<Vec<PathBuf>> {
        let mut dirs = Vec::new();
        
        // Look for the concept structure
        let concept_helper = Path::new("concept/windows-setup-helper-master/Helper");
        if concept_helper.exists() {
            for folder_name in &self.config.folders {
                let folder_path = concept_helper.join(folder_name);
                if folder_path.exists() {
                    dirs.push(folder_path);
                }
            }
        }
        
        // Look in current directory
        for folder_name in &self.config.folders {
            let folder_path = PathBuf::from(folder_name);
            if folder_path.exists() {
                dirs.push(folder_path);
            }
        }
        
        Ok(dirs)
    }
    
    fn scan_folder(&self, folder_path: &Path, category: ToolCategory) -> Result<Vec<DetectedTool>> {
        let mut tools = Vec::new();
        
        for entry in WalkDir::new(folder_path).max_depth(3) {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                // Skip hidden files (starting with .)
                let hidden = file_name.starts_with('.');
                
                // Check if it's an executable type
                let executable = self.is_executable(path);
                
                // Check if it should auto-run
                let auto_run = matches!(category, ToolCategory::PEAutoRun);
                
                if executable || self.is_script(path) {
                    tools.push(DetectedTool {
                        name: file_name.to_string(),
                        path: path.to_path_buf(),
                        category: category.clone(),
                        executable,
                        hidden,
                        auto_run,
                    });
                    
                    debug!("Detected tool: {} at {}", file_name, path.display());
                }
            }
        }
        
        Ok(tools)
    }
    
    fn scan_all_drives(&self) -> Result<Vec<DetectedTool>> {
        let mut tools = Vec::new();
        
        #[cfg(target_os = "windows")]
        {
            use std::ffi::CString;
            use winapi::um::fileapi::GetLogicalDrives;
            
            let drives = unsafe { GetLogicalDrives() };
            
            for i in 0..26 {
                if (drives >> i) & 1 != 0 {
                    let drive_letter = (b'A' + i) as char;
                    let drive_path = format!("{}:\\", drive_letter);
                    
                    for folder_name in &self.config.folders {
                        let folder_path = Path::new(&drive_path).join("Helper").join(folder_name);
                        if folder_path.exists() {
                            let category = match folder_name.as_str() {
                                name if name.starts_with("Tools") => ToolCategory::Tool,
                                name if name.starts_with("PEAutoRun") => ToolCategory::PEAutoRun,
                                name if name.starts_with("Logon") => ToolCategory::Logon,
                                _ => ToolCategory::Tool,
                            };
                            
                            if let Ok(drive_tools) = self.scan_folder(&folder_path, category) {
                                tools.extend(drive_tools);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(tools)
    }
    
    fn is_executable(&self, path: &Path) -> bool {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("exe") | Some("com") | Some("bat") | Some("cmd") => true,
            _ => false,
        }
    }
    
    fn is_script(&self, path: &Path) -> bool {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("au3") | Some("ps1") | Some("reg") | Some("vbs") => true,
            _ => false,
        }
    }
    
    pub fn load_options_file<P: AsRef<Path>>(&self, folder_path: P) -> Result<ToolOptions> {
        let options_file = folder_path.as_ref().join(".Options.txt");
        if !options_file.exists() {
            return Ok(ToolOptions::default());
        }
        
        let content = std::fs::read_to_string(&options_file)?;
        let mut options = ToolOptions::default();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if line.eq_ignore_ascii_case("CheckAll") {
                options.check_all = true;
            } else if line.eq_ignore_ascii_case("CollapseTree") {
                options.collapse_tree = true;
            } else {
                // Individual file to check by default
                options.default_checked.push(line.to_string());
            }
        }
        
        Ok(options)
    }
}

#[derive(Debug, Default)]
pub struct ToolOptions {
    pub check_all: bool,
    pub collapse_tree: bool,
    pub default_checked: Vec<String>,
}