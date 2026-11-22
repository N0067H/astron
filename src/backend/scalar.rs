use super::unsupported;
use crate::{ast::*, frontend::CheckedProgram};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Int,
    Float,
    Flag,
    Byte,
}

impl Kind {
    fn ir(self) -> &'static str {
        match self {
            Self::Int => "i64",
            Self::Float => "double",
            Self::Flag => "i1",
            Self::Byte => "i8",
        }
    }
}

#[derive(Clone)]
struct Value {
    kind: Kind,
    operand: String,
}

impl Value {
    fn new(kind: Kind, operand: impl ToString) -> Self {
        Self {
            kind,
            operand: operand.to_string(),
        }
    }
}

struct Generator<'a> {
    checked: &'a CheckedProgram,
    locals: HashMap<String, Value>,
    ir: String,
    next: usize,
    block: String,
    globals: String,
}

pub(super) fn compile(checked: &CheckedProgram, body: &[Stmt]) -> Result<String, String> {
    let mut generator = Generator {
        checked,
        locals: HashMap::new(),
        ir: String::from("define i32 @main() {\nentry:\n"),
        next: 0,
        block: String::from("entry"),
        globals: String::new(),
    };
    for stmt in body {
        if generator.statement(stmt)? {
            generator.ir.push_str("}\n");
            return Ok(generator.finish());
        }
    }

    generator.ir.push_str("  ret i32 0\n}\n");
    Ok(generator.finish())
}

