use std::path::PathBuf;
use std::process::Command;

struct Case {
    args: Vec<&'static str>,
    expected_stdout: &'static str,
    expected_code: i32,
}

fn run_case(case: &Case) {
    let bin = env!("CARGO_BIN_EXE_astron");
    let output = Command::new(bin)
        .args(case.args.iter().copied())
        .current_dir(project_root())
        .output()
        .unwrap_or_else(|err| panic!("failed to run {:?}: {err}", case.args));

    let stdout = String::from_utf8(output.stdout)
        .unwrap_or_else(|err| panic!("stdout was not valid UTF-8 for {:?}: {err}", case.args));
    let stderr = String::from_utf8(output.stderr)
        .unwrap_or_else(|err| panic!("stderr was not valid UTF-8 for {:?}: {err}", case.args));
    let code = output
        .status
        .code()
        .unwrap_or_else(|| panic!("process terminated by signal for {:?}", case.args));

    assert_eq!(stdout, case.expected_stdout, "stdout mismatch for {:?}", case.args);
    assert_eq!(code, case.expected_code, "exit code mismatch for {:?}", case.args);
    assert!(
        stderr.is_empty(),
        "expected empty stderr for {:?}, got: {}",
        case.args,
        stderr
    );
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn example_programs_match_expected_output() {
    let cases = [
        Case {
            args: vec!["exam/01_hello_launches.astrn"],
            expected_stdout: "Astron Examples 1.0.0 main launch\nhello, world\n",
            expected_code: 0,
        },
        Case {
            args: vec!["exam/01_hello_launches.astrn", "--launch", "diagnostics"],
            expected_stdout: "Astron Examples 1.0.0 diagnostics launch\nall startup variables are available here\n",
            expected_code: 0,
        },
        Case {
            args: vec!["exam/02_navigation_math.astrn"],
            expected_stdout: "distance squared 100\nclassification far\n",
            expected_code: 0,
        },
        Case {
            args: vec!["exam/03_system_checklist.astrn"],
            expected_stdout: "systems needing attention 2\n",
            expected_code: 0,
        },
        Case {
            args: vec!["exam/04_orbit_counter.astrn"],
            expected_stdout: "odd lap 1\neven lap 2\nodd lap 3\nplanned orbit complete\n",
            expected_code: 0,
        },
        Case {
            args: vec!["exam/05_route_phases.astrn"],
            expected_stdout: "phase 1 preflight\nphase 2 launch\nphase 3 orbit\nphase 4 return\nselected checksum 0x2B\n",
            expected_code: 0,
        },
        Case {
            args: vec!["exam/06_countdown_burn.astrn"],
            expected_stdout: "countdown 5\ncountdown 4\ncountdown 3\ncountdown 2\ncountdown 1\nignition confirmed\n",
            expected_code: 0,
        },
        Case {
            args: vec!["exam/07_payload_manifest.astrn"],
            expected_stdout: "probe 30\nantenna 15\nbattery 40\ncamera 10\ntotal mass 95\nmanifest approved\n",
            expected_code: 0,
        },
        Case {
            args: vec!["exam/08_abort_sequence.astrn"],
            expected_stdout: "temperature 60\ntemperature 75\ntemperature 90\nthermal limit exceeded\n",
            expected_code: 1,
        },
        Case {
            args: vec!["exam/09_multi_launch_modes.astrn"],
            expected_stdout: "Astra simulation\ndefault launch path\n",
            expected_code: 0,
        },
        Case {
            args: vec!["exam/10_signal_strength.astrn"],
            expected_stdout: "attempt 1 stable 53\nattempt 2 stable 71\nattempt 3 excellent 89\nattempt 4 excellent 100\n",
            expected_code: 0,
        },
        Case {
            args: vec!["exam/11_import_mission_tools.astrn"],
            expected_stdout: "mission ASTRON ARTEMIS-GEO\nfuel margin 85\nstatus ready\n",
            expected_code: 0,
        },
        Case {
            args: vec!["exam/12_enum_phases.astrn"],
            expected_stdout: "orbit\n",
            expected_code: 0,
        },
        Case {
            args: vec!["exam/13_builtin_functions.astrn"],
            expected_stdout: "mission astron\nname length 6\nmodule count 3\nmodule count as text 3\nfirst module nav\n",
            expected_code: 0,
        },
    ];

    for case in &cases {
        run_case(case);
    }
}
