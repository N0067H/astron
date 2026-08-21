use std::hint::black_box;
use std::time::Instant;

struct OldLexer {
    source: Vec<char>,
    pos: usize,
}

struct NewLexer {
    source: Vec<u8>,
    pos: usize,
}

impl OldLexer {
    fn new(source: &str) -> Self {
        let mut chars = source.chars().collect::<Vec<_>>();
        chars.push('\0');
        chars.push('\0');
        Self { source: chars, pos: 0 }
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
        c
    }

    fn scan(mut self) -> usize {
        let mut tokens = 0usize;
        loop {
            while self.peek().is_whitespace() {
                self.advance();
            }

            if self.peek() == '/' && self.peek_next() == '/' {
                while self.peek() != '\n' && self.peek() != '\0' {
                    self.advance();
                }
                continue;
            }

            let c = self.peek();
            if c == '\0' {
                break;
            }

            if c.is_ascii_digit() {
                while self.peek().is_ascii_digit() {
                    self.advance();
                }
                tokens += 1;
                continue;
            }

            if c.is_alphanumeric() || c == '_' {
                while self.peek().is_alphanumeric() || self.peek() == '_' {
                    self.advance();
                }
                tokens += 1;
                continue;
            }

            self.advance();
            tokens += 1;
        }

        black_box(tokens)
    }
}

impl NewLexer {
    fn new(source: &str) -> Self {
        let mut bytes = source.as_bytes().to_vec();
        bytes.push(b'\0');
        bytes.push(b'\0');
        Self { source: bytes, pos: 0 }
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
        c
    }

    fn scan(mut self) -> usize {
        let mut tokens = 0usize;
        loop {
            while self.peek().is_ascii_whitespace() {
                self.advance();
            }

            if self.peek() == b'/' && self.peek_next() == b'/' {
                while self.peek() != b'\n' && self.peek() != b'\0' {
                    self.advance();
                }
                continue;
            }

            let c = self.peek();
            if c == b'\0' {
                break;
            }

            if c.is_ascii_digit() {
                while self.peek().is_ascii_digit() {
                    self.advance();
                }
                tokens += 1;
                continue;
            }

            if c.is_ascii_alphanumeric() || c == b'_' {
                while self.peek().is_ascii_alphanumeric() || self.peek() == b'_' {
                    self.advance();
                }
                tokens += 1;
                continue;
            }

            self.advance();
            tokens += 1;
        }

        black_box(tokens)
    }
}

fn build_source() -> String {
    let mut source = String::with_capacity(1_200_000);
    for index in 0..20_000 {
        source.push_str("// bench comment for lexer locality\n");
        source.push_str(&format!("fuel signal_{}: int = {}\n", index, index));
        source.push_str(&format!("scan signal_{} >= 10 {{\n", index));
        source.push_str("    fire log(signal_0)\n");
        source.push_str("}\n");
    }
    source
}

fn measure_old(source: &str) -> f64 {
    let start = Instant::now();
    black_box(OldLexer::new(source).scan());
    start.elapsed().as_secs_f64() * 1000.0
}

fn measure_new(source: &str) -> f64 {
    let start = Instant::now();
    black_box(NewLexer::new(source).scan());
    start.elapsed().as_secs_f64() * 1000.0
}

fn main() {
    let source = build_source();
    black_box(measure_old(&source));
    black_box(measure_new(&source));

    for trial in 1..=10 {
        println!(
            "trial={} before_ms={:.3} after_ms={:.3}",
            trial,
            measure_old(&source),
            measure_new(&source)
        );
    }
}
