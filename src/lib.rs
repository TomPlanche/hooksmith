pub mod commands;

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use commands::Command;
use serde::Deserialize;

pub const GIT_ROOT: &str = ".git";
pub const GIT_HOOKS: &str = "hooks";

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

    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(flatten)]
    hooks: std::collections::HashMap<String, Hook>,
}

#[derive(Deserialize)]
pub struct Hook {
    commands: Vec<String>,
}

/// # `get_git_hooks_path`
/// Get the path to the Git hooks directory.
///
/// ## Returns
/// * `PathBuf` - Path to the Git hooks directory
#[must_use]
pub fn get_git_hooks_path() -> PathBuf {
    Path::new(GIT_ROOT).join(GIT_HOOKS)
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
    let git_hooks = get_git_hooks_path();

    git_root.exists() && git_hooks.exists()
}

/// # `read_config`
/// Read the configuration file and parse it into a Config struct.
///
/// ## Arguments
/// * `config_path` - Path to the configuration file
///
/// ## Panics
/// * If the configuration file cannot be read or parsed
///
/// ## Returns
/// * `Config` - Parsed configuration file
#[must_use]
pub fn read_config(config_path: &Path) -> Config {
    let config_string = fs::read_to_string(config_path).expect("Failed to read config file");

    serde_yaml::from_str(&config_string).expect("Failed to parse config file")
}
