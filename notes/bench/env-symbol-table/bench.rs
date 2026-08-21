use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

struct OldEnv {
    scopes: Vec<Vec<String>>,
    values: HashMap<String, Vec<usize>>,
}

impl OldEnv {
    fn new() -> Self {
        Self {
            scopes: vec![Vec::new()],
            values: HashMap::new(),
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(Vec::new());
    }

    fn pop_scope(&mut self) {
        let names = self.scopes.pop().unwrap();
        for name in names {
            if let Some(stack) = self.values.get_mut(&name) {
                stack.pop();
            }
        }
    }

    fn define(&mut self, name: &str, value: usize) {
        self.scopes.last_mut().unwrap().push(name.to_owned());
        self.values.entry(name.to_owned()).or_default().push(value);
    }

    fn lookup(&self, name: &str) -> usize {
        *self.values.get(name).and_then(|stack| stack.last()).unwrap()
    }

    fn set(&mut self, name: &str, value: usize) {
        *self.values.get_mut(name).and_then(|stack| stack.last_mut()).unwrap() = value;
    }
}

struct NewEnv {
    scopes: Vec<Vec<usize>>,
    symbols: HashMap<String, usize>,
    values: Vec<Vec<usize>>,
}

impl NewEnv {
    fn new() -> Self {
        Self {
            scopes: vec![Vec::new()],
            symbols: HashMap::new(),
            values: Vec::new(),
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(Vec::new());
    }

    fn pop_scope(&mut self) {
        let symbols = self.scopes.pop().unwrap();
        for symbol in symbols {
            self.values[symbol].pop();
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

    fn define(&mut self, name: &str, value: usize) {
        let symbol = self.intern(name);
        self.scopes.last_mut().unwrap().push(symbol);
        self.values[symbol].push(value);
    }

    fn lookup(&self, name: &str) -> usize {
        let symbol = self.symbol(name).unwrap();
        *self.values[symbol].last().unwrap()
    }

    fn set(&mut self, name: &str, value: usize) {
        let symbol = self.symbol(name).unwrap();
        *self.values[symbol].last_mut().unwrap() = value;
    }
}

fn old_workload(names: &[String]) -> usize {
    let mut env = OldEnv::new();
    let mut acc = 0usize;
    for round in 0..1200 {
        env.push_scope();
        for (index, name) in names.iter().enumerate() {
            env.define(name, round + index);
        }
        for name in names {
            let current = env.lookup(name);
            env.set(name, current.wrapping_add(round));
            acc ^= env.lookup(name);
        }
        env.pop_scope();
    }

    black_box(acc)
}

fn new_workload(names: &[String]) -> usize {
    let mut env = NewEnv::new();
    let mut acc = 0usize;
    for round in 0..1200 {
        env.push_scope();
        for (index, name) in names.iter().enumerate() {
            env.define(name, round + index);
        }
        for name in names {
            let current = env.lookup(name);
            env.set(name, current.wrapping_add(round));
            acc ^= env.lookup(name);
        }
        env.pop_scope();
    }

    black_box(acc)
}

fn measure_old(names: &[String]) -> f64 {
    let start = Instant::now();
    black_box(old_workload(names));
    start.elapsed().as_secs_f64() * 1000.0
}

fn measure_new(names: &[String]) -> f64 {
    let start = Instant::now();
    black_box(new_workload(names));
    start.elapsed().as_secs_f64() * 1000.0
}

fn main() {
    let names = (0..256)
        .map(|index| format!("signal_{}_very_long_scope_name_for_churn", index))
        .collect::<Vec<_>>();

    black_box(old_workload(&names));
    black_box(new_workload(&names));

    for trial in 1..=10 {
        println!(
            "trial={} before_ms={:.3} after_ms={:.3}",
            trial,
            measure_old(&names),
            measure_new(&names)
        );
    }
}
