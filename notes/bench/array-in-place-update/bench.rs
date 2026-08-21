use std::hint::black_box;
use std::rc::Rc;
use std::time::Instant;

#[derive(Clone)]
enum OldValue {
    Array(Vec<i64>),
}

#[derive(Clone)]
enum NewValue {
    Array(Rc<Vec<i64>>),
}

fn measure_old() -> f64 {
    let mut value = OldValue::Array((0..1024).map(|n| n as i64).collect());
    let start = Instant::now();
    let mut checksum = 0i64;
    for step in 0..200_000usize {
        let OldValue::Array(mut elems) = value.clone();
        let index = step % elems.len();
        let next = elems[index] + ((step & 7) as i64);
        elems[index] = next;
        checksum ^= next;
        value = OldValue::Array(elems);
    }
    black_box(checksum);
    start.elapsed().as_secs_f64() * 1000.0
}

fn measure_new() -> f64 {
    let mut value = NewValue::Array(Rc::new((0..1024).map(|n| n as i64).collect()));
    let start = Instant::now();
    let mut checksum = 0i64;
    for step in 0..200_000usize {
        let NewValue::Array(ref mut elems) = value;
        let elems = Rc::make_mut(elems);
        let index = step % elems.len();
        let next = elems[index] + ((step & 7) as i64);
        elems[index] = next;
        checksum ^= next;
    }
    black_box(checksum);
    start.elapsed().as_secs_f64() * 1000.0
}

fn main() {
    black_box(measure_old());
    black_box(measure_new());

    for trial in 1..=10 {
        println!(
            "trial={} before_ms={:.3} after_ms={:.3}",
            trial,
            measure_old(),
            measure_new()
        );
    }
}
