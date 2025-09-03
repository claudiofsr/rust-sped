use csv::StringRecord;
use rayon::prelude::*;
use serde::{Deserialize, Serialize, Serializer};
use serde_aux::prelude::serde_introspect;
use struct_iterable::Iterable;

// https://stackoverflow.com/questions/39638363/how-can-i-use-a-hashmap-with-f64-as-key-in-rust
// Because OrderedFloat implements Ord and Eq, it can be used as a key in a
// HashSet, HashMap, BTreeMap, or BTreeSet (unlike the primitive f32 or f64 types):
use ordered_float::OrderedFloat;

use std::{
    collections::{BTreeMap, HashMap},
    //cmp::Reverse,
    fmt,
    hash::Hash,
    ops::{Add, AddAssign, Mul},
    thread,
};

use tabled::{
    Table, Tabled,
    settings::{
        Alignment, Modify, Style,
        object::{Columns, Rows, Segment},
    },
};

use crate::{
    DECIMAL_ALIQ, DECIMAL_VALOR, Despise, DocsFiscais, EFDResult, InfoExtension, Mes, SMALL_VALUE,
    Tributos::{Cofins, Pis},
    ZERO, get_tipo_de_operacao, month_to_str, obter_descricao_da_natureza_da_bc_dos_creditos,
    obter_descricao_do_tipo_de_credito, verificar_periodo_multiplo,
};

use claudiofsr_lib::{
    BASE_CALC_SOMA, CFOP_DE_EXPORTACAO, OptionExtension, RoundFloat, svec, thousands_separator,
};

#[derive(Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Serialize, Deserialize)]
pub struct Chaves {
    pub cnpj_base: String,
    pub ano: Option<i32>,
    trimestre: Option<u32>,
    pub mes: Option<u32>,
    tipo_de_operacao: Option<u16>,
    tipo_de_credito: Option<u16>,
    pub cst: Option<u16>,
    cfop: Option<u16>,
    aliq_pis: Option<OrderedFloat<f64>>,
    aliq_cofins: Option<OrderedFloat<f64>>,
    natureza_bc: Option<u16>,
}

impl Chaves {
    /**
    Checks if the CFOP corresponds to an export operation.

    CFOP de Exportacao:

    . Grupo 7:
        valores entre 7000 e 7999;

    . Fim específico de exportação.


    The `CFOP_DE_EXPORTACAO` constant array is assumed to be sorted
    for efficient binary search.
    */
    pub fn cfop_de_exportacao(&self) -> bool {
        self.cfop
            .is_some_and(|cfop_value| CFOP_DE_EXPORTACAO.binary_search(&cfop_value).is_ok())
    }
}

#[derive(Debug, Default, PartialEq, PartialOrd, Copy, Clone, Serialize, Deserialize)]
pub struct Valores {
    valor_item: f64,
    valor_bc: f64,
    valor_rbnc_trib: f64,
    valor_rbnc_ntrib: f64,
    valor_rbnc_exp: f64,
    valor_rb_cum: f64,
}

// https://practice.rs/generics-traits/advanced-traits.html#default-generic-type-parameters
// https://stackoverflow.com/questions/73663781/how-to-implement-sum-of-optiont-variables
impl Add for Valores {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            valor_item: self.valor_item + other.valor_item,
            valor_bc: self.valor_bc + other.valor_bc,
            valor_rbnc_trib: self.valor_rbnc_trib + other.valor_rbnc_trib,
            valor_rbnc_ntrib: self.valor_rbnc_ntrib + other.valor_rbnc_ntrib,
            valor_rbnc_exp: self.valor_rbnc_exp + other.valor_rbnc_exp,
            valor_rb_cum: self.valor_rb_cum + other.valor_rb_cum,
        }
    }
}

// https://doc.rust-lang.org/std/ops/trait.AddAssign.html
/// Executa a operação +=
impl AddAssign for Valores {
    fn add_assign(&mut self, other: Self) {
        self.valor_item += other.valor_item;
        self.valor_bc += other.valor_bc;
        self.valor_rbnc_trib += other.valor_rbnc_trib;
        self.valor_rbnc_ntrib += other.valor_rbnc_ntrib;
        self.valor_rbnc_exp += other.valor_rbnc_exp;
        self.valor_rb_cum += other.valor_rb_cum;
    }
}

// Multiplying Struct (Valores) by scalars (f64) as in linear algebra
impl Mul<f64> for Valores {
    type Output = Self;
    fn mul(self, value: f64) -> Self {
        Self {
            valor_item: self.valor_item * value,
            valor_bc: self.valor_bc * value,
            valor_rbnc_trib: self.valor_rbnc_trib * value,
            valor_rbnc_ntrib: self.valor_rbnc_ntrib * value,
            valor_rbnc_exp: self.valor_rbnc_exp * value,
            valor_rb_cum: self.valor_rb_cum * value,
        }
    }
}

impl Valores {
    /// Obter valores
    fn get_values(self) -> Vec<f64> {
        vec![
            self.valor_item,
            self.valor_bc,
            self.valor_rbnc_trib,
            self.valor_rbnc_ntrib,
            self.valor_rbnc_exp,
            self.valor_rb_cum,
        ]
    }

    /// Verificar se os valores são aproximadamente iguais.
    ///
    /// # Parameters
    /// - `other`: o outro objeto do qual você está comparando
    /// - `delta`: a tolerância de erro permitida
    ///
    /// # Returns
    /// `true` se os valores forem aproximadamente iguais, `false` caso contrário
    pub fn aproximadamente_iguais(&self, other: &Self, delta: f64) -> bool {
        self.get_values()
            .iter()
            .zip(other.get_values().iter())
            .all(|(a, b)| (a - b).abs() <= delta)
    }

    /// Distribui os valores de acordo com o rateio.
    fn distribuir_conforme_rateio(&mut self, linha: &DocsFiscais, credito_rateado: Option<f64>) {
        // De acordo com 4.3.6 – Tabela Código de Tipo de Crédito
        let cod_rateio: Option<u16> = linha
            .cod_credito // valor inteiro entre 101 e 499
            .map(|value| value / 100); // valor inteiro entre 1 e 4

        match (cod_rateio, credito_rateado) {
            (Some(1), Some(valor)) => self.valor_rbnc_trib = valor,
            (Some(2), Some(valor)) => self.valor_rbnc_ntrib = valor,
            (Some(3), Some(valor)) => self.valor_rbnc_exp = valor,
            _ => (), // ignorar se o código de rateio não for válido ou crédito rateado não presente
        }
    }
}

/// Análise dos Créditos
#[derive(
    Debug, Default, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Tabled, Iterable,
)]
pub struct AnaliseDosCreditos {
    #[serde(rename = "CNPJ Base")]
    #[tabled(rename = "CNPJ Base")]
    pub cnpj_base: String,
    #[serde(rename = "Ano do Período de Apuração")]
    #[tabled(rename = "Ano", display = "display_value")]
    pub ano: Option<i32>,
    #[serde(rename = "Trimestre do Período de Apuração")]
    #[tabled(rename = "Trim", display = "display_value")]
    pub trimestre: Option<u32>,
    #[serde(
        rename = "Mês do Período de Apuração",
        serialize_with = "serialize_mes"
    )]
    #[tabled(rename = "Mês", display = "display_mes")]
    pub mes: Option<u32>,
    #[serde(
        rename = "Tipo de Operação",
        serialize_with = "serialize_tipo_de_operacao"
    )]
    #[tabled(rename = "Tipo de Operação", display = "display_tipo_de_operacao")]
    pub tipo_de_operacao: Option<u16>,
    #[serde(
        rename = "Tipo de Crédito",
        serialize_with = "serialize_tipo_de_credito"
    )]
    #[tabled(rename = "Tipo de Crédito", display = "display_tipo_de_credito")]
    pub tipo_de_credito: Option<u16>,
    #[serde(rename = "CST", serialize_with = "serialize_cst")]
    #[tabled(rename = "CST", display = "display_cst")]
    pub cst: Option<u16>,
    #[serde(rename = "Alíquota de PIS/PASEP")]
    #[tabled(rename = "Alíquota PIS/PASEP", display = "display_aliquota")]
    pub aliq_pis: Option<f64>,
    #[serde(rename = "Alíquota de COFINS")]
    #[tabled(rename = "Alíquota COFINS", display = "display_aliquota")]
    pub aliq_cofins: Option<f64>,
    #[serde(
        rename = "Natureza da Base de Cálculo dos Créditos",
        serialize_with = "serialize_natureza"
    )]
    #[tabled(
        rename = "Natureza da Base de Cálculo dos Créditos",
        display = "display_natureza"
    )]
    pub natureza_bc: Option<u16>,
    #[serde(rename = "Base de Cálculo")] // serialize_with = "serialize_f64"
    #[tabled(rename = "Base de Cálculo", display = "display_f64")]
    pub valor_bc: Option<f64>,
    #[serde(rename = "Crédito vinculado à Receita Bruta Não Cumulativa: Tributada")]
    #[tabled(rename = "RBNC_Trib", display = "display_f64")]
    pub valor_rbnc_trib: Option<f64>,
    #[serde(rename = "Crédito vinculado à Receita Bruta Não Cumulativa: Não Tributada")]
    #[tabled(rename = "RBNC_NTrib", display = "display_f64")]
    pub valor_rbnc_ntrib: Option<f64>,
    #[serde(rename = "Crédito vinculado à Receita Bruta Não Cumulativa: de Exportação")]
    #[tabled(rename = "RBNC_Exp", display = "display_f64")]
    pub valor_rbnc_exp: Option<f64>,
    #[serde(rename = "Crédito vinculado à Receita Bruta Cumulativa")]
    #[tabled(rename = "RB_Cum", display = "display_f64")]
    pub valor_rb_cum: Option<f64>,
}

