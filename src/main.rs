use astron::{backend, frontend, interpreter};
use std::{env, path::PathBuf};

struct Cli {
    path: PathBuf,
    target: String,
    build: bool,
    output: Option<PathBuf>,
}

fn main() {
    let cli = parse_cli(env::args().skip(1).collect());

    let loaded = match frontend::check_program(&cli.path) {
        Ok(program) => program,
        Err(errors) => {
            for e in errors {
                eprintln!("{e}");
            }
            std::process::exit(1);
        }
    };

    if cli.build {
        let output = cli
            .output
            .unwrap_or_else(|| cli.path.with_extension(std::env::consts::EXE_EXTENSION));
        if let Err(error) = backend::build(&loaded, &cli.target, &output) {
            eprintln!("build error: {error}");
            std::process::exit(1);
        }
        println!("Built {}", output.display());
        return;
    }

    let interp = interpreter::Interpreter::new(loaded.program());
    match interp.run(&cli.target) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            let source_path = loaded.source_path(error.span);
            eprintln!(
                "{}:{}:{}: runtime error: {}",
                source_path.display(),
                error.span.line,
                error.span.col,
                error.msg
            );
            std::process::exit(1);
        }
    }
}

fn parse_cli(args: Vec<String>) -> Cli {
    if args.is_empty() {
        eprintln!("{}", usage());
        std::process::exit(1);
    }

    let build = args.first().is_some_and(|arg| arg == "build");
    let mut output = None;
    let mut path = None;
    let mut target = String::from("main");
    let mut index = usize::from(build);
    while index < args.len() {
        match args[index].as_str() {
            "--help" => {
                println!("{}", usage());
                std::process::exit(0);
            }
            "--version" => {
                println!("{}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "-o" | "--output" if build => {
                index += 1;
                let Some(value) = args.get(index) else {
                    eprintln!("error: --output requires a path\n\n{}", usage());
                    std::process::exit(1);
                };
                output = Some(PathBuf::from(value));
            }
            "--launch" => {
                index += 1;
                let Some(name) = args.get(index) else {
                    eprintln!("error: --launch requires a name\n\n{}", usage());
                    std::process::exit(1);
                };
                target = name.clone();
            }
            arg if arg.starts_with("--") => {
                eprintln!("error: unknown option '{arg}'\n\n{}", usage());
                std::process::exit(1);
            }
            arg => {
                if path.is_some() {
                    eprintln!(
                        "error: unexpected positional argument '{arg}'\n\n{}",
                        usage()
                    );
                    std::process::exit(1);
                }
                path = Some(PathBuf::from(arg));
            }
        }
        index += 1;
    }

    let Some(path) = path else {
        eprintln!("error: missing input file\n\n{}", usage());
        std::process::exit(1);
    };

    Cli {
        path,
        target,
        build,
        output,
    }
}

fn usage() -> String {
    format!(
        "\
Astron {}

Usage:
  astron <file.astrn> [--launch <name>]
  astron build <file.astrn> [--launch <name>] [-o <path>]
  astron --help
  astron --version

Options:
  -o, --output <path>  Native executable path (build only)
  --launch <name>  Select which launch block to run
  --help           Show this help message
  --version        Show the Astron version",
        env!("CARGO_PKG_VERSION")
    )
}
