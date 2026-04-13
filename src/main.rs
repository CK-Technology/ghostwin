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
mod drivers;

use cli::*;

#[derive(Parser)]
#[command(name = "ghostwin")]
#[command(about = "Modern Windows deployment toolkit with WinPE integration")]
#[command(version = env!("CARGO_PKG_VERSION"))]
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
    /// Run post-install logon scripts
    Logon(LogonArgs),
    /// Run system setup tasks (before user logon)
    SystemSetup(SystemSetupArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    run_cli(cli).await
}

async fn run_cli(cli: Cli) -> Result<()> {
    
    // Initialize logging
    let log_level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };
    
    let _ = tracing_subscriber::fmt()
        .with_max_level(log_level)
        .try_init();

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
        Commands::Logon(args) => {
            info!("Running post-install logon scripts");
            cli::logon::execute(args).await?;
        }
        Commands::SystemSetup(args) => {
            info!("Running system setup tasks");
            cli::system_setup::execute(args).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Cli, Commands};
    use clap::Parser;

    #[test]
    fn parses_build_command_flags() {
        let cli = Cli::try_parse_from([
            "ghostwin",
            "build",
            "--source-iso",
            "src.iso",
            "--output-dir",
            "out",
            "--output-iso",
            "final.iso",
            "--verify",
            "--skip-packages",
        ])
        .unwrap();

        match cli.command {
            Commands::Build(args) => {
                assert_eq!(args.source_iso, "src.iso");
                assert_eq!(args.output_dir, "out");
                assert_eq!(args.output_iso, "final.iso");
                assert!(args.verify);
                assert!(args.skip_packages);
            }
            _ => panic!("expected build command"),
        }
    }

    #[test]
    fn parses_logon_dry_run_flag() {
        let cli = Cli::try_parse_from(["ghostwin", "logon", "--dry-run"]).unwrap();

        match cli.command {
            Commands::Logon(args) => {
                assert!(args.dry_run);
                assert!(!args.force);
            }
            _ => panic!("expected logon command"),
        }
    }

    #[test]
    fn parses_system_setup_force_flag() {
        let cli = Cli::try_parse_from(["ghostwin", "system-setup", "--force"]).unwrap();

        match cli.command {
            Commands::SystemSetup(args) => {
                assert!(args.force);
                assert!(!args.dry_run);
            }
            _ => panic!("expected system-setup command"),
        }
    }

    #[tokio::test]
    async fn run_cli_dispatches_logon_guardrail_error() {
        let cli = Cli::try_parse_from(["ghostwin", "logon"]).unwrap();
        let error = super::run_cli(cli).await.unwrap_err();
        assert!(error.to_string().contains("requires --dry-run to preview or --force"));
    }

    #[tokio::test]
    async fn run_cli_dispatches_system_setup_guardrail_error() {
        let cli = Cli::try_parse_from(["ghostwin", "system-setup"]).unwrap();
        let error = super::run_cli(cli).await.unwrap_err();
        assert!(error.to_string().contains("requires --dry-run to preview or --force"));
    }

    #[tokio::test]
    async fn run_cli_dispatches_logon_dry_run_success() {
        let cli = Cli::try_parse_from(["ghostwin", "logon", "--dry-run"]).unwrap();
        super::run_cli(cli).await.unwrap();
    }

    #[tokio::test]
    async fn run_cli_dispatches_system_setup_dry_run_success() {
        let cli = Cli::try_parse_from(["ghostwin", "system-setup", "--dry-run"]).unwrap();
        super::run_cli(cli).await.unwrap();
    }
}
