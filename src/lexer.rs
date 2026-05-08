use crate::token::{Token, TokenKind};

pub struct LexError {
    pub line: usize,
    pub col: usize,
    pub msg: String,
}

struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    errors: Vec<LexError>,
}

impl Lexer {
    fn new(source: &str) -> Self {
        let mut chars: Vec<char> = source.chars().collect();
        chars.push('\0');
        chars.push('\0');

        Lexer {
            source: chars,
            pos: 0,
            line: 1,
            col: 1,
            errors: Vec::new(),
        }
    }

    fn peek(&self) -> char {
        self.source[self.pos]
    }

    fn peek_next(&self) -> char {
        self.source[self.pos + 1]
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.pos];
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        c
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_whitespace() {
            self.advance();
        }
    }

    fn skip_comment(&mut self) {
        while self.peek() != '\n' && self.peek() != '\0' {
            self.advance();
        }
    }

    fn lex_number(&mut self) -> Token {
        let line = self.line;
        let col = self.col;
        let start = self.pos;

        if self.peek() == '0' && (self.peek_next() == 'x' || self.peek_next() == 'X') {
            self.advance();
            self.advance();
            while self.peek().is_ascii_hexdigit() {
                self.advance();
            }
            let s: String = self.source[start..self.pos].iter().collect();
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

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
            let s: String = self.source[start..self.pos].iter().collect();
            let f: f64 = s.parse().expect("invalid float literal");
            return Token::new(TokenKind::FloatLit(f), line, col);
        }

        let s: String = self.source[start..self.pos].iter().collect();
        let n: i64 = s.parse().expect("invalid int literal");
        Token::new(TokenKind::IntLit(n), line, col)
    }

    fn lex_string(&mut self) -> Token {
        let line = self.line;
        let col = self.col;
        self.advance(); // opening '"'
        let mut s = String::new();

        while self.peek() != '"' && self.peek() != '\0' {
            if self.peek() == '\\' {
                self.advance();
                let escaped = match self.advance() {
                    'n' => '\n',
                    't' => '\t',
                    '"' => '"',
                    '\\' => '\\',
                    c => c,
                };
                s.push(escaped);
            } else {
                s.push(self.advance());
            }
        }

        self.advance(); // closing '"'
        Token::new(TokenKind::StrLit(s), line, col)
    }

    fn lex_ident_or_keyword(&mut self) -> Token {
        let line = self.line;
        let col = self.col;
        let start = self.pos;

        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let word: String = self.source[start..self.pos].iter().collect();

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

        if self.peek() == '/' && self.peek_next() == '/' {
            self.advance();
            self.advance();
            self.skip_comment();
            return self.next_token();
        }

        let line = self.line;
        let col = self.col;
        let c = self.peek();

        if c == '\0' {
            return Token::new(TokenKind::Eof, line, col);
        }

        if c.is_ascii_digit() {
            return self.lex_number();
        }

        if c == '"' {
            return self.lex_string();
        }

        if c.is_alphabetic() || c == '_' {
            return self.lex_ident_or_keyword();
        }

        self.advance();

        let kind = match c {
            '+' => match self.peek() {
                '=' => {
                    self.advance();
                    TokenKind::PlusAssign
                }
                _ => TokenKind::Plus,
            },
            '-' => match self.peek() {
                '=' => {
                    self.advance();
                    TokenKind::MinusAssign
                }
                _ => TokenKind::Minus,
            },
            '*' => match self.peek() {
                '=' => {
                    self.advance();
                    TokenKind::MulAssign
                }
                _ => TokenKind::Star,
            },
            '/' => match self.peek() {
                '=' => {
                    self.advance();
                    TokenKind::DivAssign
                }
                _ => TokenKind::Slash,
            },
            '%' => match self.peek() {
                '=' => {
                    self.advance();
                    TokenKind::ModAssign
                }
                _ => TokenKind::Percent,
            },
            '=' => match self.peek() {
                '=' => {
                    self.advance();
                    TokenKind::Eq
                }
                '>' => {
                    self.advance();
                    TokenKind::FatArrow
                }
                _ => TokenKind::Assign,
            },
            '!' => match self.peek() {
                '=' => {
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
            '<' => match self.peek() {
                '=' => {
                    self.advance();
                    TokenKind::LtEq
                }
                _ => TokenKind::Lt,
            },
            '>' => match self.peek() {
                '=' => {
                    self.advance();
                    TokenKind::GtEq
                }
                _ => TokenKind::Gt,
            },
            '.' => match self.peek() {
                '.' => {
                    self.advance();
                    match self.peek() {
                        '=' => {
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
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            ',' => TokenKind::Comma,
            ':' => TokenKind::Colon,
            _ => {
                self.errors.push(LexError {
                    line,
                    col,
                    msg: format!("unexpected character '{}'", c),
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
