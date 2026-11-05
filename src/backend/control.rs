use super::{WorkDir, run_tool, unsupported};
use crate::{ast::*, frontend::CheckedProgram};
use std::{collections::HashMap, fs, process::Command};

pub(super) fn needed(program: &Program, body: &[Stmt]) -> bool {
    !program.ignite_body.is_empty()
        || program
            .items
            .iter()
            .any(|item| matches!(item, Item::Stage { .. }))
        || body.iter().any(|stmt| {
            !matches!(
                stmt.node,
                StmtKind::Let { .. } | StmtKind::Assign { .. } | StmtKind::Land(Some(_))
            )
        })
}

struct Generator<'a> {
    checked: &'a CheckedProgram,
    scopes: Vec<HashMap<String, String>>,
    code: String,
    next: usize,
    stage: bool,
    ignite: bool,
}

pub(super) fn compile(checked: &CheckedProgram, body: &[Stmt]) -> Result<String, String> {
    let mut g = Generator {
        checked,
        scopes: vec![HashMap::new()],
        code: include_str!("runtime.c").to_owned(),
        next: 0,
        stage: true,
        ignite: false,
    };
    for item in &checked.program().items {
        if let Item::Stage { name, params, .. } = item {
            g.code
                .push_str(&format!("V s_{name}({});\n", parameters(params)));
        }
    }
    for item in &checked.program().items {
        if let Item::Stage {
            name, params, body, ..
        } = item
        {
            g.function(name, params, body)?;
        }
    }
    g.stage = false;
    g.code.push_str("int main(void) {\n");
    g.ignite = true;
    g.statements(&checked.program().ignite_body)?;
    g.ignite = false;
    g.code.push_str("launch:;\n");
    g.statements(body)?;
    g.code.push_str("return 0;\n}\n");
    let work = WorkDir::new()?;
    let source = work.0.join("control.c");
    let output = work.0.join("control.ll");
    fs::write(&source, g.code).map_err(|e| format!("cannot write native source: {e}"))?;
    run_tool(
        Command::new("clang")
            .args(["-S", "-emit-llvm", "-O0", "-std=c11"])
            .arg(&source)
            .arg("-o")
            .arg(&output),
        "control-flow LLVM generation",
    )?;
    fs::read_to_string(output).map_err(|e| format!("cannot read LLVM IR: {e}"))
}

fn parameters(params: &[Param]) -> String {
    if params.is_empty() {
        return "void".to_owned();
    }
    params
        .iter()
        .map(|p| format!("V p_{}", p.name))
        .collect::<Vec<_>>()
        .join(", ")
}

