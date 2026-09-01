use std::path::PathBuf;
use std::process::Command;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn run_cli(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_astron"))
        .args(args)
        .current_dir(project_root())
        .output()
        .expect("failed to run astron");
    let code = output.status.code().expect("process terminated by signal");
    let stdout = String::from_utf8(output.stdout).expect("stdout was not valid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr was not valid UTF-8");
    (code, stdout, stderr)
}

#[test]
fn help_flag_prints_usage() {
    let (code, stdout, stderr) = run_cli(&["--help"]);

    assert_eq!(code, 0);
    assert!(stderr.is_empty(), "expected empty stderr, got: {stderr}");
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("--launch <name>"));
}

#[test]
fn version_flag_prints_package_version() {
    let (code, stdout, stderr) = run_cli(&["--version"]);

    assert_eq!(code, 0);
    assert!(stderr.is_empty(), "expected empty stderr, got: {stderr}");
    assert_eq!(stdout.trim(), env!("CARGO_PKG_VERSION"));
}

#[test]
fn unknown_flag_returns_error() {
    let (code, stdout, stderr) = run_cli(&["--wat"]);

    assert_eq!(code, 1);
    assert!(stdout.is_empty(), "expected empty stdout, got: {stdout}");
    assert!(stderr.contains("error: unknown option '--wat'"));
    assert!(stderr.contains("Usage:"));
}

#[test]
fn launch_without_name_returns_error() {
    let (code, stdout, stderr) = run_cli(&["exam/01_hello_launches.astrn", "--launch"]);

    assert_eq!(code, 1);
    assert!(stdout.is_empty(), "expected empty stdout, got: {stdout}");
    assert!(stderr.contains("error: --launch requires a name"));
    assert!(stderr.contains("Usage:"));
}
