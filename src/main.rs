mod cli;

use clap::Parser;
use cli::{Cli, Command};
use hooksmith::Hooksmith;
use std::path::Path;

/// Root directory of a Git repository.
const GIT_ROOT: &str = ".git";

fn main() -> std::io::Result<()> {
    if !Path::new(GIT_ROOT).exists() {
        format_error_message(
            ".git directory (or file for submodules) not found.",
            "Please ensure you're in a Git repository or submodule.",
        );

        std::process::exit(1);
    }

    let cli = Cli::parse();
    let config_path = Path::new(&cli.config_path);

    if !config_path.exists() {
        format_error_message(
            &format!("Could not find config file at: {}", config_path.display()),
            "The default configuration file is set to `./hooksmith.toml`. Please create a configuration file or specify its location with --config-path.",
        );

        std::process::exit(1);
    }

    let hs = Hooksmith::new_from_config(config_path, cli.dry_run, cli.verbose)?;

    match cli.command {
        Command::Install => {
            hs.validate_hooks_for_install()?;

            hs.install_hooks()
        }
        Command::Uninstall { hook_name } => {
            if hook_name.is_none() {
                hs.uninstall_hooks()
            } else {
                let hook_name = hook_name.unwrap();

                hs.uninstall_given_hook(&hook_name)
            }
        }
        Command::Run { hook_name } => hs.run_hook(&hook_name),
        Command::Compare => hs.compare_hooks(),
        Command::Validate => hs.validate_hooks(),
    }
}

/// # `format_error_message`
/// Formats a message without suggestion.
///
/// ## Arguments
/// * `title` - The title of the message.
/// * `details` - The details of the message.
///
/// ## Returns
/// * String - The formatted message.
fn format_error_message(title: &str, details: &str) -> String {
    format!("hooksmith error:\n{title}\n\n{details}")
}
