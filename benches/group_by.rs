use rand::{Rng, distr::Uniform};
use rayon::prelude::*;
use rust_decimal_macros::dec;
use std::{collections::HashMap, hint::black_box, sync::Mutex};

use criterion::{Criterion, criterion_group, criterion_main};

use efd_contribuicoes::{
    Chaves, CodigoSituacaoTributaria, DocsFiscais, MesesDoAno, NaturezaBaseCalculo, TipoDeCredito,
    TipoDeOperacao, Valores, obter_chaves_valores,
};

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
criterion = { version = "0.4", features = ["html_reports"] }
itertools = "0.12"
plotters = "0.3"

[[bench]]
name = "group_by"
harness = false

### --- ###

Create directory: benches
Create file: group_by.rs
Put the file 'group_by.rs' inside the dir 'benches'
See: cargo bench --help
Run: cargo bench --bench group_by

### --- ###
*/

/// Group By Sequencial Mode
fn consolidar_chaves_v1(lines: &[DocsFiscais]) -> HashMap<Chaves, Valores> {
    let mut hashmap: HashMap<Chaves, Valores> = HashMap::new();
    lines
        .iter()
        .filter(|&line| line.entrada_de_credito() || line.saida_de_receita_bruta())
        .map(obter_chaves_valores)
        .for_each(|(key, value)| {
            // impl Add and AddAssign for Valores: Soma de Valores
            hashmap
                .entry(key)
                .and_modify(|previous_value| *previous_value += value)
                .or_insert(value);
        });
    hashmap
}

/// Group By Parallel Mode
///
/// Rayon with a mutex to ensure that the HashMap is not
/// accessed by multiple threads at the same time.
fn consolidar_chaves_v2(lines: &[DocsFiscais]) -> HashMap<Chaves, Valores> {
    let hashmap = Mutex::new(HashMap::new());
    lines
        .par_iter()
        .filter(|&line| line.entrada_de_credito() || line.saida_de_receita_bruta())
        .map(obter_chaves_valores)
        .for_each(|(chaves, valores)| {
            hashmap
                .lock()
                .unwrap()
                .entry(chaves)
                .and_modify(|previous_value| *previous_value += valores)
                .or_insert(valores);
        });
    hashmap.into_inner().unwrap()
}

/**
Group By Parallel Mode

Consolidar Chaves (group_by)

Método adotado: MapReduce.

MapReduce é um modelo de programação desenhado para processar grandes volumes de
dados em paralelo, dividindo o trabalho em um conjunto de tarefas independentes.

The parallel fold first breaks up your list into sublists, and hence yields up
multiple HashMaps.

Fold versus reduce

The fold() and reduce() methods each take an identity element and a combining function,
but they operate rather differently.

When you use reduce(), your reduction function is sometimes called with values that were
never part of your original parallel iterator (for example, both the left and right might
be a partial sum).

With fold(), in contrast, the left value in the fold function is always the accumulator,
and the right value is always from your original sequence.

Now fold will process groups of hashmap, and we only make one hashmap per group.
We should wind up with some hashmap number roughly proportional to the number of CPUs you have
(it will ultimately depend on how busy your processors are).

Note that we still need to do a reduce afterwards to combine those groups of hashmaps
into a single hashmap.

<https://stackoverflow.com/questions/57641821/rayon-fold-into-a-hashmap>
*/
fn consolidar_chaves_v3(lines: &[DocsFiscais]) -> HashMap<Chaves, Valores> {
    let map_reduce: HashMap<Chaves, Valores> = lines
        .into_par_iter() // rayon: parallel iterator
        .filter(|&line| line.entrada_de_credito() || line.saida_de_receita_bruta())
        .map(obter_chaves_valores)
        .fold(HashMap::new, |mut hashmap_accumulator, (key, value)| {
            // impl Add and AddAssign for Valores: Soma de Valores
            hashmap_accumulator
                .entry(key)
                .and_modify(|previous_value| *previous_value += value)
                .or_insert(value);

            hashmap_accumulator
        })
        .reduce(HashMap::new, |mut hashmap_a, hashmap_b| {
            hashmap_b.into_iter().for_each(|(key_b, value_b)| {
                // impl Add and AddAssign for Valores: Soma de Valores
                hashmap_a
                    .entry(key_b)
                    .and_modify(|previous_value| *previous_value += value_b)
                    .or_insert(value_b);
            });

            hashmap_a
        });

    map_reduce
}

