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

fn assert_native_matches(source: &str, expected_code: i32) {
    let fixture = Fixture::new(source);
    let executable = fixture
        .0
        .join(format!("calculation{}", std::env::consts::EXE_SUFFIX));
    let build = fixture
        .command()
        .args(["build", "input.astrn", "-o"])
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let native = Command::new(executable).output().unwrap();
    let interpreted = fixture.command().arg("input.astrn").output().unwrap();
    assert_eq!(native.status.code(), Some(expected_code));
    assert_eq!(native.status.code(), interpreted.status.code());
    assert_eq!(native.stdout, interpreted.stdout);
    assert_eq!(native.stderr, interpreted.stderr);
}

#[test]
fn scalar_assignments_and_signed_arithmetic_match_interpreter() {
    assert_native_matches(
        "launch main {
        fuel x: int = 7
        payload saved: int = x
        x = 21
        x += saved
        x -= 4
        x *= 3
        x /= 5
        x %= 11
        land x + -7 / 3 + -7 % 3
    }",
        0,
    );
    assert_native_matches("launch main { land 4294967333 }", 37);
    assert_native_matches(include_str!("../exam/14_scalar_calculations.astrn"), 42);
}

#[test]
fn scalar_float_byte_and_logical_expressions_build() {
    assert_native_matches(
        "launch main {
        fuel f: float = 3
        f += 2
        f *= 2.5
        f /= 2
        f %= 4.0
        f = -f + 2 - 0.5
        payload b: byte = 0xFE
        fuel same: flag = b == 0xFE
        same = same and not (b != 0xFE)
        payload comparisons: flag = (f < 4.0) and (f > -4.0) and (f <= 4) and (f >= -4)
        payload mixed: flag = (1 == 1.0) and (2.0 != 1) and (1 + 2.5 == 3.5)
        payload nested: flag = (false or same) and (comparisons or mixed)
        land 17
    }",
        17,
    );
}

#[test]
fn scalar_ir_preserves_runtime_numeric_kinds_and_short_circuit_branches() {
    let fixture = Fixture::new(
        "launch main {
        fuel f: float = 7
        f /= 2
        payload result: flag = (f == 3) and (false or (2 < 3))
        land 0
    }",
    );
    let checked = astron::frontend::check_program(&fixture.0.join("input.astrn")).unwrap();
    let ir = astron::backend::emit_ir(&checked, "main").unwrap();
    assert!(ir.contains("sdiv i64 7, 2"));
    assert!(!ir.contains("sitofp"));
    assert_eq!(ir.matches("phi i1").count(), 2);
    assert_eq!(ir.matches("br i1").count(), 2);
    assert_native_matches(
        &fs::read_to_string(fixture.0.join("input.astrn")).unwrap(),
        0,
    );
}

#[test]
fn scalar_operations_rejected_by_interpreter_are_not_silently_enabled() {
    for statement in [
        "payload x: byte = 0x01 + 0x02",
        "payload x: flag = 0x01 < 0x02",
        "payload x: float = 3 % 2.0",
        "payload x: float = 3.0 % 2",
    ] {
        let fixture = Fixture::new(&format!("launch main {{ {statement} land 0 }}"));
        let interpreted = fixture.command().arg("input.astrn").output().unwrap();
        assert!(!interpreted.status.success());
        assert!(String::from_utf8_lossy(&interpreted.stderr).contains("runtime error"));
        let build = fixture
            .command()
            .args(["build", "input.astrn"])
            .output()
            .unwrap();
        assert!(!build.status.success());
        assert!(
            String::from_utf8_lossy(&build.stderr)
                .contains("operation is not supported by the interpreter")
        );
    }
}

fn assert_native_flag(expression: &str) {
    let source = format!("launch main {{ payload result: flag = {expression} land 0 }}");
    let fixture = Fixture::new(&source);
    let checked = astron::frontend::check_program(&fixture.0.join("input.astrn")).unwrap();
    let ir = astron::backend::emit_ir(&checked, "main").unwrap();
    let result = ir
        .lines()
        .rev()
        .filter(|line| line.contains(" = "))
        .nth(1)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap();
    let probe = ir.replace("trunc i64 0 to i32", &format!("zext i1 {result} to i32"));
    let ir_path = fixture.0.join("probe.ll");
    let executable = fixture
        .0
        .join(format!("probe{}", std::env::consts::EXE_SUFFIX));
    fs::write(&ir_path, probe).unwrap();
    let mut clang = Command::new("clang");
    clang.arg(&ir_path).arg("-o").arg(&executable);
    if cfg!(unix) {
        clang.arg("-lm");
    }
    let build = clang.output().unwrap();
    assert!(
        build.status.success(),
        "{expression}: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert_eq!(
        Command::new(executable).status().unwrap().code(),
        Some(1),
        "{expression}"
    );
    let assertion = source.replace("land 0", "fire assert(result) land 0");
    fs::write(fixture.0.join("input.astrn"), assertion).unwrap();
    let interpreted = fixture.command().arg("input.astrn").output().unwrap();
    assert!(
        interpreted.status.success(),
        "{expression}: {}",
        String::from_utf8_lossy(&interpreted.stderr)
    );
}

#[test]
fn native_numeric_and_logical_results_match_interpreter_assertions() {
    for expression in [
        "(7 / 2 == 3) and (-7 % 3 == -1)",
        "(7 / 2.0 == 3.5) and (7.0 / 2 == 3.5)",
        "(1 + 2.5 == 3.5) and (2.5 + 1 == 3.5)",
        "(4 - 1.5 == 2.5) and (4.5 - 2 == 2.5)",
        "(2 * 1.5 == 3.0) and (1.5 * 2 == 3.0)",
        "(5.5 % 2.0 == 1.5) and (-5.5 % 2.0 == -1.5)",
        "(3 < 3.5) and (3.5 > 3) and (3 <= 3.0) and (3.0 >= 3)",
        "(3 != 3.5) and (3.5 != 3) and not (3 == 3.5)",
        "(0xFE == 0xFE) and (0xFE != 0xFF)",
        "(true == true) and (true != false) and not false",
        "not (false and (1 / 0 == 0))",
        "true or (1 / 0 == 0)",
        "(false or true) and (true or false)",
        "not ((true and false) or (false or false))",
        "(9007199254740993 == 9007199254740992.0)",
        "(-0.0 == 0.0) and not (-0.0 < 0.0)",
    ] {
        assert_native_flag(expression);
    }
}
