use crate::error::GitError;
use std::path::PathBuf;

/// Get the path to the Git hooks directory.
///
/// # Errors
/// * If the `git` command fails to execute
///
/// # Returns
/// * `PathBuf` - Path to the Git hooks directory
pub fn get_git_hooks_path() -> Result<PathBuf, GitError> {
    let output = std::process::Command::new("git")
        .arg("rev-parse")
        .arg("--git-path")
        .arg("hooks")
        .output()?;

    if !output.status.success() {
        return Err(GitError::NotGitRepo);
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(PathBuf::from(path))
}

/// Check whether the current repository has a hooks directory.
///
/// Looks up the hooks directory using `git rev-parse --git-path hooks` and
/// returns true if that path exists. This does not validate the presence of
/// specific hook files, only the hooks directory itself.
///
/// # Returns
/// `true` if a hooks directory path could be resolved and it exists on disk,
/// otherwise `false`.
#[must_use]
pub fn check_for_git_hooks() -> bool {
    let git_hooks = get_git_hooks_path().ok();

    git_hooks.is_some_and(|path| path.exists())
}
