use chrono::NaiveDate;
use claudiofsr_lib::{CFOP_VENDA_DE_IMOBILIZADO, OUTRAS_RECEITAS_REGEX, StrExtension, match_cast};
use compact_str::CompactString;
use csv::StringRecord;
use rust_decimal::{Decimal, prelude::ToPrimitive};
use rust_decimal_macros::dec;
use rust_xlsxwriter::serialize_option_datetime_to_excel;
use serde::{Deserialize, Serialize, Serializer};
use serde_aux::prelude::serde_introspect;
use std::sync::Arc;
use struct_iterable::Iterable;

use crate::{
    CodigoDoCredito, CodigoSituacaoTributaria, ExcelExtension, FloatExt, GrupoDeContas,
    IndicadorDeOrigem, MesesDoAno, NaturezaBaseCalculo, TipoDeCredito, TipoDeOperacao,
    TipoDeRateio, TipoDoItem, obter_descricao_do_cfop,
};

#[derive(Debug, Clone)]
pub struct Informacoes {
    pub cnpj_base: u32,
    pub periodo_de_apuracao: NaiveDate,
    pub messages: String,
    pub all_docs: Vec<DocsFiscais>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone, Iterable)]
pub struct DocsFiscais {
    #[serde(rename = "Linhas")]
    pub linhas: usize,

    #[serde(rename = "Arquivo da EFD Contribuições")]
    pub arquivo_efd: Arc<str>,

