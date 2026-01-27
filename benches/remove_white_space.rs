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
criterion = { version = "0.8", features = ["html_reports"] }
itertools = "0.14"
plotters = "0.3"

[[bench]]
name = "remove_white_space"
harness = false

### --- ###

Create directory: benches
Create file: remove_white_space.rs
Put the file 'remove_white_space.rs' inside the dir 'benches'
See: cargo bench --help
Run: cargo bench --bench remove_white_space

### --- ###
*/

// --- Implementação das Engines para o Bench ---

#[inline]
fn engine_byte_push(s: &str) -> String {
    // Fast Path: Se não houver espaços duplos, retorna uma cópia imediata.
    // O contains() usa SIMD internamente, sendo extremamente veloz.
    if !s.contains("  ") {
        return s.to_string();
    }
    let bytes = s.as_bytes();
    let mut res = String::with_capacity(s.len());
    let mut last_was_space = false;
    for &byte in bytes {
        if byte == b' ' {
            if !last_was_space {
                res.push(' ');
                last_was_space = true;
            }
        } else {
            res.push(byte as char);
            last_was_space = false;
        }
    }
    res
}

#[inline]
fn engine_slice_push(s: &str) -> String {
    if !s.contains("  ") {
        return s.to_string();
    }
    let bytes = s.as_bytes();
    let mut builder = String::with_capacity(s.len());
    let mut start = 0;
    let mut last_was_space = false;

    for (i, &byte) in bytes.iter().enumerate() {
        if byte == b' ' {
            if last_was_space {
                if i > start {
                    builder.push_str(&s[start..i]);
                }
                start = i + 1;
            } else {
                last_was_space = true;
            }
        } else {
            last_was_space = false;
        }
    }
    if start < bytes.len() {
        builder.push_str(&s[start..]);
    }
    builder
}

#[inline]
pub fn without_iter_enumerate(s: &str) -> String {
    if !s.contains("  ") {
        return s.to_string();
    }
    let bytes = s.as_bytes();
    let mut builder = String::with_capacity(s.len());
    let mut start = 0;
    let mut last_was_space = false;

    for i in 0..bytes.len() {
        let byte = bytes[i];
        if byte == b' ' {
            if last_was_space {
                if i > start {
                    builder.push_str(&s[start..i]);
                }
                start = i + 1;
            } else {
                last_was_space = true;
            }
        } else {
            last_was_space = false;
        }
    }
    if start < bytes.len() {
        builder.push_str(&s[start..]);
    }
    builder
}

fn remove_multiple_whitespace_retain(s: &str) -> String {
    if !s.contains("  ") {
        return s.to_string();
    }

    let mut new_str: String = s.to_owned();
    let mut previous_char: char = 'x'; // some non-whitespace character
    new_str.retain(|current_char| {
        let keep: bool = previous_char != ' ' || current_char != ' ';
        previous_char = current_char;
        keep
    });
    new_str
}

#[inline]
fn collapse_multiple_spaces_branchless(s: &str) -> String {
    // 1. FAST PATH (SIMD)
    // O compilador otimiza o .contains("  ") para instruções SIMD (AVX2/SSE).
    // Em arquivos SPED, a maioria dos campos já vem limpa, então o retorno é instantâneo.
    if !s.contains("  ") {
        return s.to_string();
    }

    let bytes = s.as_bytes();
    let len = bytes.len();

    // 2. PRE-ALOCAÇÃO INTELIGENTE
    // Usamos o tamanho original.
    // Como vamos reduzir espaços, o buffer é garantidamente suficiente.
    let mut out = vec![0u8; len];
    let mut write_idx = 0;
    let mut last_was_space = false;

    for &byte in bytes {
        let is_space = byte == b' ';

        // Sempre escrevemos o byte na posição atual (evita branch).
        // Se for espaço multiplo, o próximo caractere válido irá sobrescrever esta posição.
        out[write_idx] = byte;

        // Só avançamos o ponteiro de escrita se NÃO for um espaço multiplo.
        // O cast 'as usize' converte true -> 1 e false -> 0 sem saltos de CPU.
        write_idx += !(is_space & last_was_space) as usize;

        // Atualiza o estado para a próxima iteração
        last_was_space = is_space;
    }

    // Converte de volta para String.
    // Como o input original era UTF-8 e só removemos bytes ASCII (espaço),
    // o resultado é garantidamente UTF-8 válido.
    let result_str = unsafe { std::str::from_utf8_unchecked(&out[..write_idx]) };
    result_str.to_string()
}

#[inline]
pub fn collapse_multiple_spaces_ultra(s: &str) -> String {
    // 1. SIMD JUMP (Fast Path)
    // O find("  ") utiliza otimizações de hardware para saltar rapidamente
    // a parte da string que já está limpa.
    let first_gap = match s.find("  ") {
        Some(idx) => idx,
        None => return s.to_string(), // Retorno imediato se não houver espaços duplos
    };

    // 2. PRE-ALOCAÇÃO ÚNICA
    // Alocamos a capacidade total original para evitar qualquer realocação durante o processo.
    let mut builder = String::with_capacity(s.len());

    // 3. COPIA O INÍCIO LIMPO
    // Copiamos até o primeiro espaço do "gap" (incluindo ele).
    // Ex: "A  B" -> copia "A " e o próximo caractere será o espaço extra a ser ignorado.
    builder.push_str(&s[..first_gap + 1]);

    let bytes = s.as_bytes();
    let mut start = first_gap + 1;
    let mut last_was_space = true;

    // 4. LOOP DE ALTA PERFORMANCE (Slice-Pushing)
    // Varremos os bytes, mas damos "push" em fatias (&str).
    for i in (first_gap + 1)..bytes.len() {
        let is_space = bytes[i] == b' ';

        if is_space && last_was_space {
            // Se detectarmos um espaço redundante:
            // 1. Movemos o texto acumulado até este ponto para o builder.
            if i > start {
                builder.push_str(&s[start..i]);
            }
            // 2. O 'start' salta este espaço extra.
            start = i + 1;
        }
        last_was_space = is_space;
    }

    // 5. FLUSH FINAL
    // Adiciona o restante da string que sobrou após o último espaço duplo.
    if start < bytes.len() {
        builder.push_str(&s[start..]);
    }

    builder
}

// --- Configuração do Benchmark ---

fn benchmark_spaces(c: &mut Criterion) {
    let input = "Este é um exemplo de string  程式   设计    com muitos    espaços para testar   eficiência.  ";

    let mut group = c.benchmark_group("Collapse Spaces");

    group.warm_up_time(std::time::Duration::from_secs(5));
    group.measurement_time(std::time::Duration::from_secs(20));
    group.sample_size(5_000);

    group.bench_function("Engine_Byte_Pushing", |b| {
        b.iter(|| engine_byte_push(black_box(input)))
    });

    group.bench_function("Engine_Slice_Pushing", |b| {
        b.iter(|| engine_slice_push(black_box(input)))
    });

    group.bench_function("Without .iter().enumerate()", |b| {
        b.iter(|| without_iter_enumerate(black_box(input)))
    });

    group.bench_function("Retain_Method", |b| {
        b.iter(|| remove_multiple_whitespace_retain(black_box(input)))
    });

    group.bench_function("Branchless (sem saltos)", |b| {
        b.iter(|| collapse_multiple_spaces_branchless(black_box(input)))
    });

    group.bench_function("Collapse Ultra", |b| {
        b.iter(|| collapse_multiple_spaces_ultra(black_box(input)))
    });

    group.finish();
}

criterion_group!(benches, benchmark_spaces);
criterion_main!(benches);
