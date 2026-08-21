use crate::token::{Token, TokenKind};

pub struct LexError {
    pub line: usize,
    pub col: usize,
    pub msg: String,
}

struct Lexer {
    source: Vec<u8>,
    pos: usize,
    line: usize,
    col: usize,
    errors: Vec<LexError>,
}

impl Lexer {
    fn new(source: &str) -> Self {
        let mut bytes = source.as_bytes().to_vec();
        bytes.push(b'\0');
        bytes.push(b'\0');

        Lexer {
            source: bytes,
            pos: 0,
            line: 1,
            col: 1,
            errors: Vec::new(),
        }
    }

    fn peek(&self) -> u8 {
        self.source[self.pos]
    }

    fn peek_next(&self) -> u8 {
        self.source[self.pos + 1]
    }

    fn advance(&mut self) -> u8 {
        let c = self.source[self.pos];
        self.pos += 1;
        if c == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        c
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_ascii_whitespace() {
            self.advance();
        }
    }

    fn skip_comment(&mut self) {
        while self.peek() != b'\n' && self.peek() != b'\0' {
            self.advance();
        }
    }

    fn lex_number(&mut self) -> Token {
        let line = self.line;
        let col = self.col;
        let start = self.pos;

        if self.peek() == b'0' && matches!(self.peek_next(), b'x' | b'X') {
            self.advance();
            self.advance();
            while self.peek().is_ascii_hexdigit() {
                self.advance();
            }
            let s = std::str::from_utf8(&self.source[start..self.pos]).expect("invalid hex literal");
            match u8::from_str_radix(&s[2..], 16) {
                Ok(n) => return Token::new(TokenKind::ByteLit(n), line, col),
                Err(_) => {
                    self.errors.push(LexError {
                        line,
                        col,
                        msg: format!("hex literal '{}' out of byte range (0x00..=0xFF)", s),
                    });
                    return Token::new(TokenKind::ByteLit(0), line, col);
                }
            }
        }

        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == b'.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
            let s =
                std::str::from_utf8(&self.source[start..self.pos]).expect("invalid float literal");
            let f: f64 = s.parse().expect("invalid float literal");
            return Token::new(TokenKind::FloatLit(f), line, col);
        }

