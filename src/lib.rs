pub mod commands;

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use commands::Command;
use serde::Deserialize;

/// Root directory of a Git repository.
pub const GIT_ROOT: &str = ".git";

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

/// # `get_git_hooks_path`
/// Get the path to the Git hooks directory.
///
/// ## Errors
/// * If the `git` command fails to execute
///
/// ## Returns
/// * `PathBuf` - Path to the Git hooks directory
pub fn get_git_hooks_path() -> std::io::Result<PathBuf> {
    // get the output of the `git rev-parse --git-path hooks` command
    let output = std::process::Command::new("git")
        .arg("rev-parse")
        .arg("--git-path")
        .arg("hooks")
        .output()?;

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(PathBuf::from(path))
}

/// # `check_for_git_hooks`
/// Check if the current directory is a Git repository and if it has hooks.
///
/// ## Arguments
/// * `path` - Path to the directory to check
///
/// ## Returns
/// * `bool` - True if the directory is a Git repository with hooks, false otherwise
#[must_use]
pub fn check_for_git_hooks() -> bool {
    let git_root = Path::new(GIT_ROOT);
    let git_hooks = get_git_hooks_path().ok();

    git_root.exists() && git_hooks.is_some_and(|path| path.exists())
}

/// # `read_config`
/// Read the configuration file and parse it into a Config struct.
///
/// ## Arguments
/// * `config_path` - Path to the configuration file
///
/// ## Errors
/// * If the configuration file cannot be read or parsed
///
/// ## Returns
/// * `Config` - Parsed configuration file
pub fn read_config(config_path: &Path) -> std::io::Result<Config> {
    let config_string = fs::read_to_string(config_path)?;
    let config = serde_yaml::from_str(&config_string)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    Ok(config)
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
    let config = read_config(config_path)?;
    let dry_run = false;
    let verbose = false;

    commands::install_hooks(&config, dry_run, verbose)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_config() -> std::io::Result<()> {
        let config_path = Path::new("tests/fixtures/hooksmith.yaml");
        let config = read_config(config_path)?;

        assert!(config.hooks.contains_key("pre-commit"));
        assert!(config.hooks.contains_key("pre-push"));
        Ok(())
    }
}
