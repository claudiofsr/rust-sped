use claudiofsr_lib::OptionExtension;
use csv::StringRecord;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::serde_introspect;
use struct_iterable::Iterable;

use std::{
    collections::HashMap,
    hash::Hash,
    //cmp::Reverse,
    ops::{Add, AddAssign},
};

use tabled::{
    Table, Tabled,
    settings::{
        Alignment, Modify, Style,
        object::{Columns, Rows, Segment},
    },
};

use crate::{
    Despise, DocsFiscais, EFDResult, InfoExtension, MesesDoAno, display_cst, display_f64,
    display_mes, display_value, realizar_somas_trimestrais, serialize_cst,
    verificar_periodo_multiplo,
};

#[derive(Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Serialize, Deserialize)]
pub struct Keys {
    pub cnpj_base: String,
    pub ano: Option<i32>,
    trimestre: Option<u32>,
    pub mes: Option<MesesDoAno>,
    ordem: Option<u16>,
    pub cst: Option<u16>,
}

impl From<&DocsFiscais> for Keys {
    fn from(linha: &DocsFiscais) -> Self {
        Self {
            ano: linha.ano,
            trimestre: linha.trimestre,
            mes: linha.mes,
            cnpj_base: linha.get_cnpj_base(),
            ordem: get_ordem(linha.cst),
            cst: linha.cst,
        }
    }
}

#[derive(Debug, Default, PartialEq, PartialOrd, Copy, Clone, Serialize, Deserialize)]
struct Values {
    valor_item: Option<f64>,
    valor_bc: Option<f64>,
    valor_pis: Option<f64>,
    valor_cofins: Option<f64>,
}

impl From<&DocsFiscais> for Values {
    fn from(linha: &DocsFiscais) -> Self {
        Self {
            // Se for None, vira 0.0, e depois embrulhamos em Some() novamente.
            valor_item: Some(linha.valor_item.unwrap_or_default()),
            valor_bc: Some(linha.valor_bc.unwrap_or_default()),
            valor_pis: Some(linha.valor_pis.unwrap_or_default()),
            valor_cofins: Some(linha.valor_cofins.unwrap_or_default()),
        }
    }
}

impl Add for Values {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            valor_item: self.valor_item.combine_with_sum(other.valor_item),
            valor_bc: self.valor_bc.combine_with_sum(other.valor_bc),
            valor_pis: self.valor_pis.combine_with_sum(other.valor_pis),
            valor_cofins: self.valor_cofins.combine_with_sum(other.valor_cofins),
        }
    }
}

/// Executar a operação +=
///
/// <https://doc.rust-lang.org/std/ops/trait.AddAssign.html>
impl AddAssign for Values {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            valor_item: self.valor_item.combine_with_sum(other.valor_item),
            valor_bc: self.valor_bc.combine_with_sum(other.valor_bc),
            valor_pis: self.valor_pis.combine_with_sum(other.valor_pis),
            valor_cofins: self.valor_cofins.combine_with_sum(other.valor_cofins),
        };
    }
}

/// Consolidação CST
#[derive(
    Debug, Default, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Tabled, Iterable,
)]
pub struct ConsolidacaoCST {
    #[serde(rename = "CNPJ Base")]
    #[tabled(rename = "CNPJ Base")]
    pub cnpj_base: String,
    #[serde(rename = "Ano do Período de Apuração")]
    #[tabled(rename = "Ano", display = "display_value")]
    pub ano: Option<i32>,
    #[serde(rename = "Trimestre do Período de Apuração")]
    #[tabled(rename = "Trim", display = "display_value")]
    pub trimestre: Option<u32>,
    #[serde(rename = "Mês do Período de Apuração")]
    #[tabled(rename = "Mês", display = "display_mes")]
    pub mes: Option<MesesDoAno>,
    #[serde(
        rename = "Código de Situação Tributária (CST)",
        serialize_with = "serialize_cst"
    )]
    #[tabled(rename = "CST", display = "display_cst")]
    pub cst: Option<u16>,

    #[serde(rename = "Valor Total do Item")] // serialize_with = "serialize_f64"
    #[tabled(rename = "Valor Total do Item", display = "display_f64")]
    pub valor_item: Option<f64>,
    #[serde(rename = "Base de Cálculo")]
    #[tabled(rename = "Base de Cálculo", display = "display_f64")]
    pub valor_bc: Option<f64>,
    #[serde(rename = "Valor de PIS/PASEP")]
    #[tabled(rename = "Valor de PIS/PASEP", display = "display_f64")]
    pub valor_pis: Option<f64>,
    #[serde(rename = "Valor de COFINS")]
    #[tabled(rename = "Valor de COFINS", display = "display_f64")]
    pub valor_cofins: Option<f64>,
}

