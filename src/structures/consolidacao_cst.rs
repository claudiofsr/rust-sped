use compact_str::CompactString;
use csv::StringRecord;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize, Serializer};
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
    CSTOption, CodigoSituacaoTributaria, Despise, DocsFiscais, EFDResult, ExcelCustomFormatter,
    InfoExtension, MesesDoAno, RowStyle, consolidar_registros, display_decimal, display_mes,
    display_value, realizar_somas_trimestrais, serialize_decimal, verificar_periodo_multiplo,
};

// ==============================================================================
// Estruturas de Chaves e Valores (Agregação Intermediária)
// ==============================================================================

#[derive(Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Serialize, Deserialize)]
pub struct Keys {
    //pub path: CompactString,
    pub cnpj_base: CompactString,
    pub ano: Option<i32>,
    pub trimestre: Option<u32>,
    pub mes: Option<MesesDoAno>,
    pub ordem: Option<u16>,
    pub cst: Option<CodigoSituacaoTributaria>,
}

impl From<&DocsFiscais> for Keys {
    fn from(linha: &DocsFiscais) -> Self {
        Self {
            //path: linha.arquivo_efd.to_compact_string(),
            cnpj_base: linha.get_cnpj_base(),
            ano: linha.ano,
            trimestre: linha.trimestre,
            mes: linha.mes,
            ordem: linha.cst.map(|c| c.get_ordem()),
            cst: linha.cst,
        }
    }
}

impl Keys {
    /// Identifica se a chave atual pertence a um grupo que requer totalização
    /// (Total Receitas/Saídas) ou (Total Aquisições).
    /// Retorna uma nova Key configurada para o totalizador ou None.
    pub fn get_total_category(&self) -> Option<Self> {
        let code = self.cst.code()?;

        let (novo_cst, nova_ordem) = match code {
            1..=49 => (CodigoSituacaoTributaria::TotalReceitasSaidas, 2),
            50..=98 => (CodigoSituacaoTributaria::TotalAquisicoes, 4),
            _ => return None,
        };

        let mut nova_chave = self.clone();
        nova_chave.ordem = Some(nova_ordem);
        nova_chave.cst = Some(novo_cst);

        Some(nova_chave)
    }
}

/// Estrutura intermediária para acumulação.
#[derive(Debug, Default, PartialEq, PartialOrd, Copy, Clone, Serialize, Deserialize)]
struct Values {
    // O Default de Decimal é Zero.
    valor_item: Decimal,
    valor_bc: Decimal,
    valor_pis: Decimal,
    valor_cofins: Decimal,
}

impl From<&DocsFiscais> for Values {
    fn from(linha: &DocsFiscais) -> Self {
        Self {
            valor_item: linha.valor_item.unwrap_or_default(),
            valor_bc: linha.valor_bc.unwrap_or_default(),
            valor_pis: linha.valor_pis.unwrap_or_default(),
            valor_cofins: linha.valor_cofins.unwrap_or_default(),
        }
    }
}

impl Add for Values {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            valor_item: self.valor_item + other.valor_item,
            valor_bc: self.valor_bc + other.valor_bc,
            valor_pis: self.valor_pis + other.valor_pis,
            valor_cofins: self.valor_cofins + other.valor_cofins,
        }
    }
}

/// Executar a operação +=
impl AddAssign for Values {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

// ==============================================================================
// Estrutura Final (Saída/Exibição)
// ==============================================================================

/// Consolidação CST
#[derive(
    Debug, Default, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Tabled, Iterable,
)]
pub struct ConsolidacaoCST {
    //#[serde(rename = "Arquivo da EFD Contribuições")] // skip
    //#[tabled(rename = "Arquivo da EFD Contribuições", skip)]
    //pub path: CompactString,
    #[serde(rename = "CNPJ Base")]
    #[tabled(rename = "CNPJ Base")]
    pub cnpj_base: CompactString,

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
    pub cst: Option<CodigoSituacaoTributaria>,

