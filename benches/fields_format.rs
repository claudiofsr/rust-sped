use claudiofsr_lib::StrExtension;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

// Testar quais das funções seguintes é a mais rápida

// https://crates.io/crates/criterion
// https://bheisler.github.io/criterion.rs/book/index.html
// https://stackoverflow.com/questions/71864137/whats-the-ideal-way-to-trim-extra-spaces-from-a-string
// Uses gnuplot to generate detailed graphs of benchmark results
// pacman -S gnuplot

/*
### --- ###
Add to Cargo.toml:

[dev-dependencies]
# cargo bench
# see: ... /projects/.../target/criterion/report/index.html
criterion = { version = "0.5", features = ["html_reports"] }
itertools = "0.12"
plotters = "0.3"

[[bench]]
name = "fields_format"
harness = false

### --- ###

Create directory: benches
Create file: fields_format.rs
Put the file 'fields_format.rs' inside the dir 'benches'
See: cargo bench --help
Run: cargo bench --bench fields_format

### --- ###
*/

pub fn format1(fields: &[&str]) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();

    for &field in fields {
        // https://stackoverflow.com/questions/43516351/how-to-convert-a-string-of-digits-into-a-vector-of-digits
        let digits = field.to_digits();

        // https://stackoverflow.com/questions/30154541/how-do-i-concatenate-strings
        // 14 digits: exemplo CNPJ: 01234567000890 --> 01.234.567/0008-90
        // 11 digits: exemplo CPF: 12345678901     --> 123.456.789-01
        //  8 digits: exemplo NCM: 01234567        --> 0123.45.67

        let formatted_text: String = match digits.len() {
            14 => [
                &field[0..2],
                ".",
                &field[2..5],
                ".",
                &field[5..8],
                "/",
                &field[8..12],
                "-",
                &field[12..],
            ]
            .concat(),
            11 => [
                &field[0..3],
                ".",
                &field[3..6],
                ".",
                &field[6..9],
                "-",
                &field[9..],
            ]
            .concat(),
            8 => [&field[0..4], ".", &field[4..6], ".", &field[6..]].concat(),
            _ => continue,
        };

        result.push(formatted_text);
    }

    //println!("fields: {:?}", fields);
    //println!("result: {:?}\n", result);

    result
}

pub fn format2(fields: &[&str]) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();

    for &field in fields {
        // Check if string contains only digits
        // Set the boolean b to true if the string field contains only characters in the range '0'..'9', false otherwise.
        if !field.contains_only_digits() {
            continue;
        }

        // https://stackoverflow.com/questions/30154541/how-do-i-concatenate-strings
        // 14 digits: exemplo CNPJ: 01234567000890 --> 01.234.567/0008-90
        // 11 digits: exemplo CPF: 12345678901     --> 123.456.789-01
        //  8 digits: exemplo NCM: 01234567        --> 0123.45.67

        let formatted_text: String = match field.len() {
            14 => [
                &field[0..2],
                ".",
                &field[2..5],
                ".",
                &field[5..8],
                "/",
                &field[8..12],
                "-",
                &field[12..],
            ]
            .concat(),
            11 => [
                &field[0..3],
                ".",
                &field[3..6],
                ".",
                &field[6..9],
                "-",
                &field[9..],
            ]
            .concat(),
            8 => [&field[0..4], ".", &field[4..6], ".", &field[6..]].concat(),
            _ => continue,
        };

        result.push(formatted_text);
    }

    //println!("fields: {:?}", fields);
    //println!("result: {:?}\n", result);

    result
}

pub fn format3(fields: &[&str]) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();

    for &field in fields {
        // https://stackoverflow.com/questions/30154541/how-do-i-concatenate-strings
        // 14 digits: exemplo CNPJ: 01234567000890 --> 01.234.567/0008-90
        // 11 digits: exemplo CPF: 12345678901     --> 123.456.789-01
        //  8 digits: exemplo NCM: 01234567        --> 0123.45.67

        if field.contains_num_digits(14) {
            result.push(
                [
                    &field[0..2],
                    ".",
                    &field[2..5],
                    ".",
                    &field[5..8],
                    "/",
                    &field[8..12],
                    "-",
                    &field[12..],
                ]
                .concat(),
            );
        } else if field.contains_num_digits(11) {
            result.push(
                [
                    &field[0..3],
                    ".",
                    &field[3..6],
                    ".",
                    &field[6..9],
                    "-",
                    &field[9..],
                ]
                .concat(),
            );
        } else if field.contains_num_digits(8) {
            result.push([&field[0..4], ".", &field[4..6], ".", &field[6..]].concat());
        }
    }

    //println!("fields: {:?}", fields);
    //println!("result: {:?}\n", result);

    result
}

fn criterion_benchmark(c: &mut Criterion) {
    // CNPJ, CPF ou NCM
    let fields = vec![
        "01234567000890",
        "12345",
        "12345678901",
        "01234567",
        "11111111111111",
        "123",
        "Text Ção 123 ",
        "88888888",
        "",
        " ",
        "12",
        "123456",
    ];

    let mut group = c.benchmark_group("Formatar Campos");

    group.warm_up_time(std::time::Duration::from_secs(10));
    group.measurement_time(std::time::Duration::from_secs(30));
    group.sample_size(5_000);

    group.bench_function("format1", |b| b.iter(|| format1(black_box(&fields))));
    group.bench_function("format2", |b| b.iter(|| format2(black_box(&fields))));
    group.bench_function("format3", |b| b.iter(|| format3(black_box(&fields))));

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