/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl InfoExtension for ConsolidacaoCST {}

impl ConsolidacaoCST {
    pub fn get_headers() -> StringRecord {
        // use serde_aux::prelude::serde_introspect;
        let colunas_vec = serde_introspect::<ConsolidacaoCST>();
        StringRecord::from(colunas_vec)
    }
}

pub fn consolidar_operacoes_por_cst(
    linhas: &[DocsFiscais],
) -> EFDResult<(String, Vec<ConsolidacaoCST>)> {
    let mut resultado: HashMap<Keys, Values> = consolidar_keys(linhas);
    realizar_soma_parcial(&mut resultado);

    let periodo_multiplo = verificar_periodo_multiplo(&resultado);

    if periodo_multiplo {
        realizar_somas_trimestrais(&mut resultado);
    }

    let resultado_ordenado: Vec<(Keys, Values)> = ordenar_cst(resultado);
    let resultado_estruturado = gerar_keysvalues(&resultado_ordenado);
    let tabela_de_resultado = gerar_tabela_cst(&resultado_estruturado, 5);

    Ok((tabela_de_resultado, resultado_estruturado))
}

fn consolidar_keys(linhas: &[DocsFiscais]) -> HashMap<Keys, Values> {
    let map_reduce: HashMap<Keys, Values> = linhas
        .into_par_iter() // rayon: parallel iterator
        .filter(|&linha| linha.operacoes_de_entrada_ou_saida()) // 1: Entrada, 2: Saída
        .map(get_keys_values)
        .fold(HashMap::new, |mut hashmap_accumulator, (key, value)| {
            // impl Add and AddAssign for Values
            hashmap_accumulator
                .entry(key)
                .and_modify(|previous_value| *previous_value += value)
                .or_insert(value);

            hashmap_accumulator
        })
        .reduce(HashMap::new, |mut hashmap_a, hashmap_b| {
            hashmap_b.into_iter().for_each(|(key_b, value_b)| {
                // impl Add and AddAssign for Values
                hashmap_a
                    .entry(key_b)
                    .and_modify(|previous_value| *previous_value += value_b)
                    .or_insert(value_b);
            });

            hashmap_a
        });

    map_reduce
}

fn get_keys_values(linha: &DocsFiscais) -> (Keys, Values) {
    // (Keys::from(linha), Values::from(linha))
    (linha.into(), linha.into())
}

fn get_ordem(cst: Option<u16>) -> Option<u16> {
    match cst {
        Some(1..=49) => Some(1),  // Saídas:   1  <= cst <= 49
        Some(50..=98) => Some(3), // Entradas: 50 <= cst <= 98
        Some(99) => Some(5),
        _ => None,
    }
}

fn realizar_soma_parcial(resultado: &mut HashMap<Keys, Values>) {
    let mut soma_parcial: HashMap<Keys, Values> = HashMap::new();

    for (chaves, &values) in resultado.iter() {
        let mut keys = chaves.clone();

        match chaves.cst {
            Some(1..=49) => {
                // Saídas: 1 <= cst <= 49
                keys.cst = Some(490); // cst temporário, valor qualquer acima de 100
                keys.ordem = Some(2);
            }
            Some(50..=98) => {
                // Entradas: 50 <= cst <= 98
                keys.cst = Some(980); // cst temporário, valor qualquer acima de 100
                keys.ordem = Some(4);
            }
            _ => continue,
        };

        // impl Add and AddAssign for Values
        soma_parcial
            .entry(keys)
            .and_modify(|previous_value| *previous_value += values)
            .or_insert(values);
    }

    // Merge two HashMaps in Rust
    resultado.extend(soma_parcial);
}

