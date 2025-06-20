use clap::{Parser, Subcommand};
use anyhow::Result;
use tracing::info;
use tracing_subscriber;

mod cli;
mod wim;
mod config;
mod tools;
mod utils;
mod vnc;
mod executor;

use cli::*;

#[derive(Parser)]
#[command(name = "ghostwin")]
#[command(about = "Modern Windows deployment toolkit with WinPE integration")]
#[command(version = "0.2.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a custom Windows ISO with WinPE integration
    Build(BuildArgs),
    /// Launch the WinPE GUI interface
    Gui,
    /// Validate configuration and tools
    Validate,
    /// Show detected tools and scripts
    Tools,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    let log_level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };
    
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();

    info!("GhostWin v{} starting", env!("CARGO_PKG_VERSION"));

    match cli.command {
        Commands::Build(args) => {
            info!("Building Windows ISO with WinPE integration");
            cli::build::execute(args).await?;
        }
        Commands::Gui => {
            info!("Launching WinPE GUI interface");
            cli::gui::execute().await?;
        }
        Commands::Validate => {
            info!("Validating configuration");
            cli::validate::execute().await?;
        }
        Commands::Tools => {
            info!("Scanning for tools and scripts");
            cli::tools::execute().await?;
        }
    }

    Ok(())
}
