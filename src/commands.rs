use clap::Subcommand;
use std::fs;

use crate::{
    Config, check_for_git_hooks, get_git_hooks_path,
    utils::{format_list, print_error, print_success, print_warning},
};

const GIT_HOOKS: [&str; 28] = [
    "applypatch-msg",
    "pre-applypatch",
    "post-applypatch",
    "pre-commit",
    "pre-merge-commit",
    "prepare-commit-msg",
    "commit-msg",
    "post-commit",
    "pre-rebase",
    "post-checkout",
    "post-merge",
    "pre-push",
    "pre-receive",
    "update",
    "proc-receive",
    "post-receive",
    "post-update",
    "reference-transaction",
    "push-to-checkout",
    "pre-auto-gc",
    "post-rewrite",
    "sendemail-validate",
    "fsmonitor-watchman",
    "p4-changelist",
    "p4-prepare-changelist",
    "p4-post-changelist",
    "p4-pre-submit",
    "post-index-change",
];

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

/// # `validate_hooks_for_install`
/// Validate hooks configuration before installation.
///
/// ## Errors
/// * `std::io::Error` - If any invalid hook names are found.
///
/// ## Arguments
/// * `config` - The configuration file containing the hooks to validate.
/// * `verbose` - Whether to print verbose output.
pub fn validate_hooks_for_install(config: &Config, verbose: bool) -> std::io::Result<()> {
    if verbose {
        println!("🔍 Validating hooks before installation...");
    }

    let mut invalid_hooks = Vec::new();
    for hook_name in config.hooks.keys() {
        if !GIT_HOOKS.contains(&hook_name.as_str()) {
            invalid_hooks.push(hook_name.clone());
        }
    }

    if !invalid_hooks.is_empty() {
        let error_message = format!(
            "Invalid hook names detected\n\nThe following hooks are not recognized by Git:\n{}\n\nPlease check your configuration file and use only valid Git hook names.",
            format_list(&invalid_hooks)
        );

        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            error_message,
        ));
    }

    Ok(())
}

/// # `install_hooks`
/// Install all hooks listed in the config file.
///
/// ## Errors
/// * If the `.git/hooks` directory cannot be created
///
/// ## Arguments
/// * `config` - Parsed configuration file
/// * `dry_run` - Whether to run the hook in dry run mode
/// * `verbose` - Whether to print verbose output
pub fn install_hooks(config: &Config, dry_run: bool, verbose: bool) -> std::io::Result<()> {
    validate_hooks(config, verbose)?;

    let git_hooks_path = get_git_hooks_path()?;

    if !check_for_git_hooks() {
        fs::create_dir_all(&git_hooks_path)?;
    }

    if verbose {
        println!("🪝 Installing hooks...");
    }

    for hook_name in config.hooks.keys() {
        install_hook(hook_name, dry_run, verbose)?;
    }

    Ok(())
}

/// # `install_hook`
/// Install a single hook.
///
/// ## Errors
/// * If the `.git/hooks` directory cannot be created
/// * If the hook cannot be installed/given permission
///
/// ## Arguments
/// * `hook_name` - Name of the hook to install
/// * `dry_run` - Whether to run the hook in dry run mode
/// * `verbose` - Whether to print verbose output
pub fn install_hook(hook_name: &str, dry_run: bool, verbose: bool) -> std::io::Result<()> {
    if verbose && !dry_run {
        println!("🪝 Installing {hook_name} hook...");
    }

    let git_hooks_path = get_git_hooks_path()?;

    if !git_hooks_path.exists() {
        if dry_run {
            println!("🪝 Skipping creation of .git/hooks directory in dry run mode");
        } else {
            if verbose {
                println!("  - Creating .git/hooks directory...");
            }
            fs::create_dir_all(&git_hooks_path)?;
        }
    }

    let hook_path = git_hooks_path.join(hook_name);

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
        fs::write(&hook_path, hook_content)?;

        if verbose {
            println!("  - Installing {hook_name} file...");
        }

        // Linux only
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mut permissions = fs::metadata(&hook_path)?.permissions();

            permissions.set_mode(0o755);

            fs::set_permissions(&hook_path, permissions)?;

            if verbose {
                println!("  - Setting file permissions...");
            }
        }
    }

    if verbose {
        println!("  ✅ Installed {hook_name} file");
    }

    Ok(())
}