fn ordenar_cst(hmap: HashMap<Keys, Values>) -> Vec<(Keys, Values)> {
    // transform hashmap to vec
    let mut vec_from_hash: Vec<(Keys, Values)> = hmap.into_iter().collect();

    vec_from_hash.sort_by_key(|(chaves, _valores)| {
        (
            chaves.cnpj_base.clone(),
            chaves.ano,
            chaves.trimestre,
            chaves.mes,
            chaves.ordem,
            chaves.cst,
        )
    });

    vec_from_hash
}

fn gerar_keysvalues(info_ordenada: &[(Keys, Values)]) -> Vec<ConsolidacaoCST> {
    let mut lines: Vec<ConsolidacaoCST> = Vec::new();

    for (chaves, valores) in info_ordenada {
        let mut line = ConsolidacaoCST {
            cnpj_base: chaves.cnpj_base.clone(),
            ano: chaves.ano,
            trimestre: chaves.trimestre,
            mes: chaves.mes,
            cst: chaves.cst,
            valor_item: valores.valor_item,
            valor_bc: valores.valor_bc,
            valor_pis: valores.valor_pis,
            valor_cofins: valores.valor_cofins,
        };

        line.despise_small_values();

        lines.push(line);
    }

    lines
}

fn gerar_tabela_cst<T: Tabled>(lines: &[T], first: usize) -> String {
    // https://crates.io/crates/tabled
    Table::new(lines)
        .with(Modify::new(Segment::all()).with(Alignment::right()))
        .with(Modify::new(Columns::new(0..first)).with(Alignment::center()))
        .with(Modify::new(Rows::one(0)).with(Alignment::center()))
        //.with(Modify::new(Rows::one(0)).with(Format::new(|s| s.blue().to_string())))
        //.with(Modify::new(Rows::new(..)).with(Format::new(|s| s.blue().to_string())))
        .with(Style::rounded())
        .to_string()
}

#[cfg(test)]
mod tests {
    // cargo test -- --help
    // cargo test -- --nocapture
    // cargo test -- --show-output
    use super::*;

    #[test]
    fn operacoes_com_vetores() {
        // cargo test -- --show-output operacoes_com_vetores

        let mut vec_a = vec![1, 3, 5];
        println!("vec_a: {vec_a:?}");

        let vec_b = vec![2, 3, 4, 1];
        println!("vec_b: {vec_b:?}\n");

        vec_a.extend(vec_b);
        println!("vec_a: {vec_a:?} extend");

        vec_a.sort_unstable();
        println!("vec_a: {vec_a:?} sort");

        vec_a.dedup();
        println!("vec_a: {vec_a:?} dedup");

        assert_eq!(vec_a, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn desprezar_valores_pequenos() {
        let mut line_a = ConsolidacaoCST {
            cnpj_base: "12.345.678".to_string(),
            ano: Some(2022),
            trimestre: Some(1),
            mes: Some(MesesDoAno::Setembro),
            cst: Some(56),
            valor_item: Some(0.00499), // <-- desprezar este valor, pois menor que SMALL_VALUE
            valor_bc: Some(27.56),
            valor_pis: Some(-0.002), // <-- desprezar este valor, pois menor que SMALL_VALUE
            valor_cofins: Some(0.0091),
        };

        println!("line_a: {line_a:?}");

        line_a.despise_small_values();

        let line_b = ConsolidacaoCST {
            cnpj_base: "12.345.678".to_string(),
            ano: Some(2022),
            trimestre: Some(1),
            mes: Some(MesesDoAno::Setembro),
            cst: Some(56),
            valor_item: None,
            valor_bc: Some(27.56),
            valor_pis: None,
            valor_cofins: Some(0.0091),
        };

        println!("line_a: {line_a:?}");
        println!("line_b: {line_b:?}");

        assert_eq!(line_a, line_b);
    }
}
