use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Byte(u8),
    Array(Vec<Value>),
    Void,
}

impl Value {
    fn as_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            v => panic!("expected bool, got {:?}", v),
        }
    }

    fn as_int(&self) -> i64 {
        match self {
            Value::Int(n) => *n,
            v => panic!("expected int, got {:?}", v),
        }
    }

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
            Value::Str(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::Byte(b) => format!("0x{:02X}", b),
            Value::Array(elems) => {
                format!(
                    "[{}]",
                    elems
                        .iter()
                        .map(|e| e.display())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Value::Void => String::new(),
        }
    }
}

enum Signal {
    Return(Value),
    Break,
    Continue,
}

struct Env {
    scopes: Vec<HashMap<String, Value>>,
}

impl Env {
    fn new() -> Self {
        Env {
            scopes: vec![HashMap::new()],
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn define(&mut self, name: String, val: Value) {
        self.scopes.last_mut().unwrap().insert(name, val);
    }

    fn get(&self, name: &str) -> &Value {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                return val;
            }
        }
        panic!("undefined variable: {}", name)
    }

    fn set(&mut self, name: &str, val: Value) {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), val);
                return;
            }
        }
        panic!("undefined variable: {}", name)
    }

    fn get_index(&self, name: &str, idx: usize) -> &Value {
        for scope in self.scopes.iter().rev() {
            if let Some(Value::Array(elems)) = scope.get(name) {
                return &elems[idx];
            }
        }
        panic!("undefined array: {}", name)
    }

    fn set_index(&mut self, name: &str, idx: usize, val: Value) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(Value::Array(elems)) = scope.get_mut(name) {
                elems[idx] = val;
                return;
            }
        }
        panic!("undefined array: {}", name)
    }
}

pub struct Interpreter {
    functions: HashMap<String, (Vec<Param>, Vec<Stmt>)>,
}

impl Interpreter {
    pub fn new(program: &Program) -> Self {
        let mut functions = HashMap::new();
        for item in &program.items {
            if let Item::Stage {
                name, params, body, ..
            } = item
            {
                functions.insert(name.clone(), (params.clone(), body.clone()));
            }
        }
        Interpreter { functions }
    }

    pub fn run(&self, program: &Program) {
        let mut env = Env::new();
        self.exec_stmts(&program.ignite_body, &mut env);

        for item in &program.items {
            if let Item::Launch { name, body } = item {
                if name == "main" {
                    let mut env = Env::new();
                    if let Some(Signal::Return(Value::Int(code))) = self.exec_stmts(body, &mut env)
                    {
                        std::process::exit(code as i32);
                    }
                    return;
                }
            }
        }
        panic!("no 'launch main' found");
    }

    fn exec_block(&self, stmts: &[Stmt], env: &mut Env) -> Option<Signal> {
        env.push_scope();
        let sig = self.exec_stmts(stmts, env);
        env.pop_scope();
        sig
    }

    fn exec_stmts(&self, stmts: &[Stmt], env: &mut Env) -> Option<Signal> {
        for stmt in stmts {
            if let Some(sig) = self.exec_stmt(stmt, env) {
                return Some(sig);
            }
        }
        None
    }