    // ==========================================================================
    // Valores em Decimal Puro
    // ==========================================================================
    #[serde(rename = "Valor Total do Item", serialize_with = "serialize_decimal")]
    #[tabled(rename = "Valor Total do Item", display = "display_decimal")]
    pub valor_item: Decimal,

    #[serde(rename = "Base de Cálculo", serialize_with = "serialize_decimal")]
    #[tabled(rename = "Base de Cálculo", display = "display_decimal")]
    pub valor_bc: Decimal,

    #[serde(rename = "Valor de PIS/PASEP", serialize_with = "serialize_decimal")]
    #[tabled(rename = "Valor de PIS/PASEP", display = "display_decimal")]
    pub valor_pis: Decimal,

    #[serde(rename = "Valor de COFINS", serialize_with = "serialize_decimal")]
    #[tabled(rename = "Valor de COFINS", display = "display_decimal")]
    pub valor_cofins: Decimal,
}

impl InfoExtension for ConsolidacaoCST {}

impl ExcelCustomFormatter for ConsolidacaoCST {
    fn row_style(&self) -> RowStyle {
        match self.cst {
            Some(CodigoSituacaoTributaria::TotalReceitasSaidas)
            | Some(CodigoSituacaoTributaria::TotalAquisicoes) => RowStyle::Soma,

            _ => RowStyle::Default,
        }
    }
}

impl ConsolidacaoCST {
    pub fn get_headers() -> StringRecord {
        // use serde_aux::prelude::serde_introspect;
        let colunas_vec = serde_introspect::<ConsolidacaoCST>();
        StringRecord::from(colunas_vec)
    }
}

// Converte a agregação intermediária (Keys, Values) para a estrutura final
impl From<(Keys, Values)> for ConsolidacaoCST {
    fn from((key, val): (Keys, Values)) -> Self {
        let mut line = Self {
            // keys
            //path: key.path,
            cnpj_base: key.cnpj_base,
            ano: key.ano,
            trimestre: key.trimestre,
            mes: key.mes,
            cst: key.cst,
            // Values
            valor_item: val.valor_item,
            valor_bc: val.valor_bc,
            valor_pis: val.valor_pis,
            valor_cofins: val.valor_cofins,
        };

        // Aplica regra de negócio para zerar/remover valores insignificantes
        line.despise_small_values();
        line
    }
}

// ==============================================================================
// Serializers e Display
// ==============================================================================

pub fn display_cst(cst_opt: &Option<CodigoSituacaoTributaria>) -> String {
    // cst_opt.map(|c| c.to_string()).unwrap_or_default()
    // cst_opt.map(|cst| format!("{:02}", cst.code())).unwrap_or_default()
    match cst_opt.code() {
        Some(490) => "Total Receitas/Saídas".to_string(),
        Some(980) => "Total Aquisições/Custos/Despesas".to_string(),
        Some(cst) => format!("{cst:02}"),
        None => String::new(),
    }
}

pub fn serialize_cst<S>(
    cst_opt: &Option<CodigoSituacaoTributaria>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let string = display_cst(cst_opt);
    serializer.serialize_str(&string)
}

// ==============================================================================
// Lógica Principal de Processamento
// ==============================================================================

pub fn consolidar_operacoes_por_cst(
    linhas: &[DocsFiscais],
) -> EFDResult<(Option<String>, Vec<ConsolidacaoCST>)> {
    // 1. Map-Reduce (Generic from Utils)
    let mut resultado = consolidar_registros(
        linhas,
        |linha| linha.operacoes_de_entrada_ou_saida(),
        |linha| (Keys::from(linha), Values::from(linha)),
    );

    // 2. Somas Parciais (Totais por grupo)
    realizar_soma_parcial(&mut resultado);

    // 3. Somas Trimestrais (Se aplicável)
    if verificar_periodo_multiplo(&resultado) {
        realizar_somas_trimestrais(&mut resultado);
    }

    // 4. Ordenação
    let resultado_ordenado = ordenar_cst(resultado);

    // 5. Transformação final para estrutura de saída
    let resultado_estruturado: Vec<ConsolidacaoCST> = resultado_ordenado
        .into_iter()
        .map(ConsolidacaoCST::from)
        .collect();

    // 6. Geração da Tabela Visual
    let tabela_de_resultado = gerar_tabela_cst(&resultado_estruturado);

    Ok((tabela_de_resultado, resultado_estruturado))
}

