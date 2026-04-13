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

pub struct ToolManager {
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
            if path.exists() && path.is_dir() && !tool_dirs.contains(&path) {
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
            
            let category = Self::category_for_folder_name(folder_name);
            
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
        #[cfg_attr(not(target_os = "windows"), allow(unused_mut))]
        let mut tools = Vec::new();
        
        #[cfg(target_os = "windows")]
        {
            use winapi::um::fileapi::GetLogicalDrives;
            
            let drives = unsafe { GetLogicalDrives() };
            
            for i in 0..26 {
                if (drives >> i) & 1 != 0 {
                    let drive_letter = (b'A' + i) as char;
                    let drive_path = format!("{}:\\", drive_letter);
                    
                    for folder_name in &self.config.folders {
                        let folder_path = Path::new(&drive_path).join("Helper").join(folder_name);
                        if folder_path.exists() {
                            let category = Self::category_for_folder_name(folder_name);
                            
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

    pub(crate) fn category_for_folder_name(folder_name: &str) -> ToolCategory {
        let leaf_name = Path::new(folder_name)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(folder_name);
        let normalized = leaf_name
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();

        if normalized.contains("peautorun") {
            ToolCategory::PEAutoRun
        } else if normalized.contains("logon") {
            ToolCategory::Logon
        } else {
            ToolCategory::Tool
        }
    }

    pub(crate) fn helper_destination_for_category(category: &ToolCategory) -> &'static str {
        match category {
            ToolCategory::Tool => "Helper/Tools",
            ToolCategory::PEAutoRun => "Helper/PEAutoRun",
            ToolCategory::Logon => "Helper/Logon",
        }
    }

    pub(crate) fn helper_destination_for_folder(folder_name: &str) -> &'static str {
        let category = Self::category_for_folder_name(folder_name);
        Self::helper_destination_for_category(&category)
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

impl ToolManager {
    pub fn new(config: &ToolsConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
    
    pub async fn scan_tools(&self) -> Result<Vec<DetectedTool>> {
        let detector = ToolDetector::new(&self.config);
        detector.detect_tools(".")
    }
}

#[derive(Debug, Default)]
pub struct ToolOptions {
    pub check_all: bool,
    pub collapse_tree: bool,
    pub default_checked: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::{ToolCategory, ToolDetector};
    use crate::cli::ToolsConfig;
    use tempfile::tempdir;

    #[test]
    fn classifies_default_tool_folders() {
        assert!(matches!(
            ToolDetector::category_for_folder_name("Tools"),
            ToolCategory::Tool
        ));
        assert!(matches!(
            ToolDetector::category_for_folder_name("PEAutoRun"),
            ToolCategory::PEAutoRun
        ));
        assert!(matches!(
            ToolDetector::category_for_folder_name("Logon"),
            ToolCategory::Logon
        ));
    }

    #[test]
    fn classifies_absolute_paths_by_leaf_folder() {
        assert!(matches!(
            ToolDetector::category_for_folder_name(r"C:\\Helper\\PEAutoRun"),
            ToolCategory::PEAutoRun
        ));
        assert!(matches!(
            ToolDetector::category_for_folder_name("/mnt/helper/Logon"),
            ToolCategory::Logon
        ));
    }

    #[test]
    fn maps_helper_destinations_by_category() {
        assert_eq!(ToolDetector::helper_destination_for_folder("Tools"), "Helper/Tools");
        assert_eq!(ToolDetector::helper_destination_for_folder("PEAutoRun"), "Helper/PEAutoRun");
        assert_eq!(ToolDetector::helper_destination_for_folder("Logon"), "Helper/Logon");
    }

    #[test]
    fn detects_tools_and_scripts_with_category_metadata() {
        let temp = tempdir().unwrap();
        let tools_dir = temp.path().join("Tools");
        let autorun_dir = temp.path().join("PEAutoRun");
        let logon_dir = temp.path().join("Logon");

        std::fs::create_dir_all(&tools_dir).unwrap();
        std::fs::create_dir_all(&autorun_dir).unwrap();
        std::fs::create_dir_all(&logon_dir).unwrap();

        std::fs::write(tools_dir.join("diskpart.exe"), "exe").unwrap();
        std::fs::write(autorun_dir.join("launch.ps1"), "ps1").unwrap();
        std::fs::write(logon_dir.join("finish.cmd"), "cmd").unwrap();
        std::fs::write(tools_dir.join("notes.txt"), "ignored").unwrap();

        let config = ToolsConfig {
            folders: vec!["Tools".into(), "PEAutoRun".into(), "Logon".into()],
            auto_detect: false,
        };
        let detector = ToolDetector::new(&config);

        let detected = detector.detect_tools(temp.path()).unwrap();
        assert_eq!(detected.len(), 3);

        let tool = detected.iter().find(|tool| tool.name == "diskpart.exe").unwrap();
        assert!(matches!(tool.category, ToolCategory::Tool));
        assert!(tool.executable);
        assert!(!tool.auto_run);

        let autorun = detected.iter().find(|tool| tool.name == "launch.ps1").unwrap();
        assert!(matches!(autorun.category, ToolCategory::PEAutoRun));
        assert!(!autorun.executable);
        assert!(autorun.auto_run);

        let logon = detected.iter().find(|tool| tool.name == "finish.cmd").unwrap();
        assert!(matches!(logon.category, ToolCategory::Logon));
        assert!(logon.executable);
        assert!(!logon.auto_run);
    }

    #[test]
    fn parses_options_file_flags_and_default_checked_entries() {
        let temp = tempdir().unwrap();
        let tools_dir = temp.path().join("Tools");
        std::fs::create_dir_all(&tools_dir).unwrap();
        std::fs::write(
            tools_dir.join(".Options.txt"),
            "# comment\nCheckAll\nCollapseTree\ninstaller.ps1\nhelper.cmd\n",
        )
        .unwrap();

        let config = ToolsConfig {
            folders: vec!["Tools".into()],
            auto_detect: false,
        };
        let detector = ToolDetector::new(&config);
        let options = detector.load_options_file(&tools_dir).unwrap();

        assert!(options.check_all);
        assert!(options.collapse_tree);
        assert_eq!(options.default_checked, vec!["installer.ps1", "helper.cmd"]);
    }
}
