use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    path::{Path, PathBuf},
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
    pub path: PathBuf,
    pub messages: String,

    // --- Metadados Globais (Registro 0000 e afins) ---
    pub pa: Option<NaiveDate>, // Período de Apuração conforme EFD
    pub cnpj_base: u32,
    pub cnpj_do_estabelecimento: String,

    pub global: HashMap<String, String>,

    // --- Tabelas de Referência ---
    pub complementar: HashMap<String, String>,
    pub contabil: HashMap<String, HashMap<String, String>>,
    pub estabelecimentos: HashMap<String, String>,
    pub nat_operacao: HashMap<String, String>,
    pub nome_do_cnpj: BTreeMap<String, String>,
    pub nome_do_cpf: BTreeMap<String, String>,
    pub participantes: BTreeMap<String, HashMap<String, String>>,
    pub produtos: BTreeMap<String, HashMap<String, String>>,
    pub unidade_de_medida: HashMap<String, String>,

    // Auxiliares de processamento
    // Correlação de Alíquotas (Cache)
    // Chave: String gerada (Chave Fraca/Forte), Valor: [AliqPis, VlPis]
    pub correlacao: HashMap<String, [String; 2]>, // arrays [type; number] are fastest than vectors
    pub linhas_inseridas: Vec<usize>,

    // Registros "State" para processamento hierárquico Pai -> Filho
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
        let mut info = Info {
            path: arquivo.to_path_buf(),
            ..Default::default()
        };

        info.global
            .insert("arquivo_efd".to_string(), arquivo.display().to_string());

        info
    }

    /// Selecionar o nome mais frequente para um dado cnpj
    pub fn obter_nome_do_cnpj_base(&self) -> HashMap<String, String> {
        get_the_most_frequent_value(&self.nome_do_cnpj)
    }
}

/// `get_the_most_frequent_value` takes a BTreeMap of full CNPJs to company names
/// and returns a HashMap mapping 8-digit CNPJ bases to their most frequent company name.
///
/// This function processes the data in a single, efficient functional pipeline:
///
/// 1.  **`filter`**: Removes entries with empty company names, ensuring only valid names are processed.
/// 2.  **`fold`**: Groups company names by their 8-digit CNPJ base. For each unique name (case-insensitive)
///     within a CNPJ base, it counts occurrences and stores an example of its original casing.
///     *   Uses `entry().and_modify().or_insert_with()` for concise and efficient counting:
///         *   If a name already exists for a CNPJ base, its count is incremented (`and_modify`).
///         *   If it's a new name, it's inserted with a count of 1 and its original casing (`or_insert_with`).
/// 3.  **`into_iter`**: Prepares the accumulated data (CNPJ bases with their name counts) for the next step.
/// 4.  **`filter_map`**: For each CNPJ base, it finds the company name that appeared most frequently.
///     *   `max_by_key`: Identifies the name with the highest count. In case of ties, `BTreeMap`'s
///         lexicographical order provides a consistent tie-breaking rule.
///     *   `map`: Extracts the original-cased name corresponding to the most frequent one.
/// 5.  **`collect`**: Gathers all results into the final `HashMap<String, String>`, which is then returned.
pub fn get_the_most_frequent_value(
    nome_do_cnpj: &BTreeMap<String, String>,
) -> HashMap<String, String> {
    nome_do_cnpj
        .iter()
        // 1. Filter out entries where the name is empty.
        .filter(|(_cnpj, name)| !name.trim().is_empty())
        // 2. Group by CNPJ base and count name frequencies, preserving original case.
        .fold(
            HashMap::new(), // Outer map: cnpj_base -> (inner map of name counts)
            |mut acc: HashMap<String, BTreeMap<String, (u32, String)>>, (cnpj, name)| {
                // Extract the 8-digit CNPJ base.
                let cnpj_base: &str = &cnpj[0..8];

                acc.entry(cnpj_base.to_string())
                    .or_default()
                    .entry(name.to_lowercase())
                    .and_modify(|(count, _name)| *count += 1)
                    .or_insert((1, name.to_string()));

                acc // Return the accumulator for the next iteration.
            },
        )
        // 3. Convert the intermediate HashMap into an iterator.
        .into_iter()
        // 4. For each CNPJ base, find its most frequent name and map it to the final output format.
        .filter_map(|(cnpj_base, entry_counts)| {
            entry_counts
                .into_values()
                // Find the entry with the maximum count.
                // BTreeMap's order provides deterministic tie-breaking for names with same max count.
                .max_by_key(|(count, _name)| *count)
                // If a most frequent name is found, extract its original casing name.
                .map(|(_count, name)| (cnpj_base, name))
        })
        // 5. Collect all results into the final HashMap.
        .collect()
}

/**
Tipo de Programação:
Esta versão é um exemplo de programação imperativa e procedural.

* Ela usa loops for explícitos.
* Declara e muta variáveis (frequencia, hashmap, counts) em vários pontos.
* O fluxo de controle é mais sequencial e explícito, passo a passo, como uma receita.
*/
#[allow(dead_code)]
fn get_the_most_frequent_value_v2(
    nome_do_cnpj: &BTreeMap<String, String>,
) -> HashMap<String, String> {
    let mut frequencia: HashMap<String, Vec<String>> = HashMap::new();

    for (cnpj, nome) in nome_do_cnpj {
        if nome.trim().is_empty() {
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

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//
//
// cargo test -- --help
// cargo test -- --nocapture
// cargo test -- --show-output

/// Run tests with:
/// cargo test -- --show-output info_tests
#[cfg(test)]
mod info_tests {
    use super::*;

    #[test]
    /// cargo test -- --show-output most_frequent_value
    fn test_most_frequent_value() {
        let cnpjs = [
            ("12345678901232", "Nome 02"),
            ("12345678901261", "Nome 03"),
            ("12345678901257", "NOME 04"), // frequencia 3
            ("12345678901239", "nome 03"),
            ("12345678901233", "Nome 02"),
            ("12345678901234", "Nome 01"),
            ("12345678901236", "nome 03"),
            ("12345678901237", "nome 04"), // frequencia 3
            ("12345678901235", "NomE 04"), // frequencia 3
            ("00000000000000", "Nome 01"),
            ("00000000000001", "Nome 02"),
            ("00000000000002", "nome 01"),
            ("00000000000003", "Nome 02"),
            ("00000000000004", "NoMe 01"),
            ("11111111000000", "Unique Name"), // Test with a unique name
            ("11111111000001", "Unique Name"),
            ("22222222000000", " "), // Teste com nome com espaços em branco
            ("99999999000000", ""),  // Teste com nome vazio
        ]
        .map(|(cnpj, nome)| (cnpj.to_string(), nome.to_string()));

        println!("cnpjs: {cnpjs:?}");

        let nome_do_cnpj: BTreeMap<String, String> = BTreeMap::from(cnpjs);
        let result: HashMap<String, String> = get_the_most_frequent_value(&nome_do_cnpj);

        println!("nome_do_cnpj: {nome_do_cnpj:#?}");
        println!("the_most_frequent_value result: {result:#?}");

        let expected = HashMap::from([
            ("00000000".to_string(), "Nome 01".to_string()),
            ("12345678".to_string(), "NomE 04".to_string()),
            ("11111111".to_string(), "Unique Name".to_string()),
        ]);

        assert_eq!(result, expected);
    }
}
