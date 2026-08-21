use crate::ast::*;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(Rc<String>),
    Bool(bool),
    Byte(u8),
    Array(Rc<Vec<Value>>),
    Enum {
        enum_name: Rc<String>,
        variant: Rc<String>,
    },
    Void,
}

impl Value {
    fn display(&self) -> String {
        match self {
            Value::Int(n) => n.to_string(),
            Value::Float(f) => {
                if f.fract() == 0.0 {
                    format!("{:.1}", f)
                } else {
                    f.to_string()
                }
            }
            Value::Str(s) => s.as_ref().clone(),
            Value::Bool(b) => b.to_string(),
            Value::Byte(b) => format!("0x{:02X}", b),
            Value::Enum { enum_name, variant } => format!("{}.{}", enum_name, variant),
            Value::Array(elems) => format!(
                "[{}]",
                elems
                    .iter()
                    .map(|e| e.display())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Void => String::new(),
        }
    }

    fn kind_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Str(_) => "str",
            Value::Bool(_) => "flag",
            Value::Byte(_) => "byte",
            Value::Array(_) => "array",
            Value::Enum { .. } => "enum",
            Value::Void => "void",
        }
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    pub span: Span,
    pub msg: String,
}

impl RuntimeError {
    fn new(span: Span, msg: impl Into<String>) -> Self {
        RuntimeError {
            span,
            msg: msg.into(),
        }
    }
}

enum Signal {
    Return(Value),
    Break,
    Continue,
    Abort,
}

struct Env {
    scopes: Vec<Vec<usize>>,
    symbols: HashMap<String, usize>,
    values: Vec<Vec<Value>>,
}

impl Env {
    fn new() -> Self {
        Env {
            scopes: vec![Vec::new()],
            symbols: HashMap::new(),
            values: Vec::new(),
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(Vec::new());
    }

    fn pop_scope(&mut self) {
        let names = self.scopes.pop().expect("scope stack is never empty");
        for symbol in names {
            if let Some(stack) = self.values.get_mut(symbol) {
                stack.pop();
            }
        }
    }

    fn symbol(&self, name: &str) -> Option<usize> {
        self.symbols.get(name).copied()
    }

    fn intern(&mut self, name: &str) -> usize {
        if let Some(symbol) = self.symbol(name) {
            return symbol;
        }

        let symbol = self.values.len();
        self.symbols.insert(name.to_owned(), symbol);
        self.values.push(Vec::new());
        symbol
    }

    fn define(&mut self, name: &str, val: Value) {
        let symbol = self.intern(name);
        self.scopes
            .last_mut()
            .expect("scope stack is never empty")
            .push(symbol);
        self.values[symbol].push(val);
    }

    fn lookup(&self, name: &str) -> Option<&Value> {
        self.symbol(name)
            .and_then(|symbol| self.values.get(symbol))
            .and_then(|stack| stack.last())
    }

    fn get(&self, name: &str, span: Span) -> Result<Value, RuntimeError> {
        self.lookup(name)
            .cloned()
            .ok_or_else(|| RuntimeError::new(span, format!("undefined variable '{}'", name)))
    }

    fn set(&mut self, name: &str, val: Value, span: Span) -> Result<(), RuntimeError> {
        if let Some(symbol) = self.symbol(name) {
            if let Some(stack) = self.values.get_mut(symbol) {
                if let Some(slot) = stack.last_mut() {
                    *slot = val;
                    return Ok(());
                }
            }
        }
        Err(RuntimeError::new(
            span,
            format!("undefined variable '{}'", name),
        ))
    }

    fn get_mut(&mut self, name: &str, span: Span) -> Result<&mut Value, RuntimeError> {
        if let Some(symbol) = self.symbol(name) {
            if let Some(stack) = self.values.get_mut(symbol) {
            if let Some(slot) = stack.last_mut() {
                    return Ok(slot);
                }
            }
        }
        Err(RuntimeError::new(
            span,
            format!("undefined variable '{}'", name),
        ))
    }
}

pub struct Interpreter<'a> {
    program: &'a Program,
    functions: HashMap<&'a str, usize>,
    enum_variants: HashMap<&'a str, &'a str>,
}

