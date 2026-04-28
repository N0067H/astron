use crate::ast::*;
use std::collections::HashMap;

pub struct TypeError {
    pub span: Span,
    pub msg: String,
}

struct TypeEnv {
    scopes: Vec<HashMap<String, (Type, bool)>>, // (type, immutable)
}

impl TypeEnv {
    fn new() -> Self {
        TypeEnv {
            scopes: vec![HashMap::new()],
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn define(&mut self, name: String, ty: Type, immutable: bool) {
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name, (ty, immutable));
    }

    fn get(&self, name: &str) -> Option<(&Type, bool)> {
        for scope in self.scopes.iter().rev() {
            if let Some((ty, imm)) = scope.get(name) {
                return Some((ty, *imm));
            }
        }
        None
    }
}

struct Checker {
    funcs: HashMap<String, (Vec<Type>, Type)>,
    errors: Vec<TypeError>,
}

impl Checker {
    fn new() -> Self {
        Checker {
            funcs: HashMap::new(),
            errors: Vec::new(),
        }
    }

    fn error(&mut self, span: Span, msg: impl Into<String>) {
        self.errors.push(TypeError {
            span,
            msg: msg.into(),
        });
    }

    fn check_item(&mut self, item: &Item) {
        match item {
            Item::Stage {
                params,
                ret_type,
                body,
                ..
            } => {
                let mut env = TypeEnv::new();
                for p in params {
                    env.define(p.name.clone(), p.ty.clone(), false);
                }
                self.check_stmts(body, &mut env, ret_type);
            }
            Item::Launch { body, .. } => {
                let mut env = TypeEnv::new();
                self.check_stmts(body, &mut env, &Type::Int);
            }
        }
    }

    fn check_block(&mut self, stmts: &[Stmt], env: &mut TypeEnv, ret: &Type) {
        env.push_scope();
        self.check_stmts(stmts, env, ret);
        env.pop_scope();
    }