    fn exec_stmt(&self, stmt: &Stmt, env: &mut Env) -> Option<Signal> {
        match &stmt.node {
            StmtKind::Let { name, init, .. } => {
                let val = self.eval_expr(init, env);
                env.define(name.clone(), val);
                None
            }

            StmtKind::Assign { target, op, value } => {
                match target {
                    AssignTarget::Ident(name) => {
                        let rhs = self.eval_expr(value, env);
                        let new_val = if matches!(op, AssignOp::Assign) {
                            rhs
                        } else {
                            let cur = env.get(name).clone();
                            apply_op(op, cur, rhs)
                        };
                        env.set(name, new_val);
                    }
                    AssignTarget::Index { name, index } => {
                        let idx = self.eval_expr(index, env).as_int() as usize;
                        let rhs = self.eval_expr(value, env);
                        let new_val = if matches!(op, AssignOp::Assign) {
                            rhs
                        } else {
                            let cur = env.get_index(name, idx).clone();
                            apply_op(op, cur, rhs)
                        };
                        env.set_index(name, idx, new_val);
                    }
                }
                None
            }

            StmtKind::If {
                cond,
                then_block,
                else_block,
            } => {
                if self.eval_expr(cond, env).as_bool() {
                    self.exec_block(then_block, env)
                } else if let Some(else_stmts) = else_block {
                    self.exec_block(else_stmts, env)
                } else {
                    None
                }
            }

            StmtKind::Route { subject, arms } => {
                let val = self.eval_expr(subject, env);
                for arm in arms {
                    let pat = self.eval_expr(&arm.pattern, env);
                    if values_equal(&val, &pat) {
                        return self.exec_stmt(&arm.body, env);
                    }
                }
                None
            }

            StmtKind::Burn { cond, body } => {
                loop {
                    if !self.eval_expr(cond, env).as_bool() {
                        break;
                    }
                    match self.exec_block(body, env) {
                        Some(Signal::Break) => break,
                        Some(Signal::Continue) => continue,
                        sig @ Some(Signal::Return(_)) => return sig,
                        None => {}
                    }
                }
                None
            }

            StmtKind::Spin { var, range, body } => {
                let (start, end, inclusive) = match &range.node {
                    ExprKind::Range {
                        start,
                        end,
                        inclusive,
                    } => (
                        self.eval_expr(start, env).as_int(),
                        self.eval_expr(end, env).as_int(),
                        *inclusive,
                    ),
                    _ => panic!("spin requires a range expression"),
                };
                let end = if inclusive { end + 1 } else { end };
                'spin: for i in start..end {
                    env.push_scope();
                    env.define(var.clone(), Value::Int(i));
                    let sig = self.exec_stmts(body, env);
                    env.pop_scope();
                    match sig {
                        Some(Signal::Break) => break 'spin,
                        Some(Signal::Continue) => continue 'spin,
                        sig @ Some(Signal::Return(_)) => return sig,
                        None => {}
                    }
                }
                None
            }

            StmtKind::Orbit { body } => {
                loop {
                    match self.exec_block(body, env) {
                        Some(Signal::Break) => break,
                        Some(Signal::Continue) => continue,
                        sig @ Some(Signal::Return(_)) => return sig,
                        None => {}
                    }
                }
                None
            }

            StmtKind::Fire { name, args } => {
                let args: Vec<Value> = args.iter().map(|a| self.eval_expr(a, env)).collect();
                self.call_function(name, args);
                None
            }

            StmtKind::Land(expr) => {
                let val = expr
                    .as_ref()
                    .map(|e| self.eval_expr(e, env))
                    .unwrap_or(Value::Void);
                Some(Signal::Return(val))
            }

            StmtKind::Eject => Some(Signal::Break),
            StmtKind::Pass => Some(Signal::Continue),
            StmtKind::Abort => std::process::exit(1),
            StmtKind::Error => unreachable!("parse error node reached interpreter"),
        }
    }

    fn eval_expr(&self, expr: &Expr, env: &mut Env) -> Value {
        match &expr.node {
            ExprKind::IntLit(n) => Value::Int(*n),
            ExprKind::FloatLit(f) => Value::Float(*f),
            ExprKind::StrLit(s) => Value::Str(s.clone()),
            ExprKind::BoolLit(b) => Value::Bool(*b),
            ExprKind::ByteLit(b) => Value::Byte(*b),
            ExprKind::Air => Value::Void,

            ExprKind::Ident(name) => env.get(name).clone(),

            ExprKind::Binary { op, lhs, rhs } => {
                match op {
                    BinOp::And => {
                        if !self.eval_expr(lhs, env).as_bool() {
                            return Value::Bool(false);
                        }
                        return Value::Bool(self.eval_expr(rhs, env).as_bool());
                    }
                    BinOp::Or => {
                        if self.eval_expr(lhs, env).as_bool() {
                            return Value::Bool(true);
                        }
                        return Value::Bool(self.eval_expr(rhs, env).as_bool());
                    }
                    _ => {}
                }
                let l = self.eval_expr(lhs, env);
                let r = self.eval_expr(rhs, env);
                eval_binary(op, l, r)
            }

            ExprKind::Unary { op, expr } => {
                let val = self.eval_expr(expr, env);
                match op {
                    UnOp::Not => Value::Bool(!val.as_bool()),
                    UnOp::Neg => match val {
                        Value::Int(n) => Value::Int(-n),
                        Value::Float(f) => Value::Float(-f),
                        v => panic!("cannot negate {:?}", v),
                    },
                }
            }

            ExprKind::Call { name, args } => {
                let args: Vec<Value> = args.iter().map(|a| self.eval_expr(a, env)).collect();
                self.call_function(name, args)
            }

            ExprKind::Index { array, index } => {
                let arr = self.eval_expr(array, env);
                let idx = self.eval_expr(index, env).as_int() as usize;
                match arr {
                    Value::Array(elems) => elems[idx].clone(),
                    v => panic!("cannot index into {:?}", v),
                }
            }

            ExprKind::ArrayLit(elems) => {
                Value::Array(elems.iter().map(|e| self.eval_expr(e, env)).collect())
            }

            ExprKind::Range { .. } => panic!("range expression outside spin"),

            ExprKind::Error => unreachable!("parse error node reached interpreter"),
        }
    }

    fn call_function(&self, name: &str, args: Vec<Value>) -> Value {
        if name == "log" {
            println!(
                "{}",
                args.iter()
                    .map(|v| v.display())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            return Value::Void;
        }

        let (params, body) = self
            .functions
            .get(name)
            .unwrap_or_else(|| panic!("undefined function: {}", name));

        let mut env = Env::new();
        for (param, val) in params.iter().zip(args.iter()) {
            env.define(param.name.clone(), val.clone());
        }

        match self.exec_stmts(body, &mut env) {
            Some(Signal::Return(val)) => val,
            _ => Value::Void,
        }
    }
}