impl<'a> Interpreter<'a> {
    pub fn new(program: &'a Program) -> Self {
        let mut functions = HashMap::new();
        let mut enum_variants = HashMap::new();
        for (index, item) in program.items.iter().enumerate() {
            match item {
                Item::Stage {
                    name, params, body, ..
                } => {
                    let _ = (params, body);
                    functions.insert(name.as_str(), index);
                }
                Item::Enum { name, variants } => {
                    for variant in variants {
                        enum_variants.insert(variant.name.as_str(), name.as_str());
                    }
                }
                Item::Launch { .. } => {}
            }
        }
        Interpreter {
            program,
            functions,
            enum_variants,
        }
    }

    pub fn run(&self, target: &str) -> Result<i32, RuntimeError> {
        let mut env = Env::new();
        match self.exec_stmts(&self.program.ignite_body, &mut env)? {
            Some(Signal::Return(Value::Int(code))) => return Ok(code as i32),
            Some(Signal::Abort) => return Ok(1),
            Some(Signal::Return(_)) | Some(Signal::Break) | Some(Signal::Continue) | None => {}
        }

        for item in &self.program.items {
            if let Item::Launch { name, body } = item {
                if name == target {
                    return match self.exec_stmts(body, &mut env)? {
                        Some(Signal::Return(Value::Int(code))) => Ok(code as i32),
                        Some(Signal::Abort) => Ok(1),
                        Some(Signal::Return(_)) | Some(Signal::Break) | Some(Signal::Continue) | None => {
                            Ok(0)
                        }
                    };
                }
            }
        }

        Err(RuntimeError::new(
            Span::new(1, 1),
            format!("no 'launch {}' found", target),
        ))
    }

    fn exec_block(&self, stmts: &[Stmt], env: &mut Env) -> Result<Option<Signal>, RuntimeError> {
        env.push_scope();
        let sig = self.exec_stmts(stmts, env);
        env.pop_scope();
        sig
    }

    fn exec_stmts(&self, stmts: &[Stmt], env: &mut Env) -> Result<Option<Signal>, RuntimeError> {
        for stmt in stmts {
            if let Some(sig) = self.exec_stmt(stmt, env)? {
                return Ok(Some(sig));
            }
        }
        Ok(None)
    }