impl Generator<'_> {
    fn fresh(&mut self) -> String {
        let name = format!("v{}", self.next);
        self.next += 1;
        name
    }

    fn value(&mut self, expression: String) -> String {
        let name = self.fresh();
        self.code.push_str(&format!("V {name} = {expression};\n"));
        name
    }

    fn function(&mut self, name: &str, params: &[Param], body: &[Stmt]) -> Result<(), String> {
        self.scopes = vec![HashMap::new()];
        for param in params {
            self.scopes[0].insert(param.name.clone(), format!("p_{}", param.name));
        }
        self.code
            .push_str(&format!("V s_{name}({}) {{\n", parameters(params)));
        self.statements(body)?;
        self.code.push_str("return air();\n}\n");
        self.scopes = vec![HashMap::new()];
        Ok(())
    }

    fn lookup(&self, name: &str, span: Span) -> Result<String, String> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name))
            .cloned()
            .ok_or_else(|| unsupported(self.checked, span))
    }

    fn statements(&mut self, body: &[Stmt]) -> Result<(), String> {
        for stmt in body {
            self.statement(stmt)?;
        }
        Ok(())
    }

    fn block(&mut self, body: &[Stmt]) -> Result<(), String> {
        self.code.push_str("{\n");
        self.scopes.push(HashMap::new());
        self.statements(body)?;
        self.scopes.pop();
        self.code.push_str("}\n");
        Ok(())
    }

    fn statement(&mut self, stmt: &Stmt) -> Result<(), String> {
        match &stmt.node {
            StmtKind::Let { name, init, .. } => self.declaration(name, init),
            StmtKind::Assign {
                target: AssignTarget::Ident(name),
                op,
                value,
            } => self.assignment(name, op, value, stmt.span),
            StmtKind::If {
                cond,
                then_block,
                else_block,
            } => self.scan(cond, then_block, else_block.as_deref()),
            StmtKind::Route { subject, arms } => self.route(subject, arms),
            StmtKind::Burn { cond, body } => self.burn(cond, body),
            StmtKind::Orbit { body } => self.orbit(body),
            StmtKind::Spin { var, range, body } => self.spin(var, range, body),
            StmtKind::Fire { name, args } => self.call(name, args, stmt.span).map(|_| ()),
            StmtKind::Land(expr) => self.land(expr.as_ref()),
            StmtKind::Abort => self.abort(),
            StmtKind::Eject => self.line("break;"),
            StmtKind::Pass => self.line("continue;"),
            _ => Err(unsupported(self.checked, stmt.span)),
        }
    }

    fn line(&mut self, line: &str) -> Result<(), String> {
        self.code.push_str(line);
        self.code.push('\n');
        Ok(())
    }

    fn declaration(&mut self, name: &str, init: &Expr) -> Result<(), String> {
        let value = self.expression(init)?;
        let slot = self.value(value);
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.to_owned(), slot);
        Ok(())
    }

    fn assignment(
        &mut self,
        name: &str,
        op: &AssignOp,
        expr: &Expr,
        span: Span,
    ) -> Result<(), String> {
        let rhs = self.expression(expr)?;
        let slot = self.lookup(name, span)?;
        let value = match op {
            AssignOp::Assign => rhs,
            AssignOp::Add => format!("binary(0, {slot}, {rhs})"),
            AssignOp::Sub => format!("binary(1, {slot}, {rhs})"),
            AssignOp::Mul => format!("binary(2, {slot}, {rhs})"),
            AssignOp::Div => format!("binary(3, {slot}, {rhs})"),
            AssignOp::Mod => format!("binary(4, {slot}, {rhs})"),
        };
        self.line(&format!("{slot} = {value};"))
    }

    fn scan(&mut self, cond: &Expr, yes: &[Stmt], no: Option<&[Stmt]>) -> Result<(), String> {
        let cond = self.expression(cond)?;
        self.code.push_str(&format!("if (truth({cond}))\n"));
        self.block(yes)?;
        if let Some(body) = no {
            self.code.push_str("else\n");
            self.block(body)?;
        }
        Ok(())
    }

    fn burn(&mut self, cond: &Expr, body: &[Stmt]) -> Result<(), String> {
        self.code.push_str("while (1) {\n");
        let cond = self.expression(cond)?;
        self.code.push_str(&format!("if (!truth({cond})) break;\n"));
        self.block(body)?;
        self.code.push_str("}\n");
        Ok(())
    }

    fn orbit(&mut self, body: &[Stmt]) -> Result<(), String> {
        self.code.push_str("while (1)\n");
        self.block(body)
    }

    fn abort(&mut self) -> Result<(), String> {
        if self.stage {
            return self.line("return air();");
        }
        self.line("return 1;")
    }

    fn land(&mut self, expr: Option<&Expr>) -> Result<(), String> {
        let value = match expr {
            Some(expr) => self.expression(expr)?,
            None => self.value("air()".into()),
        };
        let action = if self.stage {
            format!("return {value};")
        } else if self.ignite {
            format!("if ({value}.k == 0) return (int){value}.i; goto launch;")
        } else {
            format!("return {value}.k == 0 ? (int){value}.i : 0;")
        };
        self.code.push_str(&format!("{action}\n"));
        Ok(())
    }

    fn route(&mut self, subject: &Expr, arms: &[RouteArm]) -> Result<(), String> {
        let subject = self.expression(subject)?;
        let done = self.fresh();
        for arm in arms {
            let pattern = self.expression(&arm.pattern)?;
            self.code
                .push_str(&format!("if (equal({subject}, {pattern})) {{\n"));
            self.statement(&arm.body)?;
            self.code.push_str(&format!("goto {done};\n}}\n"));
        }
        self.code.push_str(&format!("{done}:;\n"));
        Ok(())
    }

    fn spin(&mut self, var: &str, range: &Expr, body: &[Stmt]) -> Result<(), String> {
        let ExprKind::Range {
            start,
            end,
            inclusive,
        } = &range.node
        else {
            return Err(unsupported(self.checked, range.span));
        };
        let start = self.expression(start)?;
        let end = self.expression(end)?;
        let index = self.fresh();
        let bound = self.fresh();
        self.code
            .push_str(&format!("{{\nint64_t {bound} = integer({end});\n"));
        if *inclusive {
            self.code
                .push_str(&format!("if ({bound} == INT64_MAX) exit(1);\n{bound}++;\n"));
        }
        self.code.push_str(&format!(
            "for (int64_t {index} = integer({start}); {index} < {bound}; {index}++) {{\n"
        ));
        self.scopes.push(HashMap::new());
        let slot = self.value(format!("vi({index})"));
        self.scopes.last_mut().unwrap().insert(var.to_owned(), slot);
        self.statements(body)?;
        self.scopes.pop();
        self.code.push_str("}\n}\n");
        Ok(())
    }

    fn expression(&mut self, expr: &Expr) -> Result<String, String> {
        let expression = match &expr.node {
            ExprKind::IntLit(n) => format!("vi((int64_t)UINT64_C({}))", *n as u64),
            ExprKind::FloatLit(n) => format!("vf({n:.17e})"),
            ExprKind::BoolLit(b) => format!("vb({})", i32::from(*b)),
            ExprKind::ByteLit(b) => format!("vy({b})"),
            ExprKind::StrLit(s) => format!("vs({})", c_string(s)),
            ExprKind::Air => "air()".into(),
            ExprKind::Ident(name) => self.lookup(name, expr.span)?,
            ExprKind::Call { name, args } => return self.call(name, args, expr.span),
            ExprKind::Unary { op, expr } => return self.unary(op, expr),
            ExprKind::Binary {
                op: BinOp::And | BinOp::Or,
                lhs,
                rhs,
            } => return self.logical(expr, lhs, rhs),
            ExprKind::Binary { op, lhs, rhs } => return self.binary(op, lhs, rhs),
            _ => return Err(unsupported(self.checked, expr.span)),
        };
        Ok(self.value(expression))
    }

    fn unary(&mut self, op: &UnOp, expr: &Expr) -> Result<String, String> {
        let value = self.expression(expr)?;
        Ok(self.value(format!("unary({}, {value})", i32::from(*op == UnOp::Not))))
    }

    fn binary(&mut self, op: &BinOp, lhs: &Expr, rhs: &Expr) -> Result<String, String> {
        let lhs = self.expression(lhs)?;
        let rhs = self.expression(rhs)?;
        Ok(self.value(format!("binary({}, {lhs}, {rhs})", opcode(op))))
    }

    fn logical(&mut self, expr: &Expr, lhs: &Expr, rhs: &Expr) -> Result<String, String> {
        let left = self.expression(lhs)?;
        let result = self.value(format!("vb(truth({left}))"));
        let ExprKind::Binary { op, .. } = &expr.node else {
            unreachable!()
        };
        let negate = if *op == BinOp::Or { "!" } else { "" };
        self.code
            .push_str(&format!("if ({negate}truth({result})) {{\n"));
        let right = self.expression(rhs)?;
        self.code
            .push_str(&format!("{result} = vb(truth({right}));\n}}\n"));
        Ok(result)
    }

    fn call(&mut self, name: &str, args: &[Expr], span: Span) -> Result<String, String> {
        let values = args
            .iter()
            .map(|arg| self.expression(arg))
            .collect::<Result<Vec<_>, _>>()?;
        if name == "log" {
            return Ok(self.log(&values));
        }
        if !self
            .checked
            .program()
            .items
            .iter()
            .any(|item| matches!(item, Item::Stage { name: stage, .. } if stage == name))
        {
            return Err(unsupported(self.checked, span));
        }
        Ok(self.value(format!("s_{name}({})", values.join(", "))))
    }
    fn separator(&mut self, index: usize) {
        if index > 0 {
            self.code.push_str("putchar(' ');\n");
        }
    }

    fn log(&mut self, values: &[String]) -> String {
        for (index, value) in values.iter().enumerate() {
            self.separator(index);
            self.code.push_str(&format!("display({value});\n"));
        }
        self.code.push_str("putchar('\\n');\n");
        self.value("air()".into())
    }
}

fn opcode(op: &BinOp) -> usize {
    match op {
        BinOp::Add => 0,
        BinOp::Sub => 1,
        BinOp::Mul => 2,
        BinOp::Div => 3,
        BinOp::Mod => 4,
        BinOp::Eq => 5,
        BinOp::NotEq => 6,
        BinOp::Lt => 7,
        BinOp::Gt => 8,
        BinOp::LtEq => 9,
        BinOp::GtEq => 10,
        BinOp::And | BinOp::Or => unreachable!(),
    }
}

fn c_string(value: &str) -> String {
    let bytes = value
        .as_bytes()
        .iter()
        .map(|b| format!("\\{b:03o}"))
        .collect::<String>();
    format!("\"{bytes}\"")
}
