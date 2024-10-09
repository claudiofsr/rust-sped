use std::collections::{HashMap, HashSet};

use crate::{
    structures::analise_dos_creditos::Chaves,
    structures::consolidacao_cst::Keys,
    AnaliseDosCreditos, ConsolidacaoCST,
    SMALL_VALUE,
};

// --- Start: Definir traits para Ano, Mes, CST e CNPJBase ---
pub trait Ano {
    fn get_ano(&self) -> Option<i32>;
}

pub trait Mes {
    fn get_mes(&self) -> Option<u32>;
    fn set_mes(&mut self, m: Option<u32>);
}

pub trait Cst {
    fn get_cst(&self) -> Option<u16>;
}

pub trait CNPJBase {
    fn get_cnpj_base(&self) -> String;
}
// --- Final: Definir traits para Ano, Mes, CST e CNPJBase ---

// --- Start: Impl traits to Chaves ---
impl Ano for Chaves {
    fn get_ano(&self) -> Option<i32> {
        self.ano
    }
}

impl Mes for Chaves {
    fn get_mes(&self) -> Option<u32> {
        self.mes
    }
    fn set_mes(&mut self, m: Option<u32>) {
        self.mes = m;
    }
}

impl Cst for Chaves {
    fn get_cst(&self) -> Option<u16> {
        self.cst
    }
}

impl CNPJBase for Chaves {
    fn get_cnpj_base(&self) -> String {
        self.cnpj_base.clone()
    }
}
// --- Final: Impl traits to Chaves ---

// --- Start: Impl traits to keys ---
impl Ano for Keys {
    fn get_ano(&self) -> Option<i32> {
        self.ano
    }
}

impl Mes for Keys {
    fn get_mes(&self) -> Option<u32> {
        self.mes
    }
    fn set_mes(&mut self, m: Option<u32>) {
        self.mes = m;
    }
}

impl Cst for Keys {
    fn get_cst(&self) -> Option<u16> {
        self.cst
    }
}

impl CNPJBase for Keys {
    fn get_cnpj_base(&self) -> String {
        self.cnpj_base.clone()
    }
}
// --- Final: Impl traits to keys ---

pub fn verificar_periodo_multiplo<T, U>(resultado: &HashMap<T, U>) -> bool
where
    T: Ano + Mes + Cst + CNPJBase,
{

    let mut cnpjs: HashSet<String> = HashSet::new();
    let mut hashmap: HashMap<String, HashSet<u32>> = HashMap::new();

    // encontrar todos os CNPJs distintos.
    for chave in resultado.keys() {
        cnpjs.insert(chave.get_cnpj_base());
    }

    for cnpj in cnpjs {

        let mut hashset: HashSet<u32> = HashSet::new();

        for chave in resultado.keys() {

            if cnpj != chave.get_cnpj_base() || chave.get_cst().is_none() {
                continue;
            }

            let ano = chave.get_ano(); // 2022
            let mes = chave.get_mes(); // 04
            let ano_mes = ano.and_then(|a: i32| Some((a as u32) * 100 + mes?)); // 202200 + 4 = 202204

            if !(mes >= Some(1) && mes <= Some(12)) {
                continue;
            }

            hashset.insert(ano_mes.unwrap());
        }

        hashmap.insert(cnpj, hashset);
    }

    let mut periodo_multiplo = false;

    for set in hashmap.values() {
        if set.len() > 1 {
            periodo_multiplo = true;
            break;
        }
    }

    //println!("hashmap: {hashmap:#?}");
    //println!("periodo_multiplo: {periodo_multiplo}");

    periodo_multiplo
}

/*
https://stackoverflow.com/questions/73680402/how-to-implement-iterator-for-array-optionf64-n-with-n-elements
https://practice.rs/generics-traits/const-generics.html
https://github.com/sunface/rust-by-practice/blob/master/solutions/generics-traits/const-generics.md
https://stackoverflow.com/questions/37410672/expected-type-parameter-found-u8-but-the-type-parameter-is-u8
https://saveriomiroddi.github.io/Rust-lulz-implementing_floating_point_approximate_equality_via_traits/

I have several structures with N (can be distinct) fields of type Option<f64>.
I want with a single function to evaluate whether or not I keep small values (values ​​< SMALL_VALUE = 0.005) ​​of these fields.
For this, I must implement an iterator that captures only the desired N fields from the structures.
 */

// --- AllValues --- //

/// Trait for types that have all their values as `Option<f64>`.
pub trait AllValues {
    /// Returns a vector of references to the `Option<f64>` values.
    fn get_all_values(&mut self) -> Vec<&mut Option<f64>>;
}

impl AllValues for ConsolidacaoCST {
    fn get_all_values(&mut self) -> Vec<&mut Option<f64>> {
        vec![
            &mut self.valor_item,
            &mut self.valor_bc,
            &mut self.valor_pis,
            &mut self.valor_cofins,
        ]
    }
}

impl AllValues for AnaliseDosCreditos {
    fn get_all_values(&mut self) -> Vec<&mut Option<f64>> {
        vec![
            &mut self.valor_bc,
            &mut self.valor_rbnc_trib,
            &mut self.valor_rbnc_ntrib,
            &mut self.valor_rbnc_exp,
            &mut self.valor_rb_cum
        ]
    }
}

/// Sets all values in `self` to `None` if their absolute value is less than `SMALL_VALUE`.
///
/// <https://stackoverflow.com/questions/73680402/how-to-implement-iterator-for-array-optionf64-n-with-n-elements>
pub fn despise_small_values<T: AllValues>(info: &mut T) {
    for value in info.get_all_values() {
        if let Some(v) = value {
            if v.abs() < SMALL_VALUE {
                *value = None;
            }
        }
    }
}