    #[serde(
        rename = "Nº da Linha da EFD",
        deserialize_with = "csv::invalid_option"
    )]
    pub num_linha_efd: Option<usize>,

    #[serde(rename = "CNPJ dos Estabelecimentos do Contribuinte")]
    pub estabelecimento_cnpj: Arc<str>,

    #[serde(rename = "Nome do Contribuinte")]
    pub estabelecimento_nome: Arc<str>,

    #[serde(
        rename = "Período de Apuração",
        serialize_with = "serialize_option_datetime_to_excel"
    )]
    pub periodo_de_apuracao: Option<NaiveDate>,

    #[serde(
        rename = "Ano do Período de Apuração",
        deserialize_with = "csv::invalid_option"
    )]
    pub ano: Option<i32>,

    #[serde(
        rename = "Trimestre do Período de Apuração",
        deserialize_with = "csv::invalid_option"
    )]
    pub trimestre: Option<u32>,

    #[serde(rename = "Mês do Período de Apuração")]
    pub mes: Option<MesesDoAno>,

    #[serde(rename = "Tipo de Operação")]
    pub tipo_de_operacao: Option<TipoDeOperacao>,

    #[serde(rename = "Indicador de Origem")]
    pub indicador_de_origem: Option<IndicadorDeOrigem>,

    #[serde(
        rename = "Código do Tipo de Crédito",
        serialize_with = "serialize_cod_credito_opt",
        deserialize_with = "csv::invalid_option"
    )]
    pub cod_credito: Option<CodigoDoCredito>,

    #[serde(
        rename = "Tipo de Crédito",
        serialize_with = "serialize_tipo_de_credito"
    )]
    pub tipo_de_credito: Option<TipoDeCredito>,

    #[serde(rename = "Registro")]
    pub registro: Arc<str>,

    #[serde(
        rename = "Código de Situação Tributária (CST)",
        serialize_with = "serialize_cst_opt"
    )]
    pub cst: Option<CodigoSituacaoTributaria>,

    #[serde(
        rename = "Código Fiscal de Operações e Prestações (CFOP)",
        // serialize_with = "serialize_cfop_excel",
        deserialize_with = "csv::invalid_option"
    )]
    pub cfop: Option<u16>,

    #[serde(
        rename = "Natureza da Base de Cálculo dos Créditos",
        serialize_with = "serialize_natureza_opt"
    )]
    pub natureza_bc: Option<NaturezaBaseCalculo>,

    #[serde(rename = "CNPJ do Participante")]
    pub participante_cnpj: Arc<str>,

    #[serde(rename = "CPF do Participante")]
    pub participante_cpf: Arc<str>,

    #[serde(rename = "Nome do Participante")]
    pub participante_nome: Arc<str>,

    #[serde(
        rename = "Nº do Documento Fiscal",
        deserialize_with = "csv::invalid_option"
    )]
    pub num_doc: Option<usize>,

    #[serde(rename = "Chave do Documento")]
    pub chave_doc: Arc<str>,

    //#[serde(rename = "Verificação da Chave")]
    //pub verificacao_chave: Arc<str>,
    #[serde(rename = "Modelo do Documento Fiscal")]
    pub modelo_doc_fiscal: Arc<str>,

    #[serde(
        rename = "Nº do Item do Documento Fiscal",
        deserialize_with = "csv::invalid_option"
    )]
    pub num_item: Option<u16>,

    #[serde(rename = "Tipo do Item")]
    pub tipo_item: Option<TipoDoItem>,

    #[serde(rename = "Descrição do Item")]
    pub descr_item: Arc<str>,

    #[serde(rename = "Código NCM")]
    pub cod_ncm: Arc<str>,

    #[serde(rename = "Natureza da Operação/Prestação")]
    pub nat_operacao: Arc<str>,

    #[serde(rename = "Informação Complementar do Documento Fiscal")]
    pub complementar: Arc<str>,

    #[serde(rename = "Escrituração Contábil: Nome da Conta")]
    pub nome_da_conta: Arc<str>,

    #[serde(
        rename = "Data da Emissão do Documento Fiscal",
        serialize_with = "serialize_option_datetime_to_excel"
    )]
    pub data_emissao: Option<NaiveDate>,

    #[serde(
        rename = "Data da Entrada / Aquisição / Execução ou da Saída / Prestação / Conclusão",
        serialize_with = "serialize_option_datetime_to_excel"
    )]
    pub data_entrada: Option<NaiveDate>,

    #[serde(
        rename = "Valor Total do Item",
        serialize_with = "serialize_option_decimal",
        deserialize_with = "csv::invalid_option"
    )]
    pub valor_item: Option<Decimal>,

    #[serde(
        rename = "Valor da Base de Cálculo das Contribuições",
        serialize_with = "serialize_option_decimal",
        deserialize_with = "csv::invalid_option"
    )]
    pub valor_bc: Option<Decimal>,
    #[serde(
        rename = "Alíquota de PIS/PASEP (em percentual)",
        serialize_with = "serialize_option_decimal",
        deserialize_with = "csv::invalid_option"
    )]
    pub aliq_pis: Option<Decimal>,

    #[serde(
        rename = "Alíquota de COFINS (em percentual)",
        serialize_with = "serialize_option_decimal",
        deserialize_with = "csv::invalid_option"
    )]
    pub aliq_cofins: Option<Decimal>,

    #[serde(
        rename = "Valor de PIS/PASEP",
        serialize_with = "serialize_option_decimal",
        deserialize_with = "csv::invalid_option"
    )]
    pub valor_pis: Option<Decimal>,

    #[serde(
        rename = "Valor de COFINS",
        serialize_with = "serialize_option_decimal",
        deserialize_with = "csv::invalid_option"
    )]
    pub valor_cofins: Option<Decimal>,

    #[serde(
        rename = "Valor de ISS",
        serialize_with = "serialize_option_decimal",
        deserialize_with = "csv::invalid_option"
    )]
    pub valor_iss: Option<Decimal>,

    #[serde(
        rename = "Valor da Base de Cálculo de ICMS",
        serialize_with = "serialize_option_decimal",
        deserialize_with = "csv::invalid_option"
    )]
    pub valor_bc_icms: Option<Decimal>,

    #[serde(
        rename = "Alíquota de ICMS (em percentual)",
        serialize_with = "serialize_option_decimal",
        deserialize_with = "csv::invalid_option"
    )]
    pub aliq_icms: Option<Decimal>,

    #[serde(
        rename = "Valor de ICMS",
        serialize_with = "serialize_option_decimal",
        deserialize_with = "csv::invalid_option"
    )]
    pub valor_icms: Option<Decimal>,
}

impl ExcelExtension for DocsFiscais {}

