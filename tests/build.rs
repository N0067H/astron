use std::{fs, path::PathBuf, process::Command};

struct Fixture(PathBuf);

impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("astron-native-test-{}-{id}", std::process::id()));
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join("input.astrn"), source).unwrap();
        Self(dir)
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_astron"));
        command.current_dir(&self.0);
        command
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn minimal_launch_builds_and_matches_interpreter() {
    let fixture = Fixture::new("launch main { land 37 } launch other { land 12 }");
    for (target, code) in [("main", 37), ("other", 12)] {
        let executable = fixture
            .0
            .join(format!("native program{}", std::env::consts::EXE_SUFFIX));
        let build = fixture
            .command()
            .args(["build", "input.astrn", "--launch", target, "-o"])
            .arg(&executable)
            .output()
            .unwrap();
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let native = Command::new(executable).output().unwrap();
        let interpreted = fixture
            .command()
            .args(["input.astrn", "--launch", target])
            .output()
            .unwrap();
        assert_eq!(native.status.code(), Some(code));
        assert_eq!(native.status.code(), interpreted.status.code());
        assert_eq!(native.stdout, interpreted.stdout);
        assert_eq!(native.stderr, interpreted.stderr);
    }
}

#[test]
fn build_defaults_to_main_and_input_stem() {
    let fixture = Fixture::new("launch main { land 0 }");
    let output = fixture
        .command()
        .args(["build", "input.astrn"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let executable = fixture
        .0
        .join(format!("input{}", std::env::consts::EXE_SUFFIX));
    assert!(Command::new(executable).status().unwrap().success());
}

#[test]
fn build_rejects_unsupported_syntax_and_missing_launch() {
    let fixture = Fixture::new("launch main { fire log(\"hello\") land 0 }");
    let output = fixture
        .command()
        .args(["build", "input.astrn"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("input.astrn:1:15: unsupported native syntax")
    );
    let output = fixture
        .command()
        .args(["build", "input.astrn", "--launch", "missing"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("no 'launch missing' found"));
}

#[test]
fn build_checks_source_before_invoking_clang() {
    let fixture = Fixture::new("launch main { land unknown }");
    let output = fixture
        .command()
        .args(["build", "input.astrn"])
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unknown"));
    assert!(!stderr.contains("cannot run clang"));
}

#[test]
fn build_reports_missing_clang_and_preserves_existing_output() {
    let fixture = Fixture::new("launch main { land 0 }");
    fs::write(fixture.0.join("existing"), "keep").unwrap();
    let output = fixture
        .command()
        .args(["build", "input.astrn", "-o", "existing"])
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot run clang"));
    assert_eq!(
        fs::read_to_string(fixture.0.join("existing")).unwrap(),
        "keep"
    );
}

#[test]
fn build_refuses_to_overwrite_source() {
    let source = "launch main { land 0 }";
    let fixture = Fixture::new(source);
    let output = fixture
        .command()
        .args(["build", "input.astrn", "-o", "input.astrn"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("overwrite source"));
    assert_eq!(
        fs::read_to_string(fixture.0.join("input.astrn")).unwrap(),
        source
    );
}
