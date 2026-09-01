use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "astron-{prefix}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp test directory");
    dir
}

fn write_file(path: &Path, source: &str) {
    fs::write(path, source).expect("failed to write test program");
}

fn run_program(path: &Path) -> (i32, String, String) {
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
fn missing_launch_reports_runtime_error() {
    let (code, stdout, stderr) =
        run_program(Path::new("exam/01_hello_launches.astrn").as_ref());

    assert_eq!(code, 0);
    assert!(stderr.is_empty(), "expected empty stderr, got: {stderr}");
    assert!(!stdout.is_empty(), "expected example program to produce stdout");

    let output = Command::new(env!("CARGO_BIN_EXE_astron"))
        .args(["exam/01_hello_launches.astrn", "--launch", "missing"])
        .current_dir(project_root())
        .output()
        .expect("failed to run astron");
    let code = output.status.code().expect("process terminated by signal");
    let stdout = String::from_utf8(output.stdout).expect("stdout was not valid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr was not valid UTF-8");

    assert_eq!(code, 1);
    assert!(stdout.is_empty(), "expected empty stdout, got: {stdout}");
    assert!(stderr.contains(":1:1: runtime error: no 'launch missing' found"));
}

#[test]
fn cyclic_import_reports_source_location() {
    let dir = temp_dir("cycle");
    let main = dir.join("main.astrn");
    let module = dir.join("module.astrn");

    write_file(
        &main,
        r#"import "./module.astrn"

launch main {
    land 0
}
"#,
    );
    write_file(
        &module,
        r#"import "./main.astrn"
"#,
    );

    let (code, stdout, stderr) = run_program(&main);

    assert_eq!(code, 1);
    assert!(stdout.is_empty(), "expected empty stdout, got: {stdout}");
    assert!(stderr.contains("module.astrn:1:1: cyclic import detected"));
}

#[test]
fn parse_error_reports_source_location() {
    let dir = temp_dir("parse");
    let path = dir.join("broken.astrn");
    write_file(
        &path,
        r#"launch main {
    payload answer: int = 
    land 0
}
"#,
    );

    let (code, stdout, stderr) = run_program(&path);

    assert_eq!(code, 1);
    assert!(stdout.is_empty(), "expected empty stdout, got: {stdout}");
    assert!(stderr.contains("broken.astrn:3:5: unexpected token in expression: Land"));
}

#[test]
fn type_error_reports_source_location() {
    let dir = temp_dir("type");
    let path = dir.join("mismatch.astrn");
    write_file(
        &path,
        r#"launch main {
    payload answer: int = "forty two"
    land 0
}
"#,
    );

    let (code, stdout, stderr) = run_program(&path);

    assert_eq!(code, 1);
    assert!(stdout.is_empty(), "expected empty stdout, got: {stdout}");
    assert!(stderr.contains("mismatch.astrn:2:27: type mismatch: declared Int, got Str"));
}