/// Helper function to serialize Option<Decimal> as f64 (Excel Number)
pub fn serialize_option_decimal<S>(
    value: &Option<Decimal>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(d) => {
            let float_val = d.to_f64().unwrap_or_default();
            serializer.serialize_f64(float_val)
        }
        None => serializer.serialize_none(),
    }
}

pub fn serialize_natureza_opt<S>(
    nat_opt: &Option<NaturezaBaseCalculo>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    nat_opt
        .map(|nat| nat.descricao_com_codigo()) // Transforma em String formatada
        .serialize(serializer) // Serializa o Option resultante (Some ou None)
}

/// Helper para serializar CodigoDoCredito como um número u16 simples para o Excel
pub fn serialize_cod_credito_opt<S>(
    val: &Option<CodigoDoCredito>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Converte para u16 para que o Excel trate como número, ou serialize_none
    val.map(|v| v.to_u16()).serialize(serializer)
}

pub fn serialize_cst_opt<S>(
    cst_opt: &Option<CodigoSituacaoTributaria>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    cst_opt
        .filter(|cst| (1..100).contains(&cst.code())) // Filtra apenas códigos válidos (01-99)
        .map(|cst| cst.descricao_com_codigo()) // Transforma em String formatada
        //.map(|cst| format!("{:02}", cst.code())) // Transforma em String formatada
        .serialize(serializer) // Serializa o Option resultante (Some ou None)
}

fn serialize_tipo_de_credito<S>(
    tipo_opt: &Option<TipoDeCredito>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    tipo_opt
        .map(|tipo| tipo.descricao_com_codigo()) // Transforma em String formatada
        //.map(|tipo| format!("{:02}", tipo.code())) // Transforma em String formatada
        .serialize(serializer) // Serializa o Option resultante (Some ou None)
}

#[allow(dead_code)]
fn serialize_cfop_excel<S>(val: &Option<u16>, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    // excel_alternative usa: obter_descricao_do_cfop(col.cfop)
    val.map(|c| obter_descricao_do_cfop(Some(c))).serialize(s)
}

trait PrintOptionDecimal {
    /// Utilizado em arquivos CSV
    fn to_some_string(&self) -> Option<String>;
}

impl PrintOptionDecimal for Option<Decimal> {
    fn to_some_string(&self) -> Option<String> {
        self.map(|decimal| {
            let float_64 = decimal.to_f64()?;

            if float_64.fract().eh_zero() {
                Some(format!("{float_64:.1}"))
            } else {
                Some(float_64.to_string())
            }
        })
        .unwrap_or_default()
    }
}

impl DocsFiscais {
    pub fn bloco(&self) -> char {
        self.registro.chars().next().unwrap_or('?')
    }

    pub fn get_headers() -> StringRecord {
        let colunas_vec = serde_introspect::<DocsFiscais>();
        StringRecord::from(colunas_vec)
    }

    pub fn get_values(&self) -> Vec<String> {
        self.iter()
            .map(|(_field, value)| {
                match_cast!( value {
                    val as Option<u16> => { val.as_ref().map(|s| s.to_string()) },
                    val as Option<u32> => { val.as_ref().map(|s| s.to_string()) },
                    val as Option<i32> => { val.as_ref().map(|s| s.to_string()) },
                    val as Option<usize> => { val.as_ref().map(|s| s.to_string()) },
                    val as Option<Decimal> => { val.to_some_string() },
                    val as Option<NaiveDate> => { val.as_ref().map(|s| s.to_string()) },
                    val as Option<GrupoDeContas> => { val.as_ref().map(|&cod| cod.code().to_string()) },
                    val as Option<CodigoDoCredito> => { val.as_ref().map(|&cod| cod.to_u16().to_string()) },
                    val as Option<CodigoSituacaoTributaria> => { val.as_ref().map(|&cst| (cst as u16).to_string()) },
                    val as Option<IndicadorDeOrigem> => { val.as_ref().map(|&indicador| (indicador as u8).to_string()) },
                    val as Option<TipoDeOperacao> => { val.as_ref().map(|&tipo| (tipo as u8).to_string()) },
                    val as Option<TipoDeCredito> => { val.as_ref().map(|&tipo| (tipo as u8).to_string()) },
                    val as Option<TipoDoItem> => { val.as_ref().map(|&tipo| tipo.descricao_com_codigo()) },
                    val as Option<MesesDoAno> => { val.as_ref().map(|&mes| (mes as u8).to_string()) },
                    val as Option<NaturezaBaseCalculo> => { val.as_ref().map(|&nat| (nat as u16).to_string()) },
                    val as Option<TipoDeRateio> => { val.as_ref().map(|&tipo| (tipo as u8).to_string()) },
                    val as usize => { Some(val.to_string()) },
                    val as Arc<str> => { Some(val.to_string()) },
                    val as String => { Some(val.to_string()) },
                }).unwrap_or_default()
            })
            .collect()
    }

