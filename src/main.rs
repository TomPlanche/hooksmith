mod cli;

use clap::Parser;
use cli::Command;
use hooksmith::{error::ConfigError, Hooksmith, Result};
use std::path::Path;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    let config_path = Path::new(&cli.config_path);

    if !config_path.exists() && !matches!(cli.command, Command::Init) {
        eprintln!(
            "{}",
            ConfigError::NotFound(config_path.to_str().unwrap().to_string())
        );

        std::process::exit(1);
    }

    if cli.command == Command::Init {
        return Hooksmith::init_interactive(config_path, cli.dry_run, cli.verbose);
    }

    let hs = Hooksmith::new_from_config(config_path, cli.dry_run, cli.verbose)?;

    match cli.command {
        Command::Compare => hs.compare_hooks(),
        Command::Init => Hooksmith::init_interactive(config_path, cli.dry_run, cli.verbose),
        Command::Install => {
            hs.validate_hooks_for_install()?;

            hs.install_hooks()
        }
        Command::Uninstall { hook_name } => {
            if let Some(item) = hook_name {
                hs.uninstall_given_hook(&item)
            } else {
                hs.uninstall_hooks()
            }
        }
        Command::Run {
            hook_names,
            interactive,
        } => {
            if hook_names.is_none() && !interactive {
                eprintln!("Error: Either provide hook names or use --interactive (-i) flag");
                std::process::exit(1);
            }

            hs.run_hook(hook_names.as_deref(), interactive)
        }
        Command::Validate => hs.validate_hooks(),
    }
}