fn eval_binary(op: &BinOp, lhs: Value, rhs: Value) -> Value {
    match op {
        BinOp::Add => match (lhs, rhs) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
            (Value::Int(a), Value::Float(b)) => Value::Float(a as f64 + b),
            (Value::Float(a), Value::Int(b)) => Value::Float(a + b as f64),
            (Value::Str(a), Value::Str(b)) => Value::Str(a + &b),
            (l, r) => panic!("type error: {:?} + {:?}", l, r),
        },
        BinOp::Sub => match (lhs, rhs) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
            (Value::Int(a), Value::Float(b)) => Value::Float(a as f64 - b),
            (Value::Float(a), Value::Int(b)) => Value::Float(a - b as f64),
            (l, r) => panic!("type error: {:?} - {:?}", l, r),
        },
        BinOp::Mul => match (lhs, rhs) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a * b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
            (Value::Int(a), Value::Float(b)) => Value::Float(a as f64 * b),
            (Value::Float(a), Value::Int(b)) => Value::Float(a * b as f64),
            (l, r) => panic!("type error: {:?} * {:?}", l, r),
        },
        BinOp::Div => match (lhs, rhs) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a / b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a / b),
            (Value::Int(a), Value::Float(b)) => Value::Float(a as f64 / b),
            (Value::Float(a), Value::Int(b)) => Value::Float(a / b as f64),
            (l, r) => panic!("type error: {:?} / {:?}", l, r),
        },
        BinOp::Mod => match (lhs, rhs) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a % b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a % b),
            (l, r) => panic!("type error: {:?} % {:?}", l, r),
        },
        BinOp::Eq => Value::Bool(values_equal(&lhs, &rhs)),
        BinOp::NotEq => Value::Bool(!values_equal(&lhs, &rhs)),
        BinOp::Lt => Value::Bool(cmp_values(&lhs, &rhs).is_lt()),
        BinOp::Gt => Value::Bool(cmp_values(&lhs, &rhs).is_gt()),
        BinOp::LtEq => Value::Bool(cmp_values(&lhs, &rhs).is_le()),
        BinOp::GtEq => Value::Bool(cmp_values(&lhs, &rhs).is_ge()),
        BinOp::And | BinOp::Or => unreachable!(),
    }
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
        (Value::Void, Value::Void) => true,
        _ => false,
    }
}

fn cmp_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x.cmp(y),
        (Value::Float(x), Value::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Int(x), Value::Float(y)) => (*x as f64)
            .partial_cmp(y)
            .unwrap_or(std::cmp::Ordering::Equal),
        (Value::Float(x), Value::Int(y)) => x
            .partial_cmp(&(*y as f64))
            .unwrap_or(std::cmp::Ordering::Equal),
        (Value::Str(x), Value::Str(y)) => x.cmp(y),
        (l, r) => panic!("cannot compare {:?} and {:?}", l, r),
    }
}

fn apply_op(op: &AssignOp, lhs: Value, rhs: Value) -> Value {
    match op {
        AssignOp::Assign => rhs,
        AssignOp::Add => eval_binary(&BinOp::Add, lhs, rhs),
        AssignOp::Sub => eval_binary(&BinOp::Sub, lhs, rhs),
        AssignOp::Mul => eval_binary(&BinOp::Mul, lhs, rhs),
        AssignOp::Div => eval_binary(&BinOp::Div, lhs, rhs),
        AssignOp::Mod => eval_binary(&BinOp::Mod, lhs, rhs),
    }
}
