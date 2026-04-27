mod ast;
mod lexer;
mod parser;
pub mod token;

use std::fs;
use token::TokenKind;

fn main() {
    let source = fs::read_to_string("exam/mission.astrn").expect("failed to read mission.astrn");
    let tokens = lexer::tokenize(&source);
    let program = parser::parse(tokens);
    println!("{program:#?}");
}
