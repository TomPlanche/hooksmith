pub mod commands;

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use commands::{Command, install_hooks};
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
/// ## Panics
/// * If the `git` command fails to execute
///
/// ## Returns
/// * `PathBuf` - Path to the Git hooks directory
#[must_use]
pub fn get_git_hooks_path() -> PathBuf {
    // get the output of the `git rev-parse --git-path hooks` command
    let output = std::process::Command::new("git")
        .arg("rev-parse")
        .arg("--git-path")
        .arg("hooks")
        .output()
        .expect("Failed to execute git command");

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    PathBuf::from(path)
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

/// # `init`
/// Initialize Hooksmith by reading the configuration file and installing hooks.
///
/// ## Arguments
/// * `config_path` - Path to the configuration file
///
/// ## Returns
/// * `Config` - Parsed configuration file
pub fn init(config_path: &Path) {
    let config = read_config(config_path);
    let dry_run = false;
    let verbose = false;

    install_hooks(&config, dry_run, verbose);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_read_config() {
        // Create a temporary config file
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("hooksmith.yaml");
        let mut config_file = File::create(&config_path).unwrap();

        let config_content = r#"
pre-commit:
  commands:
    - "echo 'Running pre-commit hook'"
"#;
        config_file.write_all(config_content.as_bytes()).unwrap();

        // Read and parse the config
        let config = read_config(&config_path);

        // Verify the config was parsed correctly
        assert!(config.hooks.contains_key("pre-commit"));
        assert_eq!(config.hooks["pre-commit"].commands.len(), 1);
        assert_eq!(
            config.hooks["pre-commit"].commands[0],
            "echo 'Running pre-commit hook'"
        );
    }
}