        let s = std::str::from_utf8(&self.source[start..self.pos]).expect("invalid int literal");
        let n: i64 = s.parse().expect("invalid int literal");
        Token::new(TokenKind::IntLit(n), line, col)
    }

    fn lex_string(&mut self) -> Token {
        let line = self.line;
        let col = self.col;
        self.advance();
        let mut bytes = Vec::new();

        while self.peek() != b'"' && self.peek() != b'\0' {
            if self.peek() == b'\\' {
                self.advance();
                let escaped = match self.advance() {
                    b'n' => b'\n',
                    b't' => b'\t',
                    b'"' => b'"',
                    b'\\' => b'\\',
                    c => c,
                };
                bytes.push(escaped);
            } else {
                bytes.push(self.advance());
            }
        }

        self.advance();
        let s = String::from_utf8(bytes).expect("invalid UTF-8 in string literal");
        Token::new(TokenKind::StrLit(s), line, col)
    }

    fn lex_ident_or_keyword(&mut self) -> Token {
        let line = self.line;
        let col = self.col;
        let start = self.pos;

        while self.peek().is_ascii_alphanumeric() || self.peek() == b'_' {
            self.advance();
        }

        let word = std::str::from_utf8(&self.source[start..self.pos])
            .expect("invalid identifier")
            .to_owned();

        let kind = match word.as_str() {
            "launch" => TokenKind::Launch,
            "ignite" => TokenKind::Ignite,
            "payload" => TokenKind::Payload,
            "fuel" => TokenKind::Fuel,
            "stage" => TokenKind::Stage,
            "enum" => TokenKind::Enum,
            "fire" => TokenKind::Fire,
            "lands" => TokenKind::Lands,
            "land" => TokenKind::Land,
            "scan" => TokenKind::Scan,
            "fallback" => TokenKind::Fallback,
            "route" => TokenKind::Route,
            "orbit" => TokenKind::Orbit,
            "spin" => TokenKind::Spin,
            "burn" => TokenKind::Burn,
            "eject" => TokenKind::Eject,
            "pass" => TokenKind::Pass,
            "abort" => TokenKind::Abort,
            "in" => TokenKind::In,
            "import" => TokenKind::Import,
            "int" => TokenKind::Int,
            "float" => TokenKind::Float,
            "str" => TokenKind::Str,
            "flag" => TokenKind::Flag,
            "byte" => TokenKind::Byte,
            "void" => TokenKind::Void,
            "air" => TokenKind::Air,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            "not" => TokenKind::Not,
            "true" => TokenKind::BoolLit(true),
            "false" => TokenKind::BoolLit(false),
            _ => TokenKind::Ident(word),
        };

        Token::new(kind, line, col)
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.peek() == b'/' && self.peek_next() == b'/' {
            self.advance();
            self.advance();
            self.skip_comment();
            return self.next_token();
        }

        let line = self.line;
        let col = self.col;
        let c = self.peek();

        if c == b'\0' {
            return Token::new(TokenKind::Eof, line, col);
        }

        if c.is_ascii_digit() {
            return self.lex_number();
        }

        if c == b'"' {
            return self.lex_string();
        }

        if c.is_ascii_alphabetic() || c == b'_' {
            return self.lex_ident_or_keyword();
        }

        self.advance();

        let kind = match c {
            b'+' => match self.peek() {
                b'=' => {
                    self.advance();
                    TokenKind::PlusAssign
                }
                _ => TokenKind::Plus,
            },
            b'-' => match self.peek() {
                b'=' => {
                    self.advance();
                    TokenKind::MinusAssign
                }
                _ => TokenKind::Minus,
            },
            b'*' => match self.peek() {
                b'=' => {
                    self.advance();
                    TokenKind::MulAssign
                }
                _ => TokenKind::Star,
            },
            b'/' => match self.peek() {
                b'=' => {
                    self.advance();
                    TokenKind::DivAssign
                }
                _ => TokenKind::Slash,
            },
            b'%' => match self.peek() {
                b'=' => {
                    self.advance();
                    TokenKind::ModAssign
                }
                _ => TokenKind::Percent,
            },
            b'=' => match self.peek() {
                b'=' => {
                    self.advance();
                    TokenKind::Eq
                }
                b'>' => {
                    self.advance();
                    TokenKind::FatArrow
                }
                _ => TokenKind::Assign,
            },
            b'!' => match self.peek() {
                b'=' => {
                    self.advance();
                    TokenKind::NotEq
                }
                _ => {
                    self.errors.push(LexError {
                        line,
                        col,
                        msg: "'!' must be followed by '='".to_string(),
                    });
                    return self.next_token();
                }
            },
            b'<' => match self.peek() {
                b'=' => {
                    self.advance();
                    TokenKind::LtEq
                }
                _ => TokenKind::Lt,
            },
            b'>' => match self.peek() {
                b'=' => {
                    self.advance();
                    TokenKind::GtEq
                }
                _ => TokenKind::Gt,
            },
            b'.' => match self.peek() {
                b'.' => {
                    self.advance();
                    match self.peek() {
                        b'=' => {
                            self.advance();
                            TokenKind::DotDotEq
                        }
                        _ => TokenKind::DotDot,
                    }
                }
                _ => {
                    self.errors.push(LexError {
                        line,
                        col,
                        msg: "unexpected '.'".to_string(),
                    });
                    return self.next_token();
                }
            },
            b'{' => TokenKind::LBrace,
            b'}' => TokenKind::RBrace,
            b'(' => TokenKind::LParen,
            b')' => TokenKind::RParen,
            b'[' => TokenKind::LBracket,
            b']' => TokenKind::RBracket,
            b',' => TokenKind::Comma,
            b':' => TokenKind::Colon,
            _ => {
                self.errors.push(LexError {
                    line,
                    col,
                    msg: format!("unexpected character '{}'", c as char),
                });
                return self.next_token();
            }
        };

        Token::new(kind, line, col)
    }
}

pub fn tokenize(source: &str) -> (Vec<Token>, Vec<LexError>) {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();

    loop {
        let tok = lexer.next_token();
        let is_eof = tok.kind == TokenKind::Eof;
        tokens.push(tok);
        if is_eof {
            break;
        }
    }

    (tokens, lexer.errors)
}
