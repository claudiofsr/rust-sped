use compact_str::CompactString;
use csv::StringRecord;
use itertools::Itertools;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::serde_introspect;
use std::collections::HashMap;
use std::fmt;
use std::ops::AddAssign;
use tabled::Tabled;

use crate::{
    CSTOption,
    CodigoSituacaoTributaria,
    DecimalExt,
    MesesDoAno,
    analise_dos_creditos::{Chaves, Valores},
    serialize_option_decimal,
    utils::{display_decimal, display_mes, display_value, serialize_decimal}, // Importa as structs de agregação do arquivo pai/irmão
};

// ==============================================================================
// Enums e Structs Específicos de Receita
// ==============================================================================

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

impl fmt::Display for ReceitaBruta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.serialize(f)
    }
}

#[derive(Debug, Default, Eq, PartialEq, PartialOrd, Clone, Hash, Serialize, Deserialize)]
pub struct PeriodoDeApuracao {
    //#[serde(rename = "Arquivo da EFD Contribuições")]
    //pub path: CompactString,
    #[serde(rename = "CNPJ Base")]
    pub cnpj_base: CompactString,
    #[serde(rename = "Ano do Período de Apuração")]
    pub ano: Option<i32>,
    #[serde(rename = "Trimestre do Período de Apuração")]
    pub trimestre: Option<u32>,
    #[serde(rename = "Mês do Período de Apuração")]
    pub mes: Option<MesesDoAno>,
    #[serde(rename = "Receita Bruta Segregada para Fins de Rateio dos Créditos")]
    pub rec_bruta: Option<ReceitaBruta>,
}

#[derive(Debug, Default, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub struct ValorDaReceita {
    #[serde(rename = "Valor")]
    pub valor: Decimal,
    #[serde(rename = "Percentual")]
    pub pct: Decimal,
    #[serde(rename = "CST")]
    pub csts: Vec<Option<CodigoSituacaoTributaria>>,
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

/// Receita Bruta Apurada e Segregada Conforme CST para Fins de Rateio dos Créditos
///
/// Estrutura final de saída para tabela
#[derive(Debug, Default, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Tabled)]
pub struct ReceitaBrutaSegregadaPorCST {
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

    #[serde(rename = "CST")]
    #[tabled(rename = "CST", display = "display_csts")]
    pub csts: Vec<Option<CodigoSituacaoTributaria>>,

    #[serde(rename = "Receita Bruta Segregada para Fins de Rateio dos Créditos")]
    #[tabled(
        rename = "Receita Bruta Segregada para Fins de Rateio dos Créditos",
        display = "display_value"
    )]
    pub rec_bruta: Option<ReceitaBruta>,

    #[serde(rename = "Valor", serialize_with = "serialize_decimal")]
    #[tabled(rename = "Valor", display = "display_decimal")]
    pub valor: Decimal,

    #[serde(rename = "Percentual", serialize_with = "serialize_option_decimal")]
    #[tabled(rename = "Percentual", display = "display_percentual")]
    pub pct: Option<Decimal>,
}

// Helpers de Display locais

fn display_csts(csts: &[Option<CodigoSituacaoTributaria>]) -> String {
    // 1. Processa o iterador sem alocar vetor intermediário
    let joined = csts
        .iter()
        .flatten() // Transforma &Option<CST> em &CST, ignorando None
        .map(|&c| c.code())
        .sorted()
        .join(", "); // Junta diretamente (Feature do itertools)

    // 2. Verifica se a string resultante está vazia
    if joined.is_empty() {
        String::new()
    } else {
        // 3. Formata adicionando os colchetes (muito mais eficiente que concat de Vec)
        format!("[{joined}]")
    }
}

fn display_percentual(valor: &Option<Decimal>) -> String {
    valor
        .as_ref()
        .map(|val| format!("{}%", val.to_formatted_string(4)))
        .unwrap_or_default() // Retorna String vazia se for None
}

// ==============================================================================
// Funções de Processamento
// ==============================================================================

/// Processa o HashMap consolidado de chaves fiscais e gera a estrutura de Receita Bruta
pub fn apurar_receita_bruta(
    receita_bruta_map: &HashMap<Chaves, Valores>,
) -> Vec<ReceitaBrutaSegregadaPorCST> {
    let receita_bruta_segregada = segregar_receita_bruta(receita_bruta_map);
    obter_informacoes_de_receita_bruta(&receita_bruta_segregada)
}