    fn exec_stmt(&self, stmt: &Stmt, env: &mut Env) -> Result<Option<Signal>, RuntimeError> {
        match &stmt.node {
            StmtKind::Let { name, init, .. } => {
                let val = self.eval_expr(init, env)?;
                env.define(name, val);
                Ok(None)
            }
            StmtKind::Assign { target, op, value } => {
                match target {
                    AssignTarget::Ident(name) => {
                        let rhs = self.eval_expr(value, env)?;
                        let new_val = if matches!(op, AssignOp::Assign) {
                            rhs
                        } else {
                            let cur = env.get(name, stmt.span)?;
                            apply_op(op, cur, rhs, stmt.span)?
                        };
                        env.set(name, new_val, stmt.span)?;
                    }
                    AssignTarget::Index { name, index } => {
                        let idx = self.expect_index_value(&self.eval_expr(index, env)?, index.span)?;
                        let rhs = self.eval_expr(value, env)?;
                        let current = env.get_mut(name, stmt.span)?;
                        match current {
                            Value::Array(elems) => {
                                let elems = Rc::make_mut(elems);
                                if idx >= elems.len() {
                                    return Err(RuntimeError::new(
                                        index.span,
                                        format!(
                                            "array index {} out of bounds for '{}' with length {}",
                                            idx,
                                            name,
                                            elems.len()
                                        ),
                                    ));
                                }
                                let new_val = if matches!(op, AssignOp::Assign) {
                                    rhs
                                } else {
                                    apply_op(op, elems[idx].clone(), rhs, stmt.span)?
                                };
                                elems[idx] = new_val;
                            }
                            other => {
                                return Err(RuntimeError::new(
                                    stmt.span,
                                    format!("cannot index into {}", other.kind_name()),
                                ));
                            }
                        }
                    }
                }
                Ok(None)
            }
            StmtKind::If {
                cond,
                then_block,
                else_block,
            } => {
                if self.expect_bool(&self.eval_expr(cond, env)?, cond.span)? {
                    self.exec_block(then_block, env)
                } else if let Some(else_stmts) = else_block {
                    self.exec_block(else_stmts, env)
                } else {
                    Ok(None)
                }
            }
            StmtKind::Route { subject, arms } => {
                let val = self.eval_expr(subject, env)?;
                for arm in arms {
                    let pat = self.eval_expr(&arm.pattern, env)?;
                    if values_equal(&val, &pat) {
                        return self.exec_stmt(&arm.body, env);
                    }
                }
                Ok(None)
            }
            StmtKind::Burn { cond, body } => {
                loop {
                    if !self.expect_bool(&self.eval_expr(cond, env)?, cond.span)? {
                        break;
                    }
                    match self.exec_block(body, env)? {
                        Some(Signal::Break) => break,
                        Some(Signal::Continue) => continue,
                        sig @ Some(Signal::Return(_)) | sig @ Some(Signal::Abort) => return Ok(sig),
                        None => {}
                    }
                }
                Ok(None)
            }
            StmtKind::Spin { var, range, body } => {
                let ExprKind::Range {
                    start,
                    end,
                    inclusive,
                } = &range.node
                else {
                    return Err(RuntimeError::new(
                        range.span,
                        "spin requires a range expression",
                    ));
                };
                let start = self.expect_int(&self.eval_expr(start, env)?, start.span)?;
                let end = self.expect_int(&self.eval_expr(end, env)?, end.span)?;
                let end = if *inclusive {
                    end.checked_add(1).ok_or_else(|| {
                        RuntimeError::new(range.span, "inclusive range end overflowed")
                    })?
                } else {
                    end
                };
                'spin: for i in start..end {
                    env.push_scope();
                    env.define(var, Value::Int(i));
                    let sig = self.exec_stmts(body, env)?;
                    env.pop_scope();
                    match sig {
                        Some(Signal::Break) => break 'spin,
                        Some(Signal::Continue) => continue 'spin,
                        sig @ Some(Signal::Return(_)) | sig @ Some(Signal::Abort) => return Ok(sig),
                        None => {}
                    }
                }
                Ok(None)
            }
            StmtKind::Orbit { body } => {
                loop {
                    match self.exec_block(body, env)? {
                        Some(Signal::Break) => break,
                        Some(Signal::Continue) => continue,
                        sig @ Some(Signal::Return(_)) | sig @ Some(Signal::Abort) => return Ok(sig),
                        None => {}
                    }
                }
                Ok(None)
            }
            StmtKind::Fire { name, args } => {
                let values = args
                    .iter()
                    .map(|arg| self.eval_expr(arg, env))
                    .collect::<Result<Vec<_>, _>>()?;
                self.call_function(name, values, stmt.span)?;
                Ok(None)
            }
            StmtKind::Land(expr) => {
                let val = match expr {
                    Some(expr) => self.eval_expr(expr, env)?,
                    None => Value::Void,
                };
                Ok(Some(Signal::Return(val)))
            }
            StmtKind::Eject => Ok(Some(Signal::Break)),
            StmtKind::Pass => Ok(Some(Signal::Continue)),
            StmtKind::Abort => Ok(Some(Signal::Abort)),
            StmtKind::Error => unreachable!("parse error node reached interpreter"),
        }
    }

    fn eval_expr(&self, expr: &Expr, env: &mut Env) -> Result<Value, RuntimeError> {
        match &expr.node {
            ExprKind::IntLit(n) => Ok(Value::Int(*n)),
            ExprKind::FloatLit(f) => Ok(Value::Float(*f)),
            ExprKind::StrLit(s) => Ok(Value::Str(Rc::new(s.clone()))),
            ExprKind::BoolLit(b) => Ok(Value::Bool(*b)),
            ExprKind::ByteLit(b) => Ok(Value::Byte(*b)),
            ExprKind::Air => Ok(Value::Void),
            ExprKind::Ident(name) => {
                if let Some(value) = env.lookup(name) {
                    return Ok(value.clone());
                }
                if let Some(enum_name) = self.enum_variants.get(name.as_str()) {
                    return Ok(Value::Enum {
                        enum_name: Rc::new((*enum_name).to_owned()),
                        variant: Rc::new(name.clone()),
                    });
                }
                Err(RuntimeError::new(
                    expr.span,
                    format!("undefined variable '{}'", name),
                ))
            }
            ExprKind::Binary { op, lhs, rhs } => match op {
                BinOp::And => {
                    if !self.expect_bool(&self.eval_expr(lhs, env)?, lhs.span)? {
                        return Ok(Value::Bool(false));
                    }
                    Ok(Value::Bool(
                        self.expect_bool(&self.eval_expr(rhs, env)?, rhs.span)?,
                    ))
                }
                BinOp::Or => {
                    if self.expect_bool(&self.eval_expr(lhs, env)?, lhs.span)? {
                        return Ok(Value::Bool(true));
                    }
                    Ok(Value::Bool(
                        self.expect_bool(&self.eval_expr(rhs, env)?, rhs.span)?,
                    ))
                }
                _ => {
                    let l = self.eval_expr(lhs, env)?;
                    let r = self.eval_expr(rhs, env)?;
                    eval_binary(op, l, r, expr.span)
                }
            },
            ExprKind::Unary { op, expr: inner } => {
                let val = self.eval_expr(inner, env)?;
                match op {
                    UnOp::Not => Ok(Value::Bool(!self.expect_bool(&val, inner.span)?)),
                    UnOp::Neg => match val {
                        Value::Int(n) => Ok(Value::Int(-n)),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        other => Err(RuntimeError::new(
                            expr.span,
                            format!("cannot negate {}", other.kind_name()),
                        )),
                    },
                }
            }
            ExprKind::Call { name, args } => {
                let values = args
                    .iter()
                    .map(|arg| self.eval_expr(arg, env))
                    .collect::<Result<Vec<_>, _>>()?;
                self.call_function(name, values, expr.span)
            }
            ExprKind::Index { array, index } => {
                let arr = self.eval_expr(array, env)?;
                let idx = self.expect_index_value(&self.eval_expr(index, env)?, index.span)?;
                match arr {
                    Value::Array(elems) => elems.get(idx).cloned().ok_or_else(|| {
                        RuntimeError::new(
                            index.span,
                            format!("array index {} out of bounds with length {}", idx, elems.len()),
                        )
                    }),
                    other => Err(RuntimeError::new(
                        array.span,
                        format!("cannot index into {}", other.kind_name()),
                    )),
                }
            }
            ExprKind::ArrayLit(elems) => Ok(Value::Array(Rc::new(
                elems
                    .iter()
                    .map(|elem| self.eval_expr(elem, env))
                    .collect::<Result<Vec<_>, _>>()?,
            ))),
            ExprKind::Range { .. } => Err(RuntimeError::new(
                expr.span,
                "range expression is only allowed in spin",
            )),
            ExprKind::Error => unreachable!("parse error node reached interpreter"),
        }
    }

    fn call_function(
        &self,
        name: &str,
        args: Vec<Value>,
        span: Span,
    ) -> Result<Value, RuntimeError> {
        if name == "log" {
            println!(
                "{}",
                args.iter()
                    .map(|v| v.display())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            return Ok(Value::Void);
        }

        if name == "len" {
            if args.len() != 1 {
                return Err(RuntimeError::new(
                    span,
                    format!("'len' expects 1 argument, got {}", args.len()),
                ));
            }
            return match &args[0] {
                Value::Array(elems) => Ok(Value::Int(elems.len() as i64)),
                Value::Str(s) => Ok(Value::Int(s.chars().count() as i64)),
                other => Err(RuntimeError::new(
                    span,
                    format!("'len' expects array or str, got {}", other.kind_name()),
                )),
            };
        }

        if name == "assert" {
            if args.len() != 1 {
                return Err(RuntimeError::new(
                    span,
                    format!("'assert' expects 1 argument, got {}", args.len()),
                ));
            }
            if !self.expect_bool(&args[0], span)? {
                return Err(RuntimeError::new(span, "assertion failed"));
            }
            return Ok(Value::Void);
        }

        if name == "to_str" {
            if args.len() != 1 {
                return Err(RuntimeError::new(
                    span,
                    format!("'to_str' expects 1 argument, got {}", args.len()),
                ));
            }
            return Ok(Value::Str(Rc::new(args[0].display())));
        }

        let function_index = self.functions.get(name).copied().ok_or_else(|| {
            RuntimeError::new(span, format!("undefined function '{}'", name))
        })?;
        let Item::Stage { params, body, .. } = &self.program.items[function_index] else {
            unreachable!("function map should only contain stage items");
        };
        if args.len() != params.len() {
            return Err(RuntimeError::new(
                span,
                format!(
                    "function '{}' expects {} arguments, got {}",
                    name,
                    params.len(),
                    args.len()
                ),
            ));
        }

        let mut env = Env::new();
        for (param, val) in params.iter().zip(args) {
            env.define(&param.name, val);
        }

        match self.exec_stmts(body, &mut env)? {
            Some(Signal::Return(val)) => Ok(val),
            Some(Signal::Abort) => Ok(Value::Void),
            Some(Signal::Break) | Some(Signal::Continue) | None => Ok(Value::Void),
        }
    }

    fn expect_bool(&self, value: &Value, span: Span) -> Result<bool, RuntimeError> {
        match value {
            Value::Bool(b) => Ok(*b),
            other => Err(RuntimeError::new(
                span,
                format!("expected flag, got {}", other.kind_name()),
            )),
        }
    }

    fn expect_int(&self, value: &Value, span: Span) -> Result<i64, RuntimeError> {
        match value {
            Value::Int(n) => Ok(*n),
            other => Err(RuntimeError::new(
                span,
                format!("expected int, got {}", other.kind_name()),
            )),
        }
    }

    fn expect_index_value(&self, value: &Value, span: Span) -> Result<usize, RuntimeError> {
        let index = self.expect_int(value, span)?;
        if index < 0 {
            return Err(RuntimeError::new(
                span,
                format!("array index must be non-negative, got {}", index),
            ));
        }
        Ok(index as usize)
    }
}

