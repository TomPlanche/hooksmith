use std::path::Path;

use clap::Parser;
use hooksmith::{
    Cli, GIT_ROOT,
    commands::{
        Command, compare_hooks, install_hooks, run_hook, uninstall_given_hook, uninstall_hooks,
        validate_hooks, validate_hooks_for_install,
    },
    read_config,
    utils::print_error,
};

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
            "Please create a configuration file or specify its location with --config-path.",
        );

        std::process::exit(1);
    }

    let config = read_config(config_path)?;

    if cli.dry_run {
        println!("🔄 DRY RUN MODE - No commands will be executed\n");
    }

    match cli.command {
        Command::Install => {
            validate_hooks_for_install(&config, cli.verbose)?;

            install_hooks(&config, cli.dry_run, cli.verbose)
        }
        Command::Uninstall { hook_name } => {
            if hook_name.is_none() {
                uninstall_hooks(&config, cli.dry_run, cli.verbose)
            } else {
                let hook_name = hook_name.unwrap();
                uninstall_given_hook(&config, &hook_name, cli.dry_run, cli.verbose)
            }
        }
        Command::Run { hook_name } => run_hook(&config, &hook_name, cli.dry_run, cli.verbose),
        Command::Compare => compare_hooks(&config, cli.verbose),
        Command::Validate => validate_hooks(&config, cli.verbose),
    }
}