/// Obter Receita Bruta segregada por CST para fins de rateio dos créditos
fn segregar_receita_bruta(
    receita_bruta: &HashMap<Chaves, Valores>,
) -> HashMap<PeriodoDeApuracao, ValorDaReceita> {
    let mut hashmap: HashMap<PeriodoDeApuracao, ValorDaReceita> = HashMap::new();

    for (chaves, valores) in receita_bruta {
        // Somar: RbncTot = RbnTrmi + RbnNtmi + RbnExpo
        // Somar: RbTotal = RbncTot + RbCumul
        let mut receitas = vec![Some(ReceitaBruta::RbTotal)];

        if chaves.aliq_pis == Some(dec!(0.65)) && chaves.aliq_cofins == Some(dec!(3.0)) {
            receitas.push(Some(ReceitaBruta::RbCumul));
        } else {
            receitas.push(Some(ReceitaBruta::RbncTot));
            match chaves.cst.code() {
                Some(1 | 2 | 3 | 5) => receitas.push(Some(ReceitaBruta::RbnTrmi)),
                Some(4 | 6 | 7 | 8 | 9 | 49) => {
                    if chaves.cfop_de_exportacao() {
                        receitas.push(Some(ReceitaBruta::RbnExpo))
                    } else {
                        receitas.push(Some(ReceitaBruta::RbnNtmi))
                    }
                }
                _ => panic!(
                    "CST (1 <= CST <= 49) inválido para receita bruta: {:?}",
                    chaves.cst
                ),
            }
        };

        let rb = ValorDaReceita {
            valor: valores.valor_item,
            pct: Decimal::ZERO,
            csts: vec![chaves.cst],
        };

        for receita in receitas {
            let pa = PeriodoDeApuracao {
                //path: chaves.path.clone(),
                cnpj_base: chaves.cnpj_base.clone(),
                ano: chaves.ano,
                trimestre: chaves.trimestre,
                mes: chaves.mes,
                rec_bruta: receita,
            };

            *hashmap.entry(pa).or_default() += rb.clone();
        }
    }

    hashmap
}

fn obter_informacoes_de_receita_bruta(
    receita_bruta_segregada: &HashMap<PeriodoDeApuracao, ValorDaReceita>,
) -> Vec<ReceitaBrutaSegregadaPorCST> {
    let mut sorted: Vec<(&PeriodoDeApuracao, &ValorDaReceita)> =
        receita_bruta_segregada.iter().collect();

    sorted.sort_unstable_by_key(|&(periodo_de_apuracao, _valor_da_receita)| {
        (
            //periodo_de_apuracao.path.clone(),
            periodo_de_apuracao.cnpj_base.clone(),
            periodo_de_apuracao.ano,
            periodo_de_apuracao.trimestre,
            periodo_de_apuracao.mes,
            periodo_de_apuracao.rec_bruta,
        )
    });

    let mut lines: Vec<ReceitaBrutaSegregadaPorCST> = Vec::new();

    for (periodo, valor_receita) in sorted {
        // Ao Remover valores nulos, evita-se divisão por Zero.
        if !valor_receita.valor.eh_maior_que_zero() {
            continue;
        }

        // Busca o total para calcular percentual
        let mut pa_total = periodo.clone();
        pa_total.rec_bruta = Some(ReceitaBruta::RbTotal);

        let pct = receita_bruta_segregada
            .get(&pa_total)
            .map(|v_total| (valor_receita.valor / v_total.valor * dec!(100)).round_dp(4));

        lines.push(ReceitaBrutaSegregadaPorCST {
            //path: periodo.path.clone(),
            cnpj_base: periodo.cnpj_base.clone(),
            ano: periodo.ano,
            trimestre: periodo.trimestre,
            mes: periodo.mes,
            csts: valor_receita.csts.clone(),
            rec_bruta: periodo.rec_bruta,
            valor: valor_receita.valor,
            pct,
        });

        // Otimização: Se já chegou em 100% no RbncTot, pode parar (regra de negócio original)
        if periodo.rec_bruta == Some(ReceitaBruta::RbncTot) && pct == Some(dec!(100.0)) {
            break;
        }
    }

    lines
}
