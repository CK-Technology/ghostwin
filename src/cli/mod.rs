use clap::Args;
use serde::{Deserialize, Serialize};

pub mod build;
pub mod gui;
pub mod validate;
pub mod tools;

#[derive(Args, Debug, Clone)]
pub struct BuildArgs {
    /// Path to the source Windows ISO
    #[arg(short, long)]
    pub source_iso: String,
    
    /// Directory to extract the ISO to
    #[arg(short, long)]
    pub output_dir: String,
    
    /// Path for the final ISO output
    #[arg(short = 'o', long)]
    pub output_iso: String,
    
    /// Additional files directory to inject
    #[arg(short, long)]
    pub extra_files: Option<String>,
    
    /// Skip WinPE package installation
    #[arg(long)]
    pub skip_packages: bool,
    
    /// Skip DPI fix registry modification
    #[arg(long)]
    pub skip_dpi_fix: bool,
    
    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostwinConfig {
    pub iso: IsoConfig,
    pub winpe: WinPEConfig,
    pub tools: ToolsConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsoConfig {
    pub wim_index: String,
    pub mount_path: Option<String>,
    pub adk_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinPEConfig {
    pub packages: Vec<String>,
    pub disable_dpi_scaling: bool,
    pub set_resolution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    pub folders: Vec<String>,
    pub auto_detect: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub password_hash: Option<String>,
    pub access_secret: Option<String>,
    pub vnc_enabled: bool,
    pub vnc_port: u16,
    pub vnc_password: Option<String>,
}

impl Default for GhostwinConfig {
    fn default() -> Self {
        Self {
            iso: IsoConfig {
                wim_index: "Microsoft Windows Setup (amd64)".to_string(),
                mount_path: None,
                adk_path: None,
            },
            winpe: WinPEConfig {
                packages: vec![
                    "WinPE-WMI".to_string(),
                    "WinPE-NetFX".to_string(),
                    "WinPE-Scripting".to_string(),
                    "WinPE-PowerShell".to_string(),
                ],
                disable_dpi_scaling: true,
                set_resolution: Some("1024x768".to_string()),
            },
            tools: ToolsConfig {
                folders: vec!["Tools".to_string(), "PEAutoRun".to_string(), "Logon".to_string()],
                auto_detect: true,
            },
            security: SecurityConfig {
                password_hash: None,
                access_secret: None,
                vnc_enabled: true,
                vnc_port: 5950,
                vnc_password: Some("vncwatch".to_string()),
            },
        }
    }
}