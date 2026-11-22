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
fn build_rejects_missing_launch() {
    let fixture = Fixture::new("launch main { fire log([1, 2]) land 0 }");
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
    assert_fixture_matches(&fixture, expected_code);
}

fn assert_fixture_matches(fixture: &Fixture, expected_code: i32) {
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
    assert_eq!(ir.matches("br i1").count(), 4);
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
        assert_native_matches(&format!("launch main {{ {statement} land 0 }}"), 1);
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

#[test]
fn control_flow_stages_and_shared_ignite_match_interpreter() {
    assert_native_matches(
        "ignite { fuel shared: int = 2 }
        stage fact(n: int) lands int {
            scan n <= 1 { land 1 }
            land n * fire fact(n - 1)
        }
        launch main {
            fuel sum: int = 0
            spin i in 0..=5 {
                scan i == 2 { pass }
                sum += i
            }
            burn shared < 5 { shared += 1 }
            orbit { sum += shared scan sum > 20 { eject } }
            route sum { 23 => land fire fact(4) 99 => land 99 }
            land 42
        }",
        24,
    );
    assert_native_matches(
        "stage stop() lands void { abort } launch main { fire stop() land 99 }",
        99,
    );
    assert_native_matches("ignite { land } launch main { land 17 }", 17);
    assert_native_matches(
        "stage empty() lands void { land } launch main { fire empty() land 3 }",
        3,
    );
}

#[test]
fn native_existing_control_examples_match_interpreter() {
    for source in [
        include_str!("../exam/01_hello_launches.astrn"),
        include_str!("../exam/02_navigation_math.astrn"),
        include_str!("../exam/04_orbit_counter.astrn"),
        include_str!("../exam/05_route_phases.astrn"),
        include_str!("../exam/06_countdown_burn.astrn"),
        include_str!("../exam/09_multi_launch_modes.astrn"),
        include_str!("../exam/10_signal_strength.astrn"),
    ] {
        assert_native_matches(source, 0);
    }
}

#[test]
fn native_abort_example_matches_interpreter() {
    assert_native_matches(include_str!("../exam/08_abort_sequence.astrn"), 1);
}

#[test]
fn native_scopes_ranges_and_numeric_kinds_match_interpreter() {
    assert_native_matches(
        "launch main {
        fuel x: float = 7
        scan true { x = 9 payload x: int = 100 }
        x /= 2
        scan x != 4 { land 99 }
        spin i in 5..2 { land 98 }
        spin i in 3..3 { land 97 }
        spin i in 3..=3 { x += i }
        scan false and (1 / 0 == 0) { land 96 }
        route 8 { 1 => land 95 }
        land 7
    }",
        7,
    );
}

#[test]
fn native_stage_calls_preserve_argument_order_and_loop_bounds() {
    assert_native_matches(
        "stage mark(n: int) lands int { fire log(n) land n }
        stage add(a: int, b: int) lands int { land a + b }
        launch main {
            fuel sum: int = fire add(fire mark(1), fire mark(2))
            fuel end: int = 4
            spin i in 1..end { end = 1 sum += i }
            land sum
        }",
        9,
    );
}

#[test]
fn native_selected_launch_sees_ignite_variables() {
    let fixture = Fixture::new(
        "ignite { fuel shared: int = 12 }
        launch main { land shared }
        launch alternate { shared += 5 land shared }",
    );
    let executable = fixture
        .0
        .join(format!("alternate{}", std::env::consts::EXE_SUFFIX));
    let build = fixture
        .command()
        .args(["build", "input.astrn", "--launch", "alternate", "-o"])
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
        .args(["input.astrn", "--launch", "alternate"])
        .output()
        .unwrap();
    assert_eq!(native.status.code(), Some(17));
    assert_eq!(native.status.code(), interpreted.status.code());
}

#[test]
fn native_string_operations_and_unicode_lengths_match_interpreter() {
    assert_native_matches(
        r#"stage join(left: str, right: str) lands str {
            land left + right
        }
        launch main {
            fuel text: str = "발사"
            payload saved: str = text
            text += " 🚀"
            payload joined: str = fire join(text, "!")
            fire log(saved, text, joined, "", fire len(""))
            scan saved != "발사" { land 90 }
            scan text == saved { land 91 }
            scan not ("a" < "aa" and "aa" <= "aa") { land 92 }
            scan not ("한" > "a" and "한" >= "한") { land 93 }
            route joined { "발사 🚀!" => fire log("matched") }
            land fire len(joined)
        }"#,
        5,
    );
    assert_native_matches(
        r#"launch main {
            payload text: str = "é한🚀" + "!"
            land fire len(text)
        }"#,
        5,
    );
}

#[test]
fn native_string_declarations_select_runtime_without_control_flow() {
    assert_native_matches(
        r#"launch main {
            fuel text: str = "hello"
            payload saved: str = text
            text += " world"
            payload same: flag = saved == "hello"
            land 7
        }"#,
        7,
    );
}

