use std::{fs, path::Path};

use clap::Subcommand;

use crate::{Config, check_for_git_hooks, get_git_hooks_path};

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
}

/// # `install_hooks`
/// Install all hooks listed in the config file.
///
/// ## Panics
/// * If the `.git/hooks` directory cannot be created
///
/// ## Arguments
/// * `config` - Parsed configuration file
/// * `dry_run` - Whether to run the hook in dry run mode
/// * `verbose` - Whether to print verbose output
pub fn install_hooks(config: &Config, dry_run: bool, verbose: bool) {
    let git_hooks_path = get_git_hooks_path();

    if !check_for_git_hooks() {
        fs::create_dir_all(git_hooks_path).expect("Failed to create .git/hooks directory");
    }

    if verbose {
        println!("🪝 Installing hooks...");
    }

    for hook_name in config.hooks.keys() {
        install_hook(hook_name, dry_run, verbose);
    }
}

/// # `install_hook`
/// Install a single hook.
///
/// ## Panics
/// * If the `.git/hooks` directory cannot be created
/// * If the hook cannot be installed/given permission
///
/// ## Arguments
/// * `hook_name` - Name of the hook to install
/// * `dry_run` - Whether to run the hook in dry run mode
/// * `verbose` - Whether to print verbose output
pub fn install_hook(hook_name: &str, dry_run: bool, verbose: bool) {
    if verbose && !dry_run {
        println!("🪝 Installing {hook_name} hook...");
    }

    let git_hooks_path = get_git_hooks_path();

    if !git_hooks_path.exists() {
        if dry_run {
            println!("🪝 Skipping creation of .git/hooks directory in dry run mode");
        } else {
            if verbose {
                println!("  - Creating .git/hooks directory...");
            }

            fs::create_dir_all(&git_hooks_path).expect("Failed to create .git/hooks directory");
        }
    }

    let hook_path = format!("{}/{}", git_hooks_path.to_str().unwrap(), hook_name);

    /*
    - >/dev/null 2>&1
        This suppresses both stdout and stderr from the hooksmith -h check
    */
    let hook_content = format!(
        "#!/bin/sh\n
if hooksmith -h >/dev/null 2>&1
then
  exec hooksmith run {hook_name}
else
  cargo install hooksmith
  exec hooksmith run {hook_name}
fi"
    );

    if dry_run {
        println!("🪝 Skipping installation of {hook_name} hook in dry run mode");
    } else {
        fs::write(&hook_path, hook_content)
            .unwrap_or_else(|_| panic!("Failed to write hook script to {hook_path}"));

        if verbose {
            println!("  - Installing {hook_name} file...");
        }

        // Linux only
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mut permissions = fs::metadata(&hook_path)
                .expect("Failed to get file permissions")
                .permissions();

            permissions.set_mode(0o755);

            fs::set_permissions(&hook_path, permissions).expect("Failed to set file permissions");

            if verbose {
                println!("  - Setting file permissions...");
            }
        }
    }

    if verbose {
        println!("  ✅ Installed {hook_name} file");
    }
}

/// # `execute_command`
/// Executes a command.
///
/// ## Panics
/// * If a command cannot be executed
///
/// # Arguments
/// * `command` - The command to execute.
/// * `dry_run` - Whether to run the command in dry run mode.
fn execute_command(command: &str, dry_run: bool) -> std::io::Result<std::process::ExitStatus> {
    if dry_run {
        println!("🔍 Would execute: {command}");

        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;

            Ok(ExitStatusExt::from_raw(0))
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::ExitStatusExt;

            Ok(ExitStatusExt::from_raw(0))
        }
    } else {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .status()
    }
}

/// # `run_hook`
/// Runs a hook by executing its commands.
///
/// ## Panics
/// * If a command cannot be executed
///
/// # Arguments
/// * `config` - A reference to the configuration.
/// * `hook_name` - The name of the hook to run.
/// * `dry_run` - Whether to run the hook in dry run mode.
/// * `verbose` - Whether to print verbose output.
pub fn run_hook(config: &Config, hook_name: &str, dry_run: bool, verbose: bool) {
    if let Some(hook) = config.hooks.get(hook_name) {
        if verbose && !dry_run {
            println!("📋 Running Hook: {hook_name}");
        }

        for (idx, command_str) in hook.commands.iter().enumerate() {
            if dry_run {
                println!("Step {} of {}:", idx + 1, hook.commands.len());
                println!("  Command: {command_str}");
                println!(
                    "  Working directory: {:?}",
                    std::env::current_dir().unwrap()
                );
                println!();
                continue;
            }

            if verbose && !dry_run {
                println!("  - Running command: {command_str}");
            }

            match execute_command(command_str, dry_run) {
                Ok(status) if status.success() => {
                    if verbose && !dry_run {
                        println!("\n  ✅ Command completed successfully");
                    }
                }
                Ok(status) => {
                    let code = status.code().unwrap_or(1);
                    eprintln!("❌ Command failed with status code {code}");

                    std::process::exit(code);
                }
                Err(e) => {
                    eprintln!("❌ Failed to execute command: {e}");

                    std::process::exit(1);
                }
            }
        }

        if dry_run {
            println!(
                "🏁 Dry run completed. {} commands would be executed",
                hook.commands.len()
            );
        }
    } else {
        let possible_hooks = config.hooks.keys().collect::<Vec<_>>();

        eprintln!("No commands defined for hook '{hook_name}'");
        eprintln!("Possible hooks: {possible_hooks:?}");

        std::process::exit(1);
    }
}

/// # `uninstall_given_hook`
/// Uninstalls a single hook by removing its file.
///
/// ## Panics
/// * Panics if the command fails to remove the file.
///
/// # Arguments
/// * `config` - A reference to the configuration.
/// * `hook_name` - The name of the hook to run.
/// * `dry_run` - Whether to perform a dry run.
/// * `verbose` - Whether to print verbose output.
pub fn uninstall_given_hook(config: &Config, hook_name: &str, dry_run: bool, verbose: bool) {
    if config.hooks.contains_key(hook_name) {
        if verbose && !dry_run {
            println!("🗑️ Uninstalling hook: {hook_name}");
        }

        let hook_path = get_git_hooks_path().join(hook_name);

        if Path::new(&hook_path).exists() {
            if dry_run {
                println!("  🚧 Dry run: Would remove hook file: {hook_path:?}");
            } else {
                fs::remove_file(&hook_path)
                    .unwrap_or_else(|_| panic!("Failed to remove hook: {hook_name}"));
            }
        } else {
            println!("  ⚠️ No hook file found for {hook_name}");
        }
    } else {
        let possible_hooks = config.hooks.keys().collect::<Vec<_>>();

        eprintln!("No file found for hook '{hook_name}'");
        eprintln!("Possible hooks: {possible_hooks:?}");

        std::process::exit(1);
    }
}

/// # `uninstall_hooks`
/// Uninstalls all hooks by removing their files.
///
/// # Arguments
/// * `config` - A reference to the configuration.
/// * `dry_run` - Whether to perform a dry run.
/// * `verbose` - Whether to print verbose output.
pub fn uninstall_hooks(config: &Config, dry_run: bool, verbose: bool) {
    if verbose && !dry_run {
        println!("🗑️ Uninstalling all hooks");
    }

    for hook_name in config.hooks.keys() {
        uninstall_given_hook(config, hook_name, dry_run, verbose);
    }

    if verbose && !dry_run {
        println!(
            "🏁 Uninstallation completed: {} hooks removed",
            config.hooks.len()
        );
    }
}
