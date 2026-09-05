use criterion::{Criterion, criterion_group, criterion_main};
use rand::distr::{Distribution, Uniform};
use std::{
    cmp::Ordering::{Greater, Less},
    collections::HashSet,
    hint::black_box,
    sync::LazyLock,
};

#[cfg(feature = "prefetch")]
use std::{
    arch::x86_64::{_MM_HINT_T0, _mm_prefetch},
    ptr,
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
criterion = { version = "0.8", features = ["html_reports"] }
itertools = "0.14"
plotters = "0.3"

[[bench]]
name = "binary_search"
harness = false

### --- ###

Create directory: benches
Create file: binary_search.rs
Put the file 'binary_search.rs' inside the dir 'benches'
See: cargo bench --help
Run: cargo bench --bench binary_search

### --- ###
*/

const CST_CREDITO: [u16; 14] = [50, 51, 52, 53, 54, 55, 56, 60, 61, 62, 63, 64, 65, 66];

const CST_CREDITO_EYTZINGER: [u16; 15] =
    [0, 60, 53, 64, 51, 55, 62, 66, 50, 52, 54, 56, 61, 63, 65];

/**
Perform a binary search on a sorted list.

Binary search is an efficient search algorithm that works on sorted lists or arrays.

It repeatedly divides the search interval in half, reducing the search space by half at each step, until the target value is found or the interval is empty.

By taking advantage of the sorted property of the input, binary search achieves a much better time complexity compared to linear search.

Its worst-case and average-case time complexity is O(log n), where n is the number of elements in the list.

```
    use efd_contribuicoes::binary_search_v2;

    let items = [1, 27, 49, 98, 210, 5432];

    let target1 = 28;
    let target2 = 210;
    let result1: Option<usize> = binary_search_v2(target1, &items);
    let result2: Option<usize> = binary_search_v2(target2, &items);
    assert!(result1.is_none());
    assert!(result2.is_some());
```
<https://doc.rust-lang.org/std/primitive.slice.html#method.binary_search>

<https://shane-o.dev/blog/binary-search-rust>
*/
pub fn binary_search_v2<T>(target: T, items: &[T]) -> Option<usize>
where
    T: Copy + Ord,
{
    let mut size = items.len();
    let mut left = 0;
    let mut right = size;

    while left < right {
        let middle: usize = left + size / 2;

        /*
        match items[middle].cmp(&target) {
            Ordering::Equal => return Some(middle),
            Ordering::Greater => right = middle,
            Ordering::Less => left = middle + 1,
        }
        */

        // SAFETY: the while condition means `size` is strictly positive, so
        // `size/2 < size`. Thus `left + size/2 < left + size`, which
        // coupled with the `left + size <= items.len()` invariant means
        // we have `left + size/2 < items.len()`, and this is in-bounds.

        let cmp = unsafe { items.get_unchecked(middle).cmp(&target) };
        //let cmp = items[middle].cmp(&target);

        // The reason why we use if/else control flow rather than match
        // is because match reorders comparison operations, which is perf sensitive.
        if cmp == Less {
            left = middle + 1;
        } else if cmp == Greater {
            right = middle;
        } else {
            return Some(middle);
        }

        size = right - left;
    }

    None
}

/// Fonte: https://github.com/bazhenov/eytzinger-layout
/// https://www.bazhenov.me/posts/faster-binary-search-in-rust
#[derive(Debug)]
pub struct Eytzinger(Vec<u16>);

impl From<&[u16]> for Eytzinger {
    fn from(input: &[u16]) -> Self {
        fn move_element(a: &[u16], b: &mut [u16], mut i: usize, k: usize) -> usize {
            if k <= a.len() {
                i = move_element(a, b, i, 2 * k);
                b[k] = a[i];
                i = move_element(a, b, i + 1, 2 * k + 1);
            }
            i
        }
        let mut result = vec![0; input.len() + 1];
        move_element(input, &mut result[..], 0, 1);
        Self(result)
    }
}

impl Eytzinger {
    /// Binary search over eytzinger layout array (branchless version)
    ///
    /// returns index of an element found or `0` is there is no match
    #[inline]
    pub fn binary_search_branchless(&self, target: u16) -> usize {
        let mut idx = 1;
        while idx < self.0.len() {
            #[cfg(feature = "prefetch")]
            unsafe {
                let prefetch = self.0.as_ptr().wrapping_offset(2 * idx as isize);
                _mm_prefetch::<_MM_HINT_T0>(ptr::addr_of!(prefetch) as *const i8);
            }
            let el = self.0[idx];
            idx = 2 * idx + usize::from(el < target);
        }
        idx >>= idx.trailing_ones() + 1;
        usize::from(self.0[idx] == target) * idx
    }
}

/**
Perform a binary search on a sorted list.

Binary search is an efficient search algorithm that works on sorted lists or arrays.

It repeatedly divides the search interval in half, reducing the search space by half at each step, until the target value is found or the interval is empty.

By taking advantage of the sorted property of the input, binary search achieves a much better time complexity compared to linear search.

Its worst-case and average-case time complexity is O(log n), where n is the number of elements in the list.

```
    use efd_contribuicoes::binary_search_v3;

    let items = [1, 27, 49, 98, 210, 5432];

    let target1 = 28;
    let target2 = 210;
    let result1: Option<usize> = binary_search_v3(target1, &items);
    let result2: Option<usize> = binary_search_v3(target2, &items);
    assert!(result1.is_none());
    assert!(result2.is_some());
```
<https://doc.rust-lang.org/std/primitive.slice.html#method.binary_search>

<https://www.bazhenov.me/posts/faster-binary-search-in-rust>
*/
pub fn binary_search_v3(target: u16) -> Option<usize> {
    let eytzinger = Eytzinger(CST_CREDITO_EYTZINGER.to_vec());
    let index = eytzinger.binary_search_branchless(target);
    if index > 0 { Some(index) } else { None }
}

// CST de Credito: [Some(50), ..., Some(56), Some(60), ..., Some(66)]
static HASH_DE_CREDITO: LazyLock<HashSet<Option<u16>>> = LazyLock::new(|| {
    let array_a: [Option<u16>; 7] = core::array::from_fn(|i| Some((50 + i) as u16));
    let array_b: [Option<u16>; 7] = core::array::from_fn(|i| Some((60 + i) as u16));

    // https://internals.rust-lang.org/t/pre-rfc-array-expansion-syntax/13490
    // https://stackoverflow.com/questions/29570607/is-there-a-good-way-to-convert-a-vect-to-an-array

    // Concat two arrays:
    let array: [Option<u16>; 14] = [array_a, array_b]
        .concat()
        .try_into()
        .expect("Error when trying to concatenate two arrays!");

    HashSet::from(array)
});

#[derive(Debug)]
struct Test {
    tipo_de_operacao: String,
    cst: Option<u16>,
    natureza_bc: Option<u16>,
}

impl Test {
    /// CST de crédito das Contribuições
    pub fn cst_de_credito_linear(&self) -> bool {
        (self.cst >= Some(50) && self.cst <= Some(56))
            || (self.cst >= Some(60) && self.cst <= Some(66))
    }

    pub fn cst_de_credito_contains(&self) -> bool {
        self.cst.is_some_and(|n| CST_CREDITO.contains(&n))
    }

    pub fn cst_de_credito_binary_search(&self) -> bool {
        self.cst
            .is_some_and(|n| CST_CREDITO.binary_search(&n).is_ok())
    }

    pub fn entrada_de_credito_binary_search_v1(&self) -> bool {
        self.tipo_de_operacao == "Entrada"
            && self.cst_de_credito_binary_search()
            && self.natureza_bc.is_some()
    }

    pub fn entrada_de_credito_binary_search_v2(&self) -> bool {
        self.tipo_de_operacao == "Entrada"
            && self
                .cst
                .and_then(|cst| binary_search_v2(cst, &CST_CREDITO))
                .is_some()
            && self.natureza_bc.is_some()
    }

    pub fn entrada_de_credito_binary_search_v3(&self) -> bool {
        self.tipo_de_operacao == "Entrada"
            && self.cst.and_then(binary_search_v3).is_some()
            && self.natureza_bc.is_some()
    }

    pub fn entrada_de_credito_hashset_search(&self) -> bool {
        self.tipo_de_operacao == "Entrada"
            && HASH_DE_CREDITO.contains(&self.cst)
            && self.natureza_bc.is_some()
    }

    pub fn entrada_de_credito_linear_search(&self) -> bool {
        self.tipo_de_operacao == "Entrada"
            && self.cst_de_credito_linear()
            && self.natureza_bc.is_some()
    }

    pub fn entrada_de_credito_contains_search(&self) -> bool {
        self.tipo_de_operacao == "Entrada"
            && self.cst_de_credito_contains()
            && self.natureza_bc.is_some()
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    // https://rust-lang-nursery.github.io/rust-cookbook/algorithms/randomness.html
    let mut rng = rand::rng();

    let cst_range1 = Uniform::new(0, 100).expect("rand error!");
    let cst_range2 = Uniform::new(49, 70).expect("rand error!");

    let nat_range1 = Uniform::new(0, 20).expect("rand error!");
    let nat_range2 = Uniform::new(0, 30).expect("rand error!");

    let mut lines: Vec<Test> = Vec::new();

    let sample_size = 10_000;

    for _ in 0..sample_size {
        let cst_v1 = cst_range1.sample(&mut rng);
        let cst_v2 = cst_range2.sample(&mut rng);

        let nat_v1 = nat_range1.sample(&mut rng);
        let nat_v2 = nat_range2.sample(&mut rng);

        let line01 = Test {
            tipo_de_operacao: "Entrada".to_string(),
            cst: Some(cst_v1),
            natureza_bc: Some(nat_v1),
        };

        let line02 = Test {
            tipo_de_operacao: "Entrada".to_string(),
            cst: Some(cst_v2),
            natureza_bc: Some(nat_v2),
        };

        lines.extend([line01, line02]);
    }

    println!("random lines: {lines:?}");
    println!("lines.len(): {}\n", lines.len());

    let eytzinger = Eytzinger::from(&CST_CREDITO[..]);
    println!("CST_CREDITO_EYTZINGER: {eytzinger:?}\n");

    let mut group = c.benchmark_group("Binary Search");

    group.warm_up_time(std::time::Duration::from_secs(10));
    group.measurement_time(std::time::Duration::from_secs(30));
    group.sample_size(2 * sample_size);

    group.bench_function("test_binary_search_v1", |b| {
        b.iter(|| {
            black_box(
                lines
                    .iter()
                    .map(|line| line.entrada_de_credito_binary_search_v1()),
            )
        })
    });
    group.bench_function("test_binary_search_v2", |b| {
        b.iter(|| {
            black_box(
                lines
                    .iter()
                    .map(|line| line.entrada_de_credito_binary_search_v2()),
            )
        })
    });
    group.bench_function("test_binary_search_v3", |b| {
        b.iter(|| {
            black_box(
                lines
                    .iter()
                    .map(|line| line.entrada_de_credito_binary_search_v3()),
            )
        })
    });
    group.bench_function("test_hashset_search", |b| {
        b.iter(|| {
            black_box(
                lines
                    .iter()
                    .map(|line| line.entrada_de_credito_hashset_search()),
            )
        })
    });
    group.bench_function("test_linear_search", |b| {
        b.iter(|| {
            black_box(
                lines
                    .iter()
                    .map(|line| line.entrada_de_credito_linear_search()),
            )
        })
    });
    group.bench_function("test_contains_search", |b| {
        b.iter(|| {
            black_box(
                lines
                    .iter()
                    .map(|line| line.entrada_de_credito_contains_search()),
            )
        })
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