#[test]
fn native_runtime_examples_match_interpreter() {
    for source in [
        include_str!("../exam/03_system_checklist.astrn"),
        include_str!("../exam/07_payload_manifest.astrn"),
        include_str!("../exam/12_enum_phases.astrn"),
        include_str!("../exam/13_builtin_functions.astrn"),
    ] {
        assert_native_matches(source, 0);
    }
}

#[test]
fn native_array_copies_mutations_and_stage_returns_match_interpreter() {
    assert_native_matches(
        r#"stage change(values: [int]) lands [int] {
            fuel local: [int] = values
            local[0] += 10
            land local
        }
        launch main {
            fuel values: [int] = [1, 2, 3]
            payload saved: [int] = values
            fuel changed: [int] = fire change(values)
            changed[1] *= 4
            changed[1] -= 1
            changed[1] /= 2
            changed[1] %= 2
            values[0] = 9
            values[2] = values[0] + 1
            fire assert(saved[0] == 1)
            fire assert(changed[0] == 11 and changed[1] == 1)
            fire assert(values[0] == 9 and values[2] == 10)
            fire assert(not (saved == saved))
            fire log(saved, values, changed, fire len(saved), fire len([]))
            fuel rows: [[int]] = [[1, 2], [3, 4]]
            payload snapshot: [[int]] = rows
            fuel first: [int] = rows[0]
            first[0] = 99
            rows[1] = first
            fire assert(snapshot[1][0] == 3)
            fire assert(rows[0][0] == 1 and rows[1][0] == 99)
            fire log(snapshot, rows, fire to_str(rows))
            land 0
        }"#,
        0,
    );
}

#[test]
fn native_enums_shadowing_and_value_formatting_match_interpreter() {
    assert_native_matches(
        r#"enum Phase { Ready Flying Done }
        stage choose(phase: Phase) lands Phase { land phase }
        launch main {
            fuel phase: Phase = Ready
            payload phases: [Phase] = [Ready, Flying, Done]
            phase = fire choose(phases[1])
            fire assert(phase == Flying and phase != Done)
            route phase { Flying => fire log(phase, phases) }
            scan true {
                payload Ready: int = 7
                fire log(Ready)
            }
            fire log(Ready, fire to_str(phase))
            fire log(fire to_str(42), fire to_str(-7), fire to_str(0xAF))
            fire log(fire to_str(true), fire to_str(false), fire to_str(air))
            fire log(fire to_str(["a", "b"]), fire to_str([]))
            fire log(fire to_str(2.5), fire to_str(3.0), fire to_str(-0.0))
            fire log(fire to_str(0.0000001), fire to_str(1.2345678901234567))
            fire log(fire to_str(1.0 / 3.0), fire to_str(0.000000000000000123))
            fire log()
            land 0
        }"#,
        0,
    );
}

#[test]
fn native_arrays_preserve_expression_evaluation_order() {
    assert_native_matches(
        r#"stage mark(n: int) lands int { fire log(n) land n }
        launch main {
            fuel values: [int] = [fire mark(1), fire mark(2)]
            values[fire mark(0)] += fire mark(3)
            fire log(values[fire mark(0)], values)
            fire assert(values[0] == 4)
            land 0
        }"#,
        0,
    );
}

#[test]
fn native_assertions_and_array_errors_match_interpreter() {
    for source in [
        "launch main { fire log(1) fire assert(false) land 0 }",
        "stage check() lands void { fire assert(false) } launch main { fire check() land 0 }",
        "launch main { fuel values: [int] = [1] fire log(values[-1]) land 0 }",
        "launch main { fuel values: [int] = [1] fire log(values[1]) land 0 }",
        "launch main { fuel values: [int] = [1] values[-1] = 2 land 0 }",
        "launch main { fuel values: [int] = [1] values[1] += 2 land 0 }",
    ] {
        assert_native_matches(source, 1);
    }
}

#[test]
fn native_imported_example_matches_interpreter() {
    let fixture = Fixture::new(include_str!("../exam/11_import_mission_tools.astrn"));
    fs::create_dir(fixture.0.join("lib")).unwrap();
    fs::write(
        fixture.0.join("lib/mission_tools.astrn"),
        include_str!("../exam/lib/mission_tools.astrn"),
    )
    .unwrap();
    assert_fixture_matches(&fixture, 0);
}