/// Map Reduce
///
/// Group By Parallel Mode com Rayon
///
/// https://doc.rust-lang.org/stable/rust-by-example/std_misc/threads/testcase_mapreduce.html
fn consolidar_chaves_v4(lines: &[DocsFiscais]) -> HashMap<Chaves, Valores> {
    lines
        .par_iter() // 1. Itera em paralelo (Rayon gerencia os chunks automaticamente)
        .filter(|line| line.entrada_de_credito() || line.saida_de_receita_bruta())
        .map(obter_chaves_valores)
        // 2. FOLD: Cria um HashMap local para cada thread/worker.
        // Isso evita contenção de memória e minimiza acessos globais.
        .fold(
            HashMap::new, // Inicializador para cada thread
            |mut acc, (key, value)| {
                acc.entry(key)
                    .and_modify(|previous_value| *previous_value += value)
                    .or_insert(value);
                acc
            },
        )
        // 3. REDUCE: Combina os HashMaps produzidos pelas threads em paralelo.
        // O Rayon faz isso de forma hierárquica (log n), muito mais rápido que o loop sequencial.
        .reduce(HashMap::new, |h1, h2| {
            // Mesclamos o mapa menor no maior para eficiência
            let (mut big, small) = if h1.len() >= h2.len() {
                (h1, h2)
            } else {
                (h2, h1)
            };

            for (key, value) in small {
                big.entry(key)
                    .and_modify(|previous_value| *previous_value += value)
                    .or_insert(value);
            }
            big
        })
}

fn criterion_benchmark(c: &mut Criterion) {
    // https://rust-lang-nursery.github.io/rust-cookbook/algorithms/randomness.html
    let mut rng = rand::rng();
    let range_cst = Uniform::new(40, 70).expect("rand error!"); // 14 cst válidos
    let range_nat = Uniform::new(0, 20).expect("rand error!"); // 18 natureza_bc válidos

    let mut data: Vec<Vec<DocsFiscais>> = Vec::new();

    // Pick a random element from a list
    // https://programming-idioms.org/idiom/11/pick-a-random-element-from-a-list
    let list = [
        10_000, 20_000, 50_000, 80_000, 100_000, 150_000, 200_000, 400_000, 500_000, 800_000,
    ];
    let choice = list[rng.random_range(0..list.len())];
    println!("choice: {choice}");

    for number_of_lines in list {
        let mut all_lines: Vec<DocsFiscais> = Vec::new();

        for index in 1..=number_of_lines {
            let cst = rng.sample(range_cst);
            let nat = rng.sample(range_nat);

            let mut colunas = DocsFiscais {
                ..Default::default()
            };

            colunas.linhas = index;
            colunas.estabelecimento_cnpj = "01234567000890".into();
            colunas.ano = Some(2023);
            colunas.trimestre = Some(4);
            colunas.mes = Some(MesesDoAno::Dezembro);
            colunas.tipo_de_credito = Some(TipoDeCredito::AliquotaBasica);
            colunas.cst = CodigoSituacaoTributaria::from_u16(cst as u16);
            colunas.aliq_pis = Some(dec!(1.65));
            colunas.aliq_cofins = Some(dec!(7.60));
            colunas.tipo_de_operacao = Some(TipoDeOperacao::Entrada);
            colunas.natureza_bc = NaturezaBaseCalculo::from_u16(nat as u16);
            colunas.cod_ncm = "01234567".into();
            colunas.valor_item = Some(dec!(20.0000));
            colunas.valor_bc = Some(dec!(10.0000));

            all_lines.push(colunas);
        }

        let chaves_consolidadas_v1: HashMap<Chaves, Valores> = consolidar_chaves_v1(&all_lines);
        let chaves_consolidadas_v2: HashMap<Chaves, Valores> = consolidar_chaves_v2(&all_lines);
        let chaves_consolidadas_v3: HashMap<Chaves, Valores> = consolidar_chaves_v3(&all_lines);
        let chaves_consolidadas_v4: HashMap<Chaves, Valores> = consolidar_chaves_v3(&all_lines);

        assert_eq!(chaves_consolidadas_v1, chaves_consolidadas_v2); // If a = b and a = c, then b = c.
        assert_eq!(chaves_consolidadas_v1, chaves_consolidadas_v3);
        assert_eq!(chaves_consolidadas_v1, chaves_consolidadas_v4);

        if number_of_lines == 500_000 {
            // max chaves_consolidadas_v1.len() = 18 * 14 = 252
            println!(
                "chaves_consolidadas_v1: {chaves_consolidadas_v1:#?} ; len: {}\n",
                chaves_consolidadas_v1.len()
            );
        }

        data.push(all_lines);
    }

    let mut group = c.benchmark_group("GroupBy MapReduce");

    group.warm_up_time(std::time::Duration::from_secs(10));
    group.measurement_time(std::time::Duration::from_secs(30));
    group.sample_size(5_000);

    group.bench_function("consolidar_chaves_v1", |b| {
        b.iter(|| black_box(data.iter().map(|all_lines| consolidar_chaves_v1(all_lines))))
    });
    group.bench_function("consolidar_chaves_v2", |b| {
        b.iter(|| black_box(data.iter().map(|all_lines| consolidar_chaves_v2(all_lines))))
    });
    group.bench_function("consolidar_chaves_v3", |b| {
        b.iter(|| black_box(data.iter().map(|all_lines| consolidar_chaves_v3(all_lines))))
    });
    group.bench_function("consolidar_chaves_v4", |b| {
        b.iter(|| black_box(data.iter().map(|all_lines| consolidar_chaves_v4(all_lines))))
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
