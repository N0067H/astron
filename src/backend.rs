use crate::{ast::*, frontend::CheckedProgram};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

mod control;
mod scalar;

pub fn emit_ir(checked: &CheckedProgram, target: &str) -> Result<String, String> {
    let program = checked.program();

    let body = program.items.iter().find_map(|item| match item {
        Item::Launch { name, body } if name == target => Some(body),
        _ => None,
    });
    let body = body.ok_or_else(|| format!("no 'launch {target}' found"))?;
    if control::needed(program, body) {
        return control::compile(checked, body);
    }
    scalar::compile(checked, body)
}

fn unsupported(checked: &CheckedProgram, span: Span) -> String {
    format!(
        "{}:{}:{}: unsupported native syntax; expected supported scalar expressions, control flow, or stage calls",
        checked.source_path(span).display(),
        span.line,
        span.col
    )
}

pub fn build(checked: &CheckedProgram, target: &str, output: &Path) -> Result<(), String> {
    let ir = emit_ir(checked, target)?;
    if let Ok(path) = fs::canonicalize(output)
        && checked.sources().contains(&path)
    {
        return Err(format!(
            "output would overwrite source '{}'",
            output.display()
        ));
    }

    let work = WorkDir::new()?;
    let ir_path = work.0.join("program.ll");
    let object_path = work.0.join("program.o");
    let executable = work
        .0
        .join(format!("program{}", std::env::consts::EXE_SUFFIX));
    fs::write(&ir_path, ir).map_err(|error| format!("cannot write LLVM IR: {error}"))?;

    run_tool(
        Command::new("clang")
            .args(["-x", "ir", "-c"])
            .arg(&ir_path)
            .arg("-o")
            .arg(&object_path),
        "LLVM object generation",
    )?;
    let mut linker = Command::new("clang");
    linker.arg(&object_path).arg("-o").arg(&executable);
    if cfg!(unix) {
        linker.arg("-lm");
    }
    run_tool(&mut linker, "native linking")?;
    fs::copy(executable, output)
        .map_err(|error| format!("cannot write executable '{}': {error}", output.display()))?;
    Ok(())
}

fn run_tool(command: &mut Command, phase: &str) -> Result<(), String> {
    let output = command.output().map_err(|error| {
        format!("{phase}: cannot run clang (install Clang and a linker): {error}")
    })?;
    if output.status.success() {
        return Ok(());
    }

    Err(format!(
        "{phase}: clang exited with {}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr).trim()
    ))
}

struct WorkDir(PathBuf);

impl WorkDir {
    fn new() -> Result<Self, String> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        loop {
            let id = NEXT.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("astron-build-{}-{id}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(format!("cannot create build directory: {error}")),
            }
        }
    }
}

impl Drop for WorkDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
