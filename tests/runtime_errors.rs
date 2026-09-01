use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn write_program(name: &str, source: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "astron-runtime-tests-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp test directory");
    let path = dir.join(name);
    fs::write(&path, source).expect("failed to write test program");
    path
}

fn run_program(path: &PathBuf) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_astron"))
        .arg(path)
        .current_dir(project_root())
        .output()
        .expect("failed to run astron");
    let code = output.status.code().expect("process terminated by signal");
    let stdout = String::from_utf8(output.stdout).expect("stdout was not valid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr was not valid UTF-8");
    (code, stdout, stderr)
}

#[test]
fn array_out_of_bounds_reports_source_location() {
    let program = write_program(
        "out_of_bounds.astrn",
        r#"launch main {
    payload values: [int] = [10, 20]
    fire log(values[5])
    land 0
}
"#,
    );

    let (code, stdout, stderr) = run_program(&program);

    assert_eq!(code, 1);
    assert!(stdout.is_empty(), "expected empty stdout, got: {stdout}");
    assert!(
        stderr.contains(":3:21: runtime error: array index 5 out of bounds with length 2"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn division_by_zero_reports_source_location() {
    let program = write_program(
        "division_by_zero.astrn",
        r#"launch main {
    payload amount: int = 10
    fire log(amount / 0)
    land 0
}
"#,
    );

    let (code, stdout, stderr) = run_program(&program);

    assert_eq!(code, 1);
    assert!(stdout.is_empty(), "expected empty stdout, got: {stdout}");
    assert!(
        stderr.contains(":3:14: runtime error: division by zero"),
        "unexpected stderr: {stderr}"
    );
}
