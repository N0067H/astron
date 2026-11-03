use crate::{ast::*, checker, lexer, parser};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Diagnostic {
    pub path: PathBuf,
    pub span: Option<Span>,
    pub msg: String,
}

impl Diagnostic {
    fn new(path: PathBuf, span: Option<Span>, msg: impl Into<String>) -> Self {
        Self {
            path,
            span,
            msg: msg.into(),
        }
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.span {
            Some(span) => write!(
                f,
                "{}:{}:{}: {}",
                self.path.display(),
                span.line,
                span.col,
                self.msg
            ),
            None => write!(f, "{}: {}", self.path.display(), self.msg),
        }
    }
}

#[derive(Debug)]
pub struct CheckedProgram {
    program: Program,
    sources: Vec<PathBuf>,
}

impl CheckedProgram {
    pub fn program(&self) -> &Program {
        &self.program
    }

    pub fn sources(&self) -> &[PathBuf] {
        &self.sources
    }

    pub fn source_path(&self, span: Span) -> &Path {
        self.sources
            .get(span.file)
            .unwrap_or(&self.sources[0])
            .as_path()
    }
}

pub fn check_program(entry: &Path) -> Result<CheckedProgram, Vec<Diagnostic>> {
    let loaded = load_program(entry)?;
    let errors = checker::check(&loaded.program);
    if errors.is_empty() {
        return Ok(CheckedProgram {
            program: loaded.program,
            sources: loaded.sources,
        });
    }

    Err(errors
        .into_iter()
        .map(|error| {
            let path = loaded
                .sources
                .get(error.span.file)
                .map(PathBuf::as_path)
                .unwrap_or(entry);
            Diagnostic::new(path.to_path_buf(), Some(error.span), error.msg)
        })
        .collect())
}

struct LoadedProgram {
    program: Program,
    sources: Vec<PathBuf>,
}

fn load_program(entry: &Path) -> Result<LoadedProgram, Vec<Diagnostic>> {
    let mut loaded = HashSet::new();
    let mut stack = Vec::new();
    let mut sources = Vec::new();
    let program = load_program_recursive(entry, None, &mut loaded, &mut stack, &mut sources)?;
    Ok(LoadedProgram { program, sources })
}

fn load_program_recursive(
    path: &Path,
    import_site: Option<(PathBuf, Span)>,
    loaded: &mut HashSet<PathBuf>,
    stack: &mut Vec<PathBuf>,
    sources: &mut Vec<PathBuf>,
) -> Result<Program, Vec<Diagnostic>> {
    let canonical = match fs::canonicalize(path) {
        Ok(path) => path,
        Err(e) => {
            let (err_path, span) =
                import_site.unwrap_or_else(|| (path.to_path_buf(), Span::new(1, 1)));
            let action = if err_path == path {
                "cannot read"
            } else {
                "cannot resolve import"
            };
            return Err(vec![Diagnostic::new(
                err_path,
                Some(span),
                format!("{} '{}': {}", action, path.display(), e),
            )]);
        }
    };

    if loaded.contains(&canonical) {
        return Ok(empty_program());
    }

    if stack.contains(&canonical) {
        let (err_path, span) = import_site.unwrap_or_else(|| (canonical.clone(), Span::new(1, 1)));
        return Err(vec![Diagnostic::new(
            err_path,
            Some(span),
            format!("cyclic import detected for '{}'", canonical.display()),
        )]);
    }

    stack.push(canonical.clone());

    let source_id = sources.len();
    sources.push(canonical.clone());

    let parsed = match parse_file(&canonical, source_id) {
        Ok(program) => program,
        Err(errors) => {
            stack.pop();
            sources.pop();
            return Err(errors);
        }
    };

    let mut merged = empty_program();
    let base_dir = canonical.parent().unwrap_or_else(|| Path::new("."));
    for import in &parsed.imports {
        let import_path = Path::new(&import.path);
        let resolved = if import_path.is_absolute() {
            import_path.to_path_buf()
        } else {
            base_dir.join(import_path)
        };

        match load_program_recursive(
            &resolved,
            Some((canonical.clone(), import.span)),
            loaded,
            stack,
            sources,
        ) {
            Ok(imported) => merge_program(&mut merged, imported),
            Err(errors) => {
                stack.pop();
                return Err(errors);
            }
        }
    }

    merged.ignite_body.extend(parsed.ignite_body);
    merged.items.extend(parsed.items);
    loaded.insert(canonical);
    stack.pop();
    Ok(merged)
}