impl Generator<'_> {
    fn finish(self) -> String {
        if self.globals.is_empty() {
            return self.ir;
        }
        let stream = if cfg!(windows) {
            "declare ptr @__acrt_iob_func(i32)\n"
        } else {
            stderr_declaration()
        };
        format!(
            "{stream}declare i64 @fwrite(ptr, i64, i64, ptr)\ndeclare void @exit(i32) noreturn\n{}{}",
            self.globals, self.ir
        )
    }

    fn guard(&mut self, condition: Value, span: Span, message: &str) {
        let id = self.next;
        self.next += 1;
        let diagnostic = format!(
            "{}:{}:{}: runtime error: {message}\n",
            self.checked.source_path(span).display(),
            span.line,
            span.col
        );
        let bytes = diagnostic
            .bytes()
            .map(|b| format!("\\{b:02X}"))
            .collect::<String>();
        let size = diagnostic.len();
        self.globals.push_str(&format!(
            "@error{id} = private constant [{size} x i8] c\"{bytes}\"\n"
        ));
        self.ir.push_str(&format!(
            "  br i1 {}, label %error{id}, label %checked{id}\nerror{id}:\n",
            condition.operand
        ));
        let stream = if cfg!(windows) {
            String::from("call ptr @__acrt_iob_func(i32 2)")
        } else {
            format!("load ptr, ptr @{}", stderr_symbol())
        };
        self.ir.push_str(&format!("  %stderr{id} = {stream}\n  call i64 @fwrite(ptr @error{id}, i64 1, i64 {size}, ptr %stderr{id})\n  call void @exit(i32 1)\n  unreachable\nchecked{id}:\n"));
        self.block = format!("checked{id}");
    }

    fn division_checks(&mut self, op: &BinOp, lhs: &Value, rhs: &Value, span: Span) {
        let (compare, zero) = if rhs.kind == Kind::Float {
            ("fcmp oeq", "0.0")
        } else {
            ("icmp eq", "0")
        };
        let condition = self.instruction(
            Kind::Flag,
            format!("{compare} {} {}, {zero}", rhs.kind.ir(), rhs.operand),
        );
        let message = if *op == BinOp::Div {
            "division by zero"
        } else {
            "modulo by zero"
        };
        self.guard(condition, span, message);
        if lhs.kind != Kind::Int {
            return;
        }
        let minimum = self.instruction(
            Kind::Flag,
            format!("icmp eq i64 {}, -9223372036854775808", lhs.operand),
        );
        let negative = self.instruction(Kind::Flag, format!("icmp eq i64 {}, -1", rhs.operand));
        let overflow = self.instruction(
            Kind::Flag,
            format!("and i1 {}, {}", minimum.operand, negative.operand),
        );
        let message = if *op == BinOp::Div {
            "integer division overflow"
        } else {
            "integer remainder overflow"
        };
        self.guard(overflow, span, message);
    }

    fn instruction(&mut self, kind: Kind, instruction: String) -> Value {
        let name = format!("%v{}", self.next);
        self.next += 1;
        self.ir.push_str(&format!("  {name} = {instruction}\n"));
        Value::new(kind, name)
    }

    fn statement(&mut self, stmt: &Stmt) -> Result<bool, String> {
        match &stmt.node {
            StmtKind::Let { name, init, .. } => self.declaration(name, init),
            StmtKind::Assign {
                target: AssignTarget::Ident(name),
                op,
                value,
            } => self.assign_statement(name, op, value, stmt.span),
            StmtKind::Land(Some(expr)) => self.land(expr),
            _ => Err(unsupported(self.checked, stmt.span)),
        }
    }

    fn declaration(&mut self, name: &str, init: &Expr) -> Result<bool, String> {
        let value = self.expression(init)?;
        self.locals.insert(name.to_owned(), value);
        Ok(false)
    }

    fn assign_statement(
        &mut self,
        name: &str,
        op: &AssignOp,
        expr: &Expr,
        span: Span,
    ) -> Result<bool, String> {
        let rhs = self.expression(expr)?;
        let value = self.assignment(name, op, rhs, span)?;
        self.locals.insert(name.to_owned(), value);
        Ok(false)
    }

    fn land(&mut self, expr: &Expr) -> Result<bool, String> {
        let value = self.expression(expr)?;
        if value.kind != Kind::Int {
            return Err(unsupported(self.checked, expr.span));
        }
        let code = self.instruction(Kind::Int, format!("trunc i64 {} to i32", value.operand));
        self.ir.push_str(&format!("  ret i32 {}\n", code.operand));
        Ok(true)
    }

    fn assignment(
        &mut self,
        name: &str,
        op: &AssignOp,
        rhs: Value,
        span: Span,
    ) -> Result<Value, String> {
        let binary = match op {
            AssignOp::Assign => return Ok(rhs),
            AssignOp::Add => BinOp::Add,
            AssignOp::Sub => BinOp::Sub,
            AssignOp::Mul => BinOp::Mul,
            AssignOp::Div => BinOp::Div,
            AssignOp::Mod => BinOp::Mod,
        };
        let lhs = self.lookup(name, span)?;
        self.binary(&binary, lhs, rhs, span)
    }

    fn lookup(&self, name: &str, span: Span) -> Result<Value, String> {
        self.locals
            .get(name)
            .cloned()
            .ok_or_else(|| unsupported(self.checked, span))
    }

    fn expression(&mut self, expr: &Expr) -> Result<Value, String> {
        match &expr.node {
            ExprKind::IntLit(value) => Ok(Value::new(Kind::Int, value)),
            ExprKind::FloatLit(value) => Ok(Value::new(
                Kind::Float,
                format!("0x{:016X}", value.to_bits()),
            )),
            ExprKind::BoolLit(value) => Ok(Value::new(
                Kind::Flag,
                if *value { "true" } else { "false" },
            )),
            ExprKind::ByteLit(value) => Ok(Value::new(Kind::Byte, value)),
            ExprKind::Ident(name) => self.lookup(name, expr.span),
            ExprKind::Unary { op, expr: inner } => self.unary_expression(op, inner, expr.span),
            ExprKind::Binary {
                op: BinOp::And,
                lhs,
                rhs,
            } => self.logical(false, lhs, rhs),
            ExprKind::Binary {
                op: BinOp::Or,
                lhs,
                rhs,
            } => self.logical(true, lhs, rhs),
            ExprKind::Binary { op, lhs, rhs } => self.binary_expression(op, lhs, rhs, expr.span),
            _ => Err(unsupported(self.checked, expr.span)),
        }
    }

    fn unary_expression(&mut self, op: &UnOp, expr: &Expr, span: Span) -> Result<Value, String> {
        let value = self.expression(expr)?;
        self.unary(op, value, span)
    }

    fn binary_expression(
        &mut self,
        op: &BinOp,
        lhs: &Expr,
        rhs: &Expr,
        span: Span,
    ) -> Result<Value, String> {
        let lhs = self.expression(lhs)?;
        let rhs = self.expression(rhs)?;
        self.binary(op, lhs, rhs, span)
    }

    fn unary(&mut self, op: &UnOp, value: Value, span: Span) -> Result<Value, String> {
        let instruction = match (op, value.kind) {
            (UnOp::Not, Kind::Flag) => format!("xor i1 {}, true", value.operand),
            (UnOp::Neg, Kind::Int) => format!("sub i64 0, {}", value.operand),
            (UnOp::Neg, Kind::Float) => format!("fneg double {}", value.operand),
            _ => return Err(unsupported(self.checked, span)),
        };
        Ok(self.instruction(value.kind, instruction))
    }

    fn logical(&mut self, is_or: bool, lhs: &Expr, rhs: &Expr) -> Result<Value, String> {
        let left = self.expression(lhs)?;
        if left.kind != Kind::Flag {
            return Err(unsupported(self.checked, lhs.span));
        }
        let origin = self.block.clone();
        let right_block = format!("rhs{}", self.next);
        let merge = format!("merge{}", self.next);
        self.next += 1;
        let (yes, no) = if is_or {
            (&merge, &right_block)
        } else {
            (&right_block, &merge)
        };
        self.ir.push_str(&format!(
            "  br i1 {}, label %{yes}, label %{no}\n{right_block}:\n",
            left.operand
        ));
        self.block = right_block;
        let right = self.expression(rhs)?;
        if right.kind != Kind::Flag {
            return Err(unsupported(self.checked, rhs.span));
        }
        self.ir
            .push_str(&format!("  br label %{merge}\n{merge}:\n"));
        let right_end = self.block.clone();
        self.block = merge;
        Ok(self.instruction(
            Kind::Flag,
            format!(
                "phi i1 [ {is_or}, %{origin} ], [ {}, %{right_end} ]",
                right.operand
            ),
        ))
    }

    fn promote(&mut self, value: Value) -> Value {
        if value.kind != Kind::Int {
            return value;
        }
        self.instruction(
            Kind::Float,
            format!("sitofp i64 {} to double", value.operand),
        )
    }

    fn binary(
        &mut self,
        op: &BinOp,
        mut lhs: Value,
        mut rhs: Value,
        span: Span,
    ) -> Result<Value, String> {
        let numeric = matches!(lhs.kind, Kind::Int | Kind::Float)
            && matches!(rhs.kind, Kind::Int | Kind::Float);
        if *op == BinOp::Mod && lhs.kind != rhs.kind {
            return Err(self.invalid_operation(span));
        }
        if numeric && (lhs.kind == Kind::Float || rhs.kind == Kind::Float) {
            lhs = self.promote(lhs);
            rhs = self.promote(rhs);
        }
        let instruction = binary_instruction(op, lhs.kind, rhs.kind)
            .ok_or_else(|| self.invalid_operation(span))?;
        if matches!(op, BinOp::Div | BinOp::Mod) {
            self.division_checks(op, &lhs, &rhs, span);
        }
        let kind = if matches!(
            op,
            BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq
        ) {
            Kind::Flag
        } else {
            lhs.kind
        };
        Ok(self.instruction(
            kind,
            format!(
                "{instruction} {} {}, {}",
                lhs.kind.ir(),
                lhs.operand,
                rhs.operand
            ),
        ))
    }

    fn invalid_operation(&self, span: Span) -> String {
        format!(
            "{}:{}:{}: operation is not supported by the interpreter for these value types",
            self.checked.source_path(span).display(),
            span.line,
            span.col
        )
    }
}