/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl InfoExtension for AnaliseDosCreditos {}

impl AnaliseDosCreditos {
    pub fn get_headers() -> StringRecord {
        // use serde_aux::prelude::serde_introspect;
        let colunas_vec = serde_introspect::<AnaliseDosCreditos>();
        StringRecord::from(colunas_vec)
    }
}

pub fn serialize_tipo_de_operacao<S>(
    tipo_de_operacao: &Option<u16>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let opt_string: Option<String> = get_tipo_de_operacao(tipo_de_operacao);
    serializer.serialize_some(&opt_string)
}

// https://doc.rust-lang.org/book/ch18-03-pattern-syntax.html
// @ Bindings
// The at operator @ lets us create a variable that holds a value
// at the same time as we’re testing that value for a pattern match.
pub fn display_tipo_de_operacao(tipo_de_operacao: &Option<u16>) -> String {
    match tipo_de_operacao {
        value @ Some(1..=7) => get_tipo_de_operacao(value).to_string(),
        _ => "".to_string(),
    }
}

pub fn serialize_tipo_de_credito<S>(tipo: &Option<u16>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let string = obter_descricao_do_tipo_de_credito(tipo, false);
    serializer.serialize_str(&string)
}

fn display_tipo_de_credito(tipo_de_credito: &Option<u16>) -> String {
    obter_descricao_do_tipo_de_credito(tipo_de_credito, false)
}

pub fn serialize_cst<S>(codigo: &Option<u16>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let string = display_cst(codigo);
    serializer.serialize_str(&string)
}

pub fn display_cst(codigo: &Option<u16>) -> String {
    match codigo {
        Some(490) => "Total Receitas/Saídas".to_string(),
        Some(980) => "Total Aquisições/Custos/Despesas".to_string(),
        Some(_) => {
            format!("{:02}", codigo.unwrap())
        }
        None => "".to_string(),
    }
}

pub fn display_value<T>(valor: &Option<T>) -> String
where
    T: std::fmt::Display + ToString,
{
    match valor {
        Some(val) => val.to_string(),
        None => "".to_string(),
    }
}

pub fn serialize_f64<S>(value: &Option<f64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(v) if v.abs() >= SMALL_VALUE => serializer.serialize_some(value),
        _ => serializer.serialize_none(),
    }
}

// ver fn despise_small_values
pub fn display_f64(valor: &Option<f64>) -> String {
    match *valor {
        Some(val) => thousands_separator(val, DECIMAL_VALOR),
        None => "".to_string(),
    }
}

pub fn serialize_mes<S>(mes: &Option<u32>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match mes {
        Some(1..=12) => serializer.serialize_str(month_to_str(mes)),
        _ => serializer.serialize_none(),
    }
}

pub fn display_mes(mes: &Option<u32>) -> String {
    match mes {
        Some(1..=12) => mes.unwrap().to_string(),
        _ => "".to_string(),
    }
}

pub fn serialize_natureza<S>(nat: &Option<u16>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let string = obter_descricao_da_natureza_da_bc_dos_creditos(nat);
    serializer.serialize_str(&string)
}

pub fn display_natureza(nat: &Option<u16>) -> String {
    obter_descricao_da_natureza_da_bc_dos_creditos(nat)
}

pub fn display_aliquota(valor: &Option<f64>) -> String {
    match *valor {
        Some(val) => [thousands_separator(val, DECIMAL_ALIQ), "%".to_string()].concat(),
        None => "".to_string(),
    }
}

// https://users.rust-lang.org/t/how-to-sort-vec-enum
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Hash, Serialize, Deserialize)]
pub enum ReceitaBruta {
    #[serde(rename = "Receita Bruta Não-Cumulativa - Tributada no Mercado Interno")]
    RbnTrmi,
    #[serde(rename = "Receita Bruta Não-Cumulativa - Não Tributada no Mercado Interno")]
    RbnNtmi,
    #[serde(rename = "Receita Bruta Não-Cumulativa - Exportação")]
    RbnExpo,
    #[serde(rename = "Receita Bruta Não Cumulativa Total")]
    RbncTot,
    #[serde(rename = "Receita Bruta Cumulativa")]
    RbCumul,
    #[serde(rename = "Receita Bruta Total")]
    RbTotal,
}

// https://docs.rs/serde/latest/serde/ser/trait.Serializer.html#method.collect_str
impl fmt::Display for ReceitaBruta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.serialize(f)
    }
}

#[derive(Debug, Default, Eq, PartialEq, PartialOrd, Clone, Hash, Serialize, Deserialize)]
pub struct PeriodoDeApuracao {
    #[serde(rename = "CNPJ Base")]
    cnpj_base: String,
    #[serde(rename = "Ano do Período de Apuração")]
    ano: Option<i32>,
    #[serde(rename = "Trimestre do Período de Apuração")]
    trimestre: Option<u32>,
    #[serde(rename = "Mês do Período de Apuração")]
    mes: Option<u32>,
    #[serde(rename = "Receita Bruta Segregada para Fins de Rateio dos Créditos")]
    rb_nome: Option<ReceitaBruta>,
}

#[derive(Debug, Default, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub struct ValorDaReceita {
    #[serde(rename = "Valor")]
    valor: f64,
    #[serde(rename = "Percentual")]
    pct: f64,
    #[serde(rename = "CST")]
    csts: Vec<Option<u16>>,
}

impl ValorDaReceita {
    pub fn get_headers() -> StringRecord {
        // use serde_aux::prelude::serde_introspect;
        let colunas_vec = serde_introspect::<ValorDaReceita>();
        StringRecord::from(colunas_vec)
    }

    fn cst_sum(&mut self, other: Self) {
        self.csts.extend(other.csts);
        self.csts.sort_unstable();
        self.csts.dedup(); // Removes consecutive repeated elements
    }
}

// https://doc.rust-lang.org/std/ops/trait.AddAssign.html
/// Executa a operação +=
impl AddAssign for ValorDaReceita {
    fn add_assign(&mut self, other: Self) {
        self.valor += other.valor;
        self.pct += other.pct;
        self.cst_sum(other);
    }
}

#[derive(Debug, Default, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Tabled)]
pub struct TabelaValorDaReceita {
    #[serde(rename = "CNPJ Base")]
    #[tabled(rename = "CNPJ Base")]
    pub cnpj_base: String,
    #[serde(rename = "Ano do Período de Apuração")]
    #[tabled(rename = "Ano", display = "display_value")]
    pub ano: Option<i32>,
    #[serde(rename = "Trimestre do Período de Apuração")]
    #[tabled(rename = "Trim", display = "display_value")]
    pub trimestre: Option<u32>,
    #[serde(
        rename = "Mês do Período de Apuração",
        serialize_with = "serialize_mes"
    )]
    #[tabled(rename = "Mês", display = "display_mes")]
    pub mes: Option<u32>,
    #[serde(rename = "CST")]
    #[tabled(rename = "CST", display = "display_csts")]
    pub csts: Vec<Option<u16>>,
    #[serde(rename = "Receita Bruta Segregada para Fins de Rateio dos Créditos")]
    #[tabled(
        rename = "Receita Bruta Segregada para Fins de Rateio dos Créditos",
        display = "display_value"
    )]
    pub rb_nome: Option<ReceitaBruta>,
    #[serde(rename = "Valor")]
    #[tabled(rename = "Valor", display = "display_f64")]
    pub valor: Option<f64>,
    #[serde(rename = "Percentual")]
    #[tabled(rename = "Percentual", display = "display_percentual")]
    pub pct: Option<f64>,
}

fn display_csts(csts: &[Option<u16>]) -> String {
    let vec_csts: Vec<String> = csts
        .iter()
        .filter(|&opt_u16| opt_u16.is_some_and(|v| v > 0))
        .map(|opt_u16| opt_u16.unwrap().to_string())
        .collect();

    if vec_csts.is_empty() {
        "".to_string()
    } else {
        svec!["[", vec_csts.join(", "), "]"].concat()
    }
}

