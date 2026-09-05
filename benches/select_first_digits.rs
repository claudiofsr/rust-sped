use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

/*
// Test which of the following functions is the fastest:
// https://crates.io/crates/criterion
// https://bheisler.github.io/criterion.rs/book/index.html
// https://stackoverflow.com/questions/71864137/whats-the-ideal-way-to-trim-extra-spaces-from-a-string
// Uses gnuplot to generate detailed graphs of benchmark results
// pacman -S gnuplot

### --- ###
Add to Cargo.toml:

[dev-dependencies]
# cargo bench
# see: ... /projects/.../target/criterion/report/index.html
criterion = { version = "0.4", features = ["html_reports"] }
itertools = "0.12"
plotters = "0.3"

[[bench]]
name = "select_first_digits"
harness = false

### --- ###
Create directory: benches
Create file: select_first_digits.rs
Put the file 'select_first_digits.rs' inside the dir 'benches'
Run: cargo bench

### --- ###
*/

pub trait ExtraProperties {
    fn select_first_digits_retain(self) -> String;
    fn select_first_digits_map_while(self) -> String;
    fn select_first_digits_take_while(self) -> String;
}

impl ExtraProperties for &str {
    // Capturar ou Reter apenas o primeiro grupo de dígitos: 1191-1  --> 1191  ou 10845/a --> 10845

    fn select_first_digits_retain(self) -> String {
        let mut new_str: String = self.to_string();
        let mut keep: bool = true;
        new_str.retain(|current_char| {
            if !current_char.is_ascii_digit() {
                keep = false;
            }
            keep
        });
        new_str
    }

    fn select_first_digits_map_while(self) -> String {
        self.chars()
            .map_while(|x| x.is_ascii_digit().then_some(x))
            .collect::<String>()
    }

    fn select_first_digits_take_while(self) -> String {
        self.chars()
            .take_while(|x| x.is_ascii_digit())
            .collect::<String>()
    }
}

fn benchmark(c: &mut Criterion) {
    let strings: &str = "1191-clá";

    let mut group = c.benchmark_group("Filtrar Primeiros Dígitos");

    group.warm_up_time(std::time::Duration::from_secs(10));
    group.measurement_time(std::time::Duration::from_secs(20));
    group.sample_size(5_000);

    group.bench_function("retain", |b| {
        b.iter(|| black_box(strings.select_first_digits_retain()))
    });
    group.bench_function("map_while", |b| {
        b.iter(|| black_box(strings.select_first_digits_map_while()))
    });
    group.bench_function("take_while", |b| {
        b.iter(|| black_box(strings.select_first_digits_take_while()))
    });

    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