    /// Retorna o CNPJ Base (10 primeiros dígitos com formatação).
    ///
    /// Retorna CompactString para evitar alocação na Heap (Zero-Allocation para strings curtas).
    ///
    /// ### Exemplo
    ///
    /// `"12.345.678/0001-12" --> "12.345.678"`
    pub fn get_cnpj_base(&self) -> CompactString {
        if self.estabelecimento_cnpj.len() >= 10 {
            // Cria a CompactString a partir do slice, sem alocar String intermediária
            CompactString::new(&self.estabelecimento_cnpj[0..10])
        } else {
            CompactString::new(&self.estabelecimento_cnpj)
        }
    }

    /// Verifica se o campo CST está preenchido (se é um Enum válido).
    pub fn cst_valido(&self) -> bool {
        // Se foi possível fazer o parse para o Enum, ele é semanticamente válido por definição.
        self.cst.is_some()
    }

    /// Verifica se o CST é de Crédito (50-56, 60-66).
    pub fn cst_de_credito(&self) -> bool {
        // Usa o as_ref() para não consumir o valor (Copy) e aplica a lógica
        self.cst.is_some_and(|cst| cst.eh_base_de_credito())
    }

    /// Verifica se o CST é de Receita Bruta (01-09, 49).
    pub fn cst_de_receita_bruta(&self) -> bool {
        self.cst.is_some_and(|cst| cst.eh_receita_bruta(false))
    }

    pub fn natureza_da_base_de_calculo(&self) -> bool {
        self.natureza_bc.is_some_and(|n| n.eh_geradora_de_credito())
    }

    pub fn aliquota_de_receita_financeira(&self) -> bool {
        self.aliq_pis == Some(dec!(0.65)) && self.aliq_cofins == Some(dec!(4.00))
    }

    pub fn aliquota_de_receita_cumulativa(&self) -> bool {
        self.aliq_pis == Some(dec!(0.65)) && self.aliq_cofins == Some(dec!(3.00))
    }

    pub fn cfop_de_venda_de_imobilizado(&self) -> bool {
        self.cfop
            .is_some_and(|value| CFOP_VENDA_DE_IMOBILIZADO.binary_search(&value).is_ok())
    }

    pub fn entrada_de_credito(&self) -> bool {
        self.cst_de_credito()
            && self.natureza_da_base_de_calculo()
            && self
                .tipo_de_operacao
                .is_some_and(|t| t == TipoDeOperacao::Entrada)
            && self.tipo_de_credito.is_some()
    }

    pub fn operacoes_de_entrada_ou_saida(&self) -> bool {
        self.tipo_de_operacao
            .is_some_and(|t| t.is_entrada_ou_saida())
    }

    pub fn descricao_de_outras_receitas(&self) -> bool {
        OUTRAS_RECEITAS_REGEX.is_match(&self.descr_item)
            || OUTRAS_RECEITAS_REGEX.is_match(&self.nome_da_conta)
            || OUTRAS_RECEITAS_REGEX.is_match(&self.complementar)
    }

    pub fn saida_de_receita_bruta(&self) -> bool {
        self.tipo_de_operacao
            .is_some_and(|t| t == TipoDeOperacao::Saida)
            && self.cst_de_receita_bruta()
            && !self.cfop_de_venda_de_imobilizado()
            && !self.aliquota_de_receita_financeira()
            && !self.descricao_de_outras_receitas()
    }

