mod ast;
mod checker;
mod interpreter;
mod lexer;
mod parser;
pub mod token;

use ast::*;
use std::{
    collections::HashSet,
    env, fs,
    path::{Path, PathBuf},
};

struct LoadError {
    path: PathBuf,
    span: Option<Span>,
    msg: String,
}

impl LoadError {
    fn new(path: PathBuf, span: Option<Span>, msg: impl Into<String>) -> Self {
        LoadError {
            path,
            span,
            msg: msg.into(),
        }
    }
}

fn main() {
    let mut args = env::args().skip(1);
    let path = args.next().unwrap_or_else(|| {
        eprintln!("usage: astron <file.astrn> [--launch <name>]");
        std::process::exit(1);
    });

    let mut target = "main".to_string();
    while let Some(arg) = args.next() {
        if arg == "--launch" {
            target = args.next().unwrap_or_else(|| {
                eprintln!("error: --launch requires a name");
                std::process::exit(1);
            });
        }
    }

    let loaded = match load_program(Path::new(&path)) {
        Ok(program) => program,
        Err(errors) => {
            for e in errors {
                print_load_error(&e);
            }
            std::process::exit(1);
        }
    };

    let errors = checker::check(&loaded.program);
    if !errors.is_empty() {
        for e in &errors {
            let source_path = loaded
                .sources
                .get(e.span.file)
                .map(|path| path.as_path())
                .unwrap_or_else(|| Path::new(&path));
            eprintln!(
                "{}:{}:{}: {}",
                source_path.display(),
                e.span.line,
                e.span.col,
                e.msg
            );
        }
        std::process::exit(1);
    }

    let interp = interpreter::Interpreter::new(&loaded.program);
    match interp.run(&loaded.program, &target) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            let source_path = loaded
                .sources
                .get(error.span.file)
                .map(|path| path.as_path())
                .unwrap_or_else(|| Path::new(&path));
            eprintln!(
                "{}:{}:{}: runtime error: {}",
                source_path.display(),
                error.span.line,
                error.span.col,
                error.msg
            );
            std::process::exit(1);
        }
    }
}

struct LoadedProgram {
    program: Program,
    sources: Vec<PathBuf>,
}

fn load_program(entry: &Path) -> Result<LoadedProgram, Vec<LoadError>> {
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
) -> Result<Program, Vec<LoadError>> {
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
            return Err(vec![LoadError::new(
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
        return Err(vec![LoadError::new(
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

fn parse_file(path: &Path, source_id: usize) -> Result<Program, Vec<LoadError>> {
    let source = fs::read_to_string(path).map_err(|e| {
        vec![LoadError::new(
            path.to_path_buf(),
            Some(Span::new(1, 1)),
            format!("cannot read '{}': {}", path.display(), e),
        )]
    })?;

    let (tokens, lex_errors) = lexer::tokenize(&source);
    let (mut program, parse_errors) = parser::parse(tokens);

    let mut errors = Vec::new();
    for e in lex_errors {
        errors.push(LoadError::new(
            path.to_path_buf(),
            Some(Span::new(e.line, e.col)),
            e.msg,
        ));
    }
    for e in parse_errors {
        errors.push(LoadError::new(path.to_path_buf(), Some(e.span), e.msg));
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

fn print_load_error(error: &LoadError) {
    match error.span {
        Some(span) => eprintln!(
            "{}:{}:{}: {}",
            error.path.display(),
            span.line,
            span.col,
            error.msg
        ),
        None => eprintln!("{}: {}", error.path.display(), error.msg),
    }
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
                for stmt in body {
                    tag_stmt_spans(stmt, source_id);
                }
            }
            Item::Enum { variants, .. } => {
                for variant in variants {
                    variant.span = variant.span.in_file(source_id);
                }
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
                for stmt in else_block {
                    tag_stmt_spans(stmt, source_id);
                }
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