fn realizar_soma_parcial(resultado: &mut HashMap<Keys, Values>) {
    let mut soma_parcial: HashMap<Keys, Values> = HashMap::new();

    for (keys, &values) in resultado.iter() {
        if let Some(chave_total) = keys.get_total_category() {
            // Como Values usa Decimal (Default = 0.0), a soma é matematicamente consistente.
            // .or_default() cria 0.0, e 0.0 + values == values.
            *soma_parcial.entry(chave_total).or_default() += values;
        }
    }

    // Merge two HashMaps in Rust
    resultado.extend(soma_parcial);
}

fn ordenar_cst(hmap: HashMap<Keys, Values>) -> Vec<(Keys, Values)> {
    // transform hashmap to vec
    let mut vec_from_hash: Vec<(Keys, Values)> = hmap.into_iter().collect();

    vec_from_hash.sort_unstable_by_key(|(chaves, _valores)| {
        (
            //chaves.path.clone(),
            chaves.cnpj_base.clone(),
            chaves.ano,
            chaves.trimestre,
            // CRITÉRIO DE OURO:
            // 1. mes.is_none() -> false (0) para meses reais, true (1) para o total.
            // Isso garante que o None (total) fique no final do trimestre.
            chaves.mes.is_none(),
            chaves.mes, // Ordenação secundária entre os meses (Jan, Fev...)
            chaves.ordem,
            chaves.cst,
        )
    });

    vec_from_hash
}

fn gerar_tabela_cst<T>(lines: &[T]) -> Option<String>
where
    T: Tabled,
{
    if lines.is_empty() {
        return None;
    }

    // O limit define até qual coluna aplicaremos o alinhamento central
    let limit = 5;

    Some(
        Table::new(lines)
            .with(Modify::new(Segment::all()).with(Alignment::right()))
            // Aplica centro para as colunas de identificação (CNPJ, Ano, Trim, Mes, CST)
            .with(Modify::new(Columns::new(0..limit)).with(Alignment::center()))
            // Aplica centro para o cabeçalho (primeira linha)
            .with(Modify::new(Rows::first()).with(Alignment::center()))
            //.with(Modify::new(Rows::one(0)).with(Format::new(|s| s.blue().to_string())))
            //.with(Modify::new(Rows::new(..)).with(Format::new(|s| s.blue().to_string())))
            .with(Style::rounded())
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

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
            //path: "file_path".into(),
            cnpj_base: "12.345.678".into(),
            ano: Some(2022),
            trimestre: Some(1),
            mes: Some(MesesDoAno::Setembro),
            cst: Some(CodigoSituacaoTributaria::CredVincRecTribENTribMIExp),
            valor_item: dec!(0.00499), // <-- desprezar este valor, pois menor que SMALL_VALUE
            valor_bc: dec!(27.56),
            valor_pis: dec!(-0.002), // <-- desprezar este valor, pois menor que SMALL_VALUE
            valor_cofins: dec!(0.0091),
        };

        println!("line_a: {line_a:?}");

        line_a.despise_small_values();

        let line_b = ConsolidacaoCST {
            //path: "file_path".into(),
            cnpj_base: "12.345.678".into(),
            ano: Some(2022),
            trimestre: Some(1),
            mes: Some(MesesDoAno::Setembro),
            cst: Some(CodigoSituacaoTributaria::CredVincRecTribENTribMIExp),
            valor_item: Decimal::ZERO,
            valor_bc: dec!(27.56),
            valor_pis: Decimal::ZERO,
            valor_cofins: dec!(0.0091),
        };

        println!("line_a: {line_a:?}");
        println!("line_b: {line_b:?}");

        assert_eq!(line_a, line_b);
    }
}
