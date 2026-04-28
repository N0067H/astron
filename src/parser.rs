use crate::ast::*;
use crate::token::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    fn span(&self) -> Span {
        let t = self.peek();
        Span::new(t.line, t.col)
    }

    fn advance(&mut self) {
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Span {
        let span = self.span();
        if self.peek_kind() != &kind {
            panic!(
                "line {}:{}: expected {:?}, found {:?}",
                span.line,
                span.col,
                kind,
                self.peek_kind()
            );
        }
        self.advance();
        span
    }

    fn eat(&mut self, kind: TokenKind) -> bool {
        if self.peek_kind() == &kind {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_ident(&mut self) -> (String, Span) {
        let span = self.span();
        match self.peek_kind().clone() {
            TokenKind::Ident(s) => {
                self.advance();
                (s, span)
            }
            _ => panic!(
                "line {}:{}: expected identifier, found {:?}",
                span.line,
                span.col,
                self.peek_kind()
            ),
        }
    }

    fn is_expr_start(&self) -> bool {
        matches!(
            self.peek_kind(),
            TokenKind::IntLit(_)
                | TokenKind::FloatLit(_)
                | TokenKind::ByteLit(_)
                | TokenKind::StrLit(_)
                | TokenKind::BoolLit(_)
                | TokenKind::Air
                | TokenKind::Ident(_)
                | TokenKind::Fire
                | TokenKind::LParen
                | TokenKind::LBracket
                | TokenKind::Not
                | TokenKind::Minus
        )
    }

    fn parse_type(&mut self) -> Type {
        let span = self.span();
        match self.peek_kind().clone() {
            TokenKind::Int => {
                self.advance();
                Type::Int
            }
            TokenKind::Float => {
                self.advance();
                Type::Float
            }
            TokenKind::Str => {
                self.advance();
                Type::Str
            }
            TokenKind::Flag => {
                self.advance();
                Type::Flag
            }
            TokenKind::Byte => {
                self.advance();
                Type::Byte
            }
            TokenKind::Void => {
                self.advance();
                Type::Void
            }
            TokenKind::LBracket => {
                self.advance();
                let inner = self.parse_type();
                self.expect(TokenKind::RBracket);
                Type::Array(Box::new(inner))
            }
            _ => panic!(
                "line {}:{}: expected type, found {:?}",
                span.line,
                span.col,
                self.peek_kind()
            ),
        }
    }

    fn parse_expr(&mut self) -> Expr {
        let lhs = self.parse_or();
        match self.peek_kind() {
            TokenKind::DotDot | TokenKind::DotDotEq => {
                let span = lhs.span;
                let inclusive = matches!(self.peek_kind(), TokenKind::DotDotEq);
                self.advance();
                let rhs = self.parse_or();
                Spanned::new(
                    ExprKind::Range {
                        start: Box::new(lhs),
                        end: Box::new(rhs),
                        inclusive,
                    },
                    span,
                )
            }
            _ => lhs,
        }
    }

    fn parse_or(&mut self) -> Expr {
        let mut lhs = self.parse_and();
        while self.eat(TokenKind::Or) {
            let span = lhs.span;
            let rhs = self.parse_and();
            lhs = Spanned::new(
                ExprKind::Binary {
                    op: BinOp::Or,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            );
        }
        lhs
    }

    fn parse_and(&mut self) -> Expr {
        let mut lhs = self.parse_cmp();
        while self.eat(TokenKind::And) {
            let span = lhs.span;
            let rhs = self.parse_cmp();
            lhs = Spanned::new(
                ExprKind::Binary {
                    op: BinOp::And,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            );
        }
        lhs
    }

    fn parse_cmp(&mut self) -> Expr {
        let lhs = self.parse_add();
        let op = match self.peek_kind() {
            TokenKind::Eq => BinOp::Eq,
            TokenKind::NotEq => BinOp::NotEq,
            TokenKind::Lt => BinOp::Lt,
            TokenKind::Gt => BinOp::Gt,
            TokenKind::LtEq => BinOp::LtEq,
            TokenKind::GtEq => BinOp::GtEq,
            _ => return lhs,
        };
        let span = lhs.span;
        self.advance();
        let rhs = self.parse_add();
        Spanned::new(
            ExprKind::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            span,
        )
    }

    fn parse_add(&mut self) -> Expr {
        let mut lhs = self.parse_mul();
        loop {
            let op = match self.peek_kind() {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };
            let span = lhs.span;
            self.advance();
            let rhs = self.parse_mul();
            lhs = Spanned::new(
                ExprKind::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            );
        }
        lhs
    }

    fn parse_mul(&mut self) -> Expr {
        let mut lhs = self.parse_unary();
        loop {
            let op = match self.peek_kind() {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Percent => BinOp::Mod,
                _ => break,
            };
            let span = lhs.span;
            self.advance();
            let rhs = self.parse_unary();
            lhs = Spanned::new(
                ExprKind::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            );
        }
        lhs
    }

    fn parse_unary(&mut self) -> Expr {
        let span = self.span();
        match self.peek_kind() {
            TokenKind::Not => {
                self.advance();
                let expr = self.parse_unary();
                Spanned::new(
                    ExprKind::Unary {
                        op: UnOp::Not,
                        expr: Box::new(expr),
                    },
                    span,
                )
            }
            TokenKind::Minus => {
                self.advance();
                let expr = self.parse_unary();
                Spanned::new(
                    ExprKind::Unary {
                        op: UnOp::Neg,
                        expr: Box::new(expr),
                    },
                    span,
                )
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Expr {
        let mut expr = self.parse_primary();
        while self.eat(TokenKind::LBracket) {
            let span = expr.span;
            let index = self.parse_expr();
            self.expect(TokenKind::RBracket);
            expr = Spanned::new(
                ExprKind::Index {
                    array: Box::new(expr),
                    index: Box::new(index),
                },
                span,
            );
        }
        expr
    }

    fn parse_primary(&mut self) -> Expr {
        let span = self.span();
        match self.peek_kind().clone() {
            TokenKind::IntLit(n) => {
                self.advance();
                Spanned::new(ExprKind::IntLit(n), span)
            }
            TokenKind::FloatLit(f) => {
                self.advance();
                Spanned::new(ExprKind::FloatLit(f), span)
            }
            TokenKind::StrLit(s) => {
                self.advance();
                Spanned::new(ExprKind::StrLit(s), span)
            }
            TokenKind::BoolLit(b) => {
                self.advance();
                Spanned::new(ExprKind::BoolLit(b), span)
            }
            TokenKind::ByteLit(b) => {
                self.advance();
                Spanned::new(ExprKind::ByteLit(b), span)
            }
            TokenKind::Air => {
                self.advance();
                Spanned::new(ExprKind::Air, span)
            }
            TokenKind::Ident(name) => {
                self.advance();
                Spanned::new(ExprKind::Ident(name), span)
            }
            TokenKind::Fire => {
                self.advance();
                let (name, _) = self.expect_ident();
                self.expect(TokenKind::LParen);
                let args = self.parse_args();
                self.expect(TokenKind::RParen);
                Spanned::new(ExprKind::Call { name, args }, span)
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr();
                self.expect(TokenKind::RParen);
                expr
            }
            TokenKind::LBracket => {
                self.advance();
                let mut elems = Vec::new();
                if self.is_expr_start() {
                    elems.push(self.parse_expr());
                    while self.eat(TokenKind::Comma) {
                        elems.push(self.parse_expr());
                    }
                }
                self.expect(TokenKind::RBracket);
                Spanned::new(ExprKind::ArrayLit(elems), span)
            }
            _ => panic!(
                "line {}:{}: unexpected token in expression: {:?}",
                span.line,
                span.col,
                self.peek_kind()
            ),
        }
    }

    fn parse_args(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();
        if self.is_expr_start() {
            args.push(self.parse_expr());
            while self.eat(TokenKind::Comma) {
                args.push(self.parse_expr());
            }
        }
        args
    }

    fn parse_block(&mut self) -> Vec<Stmt> {
        self.expect(TokenKind::LBrace);
        let mut stmts = Vec::new();
        while !matches!(self.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
            stmts.push(self.parse_stmt());
        }
        self.expect(TokenKind::RBrace);
        stmts
    }

    fn parse_stmt(&mut self) -> Stmt {
        let span = self.span();
        match self.peek_kind().clone() {
            TokenKind::Payload => {
                self.advance();
                self.parse_let(false, span)
            }
            TokenKind::Fuel => {
                self.advance();
                self.parse_let(true, span)
            }
            TokenKind::Scan => {
                self.advance();
                let cond = self.parse_expr();
                let then_block = self.parse_block();
                let else_block = if self.eat(TokenKind::Fallback) {
                    Some(self.parse_block())
                } else {
                    None
                };
                Spanned::new(
                    StmtKind::If {
                        cond,
                        then_block,
                        else_block,
                    },
                    span,
                )
            }
            TokenKind::Route => {
                self.advance();
                let subject = self.parse_expr();
                self.expect(TokenKind::LBrace);
                let mut arms = Vec::new();
                while !matches!(self.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
                    let pattern = self.parse_expr();
                    self.expect(TokenKind::FatArrow);
                    let body = self.parse_stmt();
                    arms.push(RouteArm {
                        pattern,
                        body: Box::new(body),
                    });
                }
                self.expect(TokenKind::RBrace);
                Spanned::new(StmtKind::Route { subject, arms }, span)
            }
            TokenKind::Burn => {
                self.advance();
                let cond = self.parse_expr();
                let body = self.parse_block();
                Spanned::new(StmtKind::Burn { cond, body }, span)
            }
            TokenKind::Spin => {
                self.advance();
                let (var, _) = self.expect_ident();
                self.expect(TokenKind::In);
                let range = self.parse_expr();
                let body = self.parse_block();
                Spanned::new(StmtKind::Spin { var, range, body }, span)
            }
            TokenKind::Orbit => {
                self.advance();
                let body = self.parse_block();
                Spanned::new(StmtKind::Orbit { body }, span)
            }
            TokenKind::Fire => {
                self.advance();
                let (name, _) = self.expect_ident();
                self.expect(TokenKind::LParen);
                let args = self.parse_args();
                self.expect(TokenKind::RParen);
                Spanned::new(StmtKind::Fire { name, args }, span)
            }
            TokenKind::Land => {
                self.advance();
                let expr = if self.is_expr_start() {
                    Some(self.parse_expr())
                } else {
                    None
                };
                Spanned::new(StmtKind::Land(expr), span)
            }
            TokenKind::Eject => {
                self.advance();
                Spanned::new(StmtKind::Eject, span)
            }
            TokenKind::Pass => {
                self.advance();
                Spanned::new(StmtKind::Pass, span)
            }
            TokenKind::Abort => {
                self.advance();
                Spanned::new(StmtKind::Abort, span)
            }
            TokenKind::Ident(_) => self.parse_assign(span),
            _ => panic!(
                "line {}:{}: unexpected token at start of statement: {:?}",
                span.line,
                span.col,
                self.peek_kind()
            ),
        }
    }

    fn parse_let(&mut self, mutable: bool, span: Span) -> Stmt {
        let (name, _) = self.expect_ident();
        self.expect(TokenKind::Colon);
        let ty = self.parse_type();
        self.expect(TokenKind::Assign);
        let init = self.parse_expr();
        Spanned::new(
            StmtKind::Let {
                mutable,
                name,
                ty,
                init,
            },
            span,
        )
    }

    fn parse_assign(&mut self, span: Span) -> Stmt {
        let (name, _) = self.expect_ident();

        let target = if self.eat(TokenKind::LBracket) {
            let index = self.parse_expr();
            self.expect(TokenKind::RBracket);
            AssignTarget::Index {
                name,
                index: Box::new(index),
            }
        } else {
            AssignTarget::Ident(name)
        };

        let op = match self.peek_kind() {
            TokenKind::Assign => AssignOp::Assign,
            TokenKind::PlusAssign => AssignOp::Add,
            TokenKind::MinusAssign => AssignOp::Sub,
            TokenKind::MulAssign => AssignOp::Mul,
            TokenKind::DivAssign => AssignOp::Div,
            TokenKind::ModAssign => AssignOp::Mod,
            _ => panic!(
                "line {}:{}: expected assignment operator, found {:?}",
                span.line,
                span.col,
                self.peek_kind()
            ),
        };
        self.advance();

        let value = self.parse_expr();
        Spanned::new(StmtKind::Assign { target, op, value }, span)
    }

    fn parse_params(&mut self) -> Vec<Param> {
        self.expect(TokenKind::LParen);
        let mut params = Vec::new();
        if !matches!(self.peek_kind(), TokenKind::RParen) {
            let (name, _) = self.expect_ident();
            self.expect(TokenKind::Colon);
            params.push(Param {
                name,
                ty: self.parse_type(),
            });
            while self.eat(TokenKind::Comma) {
                let (name, _) = self.expect_ident();
                self.expect(TokenKind::Colon);
                params.push(Param {
                    name,
                    ty: self.parse_type(),
                });
            }
        }
        self.expect(TokenKind::RParen);
        params
    }

    fn parse_stage(&mut self) -> Item {
        let (name, _) = self.expect_ident();
        let params = self.parse_params();
        self.expect(TokenKind::Lands);
        let ret_type = self.parse_type();
        let body = self.parse_block();
        Item::Stage {
            name,
            params,
            ret_type,
            body,
        }
    }

    fn parse_launch(&mut self) -> Item {
        let (name, _) = self.expect_ident();
        let body = self.parse_block();
        Item::Launch { name, body }
    }

    pub fn parse(&mut self) -> Program {
        let ignite_body = if self.eat(TokenKind::Ignite) {
            self.parse_block()
        } else {
            Vec::new()
        };

        let mut items = Vec::new();
        while !matches!(self.peek_kind(), TokenKind::Eof) {
            match self.peek_kind() {
                TokenKind::Stage => {
                    self.advance();
                    items.push(self.parse_stage());
                }
                TokenKind::Launch => {
                    self.advance();
                    items.push(self.parse_launch());
                }
                _ => panic!(
                    "line {}:{}: expected 'stage' or 'launch', found {:?}",
                    self.peek().line,
                    self.peek().col,
                    self.peek_kind()
                ),
            }
        }

        Program { ignite_body, items }
    }
}

pub fn parse(tokens: Vec<Token>) -> Program {
    Parser::new(tokens).parse()
}
