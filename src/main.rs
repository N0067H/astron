mod ast;
mod lexer;
pub mod token;

use std::fs;
use token::TokenKind;

fn main() {
    let source = fs::read_to_string("exam/mission.astrn").expect("failed to read mission.astrn");
    let tokens = lexer::tokenize(&source);

    for tok in &tokens {
        let label = match &tok.kind {
            TokenKind::Ident(s) => format!("Ident({s})"),
            TokenKind::StrLit(s) => format!("StrLit({s:?})"),
            TokenKind::IntLit(n) => format!("IntLit({n})"),
            TokenKind::FloatLit(f) => format!("FloatLit({f})"),
            TokenKind::BoolLit(b) => format!("BoolLit({b})"),
            other => format!("{other:?}"),
        };
        println!("line {:>3}:{:<3} | {label}", tok.line, tok.col);
    }
}