fn binary_instruction(op: &BinOp, lhs: Kind, rhs: Kind) -> Option<&'static str> {
    if lhs != rhs {
        return None;
    }
    match lhs {
        Kind::Int => integer_instruction(op),
        Kind::Float => float_instruction(op),
        Kind::Flag | Kind::Byte => match op {
            BinOp::Eq => Some("icmp eq"),
            BinOp::NotEq => Some("icmp ne"),
            _ => None,
        },
    }
}

fn integer_instruction(op: &BinOp) -> Option<&'static str> {
    match op {
        BinOp::Add => Some("add"),
        BinOp::Sub => Some("sub"),
        BinOp::Mul => Some("mul"),
        BinOp::Div => Some("sdiv"),
        BinOp::Mod => Some("srem"),
        BinOp::Eq => Some("icmp eq"),
        BinOp::NotEq => Some("icmp ne"),
        BinOp::Lt => Some("icmp slt"),
        BinOp::Gt => Some("icmp sgt"),
        BinOp::LtEq => Some("icmp sle"),
        BinOp::GtEq => Some("icmp sge"),
        _ => None,
    }
}

fn float_instruction(op: &BinOp) -> Option<&'static str> {
    match op {
        BinOp::Add => Some("fadd"),
        BinOp::Sub => Some("fsub"),
        BinOp::Mul => Some("fmul"),
        BinOp::Div => Some("fdiv"),
        BinOp::Mod => Some("frem"),
        BinOp::Eq => Some("fcmp oeq"),
        BinOp::NotEq => Some("fcmp une"),
        BinOp::Lt => Some("fcmp olt"),
        BinOp::Gt => Some("fcmp ogt"),
        BinOp::LtEq => Some("fcmp ule"),
        BinOp::GtEq => Some("fcmp uge"),
        _ => None,
    }
}

fn stderr_symbol() -> &'static str {
    if cfg!(target_vendor = "apple") {
        "__stderrp"
    } else {
        "stderr"
    }
}

fn stderr_declaration() -> &'static str {
    if cfg!(target_vendor = "apple") {
        "@__stderrp = external global ptr\n"
    } else {
        "@stderr = external global ptr\n"
    }
}
