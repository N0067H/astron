use astron::{frontend, interpreter};
use std::{env, path::PathBuf};

struct Cli {
    path: PathBuf,
    target: String,
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

    let mut path = None;
    let mut target = String::from("main");
    let mut index = 0;
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

    Cli { path, target }
}

fn usage() -> String {
    format!(
        "\
Astron {}

Usage:
  astron <file.astrn> [--launch <name>]
  astron --help
  astron --version

Options:
  --launch <name>  Select which launch block to run
  --help           Show this help message
  --version        Show the Astron version",
        env!("CARGO_PKG_VERSION")
    )
}
