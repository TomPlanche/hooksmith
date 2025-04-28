use std::path::Path;

use clap::Parser;
use hooksmith::hooksmith::Hooksmith;
use hooksmith::{Cli, Command, GIT_ROOT, utils::print_error};

fn main() -> std::io::Result<()> {
    if !Path::new(GIT_ROOT).exists() {
        print_error(
            "Git repository not found",
            ".git directory (or file for submodules) not found.",
            "Please ensure you're in a Git repository or submodule.",
        );
        std::process::exit(1);
    }

    let cli = Cli::parse();
    let config_path = Path::new(&cli.config_path);

    if !config_path.exists() {
        print_error(
            "Configuration file not found",
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