#[test]
fn native_division_and_remainder_errors_match_interpreter() {
    assert_native_matches("launch main { land 10 / 0 }", 1);
    for expression in [
        "10 / 0",
        "10 % 0",
        "10.0 / -0.0",
        "10.0 % 0.0",
        "10 / 0.0",
        "10.0 / 0",
        "(-9223372036854775807 - 1) / -1",
        "(-9223372036854775807 - 1) % -1",
    ] {
        assert_native_matches(
            &format!("launch main {{ payload value: float = {expression} land 0 }}"),
            1,
        );
        assert_native_matches(
            &format!("launch main {{ fire log(\"before\") fire log({expression}) land 0 }}"),
            1,
        );
    }
    for statement in [
        "fuel value: int = 10 value /= 0",
        "fuel value: int = 10 value %= 0",
        "fuel values: [int] = [10] values[0] /= 0",
        "fuel values: [int] = [10] values[0] %= 0",
        "fuel values: [int] = [10] values[0] += air",
    ] {
        assert_native_matches(&format!("launch main {{ {statement} land 0 }}"), 1);
    }
}

#[test]
fn native_integer_overflow_wraps_like_interpreter() {
    assert_native_matches(
        "launch main {
            payload maximum: int = 9223372036854775807
            payload minimum: int = -maximum - 1
            fire assert(maximum + 1 == minimum)
            fire assert(minimum - 1 == maximum)
            fire assert(maximum * 2 == -2)
            fire assert(-minimum == minimum)
            fuel value: int = maximum
            value += 1
            value -= 1
            value *= 2
            fire assert(value == -2)
            fuel values: [int] = [maximum]
            values[0] += 1
            fire assert(values[0] == minimum)
            fire log(value, values, minimum)
            land 0
        }",
        0,
    );
    assert_native_matches("launch main { land 9223372036854775807 + 1 }", 0);
    assert_native_matches(
        "launch main { land (-9223372036854775807 - 1) - 1 }",
        if cfg!(windows) { -1 } else { 255 },
    );
    assert_native_matches(
        "launch main { land 9223372036854775807 * 2 }",
        if cfg!(windows) { -2 } else { 254 },
    );
    assert_native_matches("launch main { land -(-9223372036854775807 - 1) }", 0);
}

#[test]
fn native_air_runtime_type_errors_match_interpreter() {
    for statement in [
        "payload value: int = air fire log(value + 1)",
        "payload value: int = air fire log(value < 1)",
        "payload value: int = air fire log(-value)",
        "payload value: flag = air scan value { land 7 }",
        "payload value: flag = air burn value { eject }",
        "payload value: flag = air fire log(not value)",
        "payload value: flag = air fire log(value and true)",
        "payload value: flag = air fire log(true and value)",
        "payload value: flag = air fire log(false or value)",
        "payload value: flag = air fire assert(value)",
        "payload value: str = air fire log(fire len(value))",
        "payload values: [int] = air fire log(values[0])",
        "fuel values: [int] = air values[0] = 1",
        "payload index: int = air payload values: [int] = [1] fire log(values[index])",
        "payload index: int = air fuel values: [int] = [1] values[index] = 2",
        "payload bound: int = air spin i in bound..1 { fire log(i) }",
        "payload bound: int = air spin i in 0..bound { fire log(i) }",
        "spin i in 0..=9223372036854775807 { eject }",
    ] {
        assert_native_matches(&format!("launch main {{ {statement} land 0 }}"), 1);
    }
}

#[test]
fn native_air_returns_abort_and_short_circuit_match_interpreter() {
    for (source, code) in [
        ("launch main { land air }", 0),
        ("ignite { land air } launch main { land 7 }", 7),
        ("ignite { abort } launch main { land 7 }", 1),
        ("launch main { abort }", 1),
        (
            "stage stop() lands int { abort } launch main { land fire stop() }",
            0,
        ),
        (
            "stage stop() lands int { land air } launch main { land fire stop() }",
            0,
        ),
        (
            "stage stop() lands int { } launch main { land fire stop() }",
            0,
        ),
        (
            "launch main { payload value: flag = air fire assert(true or value) fire assert(not (false and value)) land 0 }",
            0,
        ),
        (
            "launch main { payload result: flag = false and (0x01 < 0x02) land 0 }",
            0,
        ),
        (
            "launch main { payload result: flag = true or (1 / 0 == 0) land 0 }",
            0,
        ),
        (
            "launch main { payload result: flag = (false and (1 / 0 == 0)) or (10 / 2 == 5) land 0 }",
            0,
        ),
        (
            "launch main { payload left: int = air payload right: int = air fire assert(left == right) fire log(fire to_str(left)) land 0 }",
            0,
        ),
    ] {
        assert_native_matches(source, code);
    }
}

#[test]
fn native_imported_runtime_errors_preserve_source_locations() {
    for body in [
        "land 1 / 0",
        "payload values: [int] = [1] land values[2]",
        "fire assert(false) land 0",
    ] {
        let fixture = Fixture::new(r#"import "./helper.astrn" launch main { land fire fail() }"#);
        fs::write(
            fixture.0.join("helper.astrn"),
            format!("stage fail() lands int {{ {body} }}"),
        )
        .unwrap();
        assert_fixture_matches(&fixture, 1);
    }
}