fn eval_binary(op: &BinOp, lhs: Value, rhs: Value, span: Span) -> Result<Value, RuntimeError> {
    match op {
        BinOp::Add => match (lhs, rhs) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + b as f64)),
            (Value::Str(a), Value::Str(b)) => Ok(Value::Str(Rc::new(format!("{}{}", a, b)))),
            (l, r) => Err(binary_type_error(span, "+", &l, &r)),
        },
        BinOp::Sub => match (lhs, rhs) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 - b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - b as f64)),
            (l, r) => Err(binary_type_error(span, "-", &l, &r)),
        },
        BinOp::Mul => match (lhs, rhs) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 * b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * b as f64)),
            (l, r) => Err(binary_type_error(span, "*", &l, &r)),
        },
        BinOp::Div => match (lhs, rhs) {
            (Value::Int(_), Value::Int(0)) => Err(RuntimeError::new(span, "division by zero")),
            (Value::Float(_), Value::Float(0.0)) => {
                Err(RuntimeError::new(span, "division by zero"))
            }
            (Value::Int(_), Value::Float(0.0)) => Err(RuntimeError::new(span, "division by zero")),
            (Value::Float(_), Value::Int(0)) => Err(RuntimeError::new(span, "division by zero")),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 / b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a / b as f64)),
            (l, r) => Err(binary_type_error(span, "/", &l, &r)),
        },
        BinOp::Mod => match (lhs, rhs) {
            (Value::Int(_), Value::Int(0)) => Err(RuntimeError::new(span, "modulo by zero")),
            (Value::Float(_), Value::Float(0.0)) => Err(RuntimeError::new(span, "modulo by zero")),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a % b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a % b)),
            (l, r) => Err(binary_type_error(span, "%", &l, &r)),
        },
        BinOp::Eq => Ok(Value::Bool(values_equal(&lhs, &rhs))),
        BinOp::NotEq => Ok(Value::Bool(!values_equal(&lhs, &rhs))),
        BinOp::Lt => Ok(Value::Bool(cmp_values(&lhs, &rhs, span)?.is_lt())),
        BinOp::Gt => Ok(Value::Bool(cmp_values(&lhs, &rhs, span)?.is_gt())),
        BinOp::LtEq => Ok(Value::Bool(cmp_values(&lhs, &rhs, span)?.is_le())),
        BinOp::GtEq => Ok(Value::Bool(cmp_values(&lhs, &rhs, span)?.is_ge())),
        BinOp::And | BinOp::Or => unreachable!(),
    }
}

