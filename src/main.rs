mod ast;
mod checker;
mod interpreter;
mod lexer;
mod parser;
pub mod token;

use std::{env, fs};

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: astron <file.astrn>");
        std::process::exit(1);
    });

    let source = fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("error: cannot read '{}': {}", path, e);
        std::process::exit(1);
    });

    let (tokens, lex_errors) = lexer::tokenize(&source);
    let (program, parse_errors) = parser::parse(tokens);

    let had_syntax_error = !lex_errors.is_empty() || !parse_errors.is_empty();
    for e in &lex_errors {
        eprintln!("{}:{}:{}: {}", path, e.line, e.col, e.msg);
    }
    for e in &parse_errors {
        eprintln!("{}:{}:{}: {}", path, e.span.line, e.span.col, e.msg);
    }
    if had_syntax_error {
        std::process::exit(1);
    }

    let errors = checker::check(&program);
    if !errors.is_empty() {
        for e in &errors {
            eprintln!("{}:{}:{}: {}", path, e.span.line, e.span.col, e.msg);
        }
        std::process::exit(1);
    }

    let interp = interpreter::Interpreter::new(&program);
    interp.run(&program);
}