fn display_percentual(valor: &Option<f64>) -> String {
    match *valor {
        Some(val) => [thousands_separator(val, 4), "%".to_string()].concat(),
        None => "".to_string(),
    }
}

pub fn consolidar_natureza_da_base_de_calculo(
    linhas: &[DocsFiscais],
) -> EFDResult<(String, String, Vec<AnaliseDosCreditos>)> {
    let chaves_consolidadas: HashMap<Chaves, Valores> = consolidar_chaves(linhas); // 1 <= CST <= 99

    let mut receita_bruta: HashMap<Chaves, Valores> = HashMap::new(); //  1 <= CST <= 49
    let mut base_creditos: HashMap<Chaves, Valores> = HashMap::new(); // 50 <= CST <= 66

    for (k, v) in chaves_consolidadas {
        match k.cst {
            //Some(1..=9) => receita_bruta.insert(k, v), // Excluir CST 49 do Rateio da Receita Bruta
            Some(1..=49) => receita_bruta.insert(k, v),
            Some(50..=66) => base_creditos.insert(k, v),
            _ => continue,
        };
    }

    distribuir_creditos_rateados(linhas, &mut base_creditos);

    let receita_bruta_segregada: HashMap<PeriodoDeApuracao, ValorDaReceita> =
        apurar_receita_bruta(&receita_bruta);
    let informacoes_de_receita_bruta: Vec<TabelaValorDaReceita> =
        obter_informacoes_de_receita_bruta(&receita_bruta_segregada);
    let tabela_da_receita_bruta: String = gerar_tabela_rec(&informacoes_de_receita_bruta);

    //analisar_rateio_dos_creditos(&base_creditos, &receita_bruta_segregada)?;

    /*
    // Usar std::thread nas funções seguintes (estas funções são independentes umas das outras):
    let (result_bcparcial, result_ajustes, result_descontos) = thread::scope(|s| {

        let thread_bcparcial = s.spawn(||somar_base_de_calculo_valor_parcial(&base_creditos));

        let thread_ajustes   = s.spawn(||distribuir_ajustes_rateados(linhas));

        let thread_descontos = s.spawn(||distribuir_descontos_rateados(linhas));

        // Wait for background thread to complete
        (thread_bcparcial.join(), thread_ajustes.join(), thread_descontos.join())
    });

    let (bcparcial, ajustes, descontos) = match (result_bcparcial, result_ajustes, result_descontos) {
        (Ok(bcparcial), Ok(ajustes), Ok(descontos)) => (bcparcial, ajustes, descontos),
        _ => panic!("Falha em soma parcial ou alocação de ajustes e descontos!"),
    };
    */

    let (mut bcparcial, mut ajustes, mut descontos) =
        (HashMap::new(), HashMap::new(), HashMap::new());

    // Usar std::thread nas funções seguintes (estas funções são independentes umas das outras):
    thread::scope(|s| {
        s.spawn(|| bcparcial = somar_base_de_calculo_valor_parcial(&base_creditos));

        s.spawn(|| ajustes = distribuir_ajustes_rateados(linhas));

        s.spawn(|| descontos = distribuir_descontos_rateados(linhas));
    });

    // Merge two HashMaps in Rust
    base_creditos.extend(bcparcial);
    base_creditos.extend(ajustes);
    base_creditos.extend(descontos);

    apurar_credito_das_contribuicoes(&mut base_creditos);
    calcular_credito_apos_ajustes(&mut base_creditos);
    calcular_credito_apos_descontos(&mut base_creditos);
    somar_base_de_calculo_valor_total(&mut base_creditos);
    calcular_saldo_de_credito_passivel_de_ressarcimento(&mut base_creditos);

    let periodo_multiplo = verificar_periodo_multiplo(&base_creditos);

    if periodo_multiplo {
        realizar_somas_trimestrais(&mut base_creditos);
    }

    let base_creditos_ordenado: Vec<(Chaves, Valores)> = ordenar(base_creditos);
    let base_creditos_estruturado: Vec<AnaliseDosCreditos> = get_analises(&base_creditos_ordenado);
    let tabela_de_base_creditos: String = gerar_tabela_nat(&base_creditos_estruturado);

    Ok((
        tabela_de_base_creditos,
        tabela_da_receita_bruta,
        base_creditos_estruturado,
    ))
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
fn consolidar_chaves(lines: &[DocsFiscais]) -> HashMap<Chaves, Valores> {
    let map_reduce: HashMap<Chaves, Valores> = lines
        .into_par_iter() // rayon: parallel iterator
        .filter(|&line| line.entrada_de_credito() || line.saida_de_receita_bruta())
        //.filter(|&line| !(line.cst == Some(9) && line.registro == "C170") ) // desconsiderar: CST 9 && Registro C170
        .map(obter_chaves_valores)
        .fold(HashMap::new, |mut hashmap_accumulator, (key, value)| {
            hashmap_accumulator
                .entry(key)
                .and_modify(|previous_value| *previous_value += value)
                .or_insert(value);

            hashmap_accumulator
        })
        .reduce(HashMap::new, |mut hashmap_a, hashmap_b| {
            hashmap_b.into_iter().for_each(|(key_b, value_b)| {
                hashmap_a
                    .entry(key_b)
                    .and_modify(|previous_value| *previous_value += value_b)
                    .or_insert(value_b);
            });

            hashmap_a
        });

    map_reduce
}

pub fn obter_chaves_valores(linha: &DocsFiscais) -> (Chaves, Valores) {
    // Informações de CFOP serão utilizadas na segregação da Receita Bruta.
    // Informações de CFOP não serão utilizadas na discriminação da
    // Natureza da Base de Cálculo dos Créditos.
    let cfop = if linha.cst_de_credito() {
        None // Remover informações de CFOP
    } else {
        linha.cfop
    };

    let chaves = Chaves {
        cnpj_base: linha.get_cnpj_base(),
        ano: linha.ano,
        trimestre: linha.trimestre,
        mes: linha.mes,
        tipo_de_operacao: linha.tipo_de_operacao,
        tipo_de_credito: linha.tipo_de_credito,
        cst: linha.cst,
        cfop,
        aliq_pis: linha.aliq_pis.map(OrderedFloat),
        aliq_cofins: linha.aliq_cofins.map(OrderedFloat),
        natureza_bc: linha.natureza_bc,
    };

    let valores = Valores {
        valor_item: linha.valor_item.unwrap_or(ZERO),
        valor_bc: linha.valor_bc.unwrap_or(ZERO),
        valor_rbnc_trib: ZERO, // valor inicial
        valor_rbnc_ntrib: ZERO,
        valor_rbnc_exp: ZERO,
        valor_rb_cum: ZERO,
    };

    (chaves, valores)
}

fn distribuir_creditos_rateados(
    linhas: &[DocsFiscais],
    base_creditos: &mut HashMap<Chaves, Valores>,
) {
    // Transmitir cod_credito (crédito com informação de rateio) para base_creditos.
    // Distribuir valores de Creditos rateados nas colunas correspondentes: Trib, NTrib e Exportação.

    for linha in linhas
        .iter()
        .filter(|&line| line.tipo_de_operacao == Some(7))
    // 7: "Detalhamento"
    {
        let chaves = Chaves {
            cnpj_base: linha.get_cnpj_base(),
            ano: linha.ano,
            trimestre: linha.trimestre,
            mes: linha.mes,
            tipo_de_operacao: Some(1), // atualizar valor: Entrada
            tipo_de_credito: linha.tipo_de_credito,
            cst: linha.cst,
            cfop: linha.cfop,
            aliq_pis: linha.aliq_pis.map(OrderedFloat),
            aliq_cofins: linha.aliq_cofins.map(OrderedFloat),
            natureza_bc: linha.natureza_bc,
        };

        // usar base_creditos.get_mut(&chaves) para obter uma referência mutável do valor associado à chave.

        if let Some(valores) = base_creditos.get_mut(&chaves) {
            valores.distribuir_conforme_rateio(linha, linha.valor_bc);
        }
    }
}

/// Obter Receita Bruta segregada por CST para fins de rateio dos créditos
fn apurar_receita_bruta(
    receita_bruta: &HashMap<Chaves, Valores>,
) -> HashMap<PeriodoDeApuracao, ValorDaReceita> {
    // CST já foi filtrado e pertence ao intervalo: 01 a 49.
    let mut hashmap: HashMap<PeriodoDeApuracao, ValorDaReceita> = HashMap::new();

    receita_bruta.iter().for_each(|(chaves, valores)| {
        // Somar: RbncTot = RbnTrmi + RbnNtmi + RbnExpo
        // Somar: RbTotal = RbncTot + RbCumul
        let mut receitas: Vec<Option<ReceitaBruta>> = vec![Some(ReceitaBruta::RbTotal)];

        if chaves.aliq_pis == Some(OrderedFloat(0.65))
            && chaves.aliq_cofins == Some(OrderedFloat(3.0))
        {
            receitas.push(Some(ReceitaBruta::RbCumul));
        } else {
            receitas.push(Some(ReceitaBruta::RbncTot));
            /*
            match chaves.cst {
                Some(1 | 2 | 3 | 5) => receitas.push(Some(ReceitaBruta::RbnTrmi)),
                Some(4 | 6 | 7 | 9 | 49) => receitas.push(Some(ReceitaBruta::RbnNtmi)),
                Some(8) => {
                    if chaves.cfop_de_exportacao() {
                        receitas.push(Some(ReceitaBruta::RbnExpo))
                    } else {
                        receitas.push(Some(ReceitaBruta::RbnNtmi))
                    }
                }
                _ => panic!("1 <= CST <= 49; CST obtido: {:?}!", chaves.cst),
            }
            */
            match chaves.cst {
                Some(1 | 2 | 3 | 5) => receitas.push(Some(ReceitaBruta::RbnTrmi)),
                Some(4 | 6 | 7 | 8 | 9 | 49) if !chaves.cfop_de_exportacao() => {
                    receitas.push(Some(ReceitaBruta::RbnNtmi))
                }
                Some(4 | 6 | 7 | 8 | 9 | 49) if chaves.cfop_de_exportacao() => {
                    receitas.push(Some(ReceitaBruta::RbnExpo))
                }
                _ => panic!("1 <= CST <= 49; CST obtido: {:?}!", chaves.cst),
            }
        };

        let rb = ValorDaReceita {
            valor: valores.valor_item,
            pct: 0.0,
            csts: vec![chaves.cst],
        };

        for receita in receitas {
            let pa = PeriodoDeApuracao {
                cnpj_base: chaves.cnpj_base.clone(),
                ano: chaves.ano,
                trimestre: chaves.trimestre,
                mes: chaves.mes,
                rb_nome: receita,
            };

            // impl Add and AddAssign for Valores: Soma de Valores
            hashmap
                .entry(pa)
                .and_modify(|previous_value| *previous_value += rb.clone())
                .or_insert(rb.clone());
        }
    });

    hashmap
}

fn obter_informacoes_de_receita_bruta(
    receita_bruta_segregada: &HashMap<PeriodoDeApuracao, ValorDaReceita>,
) -> Vec<TabelaValorDaReceita> {
    let mut lines: Vec<TabelaValorDaReceita> = Vec::new();

    let mut sorted: Vec<(&PeriodoDeApuracao, &ValorDaReceita)> =
        receita_bruta_segregada.iter().collect();

    sorted.sort_by_key(|&(periodo_de_apuracao, _valor_da_receita)| {
        (
            periodo_de_apuracao.cnpj_base.clone(),
            periodo_de_apuracao.ano,
            periodo_de_apuracao.trimestre,
            periodo_de_apuracao.mes,
            periodo_de_apuracao.rb_nome,
        )
    });

    for (periodo_de_apuracao, valor_da_receita) in sorted
        .into_iter()
        // Ao Remover valores nulos, evita-se divisão por Zero.
        .filter(|&(_, valor_da_receita)| valor_da_receita.valor > ZERO)
    {
        let mut pa = periodo_de_apuracao.clone();
        pa.rb_nome = Some(ReceitaBruta::RbTotal);

        // Obter o valor da Receita Bruta Total
        let rb_total: Option<f64> = receita_bruta_segregada.get(&pa).map(|v| v.valor);

        let pct: Option<f64> = Some(100.0 * valor_da_receita.valor).combine_with_div(rb_total);

        lines.push(TabelaValorDaReceita {
            cnpj_base: periodo_de_apuracao.cnpj_base.clone(),
            ano: periodo_de_apuracao.ano,
            trimestre: periodo_de_apuracao.trimestre,
            mes: periodo_de_apuracao.mes,
            csts: valor_da_receita.csts.clone(),
            rb_nome: periodo_de_apuracao.rb_nome,
            valor: Some(valor_da_receita.valor),
            pct,
        });

        if periodo_de_apuracao.rb_nome == Some(ReceitaBruta::RbncTot) && pct == Some(100.0) {
            break;
        }
    }

    lines
}

fn gerar_tabela_rec<'a, T: Tabled + Deserialize<'a>>(lines: &[T]) -> String {
    // https://crates.io/crates/tabled
    Table::new(lines)
        .with(Modify::new(Columns::new(..4)).with(Alignment::center()))
        .with(Modify::new(Columns::one(4)).with(Alignment::left()))
        .with(Modify::new(Columns::new(5..)).with(Alignment::right()))
        .with(Modify::new(Rows::one(0)).with(Alignment::center()))
        .with(Style::rounded())
        .to_string()
}

#[allow(dead_code)]
fn analisar_rateio_dos_creditos(
    base_creditos: &HashMap<Chaves, Valores>,
    receita_bruta_segregada: &HashMap<PeriodoDeApuracao, ValorDaReceita>,
) -> EFDResult<()> {
    let bc_com_creditos_distribuidos: HashMap<Chaves, Valores> =
        distribuir_creditos_conforme_receita_bruta_segregada(
            base_creditos,
            receita_bruta_segregada,
        );

    let delta: f64 = 0.50;

    confrontar_valores(base_creditos, &bc_com_creditos_distribuidos, delta)?;

    imprimir_registros_do_bloco_m(&bc_com_creditos_distribuidos);

    Ok(())
}

/// Distribuir créditos conforme segregação da receita bruta
fn distribuir_creditos_conforme_receita_bruta_segregada(
    base_creditos: &HashMap<Chaves, Valores>,
    receita_bruta_segregada: &HashMap<PeriodoDeApuracao, ValorDaReceita>,
) -> HashMap<Chaves, Valores> {
    let mut bc_com_creditos_distribuidos: HashMap<Chaves, Valores> = HashMap::new();

    for (chaves, &valores) in base_creditos
        .iter()
        .filter(|&(_chaves, valores)| valores.valor_item > ZERO)
    {
        let mut valores_bc: Valores = valores; // Manter valor_item e valor_bc.

        let mut pa = PeriodoDeApuracao {
            cnpj_base: chaves.cnpj_base.clone(),
            ano: chaves.ano,
            trimestre: chaves.trimestre,
            mes: chaves.mes,
            rb_nome: None, // Atualizar pelo valor correspondente
        };

        // Para este periodo_de_apuracao, obter valores da Receita Bruta Segregada
        let rbn_trmi: f64 = receita_bruta_segregada
            .get({
                pa.rb_nome = Some(ReceitaBruta::RbnTrmi);
                &pa
            })
            .map(|v| v.valor)
            .unwrap_or(0.0);
        let rbn_ntmi: f64 = receita_bruta_segregada
            .get({
                pa.rb_nome = Some(ReceitaBruta::RbnNtmi);
                &pa
            })
            .map(|v| v.valor)
            .unwrap_or(0.0);
        let rbn_expo: f64 = receita_bruta_segregada
            .get({
                pa.rb_nome = Some(ReceitaBruta::RbnExpo);
                &pa
            })
            .map(|v| v.valor)
            .unwrap_or(0.0);
        let rbnc_tot: f64 = receita_bruta_segregada
            .get({
                pa.rb_nome = Some(ReceitaBruta::RbncTot);
                &pa
            })
            .map(|v| v.valor)
            .unwrap_or(0.0);
        let rb_cumul: f64 = receita_bruta_segregada
            .get({
                pa.rb_nome = Some(ReceitaBruta::RbCumul);
                &pa
            })
            .map(|v| v.valor)
            .unwrap_or(0.0);
        let rb_total: f64 = receita_bruta_segregada
            .get({
                pa.rb_nome = Some(ReceitaBruta::RbTotal);
                &pa
            })
            .map(|v| v.valor)
            .unwrap_or(0.0);

        if rb_total > ZERO {
            match chaves.cst {
                Some(50 | 60) => {
                    valores_bc.valor_rbnc_trib = valores.valor_bc * (rbnc_tot / rb_total);
                    valores_bc.valor_rbnc_ntrib = ZERO;
                    valores_bc.valor_rbnc_exp = ZERO;
                    valores_bc.valor_rb_cum = valores.valor_bc * (rb_cumul / rb_total);
                }
                Some(51 | 61) => {
                    valores_bc.valor_rbnc_trib = ZERO;
                    valores_bc.valor_rbnc_ntrib = valores.valor_bc * (rbnc_tot / rb_total);
                    valores_bc.valor_rbnc_exp = ZERO;
                    valores_bc.valor_rb_cum = valores.valor_bc * (rb_cumul / rb_total);
                }
                Some(52 | 62) => {
                    valores_bc.valor_rbnc_trib = ZERO;
                    valores_bc.valor_rbnc_ntrib = ZERO;
                    valores_bc.valor_rbnc_exp = valores.valor_bc * (rbnc_tot / rb_total);
                    valores_bc.valor_rb_cum = valores.valor_bc * (rb_cumul / rb_total);
                }
                Some(53 | 63) => {
                    valores_bc.valor_rbnc_trib = valores.valor_bc * (rbn_trmi / rb_total);
                    valores_bc.valor_rbnc_ntrib = valores.valor_bc * (rbn_ntmi / rb_total);
                    valores_bc.valor_rbnc_exp = ZERO;
                    valores_bc.valor_rb_cum = valores.valor_bc * (rb_cumul / rb_total);
                }
                Some(54 | 64) => {
                    valores_bc.valor_rbnc_trib = valores.valor_bc * (rbn_trmi / rb_total);
                    valores_bc.valor_rbnc_ntrib = ZERO;
                    valores_bc.valor_rbnc_exp = valores.valor_bc * (rbn_expo / rb_total);
                    valores_bc.valor_rb_cum = valores.valor_bc * (rb_cumul / rb_total);
                }
                Some(55 | 65) => {
                    valores_bc.valor_rbnc_trib = ZERO;
                    valores_bc.valor_rbnc_ntrib = valores.valor_bc * (rbn_ntmi / rb_total);
                    valores_bc.valor_rbnc_exp = valores.valor_bc * (rbn_expo / rb_total);
                    valores_bc.valor_rb_cum = valores.valor_bc * (rb_cumul / rb_total);
                }
                Some(56 | 66) => {
                    valores_bc.valor_rbnc_trib = valores.valor_bc * (rbn_trmi / rb_total);
                    valores_bc.valor_rbnc_ntrib = valores.valor_bc * (rbn_ntmi / rb_total);
                    valores_bc.valor_rbnc_exp = valores.valor_bc * (rbn_expo / rb_total);
                    valores_bc.valor_rb_cum = valores.valor_bc * (rb_cumul / rb_total);
                }
                _ => (),
            }
        }

        bc_com_creditos_distribuidos.insert(chaves.clone(), valores_bc);
    }

    bc_com_creditos_distribuidos
}

fn confrontar_valores(
    base_creditos: &HashMap<Chaves, Valores>,
    bc_com_creditos_distribuidos: &HashMap<Chaves, Valores>,
    delta: f64,
) -> EFDResult<()> {
    let hashmap_a = arredondar_valores_hmap(base_creditos)?;
    let hashmap_b = arredondar_valores_hmap(bc_com_creditos_distribuidos)?;

    for (k, v) in &hashmap_a {
        eprintln!("{k:?}");
        eprintln!("{v:?}");
        match hashmap_b.get(k) {
            Some(val) => {
                println!("{val:?}\n");
                assert!(
                    v.aproximadamente_iguais(val, delta),
                    "Diferença maior que delta = {delta}"
                );
            }
            None => {
                eprintln!("Erro ao executar a função confrontar_valores()!");
                eprintln!("Não foi possível encontrar os valores correspondentes após apuração.\n");
                continue;
            }
        };
    }

    assert_eq!(hashmap_a.len(), hashmap_b.len());

    Ok(())
}

/// Round f64 values
fn arredondar_valores_hmap(
    hmap_original: &HashMap<Chaves, Valores>,
) -> EFDResult<HashMap<Chaves, Valores>> {
    let mut hmap: HashMap<Chaves, Valores> = HashMap::new();

    for (chaves, valores) in hmap_original {
        let val = Valores {
            valor_item: valores.valor_item.round_float(DECIMAL_VALOR as i64),
            valor_bc: valores.valor_bc.round_float(DECIMAL_VALOR as i64),
            valor_rbnc_trib: valores.valor_rbnc_trib.round_float(DECIMAL_VALOR as i64),
            valor_rbnc_ntrib: valores.valor_rbnc_ntrib.round_float(DECIMAL_VALOR as i64),
            valor_rbnc_exp: valores.valor_rbnc_exp.round_float(DECIMAL_VALOR as i64),
            valor_rb_cum: valores.valor_rb_cum.round_float(DECIMAL_VALOR as i64),
        };
        hmap.insert(chaves.clone(), val);
    }

    Ok(hmap)
}

fn imprimir_registros_do_bloco_m(bc_com_creditos_distribuidos: &HashMap<Chaves, Valores>) {
    let mut base_de_calculo_agrupado_por_natureza: BTreeMap<Chaves, Valores> = BTreeMap::new();

    // Informações agrupadas por Natureza da Base de Cálculo.
    for (chaves, &valores) in bc_com_creditos_distribuidos {
        let mut chaves_bloco_m = chaves.clone();
        chaves_bloco_m.natureza_bc = None; // Agrupar por natureza

        // impl Add and AddAssign for Valores: Soma de Valores
        base_de_calculo_agrupado_por_natureza
            .entry(chaves_bloco_m)
            .and_modify(|previous_value| *previous_value += valores)
            .or_insert(valores);
    }

    println!("base_de_calculo_agrupado_por_natureza: {base_de_calculo_agrupado_por_natureza:#?}");

    for tributo in [Pis, Cofins] {
        for (chaves, valores) in &base_de_calculo_agrupado_por_natureza {
            let valor_bc = f64_format(valores.valor_bc);

            let (aliquota, registro_pai, registro_filho, pct) = match tributo {
                Pis => (
                    chaves
                        .aliq_pis
                        .map_or("".to_string(), |v| thousands_separator(*v, DECIMAL_ALIQ)),
                    "M100",
                    "M105",
                    80.00,
                ),
                Cofins => (
                    chaves
                        .aliq_cofins
                        .map_or("".to_string(), |v| thousands_separator(*v, DECIMAL_ALIQ)),
                    "M500",
                    "M505",
                    20.00,
                ),
            };

            // |M100|101|0|800000|1,6500|||286073,44|0,00|0,00|0,00|286073,44|1|3575,92|3575,92|
            // |M100|201|0|800000|1,6500|||332794,35|0,00|0,00|0,00|332794,35|1|4159,93|4159,93|
            // |M100|301|0|800000|1,6500|||131215,27|0,00|0,00|0,00|131215,27|1|1640,19|1640,19|

            for (coluna, valor) in [
                (100, valores.valor_rbnc_trib),
                (200, valores.valor_rbnc_ntrib),
                (300, valores.valor_rbnc_exp),
            ]
            .into_iter()
            .filter(|&(_c, v)| v > ZERO)
            {
                let cod_credito = coluna + chaves.tipo_de_credito.unwrap();

                let vl_cred = f64_format(valor);
                let vl_parcial = f64_format(valor / pct);
                println!(
                    "|{registro_pai}|{cod_credito}|0|{valor_bc}|{aliquota}|||{vl_cred}|0,00|0,00|0,00|{vl_cred}|1|{vl_parcial}|{vl_parcial}|"
                );

                for (key, value) in bc_com_creditos_distribuidos.iter().filter(|&(k, _v)| {
                    let mut filter = k.clone();
                    filter.natureza_bc = None;
                    chaves == &filter
                }) {
                    // |M105|01|56|500000||500000|178795,90||||
                    // |M105|03|56|200000||200000|71518,36||||
                    // |M105|12|56|100000||100000|35759,18||||

                    if let (Some(natureza_bc), Some(cst)) = (key.natureza_bc, key.cst) {
                        let valor_bc = f64_format(value.valor_bc);

                        for (_col, val) in [
                            (100, value.valor_rbnc_trib),
                            (200, value.valor_rbnc_ntrib),
                            (300, value.valor_rbnc_exp),
                        ]
                        .into_iter()
                        .filter(|&(c, v)| c == coluna && v > ZERO)
                        {
                            let vl_bc_rateio = f64_format(val);
                            println!(
                                "|{registro_filho}|{natureza_bc:02}|{cst:02}|{valor_bc}||{valor_bc}|{vl_bc_rateio}||||"
                            );
                        }
                    }
                }
            }
        }
    }
}

fn f64_format(value: f64) -> String {
    format!("{value:0.DECIMAL_VALOR$}").replace('.', ",")
}

fn distribuir_ajustes_rateados(linhas: &[DocsFiscais]) -> HashMap<Chaves, Valores> {
    // Transmitir cod_credito (crédito com informação de rateio) para base_creditos.
    // Distribuir valores de Ajustes rateados nas colunas correspondentes: Trib, NTrib e Exportação.

    let operacoes_selecionadas = [
        Some(3), // 3: "Ajuste de Acréscimo"
        Some(4), // 4: "Ajuste de Redução"
    ];

    let mut ajustes: HashMap<Chaves, Valores> = HashMap::new();

    for linha in linhas
        .iter()
        .filter(|&linha| operacoes_selecionadas.contains(&linha.tipo_de_operacao))
    // 3 ou 4: "Ajuste de ..."
    {
        let mut chaves = Chaves {
            cnpj_base: linha.get_cnpj_base(),
            ano: linha.ano,
            trimestre: linha.trimestre,
            mes: linha.mes,
            tipo_de_operacao: linha.tipo_de_operacao,
            tipo_de_credito: linha.tipo_de_credito,
            cst: linha.cst,
            cfop: linha.cfop,
            aliq_pis: linha.aliq_pis.map(OrderedFloat),
            aliq_cofins: linha.aliq_cofins.map(OrderedFloat),
            natureza_bc: None,
        };

        let delta = if linha.aliq_pis.is_some() {
            1 // PIS/PASEP
        } else {
            5 // COFINS
        };

        // Tipo de Operação, 3: "Ajuste de Acréscimo", 4: "Ajuste de Redução"
        chaves.natureza_bc = linha.tipo_de_operacao.map(|v| 10 * v + delta);

        let mut valores = Valores {
            valor_item: linha.valor_item.unwrap_or(ZERO),
            valor_bc: linha.valor_item.unwrap_or(ZERO),
            valor_rbnc_trib: ZERO,
            valor_rbnc_ntrib: ZERO,
            valor_rbnc_exp: ZERO,
            valor_rb_cum: ZERO,
        };

        valores.distribuir_conforme_rateio(linha, linha.valor_item);

        // impl Add and AddAssign for Valores: Soma de Valores
        ajustes
            .entry(chaves)
            .and_modify(|previous_value| *previous_value += valores)
            .or_insert(valores);
    }

    ajustes
}

fn distribuir_descontos_rateados(linhas: &[DocsFiscais]) -> HashMap<Chaves, Valores> {
    // Transmitir cod_credito (crédito com informação de rateio) para base_creditos.
    // Distribuir valores de Descontos rateados nas colunas correspondentes: Trib, NTrib e Exportação.

    let operacoes_selecionadas = [
        Some(5), // 5: "Desconto da Contribuição Apurada no Próprio Período"
        Some(6), // 6: "Desconto Efetuado em Período Posterior"
    ];

    let mut descontos: HashMap<Chaves, Valores> = HashMap::new();

    for linha in linhas
        .iter()
        .filter(|&linha| operacoes_selecionadas.contains(&linha.tipo_de_operacao))
    // 5 ou 6: "Desconto ..."
    {
        let mut chaves = Chaves {
            cnpj_base: linha.get_cnpj_base(),
            ano: linha.ano,
            trimestre: linha.trimestre,
            mes: linha.mes,
            tipo_de_operacao: linha.tipo_de_operacao,
            tipo_de_credito: linha.tipo_de_credito,
            cst: linha.cst,
            cfop: linha.cfop,
            aliq_pis: linha.aliq_pis.map(OrderedFloat),
            aliq_cofins: linha.aliq_cofins.map(OrderedFloat),
            natureza_bc: None,
        };

        let registro = linha.registro.as_str();
        let delta = match registro {
            "M100" | "1100" => 1, // PIS/PASEP
            "M500" | "1500" => 5, // COFINS
            _ => {
                eprintln!("fn distribuir_descontos_rateados()");
                panic!("Erro: Registro {registro} não suportado!")
            }
        };

        // Tipo de Operação, 5: "Desconto no Período", 6: "Desconto em Período Posterior"
        chaves.natureza_bc = linha.tipo_de_operacao.map(|v| 10 * v + delta);

        let mut valores = Valores {
            valor_item: linha.valor_item.unwrap_or(ZERO),
            valor_bc: linha.valor_item.unwrap_or(ZERO),
            valor_rbnc_trib: ZERO,
            valor_rbnc_ntrib: ZERO,
            valor_rbnc_exp: ZERO,
            valor_rb_cum: ZERO,
        };

        valores.distribuir_conforme_rateio(linha, linha.valor_item);

        // impl Add and AddAssign for Valores: Soma de Valores
        descontos
            .entry(chaves)
            .and_modify(|previous_value| *previous_value += valores)
            .or_insert(valores);
    }

    descontos
}

fn somar_base_de_calculo_valor_parcial(
    base_creditos: &HashMap<Chaves, Valores>,
) -> HashMap<Chaves, Valores> {
    let mut base_de_calculo: HashMap<Chaves, Valores> = HashMap::new();

    for (chaves, &valores) in base_creditos.iter() {
        let mut chaves_bc = chaves.clone();

        match chaves.tipo_de_credito {
            Some(tipo_de_credito) => {
                chaves_bc.cst = Some(910);
                chaves_bc.natureza_bc = Some(tipo_de_credito + 100);
            }
            _ => continue,
        };

        // Crédito vinculado à Receita Bruta Cumulativa deve ser descartado!
        let receita_bruta_nao_cumulativa =
            valores.valor_rbnc_trib + valores.valor_rbnc_ntrib + valores.valor_rbnc_exp;

        let mut valores_base_calculo_soma = valores;
        valores_base_calculo_soma.valor_bc = receita_bruta_nao_cumulativa;

        // impl Add and AddAssign for Valores: Soma de Valores
        base_de_calculo
            .entry(chaves_bc)
            .and_modify(|previous_value| *previous_value += valores_base_calculo_soma)
            .or_insert(valores_base_calculo_soma);
    }

    base_de_calculo
}

fn apurar_credito_das_contribuicoes(base_creditos: &mut HashMap<Chaves, Valores>) {
    let mut credito_apurado: HashMap<Chaves, Valores> = HashMap::new();

    for (chaves, &valores) in base_creditos
        .iter()
        // Filtrar 'Base de Cálculo dos Créditos', valores entre 101 e 199.
        // .filter(|&(chaves, _valores)| chaves.natureza_bc > Some(100) && chaves.natureza_bc < Some(200))
        .filter(|&(chaves, _valores)| {
            BASE_CALC_SOMA
                .map(Some)
                .binary_search(&chaves.natureza_bc)
                .is_ok()
        })
    {
        for (tributo, aliquota) in [(Pis, chaves.aliq_pis), (Cofins, chaves.aliq_cofins)]
            .into_iter()
            .filter(|(_tributo, aliquota)| aliquota.is_some())
            .map(|(tributo, aliquota)| (tributo, *aliquota.unwrap()))
        {
            let mut chaves_apuracao = chaves.clone();

            match tributo {
                Pis => {
                    // Crédito Apurado no Período (PIS/PASEP)
                    chaves_apuracao.cst = Some(920);
                    chaves_apuracao.aliq_cofins = None;
                    chaves_apuracao.natureza_bc = Some(201);
                }
                Cofins => {
                    // Crédito Apurado no Período (COFINS)
                    chaves_apuracao.cst = Some(930);
                    chaves_apuracao.aliq_pis = None;
                    chaves_apuracao.natureza_bc = Some(205);
                }
            }

            // Foi implementada a função de multiplicação (mul) entre Valores e f64.
            let valores_apurados = valores * (aliquota / 100.00);

            credito_apurado.insert(chaves_apuracao, valores_apurados);
        }
    }

    // Merge two HashMaps in Rust
    base_creditos.extend(credito_apurado);
}

fn calcular_credito_apos_ajustes(base_creditos: &mut HashMap<Chaves, Valores>) {
    let mut credito_apos_ajustes: HashMap<Chaves, Valores> = HashMap::new();

    for (chaves, &valores) in base_creditos.iter() {
        let mut chaves_bc = chaves.clone();

        // Filtrar 'Crédito Apurado no Período':
        let credito_apurado_pis: bool = chaves_bc.natureza_bc == Some(201);
        let credito_apurado_cof: bool = chaves_bc.natureza_bc == Some(205);
        let credito_apurado: bool = credito_apurado_pis || credito_apurado_cof;

        // Filtrar 'Ajustes':
        let ajustes_pis: bool = [Some(31), Some(41)].contains(&chaves_bc.natureza_bc);
        let ajustes_cof: bool = [Some(35), Some(45)].contains(&chaves_bc.natureza_bc);
        let ajustes: bool = ajustes_pis || ajustes_cof;

        if !(credito_apurado || ajustes) {
            continue;
        }

        /*
        if [Some(31), Some(35)].contains(&chaves_bc.natureza_bc) {
            chaves_bc.tipo_de_operacao = Some(3); // Ajuste de Acréscimo
        }
        else if [Some(41), Some(45)].contains(&chaves_bc.natureza_bc) {
            chaves_bc.tipo_de_operacao = Some(4); // Ajuste de Redução
        }
        */

        chaves_bc.aliq_pis = None;
        chaves_bc.aliq_cofins = None;

        if credito_apurado_pis || ajustes_pis {
            chaves_bc.cst = Some(920);
            chaves_bc.natureza_bc = Some(211);

            if chaves_bc.natureza_bc == Some(31) {
                chaves_bc.tipo_de_operacao = Some(3);
            } else {
                chaves_bc.tipo_de_operacao = Some(4);
            }
        }
        if credito_apurado_cof || ajustes_cof {
            chaves_bc.cst = Some(930);
            chaves_bc.natureza_bc = Some(215);

            if chaves_bc.natureza_bc == Some(35) {
                chaves_bc.tipo_de_operacao = Some(3);
            } else {
                chaves_bc.tipo_de_operacao = Some(4);
            }
        }

        // impl Add and AddAssign for Valores: Soma de Valores
        credito_apos_ajustes
            .entry(chaves_bc)
            .and_modify(|previous_value| *previous_value += valores)
            .or_insert(valores);
    }

    // Merge two HashMaps in Rust
    base_creditos.extend(credito_apos_ajustes);
}

fn calcular_credito_apos_descontos(base_creditos: &mut HashMap<Chaves, Valores>) {
    let mut credito_apos_ajustes: HashMap<Chaves, Valores> = HashMap::new();

    for (chaves, &valores) in base_creditos.iter() {
        let mut chaves_bc = chaves.clone();

        // Filtrar 'Crédito Disponível após Ajustes':
        let cred_apos_ajustes_pis: bool = chaves_bc.natureza_bc == Some(211);
        let cred_apos_ajustes_cof: bool = chaves_bc.natureza_bc == Some(215);
        let cred_apos_ajustes: bool = cred_apos_ajustes_pis || cred_apos_ajustes_cof;

        // Filtrar 'Descontos':
        let descontos_pis: bool = [Some(51), Some(61)].contains(&chaves_bc.natureza_bc);
        let descontos_cof: bool = [Some(55), Some(65)].contains(&chaves_bc.natureza_bc);
        let descontos: bool = descontos_pis || descontos_cof;

        if !(cred_apos_ajustes || descontos) {
            continue;
        }

        /*
        if [Some(51), Some(55)].contains(&chaves_bc.natureza_bc) {
            chaves_bc.tipo_de_operacao = Some(5); // Desconto da Contribuição Apurada no Próprio Período
        }
        else if [Some(61), Some(65)].contains(&chaves_bc.natureza_bc) {
            chaves_bc.tipo_de_operacao = Some(6); // Desconto Efetuado em Período Posterior
        }
        */

        chaves_bc.aliq_pis = None;
        chaves_bc.aliq_cofins = None;

        if cred_apos_ajustes_pis || descontos_pis {
            chaves_bc.cst = Some(920);
            chaves_bc.natureza_bc = Some(221);

            if chaves_bc.natureza_bc == Some(51) {
                chaves_bc.tipo_de_operacao = Some(5);
            } else {
                chaves_bc.tipo_de_operacao = Some(6);
            }
        }
        if cred_apos_ajustes_cof || descontos_cof {
            chaves_bc.cst = Some(930);
            chaves_bc.natureza_bc = Some(225);

            if chaves_bc.natureza_bc == Some(55) {
                chaves_bc.tipo_de_operacao = Some(5);
            } else {
                chaves_bc.tipo_de_operacao = Some(6);
            }
        }

        // impl Add and AddAssign for Valores: Soma de Valores
        credito_apos_ajustes
            .entry(chaves_bc)
            .and_modify(|previous_value| *previous_value += valores)
            .or_insert(valores);
    }

    // Merge two HashMaps in Rust
    base_creditos.extend(credito_apos_ajustes);
}

fn somar_base_de_calculo_valor_total(base_creditos: &mut HashMap<Chaves, Valores>) {
    let mut base_de_calculo: HashMap<Chaves, Valores> = HashMap::new();

    for (chaves, &valores) in base_creditos.iter() {
        let mut chaves_bc = chaves.clone();

        if chaves_bc.natureza_bc >= Some(101) && chaves_bc.natureza_bc <= Some(199) {
            chaves_bc.tipo_de_operacao = None;
            chaves_bc.tipo_de_credito = Some(100);
            chaves_bc.aliq_pis = None;
            chaves_bc.aliq_cofins = None;
            chaves_bc.natureza_bc = Some(300);
        } else {
            continue;
        }

        // impl Add and AddAssign for Valores: Soma de Valores
        base_de_calculo
            .entry(chaves_bc)
            .and_modify(|previous_value| *previous_value += valores)
            .or_insert(valores);
    }

    // Merge two HashMaps in Rust
    base_creditos.extend(base_de_calculo);
}

fn calcular_saldo_de_credito_passivel_de_ressarcimento(
    base_creditos: &mut HashMap<Chaves, Valores>,
) {
    let mut credito_apos_ajustes: HashMap<Chaves, Valores> = HashMap::new();

    for (chaves, &valores) in base_creditos.iter() {
        let mut chaves_bc = chaves.clone();

        // Filtrar 'Crédito Disponível após Descontos':
        let descontos_pis: bool = chaves_bc.natureza_bc == Some(221);
        let descontos_cof: bool = chaves_bc.natureza_bc == Some(225);
        let descontos: bool = descontos_pis || descontos_cof;

        if !descontos {
            continue;
        }

        chaves_bc.tipo_de_operacao = None;
        chaves_bc.tipo_de_credito = Some(100);
        chaves_bc.aliq_pis = None;
        chaves_bc.aliq_cofins = None;

        if descontos_pis {
            chaves_bc.natureza_bc = Some(301);
        }
        if descontos_cof {
            chaves_bc.natureza_bc = Some(305);
        }

        // impl Add and AddAssign for Valores: Soma de Valores
        credito_apos_ajustes
            .entry(chaves_bc)
            .and_modify(|previous_value| *previous_value += valores)
            .or_insert(valores);
    }

    // Merge two HashMaps in Rust
    base_creditos.extend(credito_apos_ajustes);
}

// https://doc.rust-lang.org/book/ch10-02-traits.html
pub fn realizar_somas_trimestrais<T, U>(resultado: &mut HashMap<T, U>)
where
    T: Mes + Eq + Hash + Clone,
    U: Clone + Add<Output = U> + AddAssign,
{
    let mut resultado_trimestral: HashMap<T, U> = HashMap::new();

    for (chaves, valores) in resultado.iter() {
        let mut chaves_trimestrais = chaves.clone();

        // mês fictício 13 para fins de soma e ordenação.
        chaves_trimestrais.set_mes(Some(13));

        // impl Add and AddAssign for Valores: Soma de Valores
        resultado_trimestral
            .entry(chaves_trimestrais)
            .and_modify(|previous_value| *previous_value += valores.clone())
            .or_insert(valores.clone());
    }

    // Merge two HashMaps in Rust
    resultado.extend(resultado_trimestral);
}

fn ordenar(hmap: HashMap<Chaves, Valores>) -> Vec<(Chaves, Valores)> {
    // transform hashmap to vec
    let mut vec_from_hash: Vec<(Chaves, Valores)> = hmap.into_iter().collect();

    vec_from_hash.sort_by_key(|(chaves, _valores)| {
        (
            chaves.cnpj_base.clone(),
            chaves.ano,
            chaves.trimestre,
            chaves.mes,
            chaves.tipo_de_credito,
            chaves.tipo_de_operacao,
            chaves.cst,
            chaves.natureza_bc,
            //(chaves.cst.is_none(), 100),
            //(chaves.cst.is_some(), Reverse(chaves.cst)),
            //(Reverse(chaves.cst), chaves.natureza_bc <= Some(18)),
            chaves.aliq_pis,
            chaves.aliq_cofins,
        )
    });

    vec_from_hash.par_iter_mut().for_each(|(chaves, _valores)| {
        // Remover valores temporários de CST.
        // Estes valores foram adicionados com a finalidade de ordenação.
        if chaves.cst >= Some(900) {
            chaves.cst = None;
        }
    });

    vec_from_hash
}

fn get_analises(info_ordenada: &[(Chaves, Valores)]) -> Vec<AnaliseDosCreditos> {
    let mut lines: Vec<AnaliseDosCreditos> = Vec::new();

    for (chaves, valores) in info_ordenada {
        let receita_bruta_cumulativa = calcular_rb_cumulativa(valores);

        let mut line = AnaliseDosCreditos {
            cnpj_base: chaves.cnpj_base.clone(),
            ano: chaves.ano,
            trimestre: chaves.trimestre,
            mes: chaves.mes,
            tipo_de_operacao: chaves.tipo_de_operacao,
            tipo_de_credito: chaves.tipo_de_credito,
            cst: chaves.cst,
            aliq_pis: chaves.aliq_pis.map(|v| *v),
            aliq_cofins: chaves.aliq_cofins.map(|v| *v),
            natureza_bc: chaves.natureza_bc,
            valor_bc: Some(valores.valor_bc),
            valor_rbnc_trib: Some(valores.valor_rbnc_trib),
            valor_rbnc_ntrib: Some(valores.valor_rbnc_ntrib),
            valor_rbnc_exp: Some(valores.valor_rbnc_exp),
            valor_rb_cum: Some(receita_bruta_cumulativa),
        };

        line.despise_small_values();

        lines.push(line);
    }

    lines
}

// Calcular Receita Bruta Cumulativa por diferença:
// Receita Bruta Total = Receita Bruta Cumulativa + Receita Bruta Não Cumulativa ->
// Receita Bruta Cumulativa = Receita Bruta Total - Receita Bruta Não Cumulativa
fn calcular_rb_cumulativa(val: &Valores) -> f64 {
    let rec_bruta_nao_cumulativa = val.valor_rbnc_trib + val.valor_rbnc_ntrib + val.valor_rbnc_exp;

    let mut rec_bruta_cumulativa = val.valor_bc - rec_bruta_nao_cumulativa;

    if rec_bruta_cumulativa < 0.10 {
        rec_bruta_cumulativa = ZERO;
    }

    rec_bruta_cumulativa
}

fn gerar_tabela_nat<'a, T: Tabled + Deserialize<'a>>(lines: &[T]) -> String {
    // use serde_aux::prelude::serde_introspect;
    let colunas_vec = serde_introspect::<T>();
    let number_of_fields = colunas_vec.len();

    // println!("number_of_fields: {number_of_fields}");

    // https://crates.io/crates/tabled
    Table::new(lines)
        .with(Modify::new(Segment::all()).with(Alignment::center()))
        .with(Modify::new(Columns::one(number_of_fields - 6)).with(Alignment::left()))
        .with(Modify::new(Columns::new(number_of_fields - 5..)).with(Alignment::right()))
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
    fn struct_sum() {
        // cargo test
        let val_a = Valores {
            valor_item: 1.2,
            valor_bc: 1.2,
            valor_rbnc_trib: 0.2,
            valor_rbnc_ntrib: 0.2,
            valor_rbnc_exp: 0.2,
            valor_rb_cum: 0.2,
        };
        let val_b = Valores {
            valor_item: 0.3,
            valor_bc: 0.3,
            valor_rbnc_trib: 1.3,
            valor_rbnc_ntrib: 1.3,
            valor_rbnc_exp: 1.3,
            valor_rb_cum: 1.3,
        };
        let val_c = Valores {
            valor_item: 3.5,
            valor_bc: 3.5,
            valor_rbnc_trib: 3.5,
            valor_rbnc_ntrib: 3.5,
            valor_rbnc_exp: 3.5,
            valor_rb_cum: 3.5,
        };
        let val_d = Valores {
            valor_item: 5.0,
            valor_bc: 5.0,
            valor_rbnc_trib: 5.0,
            valor_rbnc_ntrib: 5.0,
            valor_rbnc_exp: 5.0,
            valor_rb_cum: 5.0,
        };
        assert_eq!(val_a + val_b + val_c, val_d);
    }

    #[test]
    fn struct_multiply() {
        // cargo test
        // cargo test -- --show-output struct_mul
        let val_a1 = Valores {
            valor_item: 1.0,
            valor_bc: 1.0,
            valor_rbnc_trib: 1.0,
            valor_rbnc_ntrib: 1.0,
            valor_rbnc_exp: 1.0,
            valor_rb_cum: 1.0,
        };

        let val_a5 = val_a1 * 5.0;
        let val_a7 = val_a1.mul(7.0);

        let val_b = Valores {
            valor_item: 5.0,
            valor_bc: 5.0,
            valor_rbnc_trib: 5.0,
            valor_rbnc_ntrib: 5.0,
            valor_rbnc_exp: 5.0,
            valor_rb_cum: 5.0,
        };
        let val_c = Valores {
            valor_item: 7.0,
            valor_bc: 7.0,
            valor_rbnc_trib: 7.0,
            valor_rbnc_ntrib: 7.0,
            valor_rbnc_exp: 7.0,
            valor_rb_cum: 7.0,
        };

        assert_eq!(val_a5, val_b);
        assert_eq!(val_a7, val_c);

        println!("val_a1: {val_a1:#?}");
        println!("val_a5: {val_a5:#?}");
        println!("val_a7: {val_a7:#?}");
    }

    #[test]
    fn hashmap_sum_values_by_key() {
        // cargo test -- --show-output hashmap_sum_values_by_key

        let mut docs_fiscais1 = DocsFiscais {
            ..Default::default()
        };
        docs_fiscais1.estabelecimento_cnpj = "12.345.678/0001-23".to_string();
        docs_fiscais1.tipo_de_operacao = Some(1); // 1: Entrada
        docs_fiscais1.cst = Some(51);
        docs_fiscais1.cfop = Some(1234);
        docs_fiscais1.tipo_de_credito = Some(1);
        docs_fiscais1.natureza_bc = Some(7);
        docs_fiscais1.valor_item = Some(15.000);
        docs_fiscais1.valor_bc = Some(10.000);

        let mut docs_fiscais2 = DocsFiscais {
            ..Default::default()
        };
        docs_fiscais2.estabelecimento_cnpj = "12.345.678/0001-23".to_string();
        docs_fiscais2.tipo_de_operacao = Some(1); // 1: Entrada
        docs_fiscais2.cst = Some(51);
        docs_fiscais2.cfop = Some(2345);
        docs_fiscais2.tipo_de_credito = Some(1);
        docs_fiscais2.natureza_bc = Some(12);
        docs_fiscais2.valor_item = Some(22.000);
        docs_fiscais2.valor_bc = Some(8.000);

        let mut docs_fiscais3 = DocsFiscais {
            ..Default::default()
        };
        docs_fiscais3.estabelecimento_cnpj = "12.345.678/0001-23".to_string();
        docs_fiscais3.tipo_de_operacao = Some(1); // 1: Entrada
        docs_fiscais3.cst = Some(51);
        docs_fiscais3.cfop = Some(3456);
        docs_fiscais3.tipo_de_credito = Some(1);
        docs_fiscais3.natureza_bc = Some(12);
        docs_fiscais3.valor_item = Some(8.000);
        docs_fiscais3.valor_bc = Some(2.000);

        let mut docs_fiscais4 = DocsFiscais {
            ..Default::default()
        };
        docs_fiscais4.estabelecimento_cnpj = "12.345.678/0001-23".to_string();
        docs_fiscais4.tipo_de_operacao = Some(1); // 1: Entrada
        docs_fiscais4.cst = Some(51);
        docs_fiscais4.cfop = Some(4567);
        docs_fiscais4.tipo_de_credito = Some(1);
        docs_fiscais4.natureza_bc = Some(7);
        docs_fiscais4.valor_item = Some(25.000);
        docs_fiscais4.valor_bc = Some(18.000);

        let mut docs_fiscais5 = DocsFiscais {
            ..Default::default()
        };
        docs_fiscais5.estabelecimento_cnpj = "12.345.678/0001-23".to_string();
        docs_fiscais5.tipo_de_operacao = Some(1); // 1: Entrada
        docs_fiscais5.cst = Some(51);
        docs_fiscais5.cfop = Some(5678);
        docs_fiscais5.tipo_de_credito = Some(1);
        docs_fiscais5.natureza_bc = Some(7);
        docs_fiscais5.valor_item = Some(10.000);
        docs_fiscais5.valor_bc = Some(12.000);

        let linhas = vec![
            docs_fiscais1,
            docs_fiscais2,
            docs_fiscais3,
            docs_fiscais4,
            docs_fiscais5,
        ];

        let somas_nat: HashMap<Chaves, Valores> = consolidar_chaves(&linhas);

        println!("somas_nat: {somas_nat:#?}");

        let chaves1 = Chaves {
            cnpj_base: "12.345.678".to_string(),
            ano: None,
            trimestre: None,
            mes: None,
            tipo_de_operacao: Some(1),
            tipo_de_credito: Some(1),
            cst: Some(51),
            cfop: None,
            aliq_pis: None,
            aliq_cofins: None,
            natureza_bc: Some(7),
        };

        let valores1 = Valores {
            valor_item: 50.00,
            valor_bc: 40.00,
            valor_rbnc_trib: ZERO,
            valor_rbnc_ntrib: ZERO,
            valor_rbnc_exp: ZERO,
            valor_rb_cum: ZERO,
        };

        let chaves2 = Chaves {
            cnpj_base: "12.345.678".to_string(),
            ano: None,
            trimestre: None,
            mes: None,
            tipo_de_operacao: Some(1),
            tipo_de_credito: Some(1),
            cst: Some(51),
            cfop: None,
            aliq_pis: None,
            aliq_cofins: None,
            natureza_bc: Some(12),
        };

        let valores2 = Valores {
            valor_item: 30.00,
            valor_bc: 10.00,
            valor_rbnc_trib: ZERO,
            valor_rbnc_ntrib: ZERO,
            valor_rbnc_exp: ZERO,
            valor_rb_cum: ZERO,
        };

        let hashmap = HashMap::from([(chaves1, valores1), (chaves2, valores2)]);

        assert_eq!(hashmap, somas_nat);
    }
}
