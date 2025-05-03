use std::path::PathBuf;

/// Get the path to the Git hooks directory.
///
/// # Errors
/// * If the `git` command fails to execute
///
/// # Returns
/// * `PathBuf` - Path to the Git hooks directory
pub fn get_git_hooks_path() -> std::io::Result<PathBuf> {
    let output = std::process::Command::new("git")
        .arg("rev-parse")
        .arg("--git-path")
        .arg("hooks")
        .output()?;

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(PathBuf::from(path))
}

/// Check if the current directory is a Git repository and if it has hooks.
///
/// # Arguments
/// * `path` - Path to the directory to check
///
/// # Returns
/// * `bool` - True if the directory is a Git repository with hooks, false otherwise
#[must_use]
pub fn check_for_git_hooks() -> bool {
    let git_hooks = get_git_hooks_path().ok();

    git_hooks.is_some_and(|path| path.exists())
}
