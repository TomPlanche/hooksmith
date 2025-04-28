use std::{fs, path::Path};

use crate::{
    Config,
    git_related::{check_for_git_hooks, get_git_hooks_path},
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

pub struct Hooksmith {
    pub config: Config,
    dry_run: bool,
    verbose: bool,
}

impl Hooksmith {
    /// # `new_from_config`
    /// Create a new instance of `Hooksmith` from a configuration file.
    ///
    /// ## Errors
    /// * `std::io::Error` - If the configuration file cannot be read or parsed.
    pub fn new_from_config(config: &Path, dry_run: bool, verbose: bool) -> std::io::Result<Self> {
        let config = Self::read_config(config)?;

        if dry_run {
            println!("🔄 DRY RUN MODE - No commands will be executed\n");
        }

        Ok(Self {
            config,
            dry_run,
            verbose,
        })
    }

    /// # `validate_hooks_for_install`
    /// Validate hooks configuration before installation.
    ///
    /// ## Errors
    /// * `std::io::Error` - If any invalid hook names are found.
    pub fn validate_hooks_for_install(&self) -> std::io::Result<()> {
        if self.verbose {
            println!("🔍 Validating hooks before installation...");
        }

        let mut invalid_hooks = Vec::new();
        for hook_name in self.config.hooks.keys() {
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

    /// # `install_hook`
    /// Install a single hook.
    ///
    /// ## Errors
    /// * If the `.git/hooks` directory cannot be created
    /// * If the hook cannot be installed/given permission
    ///
    /// ## Arguments
    /// * `hook_name` - Name of the hook to install
    pub fn install_hook(&self, hook_name: &str) -> std::io::Result<()> {
        if self.verbose && !self.dry_run {
            println!("🪝 Installing {hook_name} hook...");
        }

        let git_hooks_path = get_git_hooks_path()?;

        if !git_hooks_path.exists() {
            if self.dry_run {
                println!("🪝 Skipping creation of .git/hooks directory in dry run mode");
            } else {
                if self.verbose {
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

        if self.dry_run {
            println!("🪝 Skipping installation of {hook_name} hook in dry run mode");
        } else {
            fs::write(&hook_path, hook_content)?;

            if self.verbose {
                println!("  - Installing {hook_name} file...");
            }

            // Linux only
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                let mut permissions = fs::metadata(&hook_path)?.permissions();

                permissions.set_mode(0o755);

                fs::set_permissions(&hook_path, permissions)?;

                if self.verbose {
                    println!("  - Setting file permissions...");
                }
            }
        }

        if self.verbose {
            println!("  ✅ Installed {hook_name} file");
        }

        Ok(())
    }

    /// Install all hooks listed in the config file.
    ///
    /// ## Errors
    /// * If the `.git/hooks` directory cannot be created
    ///
    /// ## Arguments
    /// * `config` - Parsed configuration file
    /// * `dry_run` - Whether to run the hook in dry run mode
    /// * `verbose` - Whether to print verbose output
    pub fn install_hooks(&self) -> std::io::Result<()> {
        self.validate_hooks()?;

        let git_hooks_path = get_git_hooks_path()?;

        if !check_for_git_hooks() {
            fs::create_dir_all(&git_hooks_path)?;
        }

        if self.verbose {
            println!("🪝 Installing hooks...");
        }

        for hook_name in self.config.hooks.keys() {
            self.install_hook(hook_name)?;
        }

        Ok(())
    }

    /// # `compare_hooks`
    /// Compare installed hooks with the configuration file.
    ///
    /// ## Errors
    /// * If there is an error reading the git hooks directory.
    pub fn compare_hooks(&self) -> std::io::Result<()> {
        let git_hooks_path = get_git_hooks_path()?;
        let mut differences_found = false;

        if self.verbose {
            println!("🔍 Comparing installed hooks with configuration file...");
        }

        // Check for hooks in config but not installed
        for hook_name in self.config.hooks.keys() {
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

                        if !self.config.hooks.contains_key(&hook_name) {
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
    /// * `hook_name` - The name of the hook to run.
    pub fn run_hook(&self, hook_name: &str) -> Result<(), std::io::Error> {
        if let Some(hook) = self.config.hooks.get(hook_name) {
            if self.verbose && !self.dry_run {
                println!("📋 Running Hook: {hook_name}");
            }

            for (idx, command_str) in hook.commands.iter().enumerate() {
                if self.dry_run {
                    let current_dir = std::env::current_dir();

                    println!("Step {} of {}:", idx + 1, hook.commands.len());
                    println!("  Command: {command_str}");

                    if let Ok(dir) = current_dir {
                        println!("  Working directory: {}", dir.display());
                    }

                    println!();
                    continue;
                }

                if self.verbose && !self.dry_run {
                    println!("  - Running command: {command_str}");
                }

                match self.execute_command(command_str) {
                    Ok(status) if status.success() => {
                        if self.verbose && !self.dry_run {
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

            if self.dry_run {
                println!(
                    "🏁 Dry run completed. {} commands would be executed",
                    hook.commands.len()
                );
            }

            Ok(())
        } else {
            let formatted_hooks = format_list(&self.config.hooks.keys().collect::<Vec<_>>());

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
    /// * `hook_name` - The name of the hook to run.
    pub fn uninstall_given_hook(&self, hook_name: &str) -> std::io::Result<()> {
        if self.config.hooks.contains_key(hook_name) {
            if self.verbose && !self.dry_run {
                println!("🗑️ Uninstalling hook: {hook_name}");
            }

            let git_hooks_path = get_git_hooks_path()?;
            let hook_path = git_hooks_path.join(hook_name);

            if hook_path.exists() {
                if self.dry_run {
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
            let possible_hooks = self.config.hooks.keys().collect::<Vec<_>>();
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
    pub fn uninstall_hooks(&self) -> std::io::Result<()> {
        if self.verbose && !self.dry_run {
            println!("🗑️ Uninstalling all hooks");
        }

        for hook_name in self.config.hooks.keys() {
            self.uninstall_given_hook(hook_name)?;
        }

        if self.verbose && !self.dry_run {
            println!(
                "🏁 Uninstallation completed: {} hooks removed",
                self.config.hooks.len()
            );
        }

        Ok(())
    }

    /// # `validate_hooks`
    /// Validate that hooks in the configuration file are standard Git hooks.
    ///
    /// ## Errors
    /// None, I just return Ok(()) to aggregate all calls in a `match` statement in the main function.
    pub fn validate_hooks(&self) -> std::io::Result<()> {
        if self.verbose {
            println!("🔍 Validating hooks in configuration file...");
        }

        let mut invalid_hooks = Vec::new();
        let mut valid_hooks = 0;

        for hook_name in self.config.hooks.keys() {
            if GIT_HOOKS.contains(&hook_name.as_str()) {
                valid_hooks += 1;
                if self.verbose {
                    println!("  ✅ Hook '{hook_name}' is valid");
                }
            } else {
                invalid_hooks.push(hook_name.clone());
            }
        }

        if invalid_hooks.is_empty() {
            if self.verbose {
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

    /// # `execute_command`
    /// Executes a command.
    ///
    /// ## Errors
    /// * If a command cannot be executed
    ///
    /// # Arguments
    /// * `command` - The command to execute.
    fn execute_command(&self, command: &str) -> std::io::Result<std::process::ExitStatus> {
        if self.dry_run {
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
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_validate_hooks_for_install_invalid() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid.yaml");

        // Create config with invalid hook
        fs::write(
            &config_path,
            "non-existent-hook:\n  commands:\n    - echo 'invalid'",
        )
        .unwrap();

        let hs = Hooksmith::new_from_config(&config_path, false, false)?;

        let result = hs.validate_hooks_for_install();
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_empty_commands_list() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("empty_commands.yaml");

        fs::write(&config_path, "pre-commit:\n  commands: []").unwrap();

        let hs = Hooksmith::new_from_config(&config_path, false, false)?;

        let result = hs.run_hook("pre-commit");
        assert!(result.is_ok()); // Should handle empty command lists gracefully

        Ok(())
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

        let hs = Hooksmith::new_from_config(Path::new("hooksmith.yaml"), false, false)?;

        // Run the function
        hs.install_hook("pre-commit")?;

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
        let temp_dir = setup_test_env()?;
        let config_path = temp_dir.path().join("hooksmith.yaml");

        // Non-dry-run test
        let hs = Hooksmith::new_from_config(&config_path, false, false)?;
        let result = hs.execute_command("echo 'test command'")?;
        assert!(result.success());

        // Dry-run test
        let hs_dry = Hooksmith::new_from_config(&config_path, true, false)?;
        let result = hs_dry.execute_command("invalid_command_that_should_fail")?;
        assert!(result.success()); // Should succeed in dry run mode

        Ok(())
    }

    #[test]
    fn test_run_hook() -> Result<(), Box<dyn Error>> {
        let hs = Hooksmith::new_from_config(Path::new("hooksmith.yaml"), true, false)?;

        // Test with dry run
        hs.run_hook("pre-commit")?;

        Ok(())
    }
}