fn parse_file(path: &Path, source_id: usize) -> Result<Program, Vec<Diagnostic>> {
    let source = fs::read_to_string(path).map_err(|e| {
        vec![Diagnostic::new(
            path.to_path_buf(),
            Some(Span::new(1, 1)),
            format!("cannot read '{}': {}", path.display(), e),
        )]
    })?;

    let (tokens, lex_errors) = lexer::tokenize(&source);
    let (mut program, parse_errors) = parser::parse(tokens);

    let mut errors = Vec::new();
    for e in lex_errors {
        errors.push(Diagnostic::new(
            path.to_path_buf(),
            Some(Span::new(e.line, e.col)),
            e.msg,
        ));
    }
    for e in parse_errors {
        errors.push(Diagnostic::new(path.to_path_buf(), Some(e.span), e.msg));
    }

    if errors.is_empty() {
        tag_program_spans(&mut program, source_id);
        Ok(program)
    } else {
        Err(errors)
    }
}

fn empty_program() -> Program {
    Program {
        imports: Vec::new(),
        ignite_body: Vec::new(),
        items: Vec::new(),
    }
}

fn merge_program(target: &mut Program, source: Program) {
    target.ignite_body.extend(source.ignite_body);
    target.items.extend(source.items);
}

fn tag_program_spans(program: &mut Program, source_id: usize) {
    for import in &mut program.imports {
        import.span = import.span.in_file(source_id);
    }
    for stmt in &mut program.ignite_body {
        tag_stmt_spans(stmt, source_id);
    }
    for item in &mut program.items {
        match item {
            Item::Stage { body, .. } | Item::Launch { body, .. } => {
                tag_block_spans(body, source_id);
            }
            Item::Enum { variants, .. } => {
                tag_variant_spans(variants, source_id);
            }
        }
    }
}

fn tag_stmt_spans(stmt: &mut Stmt, source_id: usize) {
    stmt.span = stmt.span.in_file(source_id);
    match &mut stmt.node {
        StmtKind::Let { init, .. } => tag_expr_spans(init, source_id),
        StmtKind::Assign { target, value, .. } => {
            if let AssignTarget::Index { index, .. } = target {
                tag_expr_spans(index, source_id);
            }
            tag_expr_spans(value, source_id);
        }
        StmtKind::If {
            cond,
            then_block,
            else_block,
        } => {
            tag_expr_spans(cond, source_id);
            for stmt in then_block {
                tag_stmt_spans(stmt, source_id);
            }
            if let Some(else_block) = else_block {
                tag_block_spans(else_block, source_id);
            }
        }
        StmtKind::Route { subject, arms } => {
            tag_expr_spans(subject, source_id);
            for arm in arms {
                tag_expr_spans(&mut arm.pattern, source_id);
                tag_stmt_spans(&mut arm.body, source_id);
            }
        }
        StmtKind::Burn { cond, body } => {
            tag_expr_spans(cond, source_id);
            for stmt in body {
                tag_stmt_spans(stmt, source_id);
            }
        }
        StmtKind::Spin { range, body, .. } => {
            tag_expr_spans(range, source_id);
            for stmt in body {
                tag_stmt_spans(stmt, source_id);
            }
        }
        StmtKind::Orbit { body } => {
            for stmt in body {
                tag_stmt_spans(stmt, source_id);
            }
        }
        StmtKind::Fire { args, .. } => {
            for arg in args {
                tag_expr_spans(arg, source_id);
            }
        }
        StmtKind::Land(Some(expr)) => tag_expr_spans(expr, source_id),
        StmtKind::Land(None)
        | StmtKind::Eject
        | StmtKind::Pass
        | StmtKind::Abort
        | StmtKind::Error => {}
    }
}

fn tag_expr_spans(expr: &mut Expr, source_id: usize) {
    expr.span = expr.span.in_file(source_id);
    match &mut expr.node {
        ExprKind::Binary { lhs, rhs, .. } => {
            tag_expr_spans(lhs, source_id);
            tag_expr_spans(rhs, source_id);
        }
        ExprKind::Unary { expr, .. } => tag_expr_spans(expr, source_id),
        ExprKind::Call { args, .. } | ExprKind::ArrayLit(args) => {
            for arg in args {
                tag_expr_spans(arg, source_id);
            }
        }
        ExprKind::Index { array, index } => {
            tag_expr_spans(array, source_id);
            tag_expr_spans(index, source_id);
        }
        ExprKind::Range { start, end, .. } => {
            tag_expr_spans(start, source_id);
            tag_expr_spans(end, source_id);
        }
        ExprKind::IntLit(_)
        | ExprKind::FloatLit(_)
        | ExprKind::StrLit(_)
        | ExprKind::BoolLit(_)
        | ExprKind::ByteLit(_)
        | ExprKind::Air
        | ExprKind::Ident(_)
        | ExprKind::Error => {}
    }
}

fn tag_block_spans(body: &mut [Stmt], source_id: usize) {
    for stmt in body {
        tag_stmt_spans(stmt, source_id);
    }
}

fn tag_variant_spans(variants: &mut [EnumVariant], source_id: usize) {
    for variant in variants {
        variant.span = variant.span.in_file(source_id);
    }
}
