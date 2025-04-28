pub mod git_related;
pub mod hooksmith;
pub mod utils;

use std::path::Path;

use clap::{Parser, Subcommand};
use hooksmith::Hooksmith;
use serde::Deserialize;

/// Root directory of a Git repository.
pub const GIT_ROOT: &str = ".git";

/// Commands enum for hooksmith CLI.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Install all hooks listed in the config file
    #[command(about = "Install all hooks listed in the config file")]
    Install,

    /// Run a specific hook
    #[command(about = "Run a specific hook")]
    Run {
        /// Name of the hook to run
        hook_name: String,
    },

    /// Uninstall hooks
    #[command(about = "Uninstall hooks")]
    Uninstall {
        /// Optional name of the hook to uninstall. If not provided, all hooks will be uninstalled.
        #[arg(default_value = None)]
        hook_name: Option<String>,
    },

    /// Compare installed hooks with the configuration file
    #[command(about = "Compare installed hooks with configuration file")]
    Compare,

    /// Validate hooks configuration
    #[command(about = "Validate hooks in configuration file against standard Git hooks")]
    Validate,
}

/// # `Cli`
/// Command line interface structure for hooksmith.
#[derive(Parser)]
#[command(about = "A trivial Git hooks utility.")]
#[command(author = "@TomPlanche")]
#[command(name = "hooksmith")]
pub struct Cli {
    /// Command to execute
    #[command(subcommand)]
    pub command: Command,

    /// Path to the hooksmith.yaml file
    #[arg(short, long, default_value_t = String::from("hooksmith.yaml"))]
    pub config_path: String,

    /// Whether to print verbose output
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Whether to perform a dry run
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}

/// Configuration structure for hooksmith.
#[derive(Deserialize)]
pub struct Config {
    #[serde(flatten)]
    hooks: std::collections::HashMap<String, Hook>,
}

/// Hook structure for hooksmith.
#[derive(Deserialize)]
pub struct Hook {
    commands: Vec<String>,
}

/// # `init`
/// Initialize Hooksmith by reading the configuration file and installing hooks.
///
/// ## Arguments
/// * `config_path` - Path to the configuration file
///
/// ## Errors
/// * If the configuration file cannot be read or parsed
///
/// ## Returns
/// * `Config` - Parsed configuration file
pub fn init(config_path: &Path) -> std::io::Result<()> {
    let hs = Hooksmith::new_from_config(config_path, false, false)?;

    hs.install_hooks()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        // Test basic command parsing
        let args = vec!["hooksmith", "install"];
        let cli = Cli::parse_from(args);

        match cli.command {
            Command::Install => {}
            _ => panic!("Expected Install command"),
        }

        // Test with arguments
        let args = vec!["hooksmith", "run", "pre-commit"];
        let cli = Cli::parse_from(args);

        match cli.command {
            Command::Run { hook_name } => assert_eq!(hook_name, "pre-commit"),
            _ => panic!("Expected Run command with hook_name=pre-commit"),
        }
    }
}
