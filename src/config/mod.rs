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
}