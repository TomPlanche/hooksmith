use std::process::Command;

#[test]
fn test_cli_help() -> std::io::Result<()> {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--help")
        .output()?;

    assert!(output.status.success());
    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(output_str.contains("A trivial Git hooks utility"));
    Ok(())
}
