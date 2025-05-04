use criterion::{Criterion, black_box, criterion_group, criterion_main};
use itertools::Itertools;

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
name = "remove_white_space"
harness = false

### --- ###

Create directory: benches
Create file: remove_white_space.rs
Put the file 'remove_white_space.rs' inside the dir 'benches'
See: cargo bench --help
Run: cargo bench

### --- ###
*/

fn trim_whitespace_replace(s: &str) -> String {
    let mut new_str: String = s.trim().to_owned();
    while new_str.contains("  ") {
        new_str = new_str.replace("  ", " ");
    }
    new_str
}

fn remove_multiple_whitespace1(s: &str) -> String {
    let mut new_str: String = s.trim().to_owned();
    let mut previous_char: char = ' '; // The initial value doesn't really matter
    new_str.retain(|current_char| {
        let keep: bool = current_char != ' ' || previous_char != ' ';
        previous_char = current_char;
        keep
    });
    new_str
}

fn remove_multiple_whitespace2(s: &str) -> String {
    let mut new_str: String = s.trim().to_owned();
    let mut previous_char: char = ' ';
    new_str.retain(|current_char| {
        let keep: bool = !(previous_char == ' ' && current_char == ' ');
        previous_char = current_char;
        keep
    });
    new_str
}

fn remove_multiple_whitespace3(s: &str) -> String {
    let mut new_str: String = s.trim().to_string();
    let mut previous_char: char = ' ';
    new_str.retain(|current_char| {
        let keep: bool = current_char != ' ' || previous_char != ' ';
        previous_char = current_char;
        keep
    });
    new_str
}

fn remove_multiple_whitespace4(s: &str) -> String {
    let mut new_str: String = s.trim().to_string();
    let mut previous_char: char = ' ';
    new_str.retain(|current_char| {
        let keep: bool = !(previous_char == ' ' && current_char == ' ');
        previous_char = current_char;
        keep
    });
    new_str
}

fn remove_multiple_whitespace5(s: &str) -> String {
    let mut new_str: String = s.to_string();
    let mut previous_char: char = 'x'; // some non-whitespace character
    new_str.retain(|current_char| {
        let keep: bool = previous_char != ' ' || current_char != ' ';
        previous_char = current_char;
        keep
    });
    new_str
}

fn tw_split_space(s: &str) -> String {
    s.trim()
        .split(' ')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn twss_itertools(s: &str) -> String {
    s.trim().split(' ').filter(|s| !s.is_empty()).join(" ")
}

fn tw_split_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn tw_itertools(s: &str) -> String {
    s.split_whitespace().join(" ")
}

fn tw_one_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    s.split_whitespace().for_each(|s| {
        result.push_str(s);
        result.push(' ');
    });
    result.pop(); // remove last space
    result
}

fn trim_whitespace_v2(s: &str) -> String {
    // second attempt: only allocate a string
    let mut result = String::with_capacity(s.len());
    s.split_whitespace().for_each(|w| {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(w);
    });
    result
}

fn benchmark(c: &mut Criterion) {
    let strings: &str =
        "  a   bb cc   ddd  fraç   ão foo  bar !!  1 2  3 45 程式   设计  67  && RR ";

    let mut group = c.benchmark_group("Espaços em Branco");

    group.warm_up_time(std::time::Duration::from_secs(10));
    group.measurement_time(std::time::Duration::from_secs(20));
    group.sample_size(5_000);

    group.bench_function("trim_whitespace_replace", |b| {
        b.iter(|| trim_whitespace_replace(black_box(strings)))
    });
    group.bench_function("remove_multiple_whitespace1", |b| {
        b.iter(|| remove_multiple_whitespace1(black_box(strings)))
    });
    group.bench_function("remove_multiple_whitespace2", |b| {
        b.iter(|| remove_multiple_whitespace2(black_box(strings)))
    });
    group.bench_function("remove_multiple_whitespace3", |b| {
        b.iter(|| remove_multiple_whitespace3(black_box(strings)))
    });
    group.bench_function("remove_multiple_whitespace4", |b| {
        b.iter(|| remove_multiple_whitespace4(black_box(strings)))
    });
    group.bench_function("remove_multiple_whitespace5", |b| {
        b.iter(|| remove_multiple_whitespace5(black_box(strings.trim())))
    });
    group.bench_function("tw_split_space", |b| {
        b.iter(|| tw_split_space(black_box(strings)))
    });
    group.bench_function("twss_itertools", |b| {
        b.iter(|| twss_itertools(black_box(strings)))
    });
    group.bench_function("tw_split_whitespace", |b| {
        b.iter(|| tw_split_whitespace(black_box(strings)))
    });
    group.bench_function("tw_itertools", |b| {
        b.iter(|| tw_itertools(black_box(strings)))
    });
    group.bench_function("tw_one_string", |b| {
        b.iter(|| tw_one_string(black_box(strings)))
    });
    group.bench_function("trim_whitespace_v2", |b| {
        b.iter(|| trim_whitespace_v2(black_box(strings)))
    });

    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
