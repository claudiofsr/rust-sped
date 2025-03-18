use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    path::Path,
};

/**
Tributos constantes da SPED EFD Contribuições analisados:

Contribuições de PIS/PASEP e COFINS
 */
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Hash, Serialize, Deserialize)]
pub enum Tributos {
    #[serde(rename = "PIS/PASEP")]
    Pis,
    #[serde(rename = "COFINS")]
    Cofins,
}

// https://docs.rs/serde/latest/serde/ser/trait.Serializer.html#method.collect_str
impl fmt::Display for Tributos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.serialize(f)
    }
}

#[derive(Debug, Default)]
pub struct Info {
    pub messages: String,
    pub pa: Option<NaiveDate>, // Período de Apuração conforme EFD
    pub cnpj_base: u32,
    pub cnpj_do_estabelecimento: String,
    pub linhas_inseridas: Vec<usize>,
    pub global: HashMap<String, String>,
    pub estabelecimentos: HashMap<String, String>,
    pub participantes: BTreeMap<String, HashMap<String, String>>,
    pub produtos: BTreeMap<String, HashMap<String, String>>,
    pub unidade_de_medida: HashMap<String, String>,
    pub nat_operacao: HashMap<String, String>,
    pub complementar: HashMap<String, String>,
    pub contabil: HashMap<String, HashMap<String, String>>,
    pub nome_do_cnpj: BTreeMap<String, String>,
    pub nome_do_cpf: BTreeMap<String, String>,
    pub correlacao: HashMap<String, [String; 2]>, // arrays [type; number] are fastest than vectors
    pub reg_a100: HashMap<String, String>,
    pub reg_c100: HashMap<String, String>,
    pub reg_c180: HashMap<String, String>,
    pub reg_c190: HashMap<String, String>,
    pub reg_c380: HashMap<String, String>,
    pub reg_c395: HashMap<String, String>,
    pub reg_c400: HashMap<String, String>,
    pub reg_c405: HashMap<String, String>,
    pub reg_c490: HashMap<String, String>,
    pub reg_c500: HashMap<String, String>,
    pub reg_c600: HashMap<String, String>,
    pub reg_c860: HashMap<String, String>,
    pub reg_d100: HashMap<String, String>,
    pub reg_d200: HashMap<String, String>,
    pub reg_d500: HashMap<String, String>,
    pub reg_d600: HashMap<String, String>,
    pub reg_m100: HashMap<String, String>,
    pub reg_m500: HashMap<String, String>,
    pub completa: BTreeMap<usize, HashMap<String, String>>,
}

impl Info {
    /// Get new value
    pub fn new(arquivo: &Path) -> Info {
        let mut info = Info::default();
        info.global
            .insert("arquivo_efd".to_string(), arquivo.display().to_string());

        info
    }

    /// Selecionar o nome mais frequente para um dado cnpj
    pub fn obter_nome_do_cnpj_base(&self) -> HashMap<String, String> {
        get_the_most_frequent_value(&self.nome_do_cnpj)
    }
}

pub fn get_the_most_frequent_value(
    nome_do_cnpj: &BTreeMap<String, String>,
) -> HashMap<String, String> {
    let mut frequencia: HashMap<String, Vec<String>> = HashMap::new();

    for (cnpj, nome) in nome_do_cnpj {
        if nome.is_empty() {
            continue;
        };

        let cnpj_base = &cnpj[0..8];

        frequencia
            .entry(cnpj_base.to_string())
            // If there's no entry for the key cnpj_base, create a new Vec and return a mutable ref to it
            .or_default()
            // and insert the item onto the Vec
            .push(nome.to_string());
    }

    // println!("frequencia: {frequencia:#?}");

    let mut hashmap: HashMap<String, String> = HashMap::new();

    for (cnpj_base, vetor) in frequencia {
        // Ordenado pelo nome, em caso de mesma frequência.
        let mut counts: BTreeMap<String, u32> = BTreeMap::new();

        for nome in &vetor {
            *counts.entry(nome.to_lowercase()).or_insert(0) += 1;
        }

        // println!("cnpj_base: {cnpj_base} ; counts: {counts:#?}");

        // iterate over BTreeMap to find the key more often:
        let nome_mais_frequente: String = counts
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(k, _)| k)
            .unwrap();

        // Insensitive case
        for nome in &vetor {
            if nome.to_lowercase() == nome_mais_frequente {
                hashmap.insert(cnpj_base, nome.to_string());
                break;
            }
        }
    }

    hashmap
}

#[cfg(test)]
mod tests {
    // cargo test -- --help
    // cargo test -- --nocapture
    // cargo test -- --show-output
    // cargo test -- --show-output test_most_frequent_value
    use super::*;

    #[test]
    fn test_most_frequent_value() {
        let cnpjs = [
            ("12345678901231".to_string(), "Nome 01".to_string()),
            ("12345678901232".to_string(), "Nome 02".to_string()),
            ("12345678901233".to_string(), "Nome 02".to_string()),
            ("12345678901234".to_string(), "Nome 04".to_string()),
            ("12345678901235".to_string(), "NomE 01".to_string()),
            ("12345678901236".to_string(), "nome 03".to_string()),
            ("12345678901237".to_string(), "nome 01".to_string()),
            ("12345678901257".to_string(), "NOME 01".to_string()),
            ("00000000000000".to_string(), "Nome 01".to_string()),
            ("00000000000001".to_string(), "Nome 02".to_string()),
            ("00000000000002".to_string(), "nome 01".to_string()),
            ("00000000000003".to_string(), "Nome 02".to_string()),
            ("00000000000004".to_string(), "NoMe 01".to_string()),
        ];

        let nome_do_cnpj: BTreeMap<String, String> = BTreeMap::from(cnpjs);
        let hash_map: HashMap<String, String> = get_the_most_frequent_value(&nome_do_cnpj);

        println!("nome_do_cnpj: {nome_do_cnpj:#?}");
        println!("the_most_frequent_value: {hash_map:#?}");

        let result = HashMap::from([
            ("00000000".to_string(), "Nome 01".to_string()),
            ("12345678".to_string(), "Nome 01".to_string()),
        ]);

        assert_eq!(result, hash_map);
    }
}
