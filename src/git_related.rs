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

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use tempfile::TempDir;

    fn setup_git_repo() -> Result<TempDir, Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        // Initialize git repo
        std::process::Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .output()?;

        Ok(temp_dir)
    }

    #[test]
    fn test_get_git_hooks_path() -> Result<(), Box<dyn Error>> {
        let temp_dir = setup_git_repo()?;
        std::env::set_current_dir(&temp_dir)?;

        let hooks_path = get_git_hooks_path()?;
        assert!(hooks_path.ends_with("hooks"));
        assert!(hooks_path.exists());

        Ok(())
    }

    #[test]
    fn test_get_git_hooks_path_no_git() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;

        assert!(get_git_hooks_path().is_err());

        Ok(())
    }
}
