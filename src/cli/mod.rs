use anyhow::Result;
use clap::Args;
use serde::{Deserialize, Serialize};

pub mod build;
pub mod gui;
pub mod validate;
pub mod tools;
pub mod logon;
pub mod system_setup;

#[derive(Args, Debug, Clone)]
pub struct BuildArgs {
    /// Path to the source Windows ISO
    #[arg(short, long)]
    pub source_iso: String,
    
    /// Directory to extract the ISO to
    #[arg(short = 'd', long)]
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

    /// Verify ISO integrity after creation
    #[arg(long)]
    pub verify: bool,
}

#[derive(Args, Debug, Clone, Default)]
pub struct LogonArgs {
    /// Preview actions without modifying the host
    #[arg(long)]
    pub dry_run: bool,

    /// Apply host changes for this command
    #[arg(long)]
    pub force: bool,
}

#[derive(Args, Debug, Clone, Default)]
pub struct SystemSetupArgs {
    /// Preview actions without modifying the host
    #[arg(long)]
    pub dry_run: bool,

    /// Apply host changes for this command
    #[arg(long)]
    pub force: bool,
}

pub(crate) fn validate_host_change_mode(command_name: &str, dry_run: bool, force: bool) -> Result<()> {
    match (dry_run, force) {
        (true, true) => Err(anyhow::anyhow!(
            "{} accepts either --dry-run or --force, but not both",
            command_name
        )),
        (false, false) => Err(anyhow::anyhow!(
            "{} requires --dry-run to preview or --force to modify the host",
            command_name
        )),
        _ => Ok(()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostwinConfig {
    pub iso: IsoConfig,
    pub winpe: WinPEConfig,
    pub tools: ToolsConfig,
    pub phases: PhaseConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsoConfig {
    pub wim_index: String,
    pub mount_path: Option<String>,
    pub adk_path: Option<String>,
    pub helper_source: Option<String>,
    pub windows_overlay_source: Option<String>,
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
pub struct PhaseConfig {
    pub pe_system_setup_paths: Vec<String>,
    pub pe_driver_loader_paths: Vec<String>,
    pub post_install_logon_paths: Vec<String>,
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
                wim_index: "2".to_string(),
                mount_path: None,
                adk_path: None,
                helper_source: Some("concept/windows-setup-helper-master/Helper".to_string()),
                windows_overlay_source: Some("concept/windows-setup-helper-master/Windows".to_string()),
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
            phases: PhaseConfig {
                pe_system_setup_paths: vec!["pe_autorun/system_setup".to_string()],
                pe_driver_loader_paths: vec!["pe_autorun/drivers".to_string()],
                post_install_logon_paths: vec!["scripts/basic/registry/disable_auto_logon.reg".to_string()],
            },
            security: SecurityConfig {
                password_hash: None,
                access_secret: None,
                vnc_enabled: false,
                vnc_port: 5950,
                vnc_password: None,
            },
        }
    }
}