fn binary_type_error(span: Span, op: &str, lhs: &Value, rhs: &Value) -> RuntimeError {
    RuntimeError::new(
        span,
        format!(
            "operator '{}' is not defined for {} and {}",
            op,
            lhs.kind_name(),
            rhs.kind_name()
        ),
    )
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x == y,
        (Value::Float(x), Value::Float(y)) => x == y,
        (Value::Int(x), Value::Float(y)) => *x as f64 == *y,
        (Value::Float(x), Value::Int(y)) => *x == *y as f64,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Byte(x), Value::Byte(y)) => x == y,
        (
            Value::Enum {
                enum_name: enum_x,
                variant: variant_x,
            },
            Value::Enum {
                enum_name: enum_y,
                variant: variant_y,
            },
        ) => enum_x == enum_y && variant_x == variant_y,
        (Value::Void, Value::Void) => true,
        _ => false,
    }
}

fn cmp_values(a: &Value, b: &Value, span: Span) -> Result<Ordering, RuntimeError> {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Ok(x.cmp(y)),
        (Value::Float(x), Value::Float(y)) => Ok(x.partial_cmp(y).unwrap_or(Ordering::Equal)),
        (Value::Int(x), Value::Float(y)) => {
            Ok((*x as f64).partial_cmp(y).unwrap_or(Ordering::Equal))
        }
        (Value::Float(x), Value::Int(y)) => {
            Ok(x.partial_cmp(&(*y as f64)).unwrap_or(Ordering::Equal))
        }
        (Value::Str(x), Value::Str(y)) => Ok(x.cmp(y)),
        (lhs, rhs) => Err(RuntimeError::new(
            span,
            format!(
                "cannot compare {} and {}",
                lhs.kind_name(),
                rhs.kind_name()
            ),
        )),
    }
}

fn apply_op(op: &AssignOp, lhs: Value, rhs: Value, span: Span) -> Result<Value, RuntimeError> {
    match op {
        AssignOp::Assign => Ok(rhs),
        AssignOp::Add => eval_binary(&BinOp::Add, lhs, rhs, span),
        AssignOp::Sub => eval_binary(&BinOp::Sub, lhs, rhs, span),
        AssignOp::Mul => eval_binary(&BinOp::Mul, lhs, rhs, span),
        AssignOp::Div => eval_binary(&BinOp::Div, lhs, rhs, span),
        AssignOp::Mod => eval_binary(&BinOp::Mod, lhs, rhs, span),
    }
}