    /// Formatação dos campos.
    /// Como Arc<str> é imutável, criamos novas strings formatadas e reatribuímos.
    ///
    /// 44 digits: exemplo NFe: 01234567890123456789012345678901234567890123
    /// --> 01-2345-67.890.123/4567-89-01-234-567.890.123-456.789.012-3
    ///
    /// 14 digits: exemplo CNPJ: 01234567000890
    /// --> 01.234.567/0008-90
    ///
    /// 11 digits: exemplo CPF: 12345678901
    /// --> 123.456.789-01
    ///
    ///  8 digits: exemplo NCM: 01234567
    /// --> 0123.45.67
    pub fn format(&mut self) {
        // Helper para formatar e substituir apenas se necessário
        fn format_if_needed<F>(
            target: &mut Arc<str>,
            len_check: usize,
            predicate: fn(char) -> bool,
            formatter: F,
        ) where
            F: Fn(&str) -> String,
        {
            if target.len() == len_check && target.chars().all(predicate) {
                // formatter retorna String, Arc::from toma posse sem cópia extra se possível
                *target = Arc::from(formatter(target));
            }
        }

        format_if_needed(
            &mut self.estabelecimento_cnpj,
            14,
            char::is_alphanumeric,
            |s| s.format_cnpj(),
        );
        format_if_needed(
            &mut self.participante_cnpj,
            14,
            char::is_alphanumeric,
            |s| s.format_cnpj(),
        );
        format_if_needed(&mut self.participante_cpf, 11, char::is_alphanumeric, |s| {
            s.format_cpf()
        });
        format_if_needed(&mut self.cod_ncm, 8, char::is_alphanumeric, |s| {
            s.format_ncm()
        });

        let chave = &self.chave_doc;
        if chave.len() == 44 && chave.chars().all(char::is_numeric) {
            let formatted = format!(
                "{}-{}-{}.{}.{}/{}-{}-{}-{}-{}.{}.{}-{}.{}.{}-{}",
                &chave[0..2],
                &chave[2..6],
                &chave[6..8],
                &chave[8..11],
                &chave[11..14],
                &chave[14..18],
                &chave[18..20],
                &chave[20..22],
                &chave[22..25],
                &chave[25..28],
                &chave[28..31],
                &chave[31..34],
                &chave[34..37],
                &chave[37..40],
                &chave[40..43],
                &chave[43..]
            );
            self.chave_doc = Arc::from(formatted);
            //self.verificacao_chave = Arc::from("Válida");
        } else if !chave.is_empty() {
            //self.verificacao_chave = Arc::from("Inválida/Tamanho Incorreto");
        }
    }
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//
//
// cargo test -- --help
// cargo test -- --nocapture
// cargo test -- --show-output

/// Run tests with:
/// cargo test -- --show-output docs_fiscais_new_tests
#[cfg(test)]
mod docs_fiscais_new_tests {
    use super::*;

    #[test]
    fn formatar_colunas() {
        let mut colunas = DocsFiscais {
            ..Default::default()
        };
        colunas.estabelecimento_cnpj = Arc::from("01234567000890");
        colunas.participante_cnpj = Arc::from("12345678000912");
        colunas.participante_cpf = Arc::from("12345678901");
        colunas.cod_ncm = Arc::from("01234567");
        colunas.chave_doc = Arc::from("01234567890123456789012345678901234567890123");

        colunas.format();

        assert_eq!(colunas.estabelecimento_cnpj.as_ref(), "01.234.567/0008-90");
        assert_eq!(colunas.participante_cnpj.as_ref(), "12.345.678/0009-12");
        assert_eq!(colunas.participante_cpf.as_ref(), "123.456.789-01");
        assert_eq!(colunas.cod_ncm.as_ref(), "0123.45.67");
        assert_eq!(
            colunas.chave_doc.as_ref(),
            "01-2345-67.890.123/4567-89-01-234-567.890.123-456.789.012-3"
        );
        // assert_eq!(colunas.verificacao_chave.as_ref(), "Válida");
    }
}