    fn check_stmts(&mut self, stmts: &[Stmt], env: &mut TypeEnv, ret: &Type) {
        for stmt in stmts {
            self.check_stmt(stmt, env, ret);
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt, env: &mut TypeEnv, ret: &Type) {
        let span = stmt.span;
        match &stmt.node {
            StmtKind::Let {
                mutable,
                name,
                ty,
                init,
            } => {
                if let Some(init_ty) = self.check_expr(init, env) {
                    if !assignable(ty, &init_ty) {
                        self.error(
                            init.span,
                            format!("type mismatch: declared {:?}, got {:?}", ty, init_ty),
                        );
                    }
                }
                env.define(name.clone(), ty.clone(), !mutable);
            }

            StmtKind::Assign {
                target,
                op: _,
                value,
            } => {
                let val_ty = self.check_expr(value, env);
                match target {
                    AssignTarget::Ident(name) => match env.get(name) {
                        None => self.error(span, format!("undefined variable '{}'", name)),
                        Some((var_ty, imm)) => {
                            let (var_ty, imm) = (var_ty.clone(), imm);
                            if imm {
                                self.error(
                                    span,
                                    format!("cannot assign to immutable variable '{}'", name),
                                );
                            }
                            if let Some(vt) = val_ty {
                                if !assignable(&var_ty, &vt) {
                                    self.error(
                                        span,
                                        format!(
                                            "type mismatch: expected {:?}, got {:?}",
                                            var_ty, vt
                                        ),
                                    );
                                }
                            }
                        }
                    },
                    AssignTarget::Index { name, index } => {
                        if let Some(idx_ty) = self.check_expr(index, env) {
                            if idx_ty != Type::Int {
                                self.error(
                                    index.span,
                                    format!("array index must be int, got {:?}", idx_ty),
                                );
                            }
                        }
                        match env.get(name) {
                            None => self.error(span, format!("undefined variable '{}'", name)),
                            Some((ty, imm)) => {
                                let imm = imm;
                                match ty {
                                    Type::Array(elem_ty) => {
                                        let elem_ty = (**elem_ty).clone();
                                        if imm {
                                            self.error(
                                                span,
                                                format!(
                                                    "cannot assign to immutable array '{}'",
                                                    name
                                                ),
                                            );
                                        }
                                        if let Some(vt) = val_ty {
                                            if !assignable(&elem_ty, &vt) {
                                                self.error(
                                                    span,
                                                    format!(
                                                        "type mismatch: expected {:?}, got {:?}",
                                                        elem_ty, vt
                                                    ),
                                                );
                                            }
                                        }
                                    }
                                    other => {
                                        let other = other.clone();
                                        self.error(span, format!("cannot index into {:?}", other));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            StmtKind::If {
                cond,
                then_block,
                else_block,
            } => {
                if let Some(ty) = self.check_expr(cond, env) {
                    if ty != Type::Flag {
                        self.error(cond.span, format!("condition must be flag, got {:?}", ty));
                    }
                }
                self.check_block(then_block, env, ret);
                if let Some(else_stmts) = else_block {
                    self.check_block(else_stmts, env, ret);
                }
            }

            StmtKind::Route { subject, arms } => {
                let subj_ty = self.check_expr(subject, env);
                for arm in arms {
                    if let Some(pat_ty) = self.check_expr(&arm.pattern, env) {
                        if let Some(ref st) = subj_ty {
                            if !assignable(st, &pat_ty) {
                                self.error(
                                    arm.pattern.span,
                                    format!(
                                        "pattern type mismatch: expected {:?}, got {:?}",
                                        st, pat_ty
                                    ),
                                );
                            }
                        }
                    }
                    self.check_stmt(&arm.body, env, ret);
                }
            }

            StmtKind::Burn { cond, body } => {
                if let Some(ty) = self.check_expr(cond, env) {
                    if ty != Type::Flag {
                        self.error(
                            cond.span,
                            format!("burn condition must be flag, got {:?}", ty),
                        );
                    }
                }
                self.check_block(body, env, ret);
            }

            StmtKind::Spin { var, range, body } => {
                match &range.node {
                    ExprKind::Range { start, end, .. } => {
                        for bound in [start.as_ref(), end.as_ref()] {
                            if let Some(ty) = self.check_expr(bound, env) {
                                if ty != Type::Int {
                                    self.error(
                                        bound.span,
                                        format!("range bound must be int, got {:?}", ty),
                                    );
                                }
                            }
                        }
                    }
                    _ => self.error(range.span, "spin requires a range expression"),
                }
                env.push_scope();
                env.define(var.clone(), Type::Int, true);
                self.check_stmts(body, env, ret);
                env.pop_scope();
            }

            StmtKind::Orbit { body } => {
                self.check_block(body, env, ret);
            }

            StmtKind::Fire { name, args } => {
                self.check_call(span, name, args, env);
            }

            StmtKind::Land(expr) => match expr {
                None => {
                    if *ret != Type::Void {
                        self.error(span, format!("expected return value of type {:?}", ret));
                    }
                }
                Some(e) => {
                    if let Some(ty) = self.check_expr(e, env) {
                        if !assignable(ret, &ty) {
                            self.error(
                                e.span,
                                format!("return type mismatch: expected {:?}, got {:?}", ret, ty),
                            );
                        }
                    }
                }
            },

            StmtKind::Eject | StmtKind::Pass | StmtKind::Abort | StmtKind::Error => {}
        }
    }

    fn check_expr(&mut self, expr: &Expr, env: &TypeEnv) -> Option<Type> {
        match &expr.node {
            ExprKind::IntLit(_) => Some(Type::Int),
            ExprKind::FloatLit(_) => Some(Type::Float),
            ExprKind::StrLit(_) => Some(Type::Str),
            ExprKind::BoolLit(_) => Some(Type::Flag),
            ExprKind::ByteLit(_) => Some(Type::Byte),
            ExprKind::Air => Some(Type::Void),

            ExprKind::Ident(name) => match env.get(name) {
                Some((ty, _)) => Some(ty.clone()),
                None => {
                    self.error(expr.span, format!("undefined variable '{}'", name));
                    None
                }
            },

            ExprKind::Binary { op, lhs, rhs } => {
                let l = self.check_expr(lhs, env);
                let r = self.check_expr(rhs, env);
                match (l, r) {
                    (Some(l), Some(r)) => match binary_result(op, &l, &r) {
                        Some(ty) => Some(ty),
                        None => {
                            self.error(
                                expr.span,
                                format!(
                                    "operator {:?} cannot be applied to {:?} and {:?}",
                                    op, l, r
                                ),
                            );
                            None
                        }
                    },
                    _ => None,
                }
            }

            ExprKind::Unary { op, expr: inner } => {
                let ty = self.check_expr(inner, env)?;
                match op {
                    UnOp::Not => {
                        if ty != Type::Flag {
                            self.error(inner.span, format!("'not' requires flag, got {:?}", ty));
                            None
                        } else {
                            Some(Type::Flag)
                        }
                    }
                    UnOp::Neg => {
                        if matches!(ty, Type::Int | Type::Float) {
                            Some(ty)
                        } else {
                            self.error(inner.span, format!("cannot negate {:?}", ty));
                            None
                        }
                    }
                }
            }

            ExprKind::Call { name, args } => self.check_call(expr.span, name, args, env),

            ExprKind::Index { array, index } => {
                let arr_ty = self.check_expr(array, env);
                if let Some(idx_ty) = self.check_expr(index, env) {
                    if idx_ty != Type::Int {
                        self.error(
                            index.span,
                            format!("array index must be int, got {:?}", idx_ty),
                        );
                    }
                }
                match arr_ty {
                    Some(Type::Array(elem_ty)) => Some(*elem_ty),
                    Some(ty) => {
                        self.error(array.span, format!("cannot index into {:?}", ty));
                        None
                    }
                    None => None,
                }
            }

            ExprKind::ArrayLit(elems) => {
                if elems.is_empty() {
                    return Some(Type::Array(Box::new(Type::Void)));
                }
                let first = self.check_expr(&elems[0], env);
                for elem in &elems[1..] {
                    if let Some(et) = self.check_expr(elem, env) {
                        if let Some(ref ft) = first {
                            if !assignable(ft, &et) {
                                self.error(
                                    elem.span,
                                    format!(
                                        "array element type mismatch: expected {:?}, got {:?}",
                                        ft, et
                                    ),
                                );
                            }
                        }
                    }
                }
                first.map(|t| Type::Array(Box::new(t)))
            }

            ExprKind::Range { .. } => {
                self.error(expr.span, "range expression can only appear in spin");
                None
            }

            ExprKind::Error => None,
        }
    }

    fn check_call(&mut self, span: Span, name: &str, args: &[Expr], env: &TypeEnv) -> Option<Type> {
        if name == "log" {
            for arg in args {
                self.check_expr(arg, env);
            }
            return Some(Type::Void);
        }

        let sig = self.funcs.get(name).cloned();
        match sig {
            None => {
                self.error(span, format!("undefined function '{}'", name));
                None
            }
            Some((param_tys, ret_ty)) => {
                if args.len() != param_tys.len() {
                    self.error(
                        span,
                        format!(
                            "'{}' expects {} argument(s), got {}",
                            name,
                            param_tys.len(),
                            args.len()
                        ),
                    );
                }
                for (arg, param_ty) in args.iter().zip(param_tys.iter()) {
                    if let Some(arg_ty) = self.check_expr(arg, env) {
                        if !assignable(param_ty, &arg_ty) {
                            self.error(
                                arg.span,
                                format!(
                                    "argument type mismatch: expected {:?}, got {:?}",
                                    param_ty, arg_ty
                                ),
                            );
                        }
                    }
                }
                Some(ret_ty)
            }
        }
    }
}

fn assignable(expected: &Type, got: &Type) -> bool {
    match (expected, got) {
        (_, Type::Void) => true, // air type can be assigned to anything
        (Type::Float, Type::Int) => true,
        (a, b) => a == b,
    }
}

fn binary_result(op: &BinOp, l: &Type, r: &Type) -> Option<Type> {
    match op {
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => match (l, r) {
            (Type::Int, Type::Int) => Some(Type::Int),
            (Type::Float, Type::Float) => Some(Type::Float),
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => Some(Type::Float),
            (Type::Byte, Type::Byte) => Some(Type::Byte),
            _ => None,
        },
        BinOp::Eq | BinOp::NotEq => {
            if compatible(l, r) {
                Some(Type::Flag)
            } else {
                None
            }
        }
        BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => match (l, r) {
            (Type::Str, Type::Str) => Some(Type::Flag),
            (l, r) if is_numeric(l) && is_numeric(r) => Some(Type::Flag),
            _ => None,
        },
        BinOp::And | BinOp::Or => {
            if l == &Type::Flag && r == &Type::Flag {
                Some(Type::Flag)
            } else {
                None
            }
        }
    }
}

fn compatible(a: &Type, b: &Type) -> bool {
    match (a, b) {
        (Type::Int, Type::Float) | (Type::Float, Type::Int) => true,
        (a, b) => a == b,
    }
}

fn is_numeric(t: &Type) -> bool {
    matches!(t, Type::Int | Type::Float | Type::Byte)
}

pub fn check(program: &Program) -> Vec<TypeError> {
    let mut checker = Checker::new();

    // register all stage signatures (with forward declaration)
    for item in &program.items {
        if let Item::Stage {
            name,
            params,
            ret_type,
            ..
        } = item
        {
            let param_tys = params.iter().map(|p| p.ty.clone()).collect();
            checker
                .funcs
                .insert(name.clone(), (param_tys, ret_type.clone()));
        }
    }

    // ignite body
    let mut env = TypeEnv::new();
    checker.check_stmts(&program.ignite_body, &mut env, &Type::Void);

    // check each item
    for item in &program.items {
        checker.check_item(item);
    }

    checker.errors
}
