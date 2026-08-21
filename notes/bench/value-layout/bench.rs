use std::hint::black_box;
use std::mem::size_of;
use std::rc::Rc;
use std::time::Instant;

#[derive(Clone)]
enum OldValue {
    Int(i64),
    Str(String),
    Bool(bool),
    Array(Vec<OldValue>),
    Enum { enum_name: String, variant: String },
    Void,
}

#[derive(Clone)]
enum NewValue {
    Int(i64),
    Str(Rc<String>),
    Bool(bool),
    Array(Rc<Vec<NewValue>>),
    Enum {
        enum_name: Rc<String>,
        variant: Rc<String>,
    },
    Void,
}

fn build_old_values(len: usize) -> Vec<OldValue> {
    let mut values = Vec::with_capacity(len);
    for i in 0..len {
        let value = match i % 6 {
            0 => OldValue::Int(i as i64),
            1 => OldValue::Str(format!("mission-{}", i % 97)),
            2 => OldValue::Bool(i % 2 == 0),
            3 => OldValue::Array(vec![
                OldValue::Int(i as i64),
                OldValue::Int((i + 1) as i64),
                OldValue::Int((i + 2) as i64),
            ]),
            4 => OldValue::Enum {
                enum_name: "Phase".to_owned(),
                variant: format!("V{}", i % 8),
            },
            _ => OldValue::Void,
        };
        values.push(value);
    }
    values
}

fn build_new_values(len: usize) -> Vec<NewValue> {
    let enum_name = Rc::new("Phase".to_owned());
    let mut values = Vec::with_capacity(len);
    for i in 0..len {
        let value = match i % 6 {
            0 => NewValue::Int(i as i64),
            1 => NewValue::Str(Rc::new(format!("mission-{}", i % 97))),
            2 => NewValue::Bool(i % 2 == 0),
            3 => NewValue::Array(Rc::new(vec![
                NewValue::Int(i as i64),
                NewValue::Int((i + 1) as i64),
                NewValue::Int((i + 2) as i64),
            ])),
            4 => NewValue::Enum {
                enum_name: enum_name.clone(),
                variant: Rc::new(format!("V{}", i % 8)),
            },
            _ => NewValue::Void,
        };
        values.push(value);
    }
    values
}

fn scan_old(values: &[OldValue], rounds: usize) -> usize {
    let mut acc = 0usize;
    for _ in 0..rounds {
        for value in values {
            match value.clone() {
                OldValue::Int(n) => acc ^= n as usize,
                OldValue::Str(s) => acc ^= s.len(),
                OldValue::Bool(b) => acc ^= b as usize,
                OldValue::Array(v) => acc ^= v.len(),
                OldValue::Enum { enum_name, variant } => acc ^= enum_name.len() ^ variant.len(),
                OldValue::Void => acc ^= 1,
            }
        }
    }
    black_box(acc)
}

fn scan_new(values: &[NewValue], rounds: usize) -> usize {
    let mut acc = 0usize;
    for _ in 0..rounds {
        for value in values {
            match value.clone() {
                NewValue::Int(n) => acc ^= n as usize,
                NewValue::Str(s) => acc ^= s.len(),
                NewValue::Bool(b) => acc ^= b as usize,
                NewValue::Array(v) => acc ^= v.len(),
                NewValue::Enum { enum_name, variant } => acc ^= enum_name.len() ^ variant.len(),
                NewValue::Void => acc ^= 1,
            }
        }
    }
    black_box(acc)
}

fn measure_old(values: &[OldValue]) -> f64 {
    let start = Instant::now();
    black_box(scan_old(values, 30));
    start.elapsed().as_secs_f64() * 1000.0
}

fn measure_new(values: &[NewValue]) -> f64 {
    let start = Instant::now();
    black_box(scan_new(values, 30));
    start.elapsed().as_secs_f64() * 1000.0
}

fn main() {
    let old_values = build_old_values(80_000);
    let new_values = build_new_values(80_000);

    black_box(scan_old(&old_values, 2));
    black_box(scan_new(&new_values, 2));

    println!("old_value_size_bytes={}", size_of::<OldValue>());
    println!("new_value_size_bytes={}", size_of::<NewValue>());

    for trial in 1..=10 {
        println!(
            "trial={} before_ms={:.3} after_ms={:.3}",
            trial,
            measure_old(&old_values),
            measure_new(&new_values)
        );
    }
}
