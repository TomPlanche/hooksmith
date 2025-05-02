pub(crate) mod git_related;
mod hooksmith;
pub(crate) mod utils;

pub use hooksmith::Hooksmith;

/// # `init`
/// Initialize Hooksmith by reading the configuration file and installing hooks.
///
/// ## Arguments
/// * `config_path` - Path to the configuration file
///
/// ## Errors
/// * If the configuration file cannot be read or parsed
///
/// ## Returns
/// * `Config` - Parsed configuration file
pub fn init(config_path: &std::path::Path) -> std::io::Result<()> {
    let hs = hooksmith::Hooksmith::new_from_config(config_path, false, false)?;

    hs.install_hooks()?;

    Ok(())
}
