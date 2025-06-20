use anyhow::{Result, Context};
use std::path::Path;
use crate::cli::GhostwinConfig;

pub struct ConfigManager;

impl ConfigManager {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<GhostwinConfig> {
        let content = std::fs::read_to_string(path.as_ref())
            .context("Failed to read configuration file")?;
        
        let config: GhostwinConfig = if path.as_ref().extension().and_then(|s| s.to_str()) == Some("json") {
            serde_json::from_str(&content)
                .context("Failed to parse JSON configuration")?
        } else {
            toml::from_str(&content)
                .context("Failed to parse TOML configuration")?
        };
        
        Self::validate_config(&config)?;
        Ok(config)
    }
    
    pub fn load_default() -> Result<GhostwinConfig> {
        // Look for ghostwin.toml or ghostwin.json in current directory
        if Path::new("ghostwin.toml").exists() {
            Self::load_from_file("ghostwin.toml")
        } else if Path::new("ghostwin.json").exists() {
            Self::load_from_file("ghostwin.json")
        } else {
            Ok(GhostwinConfig::default())
        }
    }
    
    pub fn save_to_file<P: AsRef<Path>>(config: &GhostwinConfig, path: P) -> Result<()> {
        let content = if path.as_ref().extension().and_then(|s| s.to_str()) == Some("json") {
            serde_json::to_string_pretty(config)
                .context("Failed to serialize configuration to JSON")?
        } else {
            toml::to_string_pretty(config)
                .context("Failed to serialize configuration to TOML")?
        };
        
        std::fs::write(path.as_ref(), content)
            .context("Failed to write configuration file")?;
        
        Ok(())
    }
    
    pub fn create_default_config<P: AsRef<Path>>(path: P) -> Result<()> {
        let config = GhostwinConfig::default();
        Self::save_to_file(&config, path)?;
        Ok(())
    }
    
    pub async fn load_config(config_path: Option<String>) -> Result<GhostwinConfig> {
        match config_path {
            Some(path) => Self::load_from_file(path),
            None => Self::load_default(),
        }
    }
    
    fn validate_config(config: &GhostwinConfig) -> Result<()> {
        // Validate WIM index format
        if config.iso.wim_index.is_empty() {
            return Err(anyhow::anyhow!("WIM index cannot be empty"));
        }
        
        // Validate VNC port range
        if config.security.vnc_port == 0 || config.security.vnc_port > 65535 {
            return Err(anyhow::anyhow!("VNC port must be between 1 and 65535"));
        }
        
        // Validate resolution format if specified
        if let Some(ref resolution) = config.winpe.set_resolution {
            if !resolution.contains('x') {
                return Err(anyhow::anyhow!("Resolution must be in format 'WIDTHxHEIGHT' (e.g., '1024x768')"));
            }
            
            let parts: Vec<&str> = resolution.split('x').collect();
            if parts.len() != 2 {
                return Err(anyhow::anyhow!("Invalid resolution format: {}", resolution));
            }
            
            for part in parts {
                if part.parse::<u32>().is_err() {
                    return Err(anyhow::anyhow!("Invalid resolution value: {}", part));
                }
            }
        }
        
        // Validate tool folder names
        for folder in &config.tools.folders {
            if folder.is_empty() {
                return Err(anyhow::anyhow!("Tool folder name cannot be empty"));
            }
            
            // Check for invalid characters
            let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
            if folder.chars().any(|c| invalid_chars.contains(&c)) {
                return Err(anyhow::anyhow!("Tool folder name contains invalid characters: {}", folder));
            }
        }
        
        // Validate password hash format if provided
        if let Some(ref hash) = config.security.password_hash {
            if !hash.is_empty() && hash.len() != 64 {
                return Err(anyhow::anyhow!("Password hash must be a 64-character SHA-256 hash"));
            }
            
            if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(anyhow::anyhow!("Password hash must contain only hexadecimal characters"));
            }
        }
        
        // Validate VNC password if provided
        if let Some(ref password) = config.security.vnc_password {
            if password.len() < 6 {
                return Err(anyhow::anyhow!("VNC password must be at least 6 characters long"));
            }
            if password.len() > 8 {
                return Err(anyhow::anyhow!("VNC password must be no longer than 8 characters"));
            }
        }
        
        // Validate WinPE package names
        for package in &config.winpe.packages {
            if !package.starts_with("WinPE-") {
                return Err(anyhow::anyhow!("Invalid WinPE package name: {}. Must start with 'WinPE-'", package));
            }
        }
        
        Ok(())
    }
}