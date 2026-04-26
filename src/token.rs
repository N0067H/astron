struct Token {
    kind: TokenKind,
    line: usize,
}

enum TokenKind {                                                                                                                       
    // Keywords
    Launch, Ignite, Payload, Fuel, Stage, Fire,
    Scan, Fallback, Route, Orbit, Spin, Burn,
    Land, Lands, Eject, Pass, Abort, In,

    // Types
    Int, Float, Str, Flag, Byte, Void, Air,
                
    // Literals
    IntLit(i64),
    FloatLit(f64),
    StrLit(String),
    Ident(String),

    // Operators
    Plus, Minus, Star, Slash, Percent,
    Eq, NotEq, Lt, Gt, LtEq, GtEq,
    Assign, PlusAssign, MinusAssign,
    And, Or, Not,
    DotDot, DotDotEq,
    FatArrow,

    // Separators
    LBrace, RBrace, LParen, RParen, LBracket, RBracket,
    Comma, Colon,

    Eof,
}
