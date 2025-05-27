use clap::{Parser, Subcommand};

/// Commands enum for hooksmith CLI.
#[derive(Subcommand)]
pub(crate) enum Command {
    /// Compare installed hooks with the configuration file
    #[command(about = "Compare installed hooks with configuration file")]
    Compare,

    /// Install all hooks listed in the config file
    #[command(about = "Install all hooks listed in the config file")]
    Install,

    /// Run a specific hook
    #[command(about = "Run a specific hook")]
    Run {
        /// Names of the hooks to run
        #[arg(default_value = None)]
        hook_names: Option<Vec<String>>,

        /// Whether to use interactive selection
        #[arg(short, long, default_value_t = false)]
        interactive: bool,
    },

    /// Uninstall hooks
    #[command(about = "Uninstall hooks")]
    Uninstall {
        /// Optional name of the hook to uninstall. If not provided, all hooks will be uninstalled.
        #[arg(default_value = None)]
        hook_name: Option<String>,
    },

    /// Validate hooks configuration
    #[command(about = "Validate hooks in configuration file against standard Git hooks")]
    Validate,
}

/// Command line interface structure for hooksmith.
#[derive(Parser)]
#[command(about = "A trivial Git hooks utility.")]
#[command(author = "Tom Planche <tomplanche@proton.me>")]
#[command(name = "hooksmith")]
#[command(version)]
pub(crate) struct Cli {
    /// Command to execute
    #[command(subcommand)]
    pub(crate) command: Command,

    /// Path to the hooksmith.yaml file
    #[arg(short, long, default_value_t = String::from("hooksmith.yaml"))]
    pub(crate) config_path: String,

    /// Whether to print verbose output
    #[arg(short, long, default_value_t = false)]
    pub(crate) verbose: bool,

    /// Whether to perform a dry run
    #[arg(long, default_value_t = false)]
    pub(crate) dry_run: bool,
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
        let args = vec!["hooksmith", "run", "pre-commit", "pre-push"];
        let cli = Cli::parse_from(args);

        match cli.command {
            Command::Run {
                hook_names,
                interactive,
            } => {
                assert_eq!(
                    hook_names,
                    Some(vec!["pre-commit".to_string(), "pre-push".to_string()])
                );
                assert!(!interactive);
            }
            _ => panic!("Expected Run command with hook_names=[pre-commit, pre-push]"),
        }
    }
}
