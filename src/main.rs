mod ast;
mod interpreter;
mod lexer;
mod parser;
pub mod token;

use std::fs;

fn main() {
    let source = fs::read_to_string("exam/mission.astrn").expect("failed to read mission.astrn");
    let tokens = lexer::tokenize(&source);
    let program = parser::parse(tokens);
    let interp = interpreter::Interpreter::new(&program);
    interp.run(&program);
}
