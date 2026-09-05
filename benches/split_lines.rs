use criterion::{Criterion, criterion_group, criterion_main};
use rayon::prelude::*;
use std::hint::black_box;

/*
### --- ###
Add to Cargo.toml:

[dev-dependencies]
# cargo bench
# see: ... /projects/.../target/criterion/report/index.html
criterion = { version = "0.4", features = ["html_reports"] }
itertools = "0.12"
plotters = "0.3"

[[bench]]
name = "split_lines"
harness = false

### --- ###

Create directory: benches
Create file: split_lines.rs
Put the file 'split_lines' inside the dir 'benches'
See: cargo bench --help
Run: cargo bench --bench split_lines

### --- ###
*/

use claudiofsr_lib::StrExtension;
use efd_contribuicoes::DELIMITER_CHAR;

pub fn split_line_v1(line: &str) -> Vec<String> {
    let mut campos: Vec<String> = line
        .split(DELIMITER_CHAR)
        .skip(1)
        .map(|campo| campo.trim().to_string())
        .collect();

    campos.pop();

    campos
}

pub fn split_line_v2(line: &str) -> Vec<String> {
    let mut campos: Vec<String> = line
        .split(DELIMITER_CHAR)
        .map(|campo| campo.trim().to_string())
        .collect();

    campos.remove(0);
    campos.pop();

    campos
}

pub fn split_line_v3(line: &str) -> Vec<String> {
    let campos: Vec<String> = line
        .strip_prefix(DELIMITER_CHAR)
        .unwrap_or(line)
        .strip_suffix(DELIMITER_CHAR)
        .unwrap_or(line)
        .split(DELIMITER_CHAR)
        .map(|campo| campo.trim().to_string())
        .collect();

    campos
}

pub fn split_line_v4(line: &str) -> Vec<String> {
    let campos: Vec<String> = line[1..line.len() - 1]
        .split(DELIMITER_CHAR)
        .map(|campo| campo.trim().to_string())
        .collect();

    campos
}

pub fn split_line_v5(line: &str) -> Vec<String> {
    let campos: Vec<String> = line[1..line.len() - 1]
        .par_split(DELIMITER_CHAR) // rayon: parallel iterator
        .map(|campo| campo.trim().to_string())
        .collect();

    campos
}

pub fn split_line_v6(line: &str) -> Vec<String> {
    let campos: Vec<String> = line
        .strip_prefix_and_sufix(DELIMITER_CHAR as u8)
        .split(DELIMITER_CHAR)
        .map(|campo| campo.trim().to_string())
        .collect();

    campos
}

fn criterion_benchmark(c: &mut Criterion) {
    let line01: &str =
        "| m210|  01  teste  |11890046,5|11890046,5| 1,65 |0||196185,7|1| 2|3|4|196185,77 |";
    let line02: &str = "| m210 |  01   teste  |25066,45|25066,45|0,00|0,00|25066,45|1,65|| |413,62|0,00|0,00|0,00|0,00| 413,6 |";
    let line03: &str = "|F010|||";
    let line04: &str = "|||";
    let line05: &str = "|C170|||||||";
    let line06: &str = "|A100|||||||||||";
    let line07: &str = "|F010|12345678901230|";
    let line08: &str = "|M505|04|50|795,82|0,00|795,82|795,82||||";
    let line09: &str = "|D100|0|1|ABC123450327|57|00|1||554|11986422715283185891401622845749354652669636|23112020|23112020|0||400,00|0,00|0|400,00|0,00|0,00|400,00||482|";
    let line10: &str = "|D100|0|1|ABC123450327|57|00|1|| 554 | 11986422715283185891401622845749354652669636 |23112020|23112020|0| | 400,00 |0,00|0|400,00|0,00|0,00|400,00| | 482| ";
    let line11: &str = " |  M505| 04|50|795,82|0,00|795,82|795,82 | | | ||| foo ||bar|| ||";
    let line12: &str = "|F990|15|";

    let lines = [
        line01, line02, line03, line04, line05, line06, line07, line08, line09, line10, line11,
        line12,
    ];

    let mut group = c.benchmark_group("Split Lines");

    group.warm_up_time(std::time::Duration::from_secs(10));
    group.measurement_time(std::time::Duration::from_secs(30));
    group.sample_size(10_000);

    group.bench_function("split_v1", |b| {
        b.iter(|| black_box(lines.iter().map(|line| split_line_v1(line))))
    });
    group.bench_function("split_v2", |b| {
        b.iter(|| black_box(lines.iter().map(|line| split_line_v2(line))))
    });
    group.bench_function("split_v3", |b| {
        b.iter(|| black_box(lines.iter().map(|line| split_line_v3(line))))
    });
    group.bench_function("split_v4", |b| {
        b.iter(|| black_box(lines.iter().map(|line| split_line_v4(line))))
    });
    group.bench_function("split_v5", |b| {
        b.iter(|| black_box(lines.iter().map(|line| split_line_v5(line))))
    });
    group.bench_function("split_v6", |b| {
        b.iter(|| black_box(lines.iter().map(|line| split_line_v6(line))))
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