/// # `execute_command`
/// Executes a command.
///
/// ## Errors
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
/// ## Errors
/// * If a command cannot be executed
///
/// ## Panics
/// * If the hook is not found in the configuration.
///
/// # Arguments
/// * `config` - A reference to the configuration.
/// * `hook_name` - The name of the hook to run.
/// * `dry_run` - Whether to run the hook in dry run mode.
/// * `verbose` - Whether to print verbose output.
pub fn run_hook(
    config: &Config,
    hook_name: &str,
    dry_run: bool,
    verbose: bool,
) -> Result<(), std::io::Error> {
    if let Some(hook) = config.hooks.get(hook_name) {
        if verbose && !dry_run {
            println!("📋 Running Hook: {hook_name}");
        }

        for (idx, command_str) in hook.commands.iter().enumerate() {
            if dry_run {
                let current_dir = std::env::current_dir();

                println!("Step {} of {}:", idx + 1, hook.commands.len());
                println!("  Command: {command_str}");

                if let Ok(dir) = current_dir {
                    println!("  Working directory: {}", dir.display());
                }

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
                    print_error(
                        "Command failed",
                        &format!("Hook '{hook_name}' command failed with status code {code}"),
                        "Please check your command and try again.",
                    );

                    std::process::exit(code);
                }
                Err(e) => {
                    print_error(
                        "Failed to execute command",
                        &format!("Error: {e}"),
                        "Please ensure the command exists and is executable.",
                    );

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

        Ok(())
    } else {
        let formatted_hooks = format_list(&config.hooks.keys().collect::<Vec<_>>());

        print_error(
            "Hook not found",
            &format!("No commands defined for hook '{hook_name}'"),
            &format!(
                "Available hooks:\n{formatted_hooks}\n\nPlease check your configuration file."
            ),
        );

        std::process::exit(1);
    }
}

/// # `uninstall_given_hook`
/// Uninstalls a single hook by removing its file.
///
/// ## Errors
/// * Errors if the command fails to remove the file.
///
/// # Arguments
/// * `config` - A reference to the configuration.
/// * `hook_name` - The name of the hook to run.
/// * `dry_run` - Whether to perform a dry run.
/// * `verbose` - Whether to print verbose output.
pub fn uninstall_given_hook(
    config: &Config,
    hook_name: &str,
    dry_run: bool,
    verbose: bool,
) -> std::io::Result<()> {
    if config.hooks.contains_key(hook_name) {
        if verbose && !dry_run {
            println!("🗑️ Uninstalling hook: {hook_name}");
        }

        let git_hooks_path = get_git_hooks_path()?;
        let hook_path = git_hooks_path.join(hook_name);

        if hook_path.exists() {
            if dry_run {
                println!(
                    "  🚧 Dry run: Would remove hook file: {}",
                    hook_path.display()
                );
            } else {
                fs::remove_file(&hook_path)?;
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

    Ok(())
}

/// # `uninstall_hooks`
/// Uninstalls all hooks by removing their files.
///
/// ## Errors
/// * If there is an error uninstalling a hook.
///
/// # Arguments
/// * `config` - A reference to the configuration.
/// * `dry_run` - Whether to perform a dry run.
/// * `verbose` - Whether to print verbose output.
pub fn uninstall_hooks(config: &Config, dry_run: bool, verbose: bool) -> std::io::Result<()> {
    if verbose && !dry_run {
        println!("🗑️ Uninstalling all hooks");
    }

    for hook_name in config.hooks.keys() {
        uninstall_given_hook(config, hook_name, dry_run, verbose)?;
    }

    if verbose && !dry_run {
        println!(
            "🏁 Uninstallation completed: {} hooks removed",
            config.hooks.len()
        );
    }

    Ok(())
}

/// # `compare_hooks`
/// Compare installed hooks with the configuration file.
///
/// ## Errors
/// * If there is an error reading the git hooks directory.
///
/// ## Arguments
/// * `config` - A reference to the configuration.
/// * `verbose` - Whether to print verbose output.
pub fn compare_hooks(config: &Config, verbose: bool) -> std::io::Result<()> {
    let git_hooks_path = get_git_hooks_path()?;
    let mut differences_found = false;

    if verbose {
        println!("🔍 Comparing installed hooks with configuration file...");
    }

    // Check for hooks in config but not installed
    for hook_name in config.hooks.keys() {
        let hook_path = git_hooks_path.join(hook_name);
        if !hook_path.exists() {
            if !differences_found {
                println!("\n❌ Differences found:");
                differences_found = true;
            }
            println!("  - Hook '{hook_name}' is in config but not installed");
        }
    }

    // Check for installed hooks not in config
    if let Ok(entries) = fs::read_dir(&git_hooks_path) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    let hook_name = entry.file_name().to_string_lossy().to_string();

                    if hook_name.ends_with(".sample") {
                        continue;
                    }

                    if !config.hooks.contains_key(&hook_name) {
                        if !differences_found {
                            println!("\n❌ Differences found:");
                            differences_found = true;
                        }
                        println!("  - Hook '{hook_name}' is installed but not in config");
                    }
                }
            }
        }
    }

    if !differences_found {
        println!("✅ All hooks match the configuration file");
    }

    Ok(())
}

/// # `validate_hooks`
/// Validate that hooks in the configuration file are standard Git hooks.
///
/// ## Errors
/// None, I just return Ok(()) to aggregate all calls in a `match` statement in the main function.
///
/// ## Arguments
/// * `config` - A reference to the configuration.
/// * `verbose` - Whether to print verbose output.
pub fn validate_hooks(config: &Config, verbose: bool) -> std::io::Result<()> {
    if verbose {
        println!("🔍 Validating hooks in configuration file...");
    }

    let mut invalid_hooks = Vec::new();
    let mut valid_hooks = 0;

    for hook_name in config.hooks.keys() {
        if GIT_HOOKS.contains(&hook_name.as_str()) {
            valid_hooks += 1;
            if verbose {
                println!("  ✅ Hook '{hook_name}' is valid");
            }
        } else {
            invalid_hooks.push(hook_name.clone());
        }
    }

    if invalid_hooks.is_empty() {
        if verbose {
            print_success(
                "All hooks are valid",
                &format!("Found {valid_hooks} valid Git hooks in your configuration."),
            );
        }
    } else {
        print_warning(
            "Invalid hooks detected",
            &format!(
                "The following hooks are not recognized by Git:\n{}\n\nPlease use only valid Git hook names in your configuration.",
                format_list(&invalid_hooks)
            ),
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cli, Hook, read_config};
    use clap::Parser;
    use std::{error::Error, fs};
    use tempfile::TempDir;

    fn setup_test_env() -> Result<TempDir, Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        // Create git structure
        let output = std::process::Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .output()?;

        if !output.status.success() {
            return Err(Box::new(std::io::Error::other(format!(
                "git init failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))));
        }

        // Create a simple config file
        let file_contents = r"
pre-commit:
    commands:
        - cargo fmt --all -- --check

pre-push:
    commands:
        - cargo test
";
        let config_file = temp_dir.path().join("hooksmith.yaml");
        fs::write(&config_file, file_contents)?;

        Ok(temp_dir)
    }

    #[test]
    fn test_validate_hooks_for_install_invalid() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid.yaml");

        // Create config with invalid hook
        fs::write(
            &config_path,
            "non-existent-hook:\n  commands:\n    - echo 'invalid'",
        )
        .unwrap();
        let config = read_config(&config_path).unwrap();

        let result = validate_hooks_for_install(&config, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_commands_list() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("empty_commands.yaml");

        fs::write(&config_path, "pre-commit:\n  commands: []").unwrap();
        let config = read_config(&config_path).unwrap();

        let result = run_hook(&config, "pre-commit", false, false);
        assert!(result.is_ok()); // Should handle empty command lists gracefully
    }

    #[test]
    fn test_setup_test_env() -> Result<(), Box<dyn Error>> {
        let temp_dir = setup_test_env()?;

        assert!(temp_dir.path().exists());
        assert!(temp_dir.path().join(".git").exists());
        assert!(temp_dir.path().join(".git").join("hooks").exists());

        assert!(temp_dir.path().join("hooksmith.yaml").exists());

        Ok(())
    }

    #[test]
    fn test_install_hook() -> Result<(), Box<dyn Error>> {
        let temp_dir = setup_test_env()?;
        std::env::set_current_dir(&temp_dir)?;

        // Run the function
        install_hook("pre-commit", false, false)?;

        // Verify hook was created
        let hook_path = temp_dir
            .path()
            .join(".git")
            .join("hooks")
            .join("pre-commit");
        assert!(hook_path.exists());

        // Check content
        let content = fs::read_to_string(hook_path)?;
        assert!(content.contains("exec hooksmith run pre-commit"));

        Ok(())
    }

    #[test]
    fn test_execute_command() -> Result<(), Box<dyn Error>> {
        let result = execute_command("echo 'test command'", false)?;
        assert!(result.success());

        // Test dry run
        let result = execute_command("invalid_command_that_should_fail", true)?;
        assert!(result.success()); // Should succeed in dry run mode

        Ok(())
    }

    #[test]
    fn test_run_hook() -> Result<(), Box<dyn Error>> {
        let temp_dir = setup_test_env()?;
        let config_path = temp_dir.path().join("hooksmith.yaml");
        let config = read_config(&config_path)?;

        // Test with dry run
        run_hook(&config, "pre-commit", true, false)?;

        Ok(())
    }

    #[test]
    fn test_validate_hooks() -> Result<(), Box<dyn Error>> {
        let temp_dir = setup_test_env()?;
        let config_path = temp_dir.path().join("hooksmith.yaml");
        let config = read_config(&config_path)?;

        validate_hooks(&config, false)?;

        // Create config with an invalid hook
        let mut hooks = std::collections::HashMap::new();
        let invalid_hook = Hook {
            commands: vec![String::from("echo 'test'")],
        };
        hooks.insert(String::from("not-a-real-hook"), invalid_hook);
        let invalid_config = Config { hooks };

        validate_hooks(&invalid_config, false)?;

        Ok(())
    }

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
