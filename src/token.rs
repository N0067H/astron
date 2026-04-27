#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, col: usize) -> Self {
        Token { kind, line, col }
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    // Keywords
    Launch,
    Ignite,
    Payload,
    Fuel,
    Stage,
    Fire,
    Scan,
    Fallback,
    Route,
    Orbit,
    Spin,
    Burn,
    Land,
    Lands,
    Eject,
    Pass,
    Abort,
    In,

    // Types
    Int,
    Float,
    Str,
    Flag,
    Byte,
    Void,
    Air,

    // Literals
    IntLit(i64),
    FloatLit(f64),
    StrLit(String),
    BoolLit(bool),
    Ident(String),

    // Arithmetic operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    // Comparison operators
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,

    // Assignment operators
    Assign,
    PlusAssign,
    MinusAssign,
    MulAssign,
    DivAssign,
    ModAssign,

    // Logical operators
    And,
    Or,
    Not,

    // Range & arrow
    DotDot,
    DotDotEq,
    FatArrow,

    // Separators
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Colon,

    Eof,
}
