use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

#[derive(Clone)]
struct Stage {
    name: String,
    params: Vec<String>,
    body: Vec<Stmt>,
}

#[derive(Clone)]
struct Stmt {
    opcode: u8,
    operand: i64,
}

struct Program {
    stages: Vec<Stage>,
}

struct OldInterpreter {
    functions: HashMap<String, (Vec<String>, Vec<Stmt>)>,
}

struct NewInterpreter<'a> {
    program: &'a Program,
    functions: HashMap<&'a str, usize>,
}

impl OldInterpreter {
    fn new(program: &Program) -> Self {
        let mut functions = HashMap::new();
        for stage in &program.stages {
            functions.insert(stage.name.clone(), (stage.params.clone(), stage.body.clone()));
        }
        Self { functions }
    }

    fn dispatch_checksum(&self) -> usize {
        let mut acc = 0usize;
        for (name, (params, body)) in &self.functions {
            acc ^= name.len();
            acc ^= params.len();
            acc ^= body.len();
            acc ^= body.iter().map(|stmt| stmt.opcode as usize ^ stmt.operand as usize).sum::<usize>();
        }
        black_box(acc)
    }
}

impl<'a> NewInterpreter<'a> {
    fn new(program: &'a Program) -> Self {
        let mut functions = HashMap::new();
        for (index, stage) in program.stages.iter().enumerate() {
            functions.insert(stage.name.as_str(), index);
        }
        Self { program, functions }
    }

    fn dispatch_checksum(&self) -> usize {
        let mut acc = 0usize;
        for (name, index) in &self.functions {
            let stage = &self.program.stages[*index];
            acc ^= name.len();
            acc ^= stage.params.len();
            acc ^= stage.body.len();
            acc ^= stage
                .body
                .iter()
                .map(|stmt| stmt.opcode as usize ^ stmt.operand as usize)
                .sum::<usize>();
        }
        black_box(acc)
    }
}

fn build_program() -> Program {
    let mut stages = Vec::with_capacity(1500);
    for stage_index in 0..1500 {
        let params = (0..8)
            .map(|param| format!("p{}_{}", stage_index, param))
            .collect::<Vec<_>>();
        let body = (0..96)
            .map(|stmt| Stmt {
                opcode: (stmt % 7) as u8,
                operand: (stage_index as i64) * 17 + stmt as i64,
            })
            .collect::<Vec<_>>();
        stages.push(Stage {
            name: format!("stage_{}", stage_index),
            params,
            body,
        });
    }
    Program { stages }
}

fn measure_old(program: &Program) -> f64 {
    let start = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..60 {
        let interp = OldInterpreter::new(program);
        checksum ^= interp.dispatch_checksum();
    }
    black_box(checksum);
    start.elapsed().as_secs_f64() * 1000.0
}

fn measure_new(program: &Program) -> f64 {
    let start = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..60 {
        let interp = NewInterpreter::new(program);
        checksum ^= interp.dispatch_checksum();
    }
    black_box(checksum);
    start.elapsed().as_secs_f64() * 1000.0
}

fn main() {
    let program = build_program();
    black_box(measure_old(&program));
    black_box(measure_new(&program));

    for trial in 1..=10 {
        println!(
            "trial={} before_ms={:.3} after_ms={:.3}",
            trial,
            measure_old(&program),
            measure_new(&program)
        );
    }
}
